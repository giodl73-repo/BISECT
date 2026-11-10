"""Retain the bounded Policy source-chain checkpoint and its exact artifacts."""
import hashlib
import json
import shutil
from pathlib import Path
from evidence_blob_store import SnapshotBlobs

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'target/policy-fallback'
PACKAGE = ROOT / 'dist/wasm-policy-fallback-final'
CASES = ['run-a8onfP', 'run-oAfDmM', 'browser-XP4m0w', 'browser-EcL2cK']


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
    for directory in ['target/policy-cli-contract/snapshot', 'target/config-cli-contract/snapshot']:
        for path in (ROOT / directory).iterdir():
            if path.is_file():
                known[path.name] = path
    files, accepted = set(), []
    for name in CASES:
        folder = OUT / name
        data = read(folder / 'evidence.json')
        browser = name.startswith('browser-')
        assert data['status'] == ('passed-policy-fallback-workbench' if browser else 'passed-native-policy-fallback')
        assert len(data['outcomes'] if browser else data['records']) == 15
        for ref in data['sources']:
            path = ROOT / ref['path']
            assert digest(path) == ref['sha256'], ref['path']
            files.add(path)
        if browser:
            assert data['test_sha256'] == digest(ROOT / 'scripts/web/test_policy_fallback_browser.mjs')
            assert data['controlled_cancel_late_and_reentrant_reads']
            ref = data['reference']
            assert digest(ROOT / ref['path']) == ref['sha256']
        else:
            assert data['backward_strict_cli_cases'] == 68
            for key in ['native', 'wasm', 'strict_cli_reference']:
                ref = data[key]
                path = ROOT / ref['path']
                assert digest(path) == ref['sha256']
                files.add(path)
            files.update(p for p in (ROOT / data['strict_cli_reference']['path']).parent.rglob('*') if p.is_file())
        files.update(p for p in folder.rglob('*') if p.is_file())
        accepted.append((folder / 'evidence.json').relative_to(ROOT).as_posix())
    assert read(OUT / 'build/status.json')['status'] == 'passed'
    for ref in read(OUT / 'build/source-inputs.json'):
        path = ROOT / ref['path']
        assert digest(path) == ref['sha256'], ref['path']
        files.add(path)
    for folder in ['build', 'before', 'c-engine']:
        files.update(p for p in (OUT / folder).rglob('*') if p.is_file())
    for path in OUT.glob('*.log'):
        files.add(path)
    engine = read(OUT / 'c-engine/engine-evidence.json')
    assert engine['native_matches'] == 52 and engine['expected_native_failures'] == 4
    assert engine['test_source_sha256'] == digest(ROOT / 'scripts/web/test_cffi_engine.mjs')
    for profile in ['default', 'c-sidecar']:
        evidence = Path(json.loads((OUT / ('fetch-' + profile + '.log')).read_text(encoding='utf-8-sig').strip())['evidence'])
        data = read(evidence)
        files.update(p for p in evidence.parent.rglob('*') if p.is_file())
    manifest = read(PACKAGE / 'offline-manifest.json')
    assert len(manifest['assets']) == 733
    assert engine['loaded_artifacts']['engine']['sha256'] == digest(PACKAGE / 'toolkit/bisect_wasm.wasm')
    for name, ref in manifest['assets'].items():
        path = PACKAGE / name
        assert path.stat().st_size == ref['bytes'] and digest(path) == ref['sha256']
        files.add(path)
    files.add(PACKAGE / 'offline-manifest.json')
    for name in ['README.md', 'docs/specs/wasm-policy-fallback.md', 'docs/specs/wasm-policy-cli-contract.md',
                 'docs/specs/wasm-desktop-parity-plan.md', 'docs/specs/wasm-capability-inventory.json',
                 'docs/specs/wasm-engine-option-inventory.json', 'docs/specs/wasm-practitioner-actions.json',
                 'docs/specs/wasm-fetch-source-coverage.json', 'design/pitfalls/pitfalls-pipeline.md',
                 'signals/roles/check/wasm-policy-fallback-roles-check-2026-10-27.md',
                 'scripts/web/snapshot_policy_fallback.py', 'scripts/web/test_policy_fallback_inputs.mjs',
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
    record = dict(status='passed-bounded-policy-fallback-checkpoint', files=rows,
                  accepted_evidence=accepted, offline_version=manifest['version'], new_blob_bytes=new_bytes,
                  previous_archive=release, previous_archive_unchanged=True,
                  new_whole_builder_passed=False, full_parent_acceptance=False, deployed=False,
                  scope='Explicit primary/alias source-chain adaptation with actual CLI and worker comparisons. '
                        'Unavailable inputs are data, not authenticated live access. Broader historical nested '
                        'retention gaps are not reclassified by this checkpoint.')
    destination.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'closure': destination.relative_to(ROOT).as_posix(), 'sha256': digest(destination),
                      'files': len(rows), 'verified_blob_identities': len(verified), 'new_blob_bytes': new_bytes}))


if __name__ == '__main__':
    main()
