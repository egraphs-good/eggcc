"""Fixed MAX=true oracle. Exit 1 is red; only valid_original_red authorizes control mutation."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

from observation_evidence import archive_capture, retain_receipt, validate_source_package

EXACT_CONFLICT = ['(ObservationConflict 5 2 1 true false)']


def classify_capture(directory, mode, capture_exit, *, expected_argv, expected_cwd):
    """Separate the complete bug signature from malformed, killed or unrelated outcomes."""
    directory = Path(directory)
    row = {'mode': mode, 'passed': False, 'original_red_component': False,
           'receipt': str(directory / 'command.json'), 'classification': 'invalid-capture'}
    try:
        receipt = json.loads((directory / 'command.json').read_text())
        row['exit'] = receipt['returncode']
        if receipt['argv'] != expected_argv or receipt['cwd'] != str(expected_cwd):
            raise ValueError('captured argv/cwd differs from exact requested invocation')
        if capture_exit != receipt['returncode']:
            raise ValueError('capture return code disagrees with command receipt')
        if receipt['timed_out'] or receipt['returncode'] < 0:
            row['classification'] = 'timeout-or-signal'
            return row
        stderr = (directory / 'command.stderr').read_text()
        tiger_files = list((directory / 'egraphs').glob('tiger_egraph_*.json'))
        passes = list(map(int, re.findall(r'Running pass (\d+)\.\.\.', stderr)))
        full = passes == [0, 1, 2] and len(tiger_files) == 3
        row.update(passes=passes, native_egraph_files=len(tiger_files), full_three_native_passes=full,
            command_receipt_sha256=hashlib.sha256((directory / 'command.json').read_bytes()).hexdigest(),
            full_stderr_sha256=hashlib.sha256((directory / 'command.stderr').read_bytes()).hexdigest(),
            native_egraphs_sha256={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in tiger_files})
        binding = None
        if mode != 'off' and receipt['returncode'] in (0, 101):
            binding = archive_capture(directory / 'command.json', mode=mode, root=directory / 'evidence-root',
                output=directory / 'evidence-archive', allow_conflicts=(mode == 'common-seed'))
            (directory / 'evidence-binding.json').write_text(json.dumps(binding, indent=2) + '\n')
        if receipt['returncode'] == 0:
            if binding:
                normal = binding['normal_receipt']
                data = json.loads(Path(normal['retained_path']).read_text())
            else:
                argv = receipt['argv']
                original = Path(argv[argv.index('--run-data-out') + 1])
                data, normal = retain_receipt(original, directory / 'captured-run-data.json')
                if data.get('observation_evidence') is not None:
                    raise ValueError('Off unexpectedly emitted evidence')
            row.update(original=data.get('original_interpreted'), optimized=data.get('result_interpreted'), normal_receipt=normal)
            if not full:
                row['classification'] = 'incomplete-native-pipeline'
            elif row['original'] is None or row['optimized'] is None:
                row['classification'] = 'missing-interpreter-output'
            elif row['original'] == row['optimized'] == 'true\n':
                conflicts = [] if binding is None else [json.loads(p.read_text())['conflicts'] for p in (directory / 'evidence-archive').glob('pass-*/committed-before-audit.json')]
                row['passed'] = not any(conflicts)
                row['classification'] = 'oracle-pass' if row['passed'] else 'unexpected-committed-conflict'
            elif row['original'] == 'true\n' and row['optimized'] == 'false\n':
                row['classification'] = 'complete-known-wrong-answer'
                row['original_red_component'] = mode in ('off', 'common-seed') and (binding is None or binding['status'] == 'complete')
            else:
                row['classification'] = 'unrelated-output'
        elif binding and binding['status'] == 'rejected':
            conflict = json.loads((directory / 'evidence-archive/pass-00/committed-before-audit.json').read_text())['conflicts']
            witnessed = conflict == EXACT_CONFLICT and not tiger_files and passes == [0]
            row.update(classification='exact-pre-tiger-witness' if witnessed else 'unrelated-observer-rejection', conflicts=conflict)
            row['original_red_component'] = mode in ('manual', 'injective') and witnessed
        elif binding and binding['status'] == 'unsupported':
            row['classification'] = 'unrelated-domain-error'
        else:
            row['classification'] = 'unrelated-process-error'
    except (ValueError, KeyError, OSError, TypeError, IndexError) as error:
        row.update(passed=False, original_red_component=False, classification='invalid-capture', error=str(error))
    return row


def summarize(rows):
    """Require all four distinct modes and exact signatures, rather than any exit-1 failure."""
    all_modes = len(rows) == 4 and {r['mode'] for r in rows} == {'off', 'common-seed', 'manual', 'injective'}
    return {'mode_assertions': len(rows), 'passed': sum(r['passed'] for r in rows),
        'failed': sum(not r['passed'] for r in rows), 'expected_output': 'true\n',
        'capture_complete': all_modes and all(r.get('full_three_native_passes') or r['classification'] == 'exact-pre-tiger-witness' for r in rows),
        'valid_original_red': all_modes and all(r['original_red_component'] for r in rows),
        'all_corrected_green': all_modes and all(r['passed'] for r in rows), 'rows': rows}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('binary', 'source-package', 'capture-runner', 'out'):
        parser.add_argument(f'--{name}', required=True, type=Path)
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=False)
    source = Path(__file__).resolve().parents[3]
    input_path = source / 'tests/encoding_impact/arithmetic/mul-minimal-max.bril'
    identity = {}
    rows = []
    try:
        identity = {name + '_sha256': hashlib.sha256(path.read_bytes()).hexdigest() for name, path in
                    [('driver', Path(__file__)), ('helper', Path(__file__).with_name('observation_evidence.py')),
                     ('capture_runner', args.capture_runner), ('input', input_path), ('binary', args.binary),
                     ('source_package', args.source_package)]}
        package = validate_source_package(args.source_package, args.binary)
        source_manifest = Path(package.get('source_manifest', package.get('source', '')))
        source_identity = json.loads(source_manifest.read_text())
        if source_identity['source_sha256'][str(input_path.relative_to(source))] != identity['input_sha256']:
            raise ValueError('MAX input differs from frozen source package')
        identity['source_manifest_sha256'] = package['source_manifest_sha256']
        for mode in ['off', 'common-seed', 'manual', 'injective']:
            directory = args.out / mode
            directory.mkdir()
            argv = [str(args.binary.resolve()), '--with-context', '--run-mode', 'optimize', '--interp',
                '--run-data-out', str(directory / 'run-data.json'), '--egraph-out-dir', str(directory / 'egraphs'),
                '--observations', mode]
            if mode != 'off':
                argv += ['--observation-evidence-dir', str(directory / 'evidence-root')]
            argv.append(str(input_path))
            try:
                result = subprocess.run([sys.executable, str(args.capture_runner), str(directory / 'command'),
                    '120', str(source), *argv], env=dict(os.environ, RAYON_NUM_THREADS='1', RUST_LOG='info'))
                row = classify_capture(directory, mode, result.returncode, expected_argv=argv, expected_cwd=source)
            except OSError as error:
                row = {'mode': mode, 'passed': False, 'original_red_component': False,
                       'classification': 'invalid-capture', 'error': str(error)}
            rows.append(row)
            (args.out / 'results.partial.json').write_text(json.dumps({**identity, **summarize(rows)}, indent=2) + '\n')
    except (ValueError, KeyError, OSError) as error:
        identity['preflight_error'] = str(error)
    summary = {**identity, **summarize(rows), 'input_path': str(input_path), 'source_package': str(args.source_package.resolve())}
    (args.out / 'results.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps({k: summary[k] for k in ['mode_assertions', 'passed', 'failed', 'valid_original_red', 'capture_complete']}))
    return 0 if summary['all_corrected_green'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
