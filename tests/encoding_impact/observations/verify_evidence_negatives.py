"""Protocol tests: relocated real evidence and synthetic collector rows, no compiler execution."""
import argparse
import copy
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import sys
import tempfile

from check_corrected_specificity import classify_capture, summarize
from observation_evidence import (VerifiedCohort, archive_tree, retain_receipt, validate_evidence)


def relocate(source, root, suffix):
    """Keep authentic pass bytes, rebinding only absolute invocation/index identities."""
    old = json.loads((source / 'index.json').read_text())
    child = root / f"invocation-{old['pid']}-{suffix}"
    shutil.copytree(source, child)
    index_path = child / 'index.json'
    old.update(root=str(root), directory=str(child), index=str(index_path))
    for key, entry in old['passes'].items():
        directory = child / f'pass-{key}'
        entry['directory'] = str(directory)
        entry['evidence'] = [str(directory / Path(p).name) for p in entry['evidence']]
    index_path.write_text(json.dumps(old))
    return {'directory': str(child), 'index': str(index_path)}, old


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('binding', 'rejected-binding', 'normal-receipt', 'binary', 'source-package', 'collector', 'out'):
        parser.add_argument(f'--{name}', required=True, type=Path)
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=False)
    binding = json.loads(args.binding.read_text())
    source = Path(binding['locator']['directory'])
    mode, pid = binding['mode'], binding['pid']
    normal = json.loads(args.normal_receipt.read_text())
    checks = []

    def check(name, operation, *, reject=False):
        try:
            operation()
        except (ValueError, KeyError, OSError, TypeError, IndexError) as error:
            checks.append({'case': name, 'passed': reject, 'rejected': True, 'reason': str(error)})
        except AssertionError as error:
            checks.append({'case': name, 'passed': False, 'reason': str(error)})
        else:
            checks.append({'case': name, 'passed': not reject, 'rejected': False})
        (args.out / 'results.partial.json').write_text(json.dumps(checks, indent=2) + '\n')

    with tempfile.TemporaryDirectory() as temporary:
        scratch = Path(temporary).resolve()
        counter = 0

        def tree_case(mutate=None):
            nonlocal counter
            counter += 1
            root = scratch / f'tree-{counter}'
            root.mkdir()
            locator, index = relocate(source, root, 'test')
            child = Path(locator['directory'])
            if mutate:
                mutate(child, index)
                (child / 'index.json').write_text(json.dumps(index))
            return validate_evidence(locator, root=root, mode=mode, pid=pid, status='complete')

        check('real-complete-evidence', tree_case)
        for name, mutate in [
            ('stale-pid', lambda c, d: d.update(pid=pid + 1)),
            ('swapped-mode', lambda c, d: d.update(mode='Injective')),
            ('partial-terminal', lambda c, d: d.update(status='in-progress')),
            ('missing-pass', lambda c, d: d['passes'].pop('02')),
            ('pre-native-stage', lambda c, d: d['passes']['01'].update(stage='audit-accepted')),
            ('wrong-root', lambda c, d: d.update(root=str(scratch))),
        ]:
            check(name, lambda m=mutate: tree_case(m), reject=True)
        for filename in ['initialization.egg', 'source-schedule.egg', 'audit.json']:
            def omit(child, index, filename=filename):
                path = child / 'pass-00' / filename
                path.unlink()
                index['passes']['00']['evidence'].remove(str(path))
            check(f'missing-{filename}-and-index-entry', lambda m=omit: tree_case(m), reject=True)
        for filename, field, value in [
            ('audit.json', 'accepted', False), ('audit.json', 'mode', 'Injective'),
            ('audit.json', 'stage', 'after-extraction'), ('committed-before-audit.json', 'stage', 'uncommitted'),
            ('committed-before-audit.json', 'native_extraction_entered', True),
            ('serialization-entry.json', 'stage', 'not-serialization'), ('observations.json', 'mode', 'Injective'),
        ]:
            def alter(child, index, filename=filename, field=field, value=value):
                path = child / 'pass-00' / filename
                data = json.loads(path.read_text())
                data[field] = value
                path.write_text(json.dumps(data))
            check(f'altered-{filename}-{field}', lambda m=alter: tree_case(m), reject=True)
        external = scratch / 'own-fake-external.txt'
        external.write_text('must never be archived')
        external_dir = scratch / 'own-fake-external-directory'
        external_dir.mkdir()
        (external_dir / 'file').write_text('must never be archived')
        for name, target in [('root-file-symlink', external), ('directory-symlink', external_dir)]:
            check(name, lambda target=target: tree_case(lambda c, d: (c / 'extra').symlink_to(target)), reject=True)
        check('unexpected-root-file', lambda: tree_case(lambda c, d: (c / 'extra').write_text('unindexed')), reject=True)
        archive_source = scratch / 'archive-root'
        archive_source.mkdir()
        loc, _ = relocate(source, archive_source, 'destination')
        child = Path(loc['directory'])
        alias = scratch / 'alias'
        alias.symlink_to(child, target_is_directory=True)
        check('symlink-aliased-destination-inside-source', lambda: archive_tree(child, alias / 'copy'), reject=True)
        check('relative-destination-inside-source', lambda: archive_tree(child, Path(os.path.relpath(child / 'copy'))), reject=True)
        receipt = scratch / 'shared.json'
        receipt.write_text(json.dumps(normal))
        _, retained = retain_receipt(receipt, scratch / 'retained.json')
        receipt.write_text('{}')
        check('retained-copy-after-original-overwrite', lambda: retain_receipt(receipt, scratch / 'retained.json', expected_sha256=retained['sha256'], expected_data=normal))
        (scratch / 'retained.json').write_text('{}')
        check('altered-retained-copy', lambda: retain_receipt(receipt, scratch / 'retained.json', expected_sha256=retained['sha256']), reject=True)

        spec = importlib.util.spec_from_file_location('protocol_fixture_collector', args.collector)
        collector = importlib.util.module_from_spec(spec)
        sys.modules[spec.name] = collector
        spec.loader.exec_module(collector)

        def cohort_case(change=None, action=None):
            """Synthetic row metadata uses real frozen verify, never executes collect/optimizer."""
            nonlocal counter
            counter += 1
            base = scratch / f'cohort-{counter}'
            base.mkdir()
            root = base / 'evidence'
            root.mkdir()
            reference = base / 'reference.json'
            reference.write_text(json.dumps({'cases': {'x1': {'input_sha256': hashlib.sha256(args.normal_receipt.read_bytes()).hexdigest(), 'expected_output': 'false\n'}}}))
            package = base / 'package.json'
            shutil.copyfile(args.source_package, package)
            study_path, results_path = base / 'study.json', base / 'results.json'
            study_data = {'schema_version': 1, 'study_id': 'synthetic-protocol-not-executed', 'warmups': 1, 'observations': 1,
                'cases': [{'id': 'x1', 'input': str(args.normal_receipt.resolve()), 'origin': 'synthetic protocol input identity only'}],
                'arms': [{'id': 'Manual', 'binary': str(args.binary.resolve()), 'cwd': str(base), 'engine': 'synthetic',
                    'configuration': 'not-a-compiler-run', 'env': {}, 'args': ['--run-mode', 'optimize', '--observations', mode, '--observation-evidence-dir', str(root)]}],
                'artifacts': [{'id': 'source-manifest', 'path': json.loads(package.read_text())['source_manifest'], 'origin': 'frozen build source'}, {'id': 'source', 'path': str(package), 'origin': 'frozen endpoint package'}, {'id': 'reference', 'path': str(reference), 'origin': 'synthetic reference'}]}
            study_path.write_text(json.dumps(study_data))
            study = collector.load_study(study_path)
            rows = []
            for phase, repeat, case, arm in collector.schedule(study):
                locator, _ = relocate(source, root, phase)
                data = {**normal, 'observation_evidence': locator,
                    'original_interpreted': 'false\n' if phase == 'correctness' else None,
                    'result_interpreted': 'false\n' if phase == 'correctness' else None}
                path = base / f'{phase}.json'
                path.write_text(json.dumps(data))
                rows.append({'phase': phase, 'repetition': repeat, 'case': case['id'], 'arm': arm['id'],
                    'command': collector.invocation(arm, case, phase, path), 'cwd': str(base), 'engine': arm['engine'], 'configuration': arm['configuration'],
                    'env_overrides': {'RUST_LOG': 'error'}, 'environment_sha256': {'RUST_LOG': hashlib.sha256(b'error').hexdigest()}, 'inherited_environment': {},
                    'receipt_path': str(path), 'receipt_sha256': hashlib.sha256(path.read_bytes()).hexdigest(), 'receipt': data,
                    'rss_scope': collector.RSS_SCOPE, 'wall_sec': 1.0, 'max_rss_bytes': 1, 'status': 'success', 'exit_code': 0, 'signal': None, 'error': None})
            if change:
                change(study, rows, package, reference)
            study_path.write_text(json.dumps(study))
            result = {'schema_version': 1, 'study_path': str(study_path), 'study_sha256': collector.sha256(study_path),
                'study': study, 'provenance': collector.provenance(study), 'status': 'complete', 'rows': rows, 'rss_scope': collector.RSS_SCOPE}
            results_path.write_text(json.dumps(result))
            verified = VerifiedCohort(collector=args.collector, study=study_path, results=results_path, source_artifact='source', reference_artifact='reference')
            if action:
                action(verified, base)
            else:
                for n in range(3):
                    verified.archive_row(n, base / f'archive-{n}')

        check('real-verifier-correctness-plus-warmup-and-measurement-null', cohort_case)

        def change_receipt(rows, n, field, value):
            rows[n]['receipt'][field] = value
            path = Path(rows[n]['receipt_path'])
            path.write_text(json.dumps(rows[n]['receipt']))
            rows[n]['receipt_sha256'] = hashlib.sha256(path.read_bytes()).hexdigest()

        check('timed-nonnull-output', lambda: cohort_case(lambda s, r, p, e: change_receipt(r, 1, 'result_interpreted', 'false\n')), reject=True)
        check('correctness-null-output', lambda: cohort_case(lambda s, r, p, e: change_receipt(r, 0, 'original_interpreted', None)), reject=True)
        check('missing-correctness', lambda: cohort_case(lambda s, r, p, e: r.pop(0)), reject=True)
        check('wrong-phase', lambda: cohort_case(lambda s, r, p, e: r[1].update(phase='correctness')), reject=True)
        check('wrong-correctness-arm', lambda: cohort_case(lambda s, r, p, e: r[0].update(arm='Other')), reject=True)
        check('wrong-correctness-case', lambda: cohort_case(lambda s, r, p, e: r[0].update(case='Other')), reject=True)
        def wrong_reference(study, rows, package, reference):
            data = json.loads(reference.read_text())
            data['cases']['x1']['expected_output'] = 'true\n'
            reference.write_text(json.dumps(data))
        check('correctness-reference-output-mismatch', lambda: cohort_case(wrong_reference), reject=True)
        def wrong_source(study, rows, package, reference):
            data = json.loads(package.read_text())
            data['binary_sha256'] = '0' * 64
            package.write_text(json.dumps(data))
        check('source-package-binary-mismatch', lambda: cohort_case(wrong_source), reject=True)
        def wrong_root(study, rows, package, reference):
            for row in rows:
                path = Path(row['receipt']['observation_evidence']['index'])
                data = json.loads(path.read_text())
                data['root'] = str(scratch)
                path.write_text(json.dumps(data))
        check('actual-child-wrong-root', lambda: cohort_case(wrong_root), reject=True)
        def duplicate(study, rows, package, reference):
            change_receipt(rows, 1, 'observation_evidence', rows[0]['receipt']['observation_evidence'])
        check('duplicate-child-in-verified-cohort', lambda: cohort_case(duplicate), reject=True)
        check('changed-cohort-after-verify', lambda: cohort_case(action=lambda v, b: (v.results_path.write_text('{}'), v.archive_row(1, b / 'archive'))), reject=True)

        check('negative-row-index', lambda: cohort_case(action=lambda v, b: v.archive_row(-1, b / 'archive')), reject=True)
        check('unknown-row-index', lambda: cohort_case(action=lambda v, b: v.archive_row(3, b / 'archive')), reject=True)
        def duplicate_row(verified, base):
            verified.archive_row(0, base / 'archive-a')
            verified.archive_row(0, base / 'archive-b')
        check('duplicate-row-selection', lambda: cohort_case(action=duplicate_row), reject=True)
        def wrong_input_reference(study, rows, package, reference):
            data = json.loads(reference.read_text())
            data['cases']['x1']['input_sha256'] = '0' * 64
            reference.write_text(json.dumps(data))
        check('reference-input-hash-mismatch', lambda: cohort_case(wrong_input_reference), reject=True)
        def wrong_manifest(study, rows, package, reference):
            data = json.loads(package.read_text())
            data['source_manifest_sha256'] = '0' * 64
            package.write_text(json.dumps(data))
        check('source-manifest-hash-mismatch', lambda: cohort_case(wrong_manifest), reject=True)
        def wrong_artifact(study, rows, package, reference):
            study['artifacts'][1]['id'] = 'unselected-source'
        check('source-not-associated-study-artifact', lambda: cohort_case(wrong_artifact), reject=True)
        check('manifest-not-individually-verified', lambda: cohort_case(lambda s, r, p, e: s['artifacts'].pop(0)), reject=True)
        def partial_index(study, rows, package, reference):
            path = Path(rows[1]['receipt']['observation_evidence']['index'])
            data = json.loads(path.read_text())
            data.update(status='in-progress', stage='source-schedule')
            path.write_text(json.dumps(data))
        check('timed-child-partial-stage', lambda: cohort_case(partial_index), reject=True)
        def swapped_index(study, rows, package, reference):
            path = Path(rows[1]['receipt']['observation_evidence']['index'])
            data = json.loads(path.read_text())
            data['mode'] = 'Injective'
            path.write_text(json.dumps(data))
        check('timed-child-swapped-mode', lambda: cohort_case(swapped_index), reject=True)
        def nested_destination(verified, base):
            child = Path(verified.result['rows'][0]['receipt']['observation_evidence']['directory'])
            output = child / 'must-not-be-created'
            try:
                verified.archive_row(0, output)
            finally:
                if output.exists():
                    raise AssertionError('invalid destination wrote into source child')
        check('cohort-destination-inside-child-before-any-copy', lambda: cohort_case(action=nested_destination), reject=True)
        rejected_source = Path(json.loads(args.rejected_binding.read_text())['locator']['directory'])
        for variant in ['exact-witness', 'different-witness', 'empty-witness']:
            directory = scratch / f'rejection-{variant}'
            directory.mkdir()
            root = directory / 'evidence-root'
            root.mkdir()
            locator, rejected_index = relocate(rejected_source, root, variant)
            if variant != 'exact-witness':
                path = Path(locator['directory']) / 'pass-00/committed-before-audit.json'
                data = json.loads(path.read_text())
                data['conflicts'] = [] if variant == 'empty-witness' else ['(ObservationConflict 9 8 7 true false)']
                path.write_text(json.dumps(data))
            argv = ['binary']
            command = {'returncode': 101, 'timed_out': False, 'argv': argv, 'cwd': str(scratch), 'pid': rejected_index['pid']}
            (directory / 'command.json').write_text(json.dumps(command))
            (directory / 'command.stderr').write_text('observation-evidence: ' + json.dumps(locator) + '\nRunning pass 0...\n')
            def reject_classification(directory=directory, variant=variant, argv=argv):
                row = classify_capture(directory, 'manual', 101, expected_argv=argv, expected_cwd=scratch)
                if row['original_red_component'] != (variant == 'exact-witness') or row['passed']:
                    raise AssertionError('wrong exact pre-Tiger witness classification')
            check(f'{variant}-rejection-classification', reject_classification)
        # Simulated diagnostic captures test classification, not compiler semantics.
        for variant in ['complete-wrong-answer', 'missing-run-data', 'malformed-run-data', 'incomplete-pipeline']:
            directory = scratch / variant
            directory.mkdir()
            path = directory / 'run-data.json'
            argv = ['binary', '--run-data-out', str(path)]
            command = {'returncode': 0, 'timed_out': False, 'argv': argv, 'cwd': str(scratch)}
            (directory / 'command.json').write_text(json.dumps(command))
            (directory / 'command.stderr').write_text(''.join(f'Running pass {n}...\n' for n in range(1 if variant == 'incomplete-pipeline' else 3)))
            (directory / 'egraphs').mkdir()
            for n in range(1 if variant == 'incomplete-pipeline' else 3):
                (directory / f'egraphs/tiger_egraph_{n}.json').write_text('{}')
            if variant != 'missing-run-data':
                path.write_text('{' if variant == 'malformed-run-data' else json.dumps({'original_interpreted': 'true\n', 'result_interpreted': 'false\n'}))
            def check_classification(directory=directory, variant=variant, argv=argv):
                row = classify_capture(directory, 'off', 0, expected_argv=argv, expected_cwd=scratch)
                if row['original_red_component'] != (variant == 'complete-wrong-answer') or row['passed']:
                    raise AssertionError('wrong complete/invalid pipeline classification')
            check(f'{variant}-classification', check_classification)
        # All four mode classifiers are exercised separately from compiler executions.
        valid_rows = [dict(mode=m, passed=False, original_red_component=True, classification='complete-known-wrong-answer' if m in ('off', 'common-seed') else 'exact-pre-tiger-witness') for m in ('off', 'common-seed', 'manual', 'injective')]
        def require_signature(rows, expected):
            if summarize(rows)['valid_original_red'] is not expected:
                raise AssertionError('wrong original-red admission')
        check('strict-four-mode-signature', lambda: require_signature(valid_rows, True))
        check('missing-mode-not-valid-red', lambda: require_signature(valid_rows[:3], False))
        for malformed in ['missing-command', 'malformed-command', 'timeout', 'wrong-capture-exit']:
            directory = scratch / malformed
            directory.mkdir()
            command = {'returncode': 0, 'timed_out': False, 'argv': ['binary'], 'cwd': str(scratch)}
            if malformed == 'timeout':
                command['timed_out'] = True
            if malformed != 'missing-command':
                (directory / 'command.json').write_text('{' if malformed == 'malformed-command' else json.dumps(command))
            def classify_invalid(directory=directory, malformed=malformed):
                row = classify_capture(directory, 'off', 101 if malformed == 'wrong-capture-exit' else 0, expected_argv=['binary'], expected_cwd=scratch)
                if row['original_red_component'] or row['passed']:
                    raise AssertionError('invalid capture counted as red/green')
                altered = copy.deepcopy(valid_rows)
                altered[0] = row
                require_signature(altered, False)
            check(f'{malformed}-not-valid-red', classify_invalid)
    summary = {'checks': checks, 'passed': sum(r['passed'] for r in checks), 'failed': sum(not r['passed'] for r in checks),
        'fixture_kind': 'real captured pass bytes relocated; synthetic protocol rows verified by real frozen collector, no new compiler execution',
        'source_sha256': {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in Path(__file__).parent.glob('*.py')}}
    (args.out / 'results.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps({k: summary[k] for k in ('passed', 'failed')}))
    return 0 if summary['failed'] == 0 else 1


if __name__ == '__main__':
    raise SystemExit(main())
