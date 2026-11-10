"""Retain the bounded Doctor report checkpoint without rewriting old receipts."""
import hashlib
import json
import shutil
from pathlib import Path
from evidence_blob_store import SnapshotBlobs

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'target/doctor-command'
PACKAGE = ROOT / 'dist/wasm-doctor-command-final'
CASES = ['run-Ykmctg', 'run-dSCSOX', 'browser-GkosJ0', 'browser-yHhlzJ']


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
    for directory in ['target/policy-fallback/snapshot', 'target/policy-cli-contract/snapshot',
                      'target/config-cli-contract/snapshot']:
        for path in (ROOT / directory).iterdir():
            if path.is_file():
                known[path.name] = path
    files, accepted = set(), []
    for name in CASES:
        folder = OUT / name
        data = read(folder / 'evidence.json')
        browser = name.startswith('browser-')
        assert data['status'] == ('passed-doctor-command-workbench' if browser else 'passed-doctor-plan-command')
        assert len(data['outcomes'] if browser else data['records']) == 8
        for ref in data['sources']:
            path = ROOT / ref['path']
            assert digest(path) == ref['sha256'], ref['path']
            files.add(path)
        if browser:
            assert data['test_sha256'] == digest(ROOT / 'scripts/web/test_doctor_command_browser.mjs')
            assert data['explicit_runs'] == 8 and not data['actual_os_picker_grant']
            ref = data['reference']
            assert digest(ROOT / ref['path']) == ref['sha256']
        else:
            assert len(data['dispatch']) == 4 and data['unsupported_pointer_width']['native_exit_code'] == 0
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
    files.update(p for p in (OUT / 'before').rglob('*') if p.is_file())
    files.update(OUT.glob('*.log'))
    for name in ['README.md', 'docs/specs/wasm-doctor-command.md', 'docs/specs/wasm-desktop-parity-plan.md',
                 'docs/specs/wasm-capability-inventory.json', 'docs/specs/wasm-practitioner-actions.json',
                 'docs/specs/wasm-fetch-source-coverage.json',
                 'signals/roles/check/wasm-doctor-command-roles-check-2026-10-27.md',
                 'scripts/web/snapshot_doctor_command.py', 'scripts/web/test_doctor_command_browser.mjs',
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
    record = dict(status='passed-bounded-doctor-command-checkpoint', files=rows,
                  accepted_evidence=accepted, offline_version=manifest['version'], new_blob_bytes=new_bytes,
                  previous_archive=release, previous_archive_unchanged=True,
                  new_whole_builder_passed=False, full_parent_acceptance=False, deployed=False,
                  scope='Selected-plan all/label reports only. Actual CLI and worker evidence; '
                        'native location, pointer-width metadata and verifier/tutorial reconciliation remain open. '
                        'Historical runtime receipts retain their original identities.')
    destination.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'closure': destination.relative_to(ROOT).as_posix(), 'sha256': digest(destination),
                      'files': len(rows), 'verified_blob_identities': len(verified), 'new_blob_bytes': new_bytes}))


if __name__ == '__main__':
    main()
