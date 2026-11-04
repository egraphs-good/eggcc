"""Named post-capture validation; the frozen collector does not archive evidence trees."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import stat
import sys

MODES = {'common-seed': 'CommonSeed', 'manual': 'Manual', 'injective': 'Injective'}
COLLECTOR_SHA256 = '79c4147dbe5b74d1d9138e170643a7c09205f55a6d49d355f1fc201c3a811380'


def inspect_tree(directory):
    """Reject every symlink/special entry before any content read or copy."""
    directory = Path(directory)
    if not stat.S_ISDIR(directory.lstat().st_mode):
        raise ValueError('evidence child must be a real directory')
    files = []
    pending = [directory]
    while pending:
        parent = pending.pop()
        with os.scandir(parent) as entries:
            for entry in entries:
                kind = entry.stat(follow_symlinks=False).st_mode
                path = Path(entry.path)
                if stat.S_ISDIR(kind):
                    pending.append(path)
                elif stat.S_ISREG(kind):
                    files.append(path)
                else:
                    raise ValueError('symlink or non-regular evidence entry')
    return files


def validate_evidence(locator, *, root, mode, pid, status, allow_conflicts=False):
    """Bind a terminal invocation, mandatory replay inputs and pre-Tiger audit stages."""
    root = Path(root).resolve(strict=True)
    directory, index_path = Path(locator['directory']), Path(locator['index'])
    if not directory.is_absolute() or not index_path.is_absolute():
        raise ValueError('locator must be absolute')
    # Inspect before resolve/read: resolving the child/index alone could hide a symlink.
    inspect_tree(directory)
    if directory != directory.resolve(strict=True) or index_path != directory / 'index.json':
        raise ValueError('non-canonical or swapped locator')
    if directory.parent != root:
        raise ValueError('locator outside expected root/child')
    index = json.loads(index_path.read_text())
    if (index['directory'] != str(directory) or index['index'] != str(index_path)
            or index['root'] != str(root) or index['mode'] != MODES[mode]
            or (pid is not None and index['pid'] != pid)):
        raise ValueError('stale/swapped invocation identity or mode')
    if type(index['pid']) is not int or index['pid'] <= 0 or not directory.name.startswith(f"invocation-{index['pid']}-"):
        raise ValueError('inconsistent internal invocation PID')
    if status is None:
        status = index['status']
    if index['status'] != status:
        raise ValueError('incomplete or unexpected terminal status')
    passes = index['passes']
    expected = {'complete': ('optimizer-complete', {'00', '01', '02'}),
                'rejected': ('observation-audit-rejected', {'00'}),
                'unsupported': ('unsupported-domain', {'00'})}
    if status not in expected or (index['stage'], set(passes)) != expected[status]:
        raise ValueError('incomplete/unexpected terminal stage or pass set')
    if {p.name for p in directory.iterdir()} != {'index.json', *(f'pass-{key}' for key in passes)}:
        raise ValueError('unexpected invocation-root entry')
    required = {'observations.json', 'initialization.egg', 'source-schedule.egg',
                'committed-before-audit.json', 'audit.json'}
    for key, record in passes.items():
        path = directory / f'pass-{key}'
        if record['directory'] != str(path) or not path.is_dir():
            raise ValueError('swapped pass directory')
        files = set(path.iterdir())
        indexed = {Path(p) for p in record['evidence']}
        if (len(indexed) != len(record['evidence']) or files != indexed
                or any(p.parent != path or not p.is_file() for p in files)):
            raise ValueError('missing/unindexed/outside pass evidence')
        names = {p.name for p in files}
        if status == 'unsupported':
            if names != {'domain-error.json'} or record['stage'] != 'unsupported-domain':
                raise ValueError('unsupported outcome contains seeds or unexpected evidence')
            domain = json.loads((path / 'domain-error.json').read_text())
            if domain['stage'] != 'before-seeding' or domain['emitted_observation_seeds'] != 0:
                raise ValueError('domain rejection was not before seeding')
            continue
        mandatory = required | ({'serialization-entry.json'} if status == 'complete' else set())
        if not mandatory <= names or names - mandatory - {'prepass-probe.json'}:
            raise ValueError('missing mandatory or unexpected replay evidence')
        observations = json.loads((path / 'observations.json').read_text())
        audit = json.loads((path / 'audit.json').read_text())
        committed = json.loads((path / 'committed-before-audit.json').read_text())
        if observations['pass'] != int(key) or observations['mode'] != MODES[mode]:
            raise ValueError('observation pass/mode mismatch')
        if (audit['stage'] != 'pre-serialization-observation-audit' or audit['mode'] != MODES[mode]
                or audit['native_extraction_entered'] is not False
                or committed['stage'] != 'source-schedule-committed-before-serialization'
                or committed['native_extraction_entered'] is not False):
            raise ValueError('wrong audit/committed stage, mode or extraction boundary')
        expected_audit = {'common-seed': '', 'manual': '(run observation-manual-audit 1)',
                          'injective': '(run-schedule (saturate injectivity))'}[mode]
        if committed['audit'] != expected_audit:
            raise ValueError('wrong committed audit schedule')
        if status == 'complete':
            if record['stage'] != 'native-round-trip-complete' or audit['accepted'] is not True or audit['error'] is not None:
                raise ValueError('incomplete native pass/audit')
            if committed['conflicts'] and not (allow_conflicts and mode == 'common-seed'):
                raise ValueError('accepted invocation contains observer conflict')
            marker = json.loads((path / 'serialization-entry.json').read_text())
            if marker != {'stage': 'observer-accepted-entering-serialization'}:
                raise ValueError('wrong serialization-entry stage')
        else:
            diagnostic = {'manual': 'observation-manual-audit:',
                          'injective': 'injectivity violation in constructor `ObservedBool`'}.get(mode)
            if (record['stage'] != 'observation-audit-rejected' or diagnostic is None
                    or audit['accepted'] is not False or not committed['conflicts']
                    or diagnostic not in audit['error']):
                raise ValueError('abort is not the named observer conflict')
    return directory, index


def archive_tree(directory, output):
    """Validate the whole tree and canonical destination before reading/copying bytes."""
    inspect_tree(Path(directory))
    directory = Path(directory).resolve(strict=True)
    output = Path(output).resolve()
    if output.exists() or directory == output or directory in output.parents or output in directory.parents:
        raise ValueError('archive destination must be fresh and outside source child')
    files = inspect_tree(directory)
    before = {str(p.relative_to(directory)): hashlib.sha256(p.read_bytes()).hexdigest() for p in files}
    shutil.copytree(directory, output, symlinks=True)
    after = {str(p.relative_to(directory)): hashlib.sha256(p.read_bytes()).hexdigest() for p in inspect_tree(directory)}
    copied = {str(p.relative_to(output)): hashlib.sha256(p.read_bytes()).hexdigest() for p in inspect_tree(output)}
    if before != after or before != copied:
        raise ValueError('evidence changed during post-capture archival')
    return before


def retain_receipt(original, destination, *, expected_sha256=None, expected_data=None):
    """Snapshot once; revalidation reads the hash-bound copy, never rewrites historical argv."""
    original, destination = Path(original), Path(destination).resolve()
    if destination.exists():
        if expected_sha256 is None:
            raise ValueError('existing retained receipt requires its previously bound hash')
        contents = destination.read_bytes()
    else:
        contents = original.read_bytes()
        destination.parent.mkdir(parents=True, exist_ok=True)
        with destination.open('xb') as handle:
            handle.write(contents)
    digest = hashlib.sha256(contents).hexdigest()
    data = json.loads(contents)
    if ((expected_sha256 is not None and digest != expected_sha256)
            or (expected_data is not None and data != expected_data)):
        raise ValueError('retained normal receipt hash/content mismatch')
    return data, {'original_path': str(original), 'retained_path': str(destination), 'sha256': digest}


def archive_capture(command_receipt, *, mode, root, output, allow_conflicts=False,
                    retained_receipt=None, retained_sha256=None):
    """Diagnostic capture owns full stderr/PID; successful normal JSON is retained per run."""
    command_receipt = Path(command_receipt).resolve()
    command = json.loads(command_receipt.read_text())
    if command['timed_out'] or command['returncode'] < 0:
        raise ValueError('timeout/kill is incomplete evidence, not expected rejection')
    argv = command['argv']
    stderr_path = command_receipt.with_suffix('.stderr')
    stderr = stderr_path.read_text()
    locators = [json.loads(line.removeprefix('observation-evidence: '))
                for line in stderr.splitlines() if line.startswith('observation-evidence: ')]
    if len(locators) != 1:
        raise ValueError('full stderr must retain exactly one invocation locator')
    locator = locators[0]
    normal = None
    if command['returncode'] == 0:
        historical = Path(argv[argv.index('--run-data-out') + 1])
        if not historical.is_absolute():
            historical = Path(command['cwd']) / historical
        destination = retained_receipt or command_receipt.with_name('captured-run-data.json')
        data, normal = retain_receipt(historical, destination, expected_sha256=retained_sha256)
        if data.get('observation_evidence') != locator:
            raise ValueError('normal receipt and full stderr locators differ')
        if data.get('original_interpreted') is None or data.get('result_interpreted') is None:
            raise ValueError('missing non-null diagnostic interpreter outputs')
        status = 'complete'
    else:
        if command['returncode'] != 101:
            raise ValueError('unexpected process failure')
        status = None  # Entire-tree validation precedes reading the terminal index.
    directory, index = validate_evidence(locator, root=root, mode=mode,
        pid=command['pid'], status=status, allow_conflicts=allow_conflicts)
    if command['returncode'] != 0 and index['status'] not in ('rejected', 'unsupported'):
        raise ValueError('unexpected process failure')
    files = archive_tree(directory, output)
    return {'locator': locator, 'archive': str(Path(output).resolve()), 'mode': mode,
        'pid': command['pid'], 'status': index['status'], 'command_receipt': str(command_receipt),
        'command_receipt_sha256': hashlib.sha256(command_receipt.read_bytes()).hexdigest(),
        'full_stderr': str(stderr_path), 'full_stderr_sha256': hashlib.sha256(stderr_path.read_bytes()).hexdigest(),
        'normal_receipt': normal, 'files_sha256': files, 'independent_pid_available': True,
        'collector_recursive_archival': False}


def validate_source_package(path, binary):
    """Bind a frozen endpoint's binary, source manifest and native extractor bytes."""
    package = json.loads(Path(path).read_text())
    source = Path(package.get('source_manifest', package.get('source', '')))
    if (hashlib.sha256(source.read_bytes()).hexdigest() != package['source_manifest_sha256']
            or hashlib.sha256(Path(binary).read_bytes()).hexdigest() != package['binary_sha256']
            or hashlib.sha256(Path(package['tiger']).read_bytes()).hexdigest() != package['tiger_sha256']):
        raise ValueError('frozen source package/binary/Tiger mismatch')
    if 'compiler_endpoint_manifest' in package:
        endpoint = Path(package['compiler_endpoint_manifest'])
        if hashlib.sha256(endpoint.read_bytes()).hexdigest() != package['compiler_endpoint_manifest_sha256']:
            raise ValueError('frozen endpoint manifest mismatch')
        authority = json.loads(endpoint.read_text())
        if any(authority[k] != package[k] for k in ('binary_sha256', 'source_manifest_sha256', 'tiger_sha256', 'source_patch_sha256')):
            raise ValueError('source package disagrees with frozen endpoint authority')
    return package


class VerifiedCohort:
    """Only the pinned real collector verifier establishes a cohort/row association."""
    def __init__(self, *, collector, study, results, source_artifact, reference_artifact):
        collector, self.study_path, self.results_path = map(lambda p: Path(p).resolve(), (collector, study, results))
        if hashlib.sha256(collector.read_bytes()).hexdigest() != COLLECTOR_SHA256:
            raise ValueError('not the reviewed frozen collector')
        spec = importlib.util.spec_from_file_location('observation_frozen_collector', collector)
        module = importlib.util.module_from_spec(spec)
        sys.modules[spec.name] = module
        spec.loader.exec_module(module)
        self.study_sha256 = hashlib.sha256(self.study_path.read_bytes()).hexdigest()
        self.results_sha256 = hashlib.sha256(self.results_path.read_bytes()).hexdigest()
        self.result = module.verify(self.study_path, self.results_path)
        if (hashlib.sha256(self.study_path.read_bytes()).hexdigest() != self.study_sha256
                or hashlib.sha256(self.results_path.read_bytes()).hexdigest() != self.results_sha256):
            raise ValueError('cohort changed while collector verify was reading it')
        provenance = self.result['provenance']
        sources = [p for p in provenance if p['kind'] == 'artifact' and p['id'] == source_artifact]
        references = [p for p in provenance if p['kind'] == 'artifact' and p['id'] == reference_artifact]
        if len(sources) != 1 or len(references) != 1:
            raise ValueError('source/reference must be actual verified-study artifacts')
        self.source, self.reference = sources[0], references[0]
        package = json.loads(Path(self.source['path']).read_text())
        self.package = validate_source_package(self.source['path'], package['binary'])
        source_path = str(Path(package.get('source_manifest', package.get('source', ''))).resolve())
        manifests = [p for p in provenance if p['kind'] == 'artifact' and p['path'] == source_path and p['sha256'] == package['source_manifest_sha256']]
        if len(manifests) != 1:
            raise ValueError('source manifest must be an individually verified study artifact')
        self.source_manifest = manifests[0]
        self.expected = json.loads(Path(self.reference['path']).read_text())['cases']
        self.seen_children = set()
        self.seen_rows = set()

    def archive_row(self, row_index, output):
        """Phase-aware null contract linked to exact verified correctness/reference identities."""
        if (hashlib.sha256(self.results_path.read_bytes()).hexdigest() != self.results_sha256
                or hashlib.sha256(self.study_path.read_bytes()).hexdigest() != self.study_sha256):
            raise ValueError('verified cohort changed before archival')
        if type(row_index) is not int or not 0 <= row_index < len(self.result['rows']) or row_index in self.seen_rows:
            raise ValueError('unknown or duplicate row selection')
        row = self.result['rows'][row_index]
        phase, case, arm = row['phase'], row['case'], row['arm']
        correctness = [(n, r) for n, r in enumerate(self.result['rows'])
                       if r['phase'] == 'correctness' and r['repetition'] == 0 and r['case'] == case and r['arm'] == arm]
        if len(correctness) != 1:
            raise ValueError('missing/ambiguous matching correctness row')
        correctness_index, correct = correctness[0]
        if phase not in ('correctness', 'warmup', 'measurement'):
            raise ValueError('unknown collector phase')
        for key in ('engine', 'configuration', 'cwd', 'env_overrides', 'environment_sha256'):
            if row[key] != correct[key]:
                raise ValueError('correctness/phase context mismatch')
        provenance = self.result['provenance']
        binary = next(p for p in provenance if p['kind'] == 'binary' and p['id'] == arm)
        input_entry = next(p for p in provenance if p['kind'] == 'input' and p['id'] == case)
        if binary['sha256'] != self.package['binary_sha256'] or row['command'][0] != binary['path']:
            raise ValueError('row/source package binary mismatch')
        reference = self.expected[case]
        if reference['input_sha256'] != input_entry['sha256'] or not isinstance(reference['expected_output'], str):
            raise ValueError('reference case/input mismatch')
        expected = reference['expected_output']
        if (correct['receipt']['original_interpreted'] != expected
                or correct['receipt']['result_interpreted'] != expected):
            raise ValueError('correctness/reference output mismatch')
        output = Path(output).resolve()
        if output.exists():
            raise ValueError('row archive destination must be fresh')
        # Validate phase outputs even after real verify; do not infer phase from nulls.
        fields = (row['receipt']['original_interpreted'], row['receipt']['result_interpreted'])
        if fields != ((expected, expected) if phase == 'correctness' else (None, None)):
            raise ValueError('phase/output contract mismatch')
        argv = row['command']
        mode = argv[argv.index('--observations') + 1]
        root = None if mode == 'off' else Path(argv[argv.index('--observation-evidence-dir') + 1])
        if root is not None and not root.is_absolute():
            root = Path(row['cwd']) / root
        correct_argv = correct['command']
        if correct_argv[correct_argv.index('--observations') + 1] != mode:
            raise ValueError('correctness mode mismatch')
        if mode != 'off' and correct_argv[correct_argv.index('--observation-evidence-dir') + 1] != str(root):
            raise ValueError('correctness evidence root mismatch')
        directory, index = None, None
        if mode == 'off':
            if row['receipt'].get('observation_evidence') is not None:
                raise ValueError('Off receipt unexpectedly has evidence locator')
        else:
            locator = row['receipt']['observation_evidence']
            directory, index = validate_evidence(locator, root=root, mode=mode, pid=None, status='complete')
            if str(directory) in self.seen_children:
                raise ValueError('stale/duplicate child reused by another captured row')
            if directory == output or directory in output.parents or output in directory.parents:
                raise ValueError('row destination must be outside source evidence child')
        # Recheck originals against the verified row before retaining immutable per-row copies.
        snapshots = {}
        for name, selected in [('row', row), ('correctness', correct)]:
            path = Path(selected['receipt_path'])
            if hashlib.sha256(path.read_bytes()).hexdigest() != selected['receipt_sha256']:
                raise ValueError('verified original receipt changed')
            _, snapshots[name] = retain_receipt(path, output / f'{name}-run-data.json',
                expected_sha256=selected['receipt_sha256'], expected_data=selected['receipt'])
        binding = {'row_index': row_index, 'row_identity': {k: row[k] for k in ('phase', 'repetition', 'case', 'arm')},
            'row_metadata': {k: row[k] for k in ('command', 'cwd', 'engine', 'configuration', 'env_overrides', 'environment_sha256', 'status', 'exit_code', 'signal', 'error')},
            'study': str(self.study_path), 'study_sha256': self.study_sha256,
            'results': str(self.results_path), 'results_sha256': self.results_sha256,
            'collector_sha256': COLLECTOR_SHA256, 'postprocessor_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(), 'source_artifact': self.source, 'reference_artifact': self.reference,
            'source_manifest_artifact': self.source_manifest, 'expected_output': expected,
            'correctness_identity': {k: correct[k] for k in ('phase', 'repetition', 'case', 'arm')},
            'binary': binary, 'input': input_entry, 'mode': mode, 'root': str(root) if root else None,
            'normal_receipt': snapshots['row'], 'correctness_row_index': correctness_index,
            'correctness_receipt': snapshots['correctness'], 'independent_pid_available': False,
            'full_stderr_available': False, 'collector_recursive_archival': False}
        if directory is not None:
            binding.update(locator=locator, pid=index['pid'], files_sha256=archive_tree(directory, output / 'evidence'))
            self.seen_children.add(str(directory))
        (output / 'binding.json').write_text(json.dumps(binding, indent=2) + '\n')
        self.seen_rows.add(row_index)
        return binding


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='operation', required=True)
    diagnostic = sub.add_parser('diagnostic')
    diagnostic.add_argument('--command-receipt', required=True, type=Path)
    diagnostic.add_argument('--mode', required=True, choices=MODES)
    diagnostic.add_argument('--root', required=True, type=Path)
    diagnostic.add_argument('--archive', required=True, type=Path)
    diagnostic.add_argument('--binding', required=True, type=Path)
    diagnostic.add_argument('--allow-conflicts', action='store_true')
    cohort = sub.add_parser('collector')
    for name in ('collector', 'study', 'results', 'archive'):
        cohort.add_argument(f'--{name}', required=True, type=Path)
    for name in ('source-artifact', 'reference-artifact'):
        cohort.add_argument(f'--{name}', required=True)
    args = parser.parse_args()
    if args.operation == 'diagnostic':
        binding = archive_capture(args.command_receipt, mode=args.mode, root=args.root,
            output=args.archive, allow_conflicts=args.allow_conflicts)
        args.binding.write_text(json.dumps(binding, indent=2) + '\n')
    else:
        args.archive.mkdir(parents=True, exist_ok=False)
        rows = []
        try:
            verified = VerifiedCohort(collector=args.collector, study=args.study, results=args.results,
                source_artifact=args.source_artifact, reference_artifact=args.reference_artifact)
            for n in range(len(verified.result['rows'])):
                rows.append(verified.archive_row(n, args.archive / f'row-{n:03}'))
        except (ValueError, KeyError, OSError) as error:
            (args.archive / 'results.json').write_text(json.dumps({'status': 'invalid', 'error': str(error), 'rows': rows}, indent=2) + '\n')
            raise
        (args.archive / 'results.json').write_text(json.dumps({'status': 'complete', 'rows': rows,
            'identity_basis': 'frozen collector verify + exact phase/case/arm row, copied receipt and matching correctness/reference; internal PID only'}, indent=2) + '\n')


if __name__ == '__main__':
    main()
