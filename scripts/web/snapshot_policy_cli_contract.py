"""Freeze Policy CLI/reader evidence; preserve the previous complete package."""
import hashlib
import json
import shutil
from pathlib import Path
from evidence_blob_store import SnapshotBlobs

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'target/policy-cli-contract'
PACKAGE = ROOT / 'dist/wasm-policy-cli-preview'
CASES = ['run-wIXI3P', 'run-jSM2b5', 'browser-oyQvlw', 'browser-nkMFXq']


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        while chunk := stream.read(1024 * 1024):
            h.update(chunk)
    return h.hexdigest()


def main():
    destination = OUT / 'closure.json'
    if destination.exists():
        raise SystemExit('Preserve existing checkpoint; choose a new output.')
    base = SnapshotBlobs('target/config-publication/full-build/snapshot-scope/snapshot.json',
                         '519c6eeb7c8d121057c0bb0534c90b15fa677aa20736b9375fad36e0d93f478d')
    files, evidence = set(), []
    for name in CASES:
        folder = OUT / name
        record = folder / 'evidence.json'
        data = json.loads(record.read_text(encoding='utf-8'))
        browser = name.startswith('browser-')
        assert data['status'] == ('passed-actual-policy-CLI-browser' if browser else 'passed-actual-policy-CLI-contract')
        assert len(data['outcomes'] if browser else data['records']) == (10 if browser else 68)
        for ref in data['sources']:
            source = ROOT / ref['path']
            assert digest(source) == ref['sha256'], ref['path']
            files.add(source)
        if browser:
            assert data['test_sha256'] == digest(ROOT / 'scripts/web/test_policy_cli_browser.mjs')
            ref = data['reference']
            assert digest(ROOT / ref['path']) == ref['sha256']
        else:
            assert len(data['fallbacks']) == 6
            for key in ['native', 'wasm']:
                ref = data[key]
                assert digest(ROOT / ref['path']) == ref['sha256']
                files.add(ROOT / ref['path'])
        files.update(p for p in folder.rglob('*') if p.is_file())
        evidence.append(record.relative_to(ROOT).as_posix())
    manifest = json.loads((PACKAGE / 'offline-manifest.json').read_text(encoding='utf-8'))
    assert len(manifest['assets']) == 729
    for name, ref in manifest['assets'].items():
        path = PACKAGE / name
        assert path.stat().st_size == ref['bytes'] and digest(path) == ref['sha256']
        files.add(path)
    files.add(PACKAGE / 'offline-manifest.json')
    files.add(ROOT / 'dist/wasm-parity-config-final/toolkit/policy-files.js')
    for folder in ['target/policy-cli-probe', 'target/policy-cli-contract/before']:
        files.update(p for p in (ROOT / folder).rglob('*') if p.is_file())
    for name in ['README.md', 'docs/specs/wasm-policy-cli-contract.md',
                 'docs/specs/wasm-desktop-parity-plan.md', 'docs/specs/wasm-capability-inventory.json',
                 'docs/specs/wasm-engine-option-inventory.json', 'docs/specs/wasm-practitioner-actions.json',
                 'docs/specs/wasm-fetch-source-coverage.json', 'design/pitfalls/pitfalls-pipeline.md',
                 'signals/roles/check/wasm-policy-cli-contract-roles-check-2026-10-27.md',
                 'scripts/web/snapshot_policy_cli_contract.py', 'scripts/web/test_policy_cli_contract.mjs',
                 'scripts/web/test_policy_cli_browser.mjs', 'scripts/web/check_cffi_legacy_gates.ps1',
                 'scripts/web/build_cffi_preview.ps1']:
        files.add(ROOT / name)
    blobs = OUT / 'snapshot'
    blobs.mkdir(exist_ok=True)
    rows, verified, new_bytes = [], {}, 0
    for path in sorted(files):
        identity, size = digest(path), path.stat().st_size
        blob = verified.get(identity)
        if blob is None:
            if identity in base.rows:
                blob = base.blob(identity, size)
            else:
                blob = blobs / identity
                if blob.exists():
                    assert blob.stat().st_size == size and digest(blob) == identity
                else:
                    shutil.copyfile(path, blob)
                    new_bytes += size
            assert blob.stat().st_size == size and digest(blob) == identity
            verified[identity] = blob
        rows.append({'source': {'path': path.relative_to(ROOT).as_posix(), 'sha256': identity,
                                'byte_length': size},
                     'snapshot': {'path': blob.relative_to(ROOT).as_posix(), 'sha256': identity,
                                  'byte_length': size}})
    record = dict(status='passed-bounded-policy-CLI-reader-checkpoint', files=rows,
                  accepted_evidence=evidence, offline_version=manifest['version'],
                  new_blob_bytes=new_bytes, prior_complete_snapshot={'path': base.path.relative_to(ROOT).as_posix(),
                                                                     'sha256': base.identity},
                  browser_native_override_fallback_implemented=False, new_whole_builder_passed=False,
                  previous_zip_unchanged=True, full_parent_acceptance=False, deployed=False,
                  scope='Actual Policy command and explicit original-file browser inspection. Native fallback '
                        'observed separately and remains required missing browser work. Broader prior nested '
                        'historical retention gaps are not reclassified or repaired by this checkpoint.')
    destination.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'closure': destination.relative_to(ROOT).as_posix(), 'sha256': digest(destination),
                      'files': len(rows), 'new_blob_bytes': new_bytes}))


if __name__ == '__main__':
    main()
