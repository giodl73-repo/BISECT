"""Retain a new, source-bound reconciliation/matrix evidence snapshot; never overwrite."""
import argparse
import hashlib
import json
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def local(name):
    path = (ROOT / name).resolve()
    if not path.is_relative_to(ROOT):
        raise ValueError('Evidence must remain inside the workspace: ' + str(name))
    return path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--coverage', action='append', required=True)
    parser.add_argument('--matrix', action='append', required=True)
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    target = local(args.output)
    if not target.is_relative_to(ROOT / 'target/fetch-source-reconciliation'):
        raise ValueError('Choose an owned target/fetch-source-reconciliation output')
    if target.exists():
        raise ValueError('Existing snapshots are immutable; choose a new output')
    names = set()

    def include(name, expected=None):
        path = local(name)
        if not path.is_file():
            raise ValueError('Missing evidence: ' + str(path))
        if expected and digest(path) != expected:
            raise ValueError('Evidence changed: ' + str(path))
        names.add(path)

    coverage_file = local('docs/specs/wasm-fetch-source-coverage.json')
    coverage = json.loads(coverage_file.read_text(encoding='utf-8'))
    for pin in coverage['authority']:
        include(pin['path'], pin['sha256'])
    profiles = set()
    for name in args.coverage:
        folder = local(name)
        report = json.loads((folder / 'evidence.json').read_text(encoding='utf-8'))
        assert report['coverage_sha256'] == digest(coverage_file)
        assert len(report['cases']) == 42
        assert all(row['native_plan_equal'] and row['native_handoff_equal'] for row in report['cases'])
        assert report['source_pins'] == coverage['authority']
        for key in ['wasm', 'native_planner', 'native_handoff']:
            include(report[key]['path'], report[key]['sha256'])
        profiles.add(report['wasm']['sha256'])
        for path in folder.rglob('*'):
            if path.is_file():
                include(path)
    matrix_profiles = set()
    for name in args.matrix:
        folder = local(name)
        report = json.loads((folder / 'evidence.json').read_text(encoding='utf-8'))
        assert report['status'] == 'passed-declared-native-adapter-tier'
        assert report['manifest_sha256'] == digest(local('docs/specs/wasm-real-data-matrix.json'))
        assert report['counts'] == dict(attempts=3, successful_executions=1, matching_native_failures=2,
                                        failures=0, not_run=0, browser_executions=0)
        include(folder / 'runtime/engine.wasm', report['wasm_sha256'])
        include(folder / 'runtime/native.exe', report['native_sha256'])
        matrix_profiles.add(report['wasm_sha256'])
        browsers = list(folder.glob('browser-*/evidence.json'))
        assert browsers, 'Missing actual packaged GUI evidence'
        for browser in browsers:
            result = json.loads(browser.read_text(encoding='utf-8'))
            assert result['wasm_sha256'] == report['wasm_sha256']
            assert result['manifest_sha256'] == report['manifest_sha256']
            assert len(result['rows']) == 3
            assert [row['status'] for row in result['rows']] == [
                'passed-gui-run-save-open', 'matching-native-failure', 'matching-native-failure']
            for pin in result['served_assets']:
                include(pin['path'], pin['sha256'])
        for path in folder.rglob('*'):
            if path.is_file():
                include(path)
    assert len(profiles) == 2 and matrix_profiles == profiles, 'Both actual engine identities required'
    for name in ['README.md', 'docs/specs/wasm-fetch-source-coverage.json',
                 'docs/specs/wasm-real-data-matrix.json', 'docs/specs/wasm-fetch-source-reconciliation.md',
                 'docs/specs/wasm-desktop-parity-plan.md', 'docs/specs/wasm-parity-ledger.md',
                 'signals/roles/check/wasm-fetch-source-reconciliation-roles-check-2026-10-23.md',
                 'scripts/web/test_fetch_source_coverage.mjs', 'scripts/web/declare_real_data_matrix.mjs',
                 'scripts/web/test_real_data_matrix.mjs', 'scripts/web/test_real_data_matrix_browser.mjs',
                 'scripts/web/fixtures/real-data-matrix-case.mjs',
                 'scripts/web/build_wasm.ps1', 'scripts/web/build_wasm_lab.ps1',
                 'scripts/web/build_workspace_preview.ps1',
                 'scripts/web/snapshot_fetch_source_reconciliation.py']:
        include(name)
    target.mkdir(parents=True)
    files = []
    for path in sorted(names):
        relative = path.relative_to(ROOT)
        destination = target / 'files' / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, destination)
        assert digest(destination) == digest(path)
        files.append(dict(path=relative.as_posix(), sha256=digest(destination),
                          byte_length=destination.stat().st_size))
    snapshot = dict(schema_version='bisect-fetch-reconciliation-snapshot-v1', files=files,
                    wasm_identities=sorted(profiles), scope='Source reconciliation plus first retained-graph '
                    'native-adapter/GUI tier. Raw integration inputs remain external immutable prerequisites. '
                    'Not a new release package, fresh derivation, live acquisition or full parity.', deployed=False)
    (target / 'snapshot.json').write_text(json.dumps(snapshot, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(dict(snapshot=str(target / 'snapshot.json'), files=len(files),
                         bytes=sum(row['byte_length'] for row in files))))


if __name__ == '__main__':
    main()
