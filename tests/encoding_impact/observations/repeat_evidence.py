"""Untimed identical-argv captures, including revalidation after shared receipt overwrite."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

from observation_evidence import archive_capture


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('binary', 'capture-runner', 'out'):
        parser.add_argument(f'--{name}', required=True, type=Path)
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=False)
    source = Path(__file__).resolve().parents[3]
    rows, pairs = [], []
    for case, mode in [('one', 'common-seed'), ('one', 'manual'), ('one', 'injective'),
                       ('max', 'manual'), ('max', 'injective')]:
        pair = args.out / f'{case}-{mode}'
        pair.mkdir()
        input_path = source / f'tests/encoding_impact/arithmetic/mul-minimal-{case}.bril'
        argv = [str(args.binary.resolve()), '--with-context', '--run-mode', 'optimize', '--interp',
                '--run-data-out', str(pair / 'static-run-data.json'), '--observations', mode,
                '--observation-evidence-dir', str(pair / 'evidence-root'), str(input_path)]
        prior = []
        for n in range(2):
            run = pair / f'run-{n}'
            run.mkdir()
            row = {'case': case, 'mode': mode, 'repeat': n, 'classification': 'invalid-capture',
                'input_sha256': hashlib.sha256(input_path.read_bytes()).hexdigest(),
                'binary_sha256': hashlib.sha256(args.binary.read_bytes()).hexdigest(),
                'receipt': str(run / 'command.json')}
            try:
                result = subprocess.run([sys.executable, str(args.capture_runner), str(run / 'command'),
                    '120', str(source), *argv], env=dict(os.environ, RAYON_NUM_THREADS='1', RUST_LOG='info'))
                command = json.loads((run / 'command.json').read_text())
                if result.returncode != command['returncode'] or command['timed_out'] or result.returncode != (0 if case == 'one' else 101):
                    raise ValueError('unexpected capture/command exit or timeout')
                binding = archive_capture(run / 'command.json', root=pair / 'evidence-root', mode=mode,
                    output=run / 'evidence-archive')
                if case == 'one':
                    data = json.loads(Path(binding['normal_receipt']['retained_path']).read_text())
                    if data['original_interpreted'] != 'false\n' or data['result_interpreted'] != 'false\n':
                        raise ValueError('wrong one-input output')
                elif binding['status'] != 'rejected':
                    raise ValueError('MAX was not a named observer rejection')
                (run / 'evidence-binding.json').write_text(json.dumps(binding, indent=2) + '\n')
                row.update(exit=result.returncode, child=binding['locator']['directory'],
                    binding=str(run / 'evidence-binding.json'), classification='expected-success' if case == 'one' else 'expected-rejection')
                prior.append((command, binding))
            except (ValueError, KeyError, OSError) as error:
                row['error'] = str(error)
            rows.append(row)
            (args.out / 'results.partial.json').write_text(json.dumps({'rows': rows, 'pairs': pairs}, indent=2) + '\n')
        check = {'case': case, 'mode': mode, 'passed': False}
        try:
            if len(prior) != 2 or prior[0][0]['argv'] != prior[1][0]['argv'] or prior[0][1]['locator'] == prior[1][1]['locator']:
                raise ValueError('pair incomplete, changed argv or reused child')
            first = prior[0][1]
            normal = first['normal_receipt']
            revalidated = archive_capture(pair / 'run-0/command.json', root=pair / 'evidence-root', mode=mode,
                output=pair / 'run-0/revalidated-after-second-repeat',
                retained_receipt=normal['retained_path'] if normal else None,
                retained_sha256=normal['sha256'] if normal else None)
            if revalidated['locator'] != first['locator'] or revalidated['files_sha256'] != first['files_sha256']:
                raise ValueError('historical revalidation changed first capture')
            (pair / 'run-0/revalidated-binding.json').write_text(json.dumps(revalidated, indent=2) + '\n')
            check.update(passed=True, byte_identical_argv=True, distinct_children=True, first_revalidated_after_second=True)
        except (ValueError, KeyError, OSError) as error:
            check['error'] = str(error)
        pairs.append(check)
    summary = {'invocations': len(rows), 'rows': rows, 'pairs': pairs,
        'driver_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        'helper_sha256': hashlib.sha256(Path(__file__).with_name('observation_evidence.py').read_bytes()).hexdigest(),
        'capture_runner_sha256': hashlib.sha256(args.capture_runner.read_bytes()).hexdigest(),
        'all_expected': len(rows) == 10 and all(r['classification'].startswith('expected-') for r in rows) and all(p['passed'] for p in pairs)}
    (args.out / 'results.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps({'invocations': len(rows), 'validated_pairs': sum(p['passed'] for p in pairs), 'all_expected': summary['all_expected']}))
    return 0 if summary['all_expected'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
