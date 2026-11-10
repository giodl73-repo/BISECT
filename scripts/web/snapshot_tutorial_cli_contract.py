"""Retain tutorial/empty-JSON evidence; preserve historical execution identities."""
import hashlib
import json
import os
import shutil
from pathlib import Path
from evidence_blob_store import SnapshotBlobs

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'target/tutorial-cli-contract'
PACKAGE = ROOT / 'dist/wasm-tutorial-cli-verified'
CASES = [
    ('target/manifest-cli-contract/run-z1OUO8', 21, 'manifest', False),
    ('target/manifest-cli-contract/run-GkcwRl', 21, 'manifest', False),
    ('target/manifest-cli-contract/browser-NR4eQn', 21, 'manifest', True),
    ('target/manifest-cli-contract/browser-tcctP9', 21, 'manifest', True),
    ('target/tutorial-cli-contract/run-xbnX4o', 19, 'tutorial', False),
    ('target/tutorial-cli-contract/run-Mnydu0', 19, 'tutorial', False),
    ('target/tutorial-cli-contract/browser-7gVHbN', 19, 'tutorial', True),
    ('target/tutorial-cli-contract/browser-sTNBQG', 19, 'tutorial', True),
]


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        while chunk := stream.read(1024 * 1024):
            h.update(chunk)
    return h.hexdigest()


def read(path):
    return json.loads(path.read_text(encoding='utf-8-sig'))


def main():
    destination = OUT / 'closure.json'
    assert not destination.exists(), 'Preserve previous checkpoints.'
    base = SnapshotBlobs('target/config-publication/full-build/snapshot-scope/snapshot.json',
                         '519c6eeb7c8d121057c0bb0534c90b15fa677aa20736b9375fad36e0d93f478d')
    known = {}
    for directory in ['target/manifest-cli-contract/snapshot', 'target/doctor-command/snapshot',
                      'target/policy-fallback/snapshot', 'target/policy-cli-contract/snapshot',
                      'target/config-cli-contract/snapshot']:
        for path in (ROOT / directory).iterdir():
            if path.is_file():
                known[path.name] = path
    files, accepted = set(), []
    for name, count, kind, browser in CASES:
        folder = ROOT / name
        data = read(folder / 'evidence.json')
        assert data['status'] == ('passed-' + kind + '-cli-workbench' if browser else
                                  'passed-actual-' + kind + '-cli-contract')
        assert len(data['outcomes'] if browser else data['records']) == count
        for ref in data['sources']:
            path = ROOT / ref['path']
            assert digest(path) == ref['sha256'], ref['path']
            files.add(path)
        if browser:
            assert data['test_sha256'] == digest(ROOT / ('scripts/web/test_' + kind + '_cli_browser.mjs'))
            assert data['explicit_runs'] == count and not data['actual_os_picker_grant']
            if data['package_identity']:
                assert data['package_identity']['offline_manifest_sha256'] == digest(PACKAGE / 'offline-manifest.json')
                assert data['package_identity']['workbench_aliases_match'] and data['package_identity']['laboratory_landing_preserved']
            ref = data['reference']
            assert digest(ROOT / ref['path']) == ref['sha256']
        else:
            assert len(data['guards']) == 2
            assert data['historical_bom_loss'] == {'native_exit': 2, 'old_decoder_exit': 0, 'corrected_exit': 2}
            if kind == 'tutorial':
                for row in data['records']:
                    supplied = {r['path'] for r in row['selected_files']}
                    expected = list(dict.fromkeys(r['path'] for r in row['result'].get('rows', [])))
                    assert row['hashed_paths'] == [p for p in expected if p in supplied]
            for key in ['native', 'wasm']:
                ref = data[key]
                path = ROOT / ref['path']
                assert digest(path) == ref['sha256']
                files.add(path)
        files.update(p for p in folder.rglob('*') if p.is_file())
        accepted.append((folder / 'evidence.json').relative_to(ROOT).as_posix())
    assert read(OUT / 'build/status.json')['status'] == 'passed'
    for ref in read(OUT / 'build/source-inputs.json'):
        path = ROOT / ref['path']
        assert digest(path) == ref['sha256'], ref['path']
        files.add(path)
    for folder in ['build', 'before', 'c-engine']:
        files.update(p for p in (OUT / folder).rglob('*') if p.is_file())
    engine = read(OUT / 'c-engine/engine-evidence.json')
    assert engine['native_matches'] == 52 and engine['expected_native_failures'] == 4
    assert engine['test_source_sha256'] == digest(ROOT / 'scripts/web/test_cffi_engine.mjs')
    files.add(ROOT / 'scripts/web/test_cffi_engine.mjs')
    assert engine['loaded_artifacts']['engine']['sha256'] == digest(PACKAGE / 'toolkit/bisect_wasm.wasm')
    for name in ['run-S1SwA1', 'run-87Ywhg']:
        folder = ROOT / 'target/policy-fallback' / name
        data = read(folder / 'evidence.json')
        assert len(data['records']) == 15 and data['backward_strict_cli_cases'] == 68
        files.update(p for p in folder.rglob('*') if p.is_file())
        ref = data['strict_cli_reference']
        assert digest(ROOT / ref['path']) == ref['sha256']
        files.add(ROOT / ref['path'])
        accepted.append((folder / 'evidence.json').relative_to(ROOT).as_posix())
    for profile in ['default', 'c']:
        data = read(OUT / ('fetch-' + profile + '.log'))
        assert data['cases'] == 42 and not data['acquisition_performed']
        files.update(p for p in Path(data['evidence']).parent.rglob('*') if p.is_file())
    manifest = read(PACKAGE / 'offline-manifest.json')
    assert len(manifest['assets']) == 737
    for name, ref in manifest['assets'].items():
        path = PACKAGE / name
        assert path.stat().st_size == ref['bytes'] and digest(path) == ref['sha256']
        files.add(path)
    files.add(PACKAGE / 'offline-manifest.json')
    files.add(OUT / 'staging-prototype.json')
    files.add(OUT / 'debug-symbol-cache-cleanup.json')
    files.add(OUT / 'snapshot-attempt-1.py')
    files.update(OUT.glob('*.log'))
    files.update(ROOT.glob('target/tutorial-cli-contract-*.log'))
    for name in ['README.md', 'docs/specs/wasm-tutorial-cli-contract.md', 'docs/specs/wasm-desktop-parity-plan.md',
                 'docs/specs/wasm-capability-inventory.json', 'docs/specs/wasm-engine-option-inventory.json',
                 'docs/specs/wasm-practitioner-actions.json', 'docs/specs/wasm-fetch-source-coverage.json',
                 'signals/roles/check/wasm-tutorial-cli-contract-roles-check-2026-10-27.md',
                 'scripts/web/snapshot_tutorial_cli_contract.py', 'scripts/web/test_tutorial_cli_browser.mjs',
                 'scripts/web/test_manifest_cli_browser.mjs', 'scripts/web/test_manifest_wasm.mjs',
                 'scripts/web/check_cffi_legacy_gates.ps1', 'scripts/web/build_cffi_preview.ps1']:
        files.add(ROOT / name)
    release = read(ROOT / 'target/config-publication/full-build/release.json')['archive']
    assert digest(ROOT / release['path']) == release['sha256']
    blobs = OUT / 'snapshot'
    blobs.mkdir(exist_ok=True)
    rows, verified, copied_bytes, linked_bytes = [], {}, 0, 0
    for path in sorted(files):
        identity, size = digest(path), path.stat().st_size
        blob = verified.get(identity)
        if blob is None:
            if identity in base.rows:
                blob = base.blob(identity, size)
            elif identity in known and known[identity].stat().st_size == size and digest(known[identity]) == identity:
                blob = known[identity]
            else:
                blob = blobs / identity
                if not blob.exists():
                    # Only sealed package/owned build binaries are linked; working sources are copied.
                    if path.suffix == '.wasm' and (path.is_relative_to(PACKAGE) or path.is_relative_to(OUT / 'build')):
                        os.link(path, blob)
                        linked_bytes += size
                    else:
                        assert shutil.disk_usage(ROOT).free > size + 256 * 1024**2
                        shutil.copyfile(path, blob)
                        copied_bytes += size
            assert blob.stat().st_size == size and digest(blob) == identity
            verified[identity] = blob
        rows.append({'source': {'path': path.relative_to(ROOT).as_posix(), 'sha256': identity, 'byte_length': size},
                     'snapshot': {'path': blob.relative_to(ROOT).as_posix(), 'sha256': identity, 'byte_length': size}})
    record = dict(status='passed-bounded-tutorial-empty-json-checkpoint', files=rows,
                  accepted_evidence=accepted, offline_version=manifest['version'],
                  new_copied_blob_bytes=copied_bytes, hardlinked_artifact_logical_bytes=linked_bytes,
                  immutable_artifact_rule='Owned build and sealed package WASM binaries must never be overwritten in place; use a new build/output directory.',
                  previous_archive=release, previous_archive_unchanged=True,
                  new_whole_builder_passed=False, full_parent_acceptance=False, deployed=False,
                  scope='Selected tutorial and empty manifest/checksum workflows only; native location, wide metadata, '
                        'wider host/capacity, clean-checkout and formal acceptance remain open. '
                        'Failed wrapper/shim/staging attempts are diagnostics, not accepted final packages.')
    destination.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'closure': destination.relative_to(ROOT).as_posix(), 'sha256': digest(destination),
                      'files': len(rows), 'verified_blob_identities': len(verified),
                      'new_copied_blob_bytes': copied_bytes, 'hardlinked_artifact_logical_bytes': linked_bytes}))


if __name__ == '__main__':
    main()
