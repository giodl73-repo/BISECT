"""Retain final-package Fetch archive evidence without replacing prior attempts."""
import argparse
import json
import shutil
from pathlib import Path
from snapshot_fetch_source_reconciliation import ROOT, local, digest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--package', required=True)
    parser.add_argument('--real', action='append', default=[])
    args = parser.parse_args()
    package = local(args.package)
    assert package.is_relative_to(ROOT / 'dist')
    target = ROOT / 'target/fetch-archive-acquisition/package-snapshots' / package.name
    if target.exists():
        raise ValueError('Keep existing snapshots; choose a new package identity')
    files = set()

    def include(name, expected=None):
        path = local(name)
        assert path.is_file(), str(path)
        if expected:
            assert digest(path) == expected, 'Changed evidence: ' + str(path)
        files.add(path)

    def include_ref(ref):
        include(ref['path'], ref['sha256'])

    def tree(folder):
        for path in folder.rglob('*'):
            if path.is_file():
                include(path)

    def archive_run(folder, engine, require_browser=False):
        folder = local(folder)
        evidence = json.loads((folder / 'evidence.json').read_text(encoding='utf-8'))
        assert evidence['wasm_sha256'] == engine
        for row in evidence['cases']:
            assert row['native_files_equal']
            source = local(row['source_project']).parent / 'source.zip'
            include(source, row['source_sha256'])
        include(folder / 'native-fetch-reference.exe', evidence['native_sha256'])
        browsers = list(folder.glob('browser-*/evidence.json'))
        if require_browser:
            assert browsers, 'Actual browser record required'
        for name in browsers:
            browser = json.loads(name.read_text(encoding='utf-8'))
            assert browser['wasm_sha256'] == engine
            for row in browser['rows']:
                assert all(row[key] for key in ['exact_native_package', 'inert_open',
                    'edits_retire_download', 'real_indexed_cache_offline',
                    'local_import_no_network', 'unavailable_404', 'cancel_without_source_install'])
            for pin in browser['served_assets']:
                include_ref(pin)
        tree(folder)

    record = ROOT / 'target/native-workspace-source/package-evidence.json'
    bundle = json.loads(record.read_text(encoding='utf-8'))
    assert local(bundle['package']) == package
    assert bundle['browser_status'] == 'passed' and bundle['deployed'] is False
    include(record)
    for pin in bundle['source_files'] + bundle['results']:
        include_ref(pin)
    for pin in bundle.get('explicit_c_batch_evidence', []):
        include_ref(pin)
    if bundle.get('optional_c_browser_evidence'):
        include_ref(bundle['optional_c_browser_evidence'])
    offline = json.loads((package / 'offline-manifest.json').read_text(encoding='utf-8'))
    assert offline['version'] == bundle['offline_version']
    include_ref(bundle['offline_manifest'])
    engine = digest(package / 'toolkit/bisect_wasm.wasm')
    include(package / 'toolkit/bisect_wasm.wasm', engine)
    archive = bundle['fetch_archive_evidence']
    include_ref(archive['node'])
    archive_run(archive['root'], engine, require_browser=True)
    assert len(json.loads(local(archive['node']['path']).read_text(encoding='utf-8'))['cases']) == 6
    for pin in archive['browser']:
        include_ref(pin)
        browser = json.loads(local(pin['path']).read_text(encoding='utf-8'))
        assert len(browser['rows']) == 2
        assert all(local(row['path']).is_relative_to(package) for row in browser['served_assets'])
    profiles = set()
    for profile in ['default', 'c-sidecar']:
        folder = ROOT / 'target/cffi-legacy-gates' / profile
        gate = json.loads((folder / 'build.json').read_text(encoding='utf-8-sig'))
        assert gate['status'] == 'passed'
        profiles.add(gate['wasm_sha256'])
        for pin in gate['source_files']:
            include_ref(pin)
        include(folder.parent / 'state-core-reference.exe', gate['native_oracle_sha256'])
        include(folder.parent / (profile + '.wasm'), gate['wasm_sha256'])
        archive_run(gate['fetch_archive_evidence']['root'], gate['wasm_sha256'], require_browser=profile == 'default')
        tree(folder)
    assert engine in profiles and len(profiles) == 2
    for name in args.real:
        folder = local(name)
        real = json.loads((folder / 'evidence.json').read_text(encoding='utf-8'))
        assert real['wasm_sha256'] in profiles
        archive_run(folder, real['wasm_sha256'], require_browser=True)
        include_ref(real['observation'])
        tree(local(real['observation']['path']).parent)
    coverage = ROOT / 'docs/specs/wasm-fetch-source-coverage.json'
    for pin in json.loads(coverage.read_text(encoding='utf-8'))['authority']:
        include_ref(pin)
    for name in ['README.md', 'docs/specs/wasm-fetch-archive-acquisition.md',
                 'docs/specs/wasm-fetch-source-reconciliation.md',
                 'docs/specs/wasm-desktop-parity-plan.md', 'docs/specs/wasm-parity-ledger.md',
                 'signals/roles/check/wasm-fetch-archive-acquisition-roles-check-2026-10-23.md',
                 'design/pitfalls/README.md', 'design/pitfalls/pitfalls-pipeline.md',
                 'scripts/web/test_fetch_archive_real.mjs', 'scripts/web/inspect_public_fetch_archives.mjs',
                 'scripts/web/snapshot_fetch_archive_acquisition.py',
                 'scripts/web/snapshot_fetch_source_reconciliation.py',
                 'target/fetch-archive-acquisition/dual-build.log',
                 'target/fetch-archive-acquisition/package-build.log',
                 'target/fetch-archive-acquisition/package-build-v2.log',
                 'target/fetch-archive-acquisition/project-regression-final.log']:
        include(name)
    target.mkdir(parents=True)
    records = []
    for name in sorted(files):
        relative = name.relative_to(ROOT)
        output = target / 'files' / relative
        output.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(name, output)
        assert digest(output) == digest(name)
        records.append(dict(path=relative.as_posix(), sha256=digest(output), byte_length=output.stat().st_size))
    (target / 'snapshot.json').write_text(json.dumps(dict(
        schema_version='bisect-fetch-archive-snapshot-v1', package=package.relative_to(ROOT).as_posix(),
        offline_version=offline['version'], wasm_identities=sorted(profiles), files=records,
        deployed=False, scope='Scoped archive acquisition/local import/cache/native-layout export. '
        'Both actual engines, final C-enabled package and explicit retained RI integration. '
        'Not full Fetch orchestration, zone joins, source authenticity or release acceptance.'), indent=2) + '\n', encoding='utf-8')
    print(json.dumps(dict(snapshot=str(target / 'snapshot.json'), files=len(records),
                         bytes=sum(row['byte_length'] for row in records))))


if __name__ == '__main__':
    main()
