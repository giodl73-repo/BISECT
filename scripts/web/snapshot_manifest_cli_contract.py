"""Retain the bounded Doctor manifest checkpoint without rewriting old receipts."""
import hashlib
import json
import shutil
from pathlib import Path
from evidence_blob_store import SnapshotBlobs

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'target/manifest-cli-contract'
PACKAGE = ROOT / 'dist/wasm-manifest-cli-complete'
CASES = ['run-drCjqu', 'run-SrNrMH', 'browser-gfVB9u', 'browser-EBJeJD']


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        while chunk := stream.read(1024 * 1024):
            h.update(chunk)
    return h.hexdigest()


def read(path):
    return json.loads(path.read_text(encoding='utf-8-sig'))


def main():
    destination = OUT / 'closure-final.json'
    assert not destination.exists(), 'Preserve previous checkpoints.'
    base = SnapshotBlobs('target/config-publication/full-build/snapshot-scope/snapshot.json',
                         '519c6eeb7c8d121057c0bb0534c90b15fa677aa20736b9375fad36e0d93f478d')
    known = {}
    for directory in ['target/manifest-cli-contract/snapshot', 'target/doctor-command/snapshot', 'target/policy-fallback/snapshot', 'target/policy-cli-contract/snapshot',
                      'target/config-cli-contract/snapshot']:
        for path in (ROOT / directory).iterdir():
            if path.is_file():
                known[path.name] = path
    files, accepted = set(), []
    for name in CASES:
        folder = OUT / name
        data = read(folder / 'evidence.json')
        browser = name.startswith('browser-')
        assert data['status'] == ('passed-manifest-cli-workbench' if browser else 'passed-actual-manifest-cli-contract')
        assert len(data['outcomes'] if browser else data['records']) == 20
        for ref in data['sources']:
            path = ROOT / ref['path']
            assert digest(path) == ref['sha256'], ref['path']
            files.add(path)
        if browser:
            assert data['test_sha256'] == digest(ROOT / 'scripts/web/test_manifest_cli_browser.mjs')
            assert data['explicit_runs'] == 20 and not data['actual_os_picker_grant']
            if name == 'browser-EBJeJD':
                assert data['package_identity']['offline_manifest_sha256'] == digest(PACKAGE / 'offline-manifest.json')
                assert data['package_identity']['workbench_aliases_match'] and data['package_identity']['laboratory_landing_preserved']
            ref = data['reference']
            assert digest(ROOT / ref['path']) == ref['sha256']
        else:
            assert len(data['guards']) == 3 and data['historical_bom_loss'] == {'native_exit': 2, 'old_decoder_exit': 0, 'corrected_exit': 2}
            for key in ['native', 'wasm']:
                ref = data[key]
                path = ROOT / ref['path']
                assert digest(path) == ref['sha256']
                files.add(path)
        files.update(p for p in folder.rglob('*') if p.is_file())
        accepted.append((folder / 'evidence.json').relative_to(ROOT).as_posix())
    manifest = read(PACKAGE / 'offline-manifest.json')
    assert len(manifest['assets']) == 737
    for name, ref in manifest['assets'].items():
        path = PACKAGE / name
        assert path.stat().st_size == ref['bytes'] and digest(path) == ref['sha256']
        files.add(path)
    files.add(PACKAGE / 'offline-manifest.json')
    files.add(OUT / 'closure.json')
    files.update(p for p in (OUT / 'before').rglob('*') if p.is_file())
    files.update(OUT.glob('*.log'))
    files.update(ROOT.glob('target/manifest-*.log'))
    for profile in ['default', 'c']:
        data = read(OUT / ('fetch-' + profile + '.log'))
        evidence = Path(data['evidence'])
        assert data['cases'] == 42 and not data['acquisition_performed']
        files.update(p for p in evidence.parent.rglob('*') if p.is_file())
    # Retain the UI prototype's central-predicate failure as historical bytes.
    for name in ['workbench.js', 'offline-manifest.json']:
        files.add(ROOT / 'dist/wasm-manifest-cli-final' / name)

    for name in ['README.md', 'docs/specs/wasm-manifest-cli-contract.md', 'docs/specs/wasm-desktop-parity-plan.md',
                 'docs/specs/wasm-capability-inventory.json', 'docs/specs/wasm-practitioner-actions.json',
                 'docs/specs/wasm-fetch-source-coverage.json',
                 'signals/roles/check/wasm-manifest-cli-contract-roles-check-2026-10-27.md',
                 'scripts/web/snapshot_manifest_cli_contract.py', 'scripts/web/test_manifest_cli_browser.mjs',
                 'scripts/web/check_cffi_legacy_gates.ps1', 'scripts/web/build_cffi_preview.ps1']:
        files.add(ROOT / name)
    release = read(ROOT / 'target/config-publication/full-build/release.json')['archive']
    assert digest(ROOT / release['path']) == release['sha256']
    blobs = OUT / 'snapshot'
    blobs.mkdir(exist_ok=True)
    rows, verified, new_bytes = [], {}, 0
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
                    assert shutil.disk_usage(ROOT).free > size + 256 * 1024**2
                    shutil.copyfile(path, blob)
                    new_bytes += size
            assert blob.stat().st_size == size and digest(blob) == identity
            verified[identity] = blob
        rows.append({'source': {'path': path.relative_to(ROOT).as_posix(), 'sha256': identity, 'byte_length': size},
                     'snapshot': {'path': blob.relative_to(ROOT).as_posix(), 'sha256': identity, 'byte_length': size}})
    record = dict(status='passed-bounded-manifest-cli-checkpoint', files=rows,
                  accepted_evidence=accepted, offline_version=manifest['version'], new_blob_bytes=new_bytes,
                  superseded_prototype={'path': 'target/manifest-cli-contract/closure.json', 'sha256': digest(OUT / 'closure.json'), 'accepted_final_package': False},
                  previous_archive=release, previous_archive_unchanged=True,
                  new_whole_builder_passed=False, full_parent_acceptance=False, deployed=False,
                  scope='Selected nonempty UTF-8 manifest verification and console only. Actual CLI and worker evidence; '
                        'native location, pointer-width metadata, tutorial and empty manifest execution remain open. '
                        'Historical runtime receipts retain their original identities.')
    destination.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'closure': destination.relative_to(ROOT).as_posix(), 'sha256': digest(destination),
                      'files': len(rows), 'verified_blob_identities': len(verified), 'new_blob_bytes': new_bytes}))


if __name__ == '__main__':
    main()
