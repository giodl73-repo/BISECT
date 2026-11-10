"""Retain the bounded read-only manifest-width checkpoint without repinning history."""
import json
import os
import shutil
from pathlib import Path
from evidence_blob_store import ROOT, SnapshotBlobs, digest

OUT = ROOT / 'target/manifest-reader-width'
PACKAGE = ROOT / 'dist/wasm-manifest-reader-width'


def read(path):
    return json.loads(path.read_text(encoding='utf-8-sig'))


def main():
    destination = OUT / 'closure.json'
    assert not destination.exists(), 'Preserve previous checkpoints.'
    files, accepted, audits = set(), [], []

    def folder(path):
        files.update(p for p in path.rglob('*') if p.is_file())

    def reference(ref):
        path = ROOT / ref['path']
        assert digest(path) == ref['sha256'], ref['path']
        files.add(path)

    for name, browser in [('run-Hk4a9t', False), ('run-HxkKWl', False),
                          ('browser-829MoN', True), ('browser-CNDBmw', True)]:
        path = OUT / name
        data = read(path / 'evidence.json')
        assert data['status'] == ('passed-manifest-reader-width-workbench' if browser else 'passed-manifest-reader-width')
        assert len(data['outcomes' if browser else 'records']) == 72
        for ref in data['sources']:
            reference(ref)
        if browser:
            reference(data['reference'])
            assert data['explicit_runs'] == 72
            assert data['test_sha256'] == digest(ROOT / 'scripts/web/test_manifest_reader_width_browser.mjs')
        else:
            reference(data['native'])
            reference(data['cli'])
            assert len(data['cli_records']) == 18
            reference(data['wasm'])
        folder(path)
        accepted.append((path / 'evidence.json').relative_to(ROOT).as_posix())
    for name in ['run-Psc7bh', 'run-Ok5NjU']:
        path = ROOT / 'target/policy-fallback' / name
        data = read(path / 'evidence.json')
        assert len(data['records']) == 15 and data['backward_strict_cli_cases'] == 68
        reference(data['strict_cli_reference'])
        folder(path)
        accepted.append((path / 'evidence.json').relative_to(ROOT).as_posix())
    for profile in ['default', 'c']:
        data = read(OUT / f'fetch-{profile}.log')
        assert data['cases'] == 42 and not data['acquisition_performed']
        folder(Path(data['evidence']).parent)
    assert read(OUT / 'build/status.json')['status'] == 'passed'
    for ref in read(OUT / 'build/source-inputs.json'):
        reference(ref)
    for name in ['build', 'before', 'c-engine']:
        folder(OUT / name)
    files.update(p for p in OUT.glob('*.log') if p.name != 'snapshot.log')
    files.update(OUT.glob('*cleanup.json'))
    files.add(OUT / 'full-parity-negative.json')
    engine = read(OUT / 'c-engine/engine-evidence.json')
    assert engine['native_matches'] == 52 and engine['expected_native_failures'] == 4
    assert engine['loaded_artifacts']['engine']['sha256'] == digest(PACKAGE / 'toolkit/bisect_wasm.wasm')
    manifest = read(PACKAGE / 'offline-manifest.json')
    assert len(manifest['assets']) == 741
    for name, ref in manifest['assets'].items():
        path = PACKAGE / name
        assert path.stat().st_size == ref['bytes'] and digest(path) == ref['sha256']
        files.add(path)
    files.add(PACKAGE / 'offline-manifest.json')
    for name in ['README.md', 'docs/specs/wasm-manifest-reader-width.md',
                 'docs/specs/wasm-desktop-parity-plan.md', 'docs/specs/wasm-capability-inventory.json',
                 'docs/specs/wasm-engine-option-inventory.json', 'docs/specs/wasm-practitioner-actions.json',
                 'docs/specs/wasm-fetch-source-coverage.json',
                 'signals/roles/check/wasm-manifest-reader-width-roles-check-2026-10-27.md',
                 'scripts/web/snapshot_manifest_reader_width.py', 'scripts/web/test_manifest_reader_width.mjs',
                 'scripts/web/test_manifest_reader_width_browser.mjs', 
                 'scripts/web/test_cffi_engine.mjs', 'crates/bisect-cli/src/verify.rs',
                 'design/pitfalls/README.md', 'design/pitfalls/pitfalls-pipeline.md',
                 'scripts/web/check_cffi_legacy_gates.ps1',
                 'scripts/web/build_cffi_preview.ps1']:
        files.add(ROOT / name)
    base = SnapshotBlobs('target/config-publication/full-build/snapshot-scope/snapshot.json',
                         '519c6eeb7c8d121057c0bb0534c90b15fa677aa20736b9375fad36e0d93f478d')
    known = {}
    for name in ['plan-manifest-width', 'doctor-workspace', 'tutorial-cli-contract', 'manifest-cli-contract', 'doctor-command',
                 'policy-fallback', 'policy-cli-contract', 'config-cli-contract']:
        for path in (ROOT / 'target' / name / 'snapshot').iterdir():
            if path.is_file():
                known[path.name] = path
    blobs = OUT / 'snapshot'
    blobs.mkdir(exist_ok=True)
    rows, verified = [], {}
    for path in sorted(files):
        identity, size = digest(path), path.stat().st_size
        if identity not in verified:
            if identity in base.rows:
                blob = base.blob(identity, size)
            elif identity in known:
                blob = known[identity]
            else:
                blob = blobs / identity
                if not blob.exists():
                    if path.suffix == '.wasm' and (path.is_relative_to(PACKAGE) or path.is_relative_to(OUT / 'build')):
                        os.link(path, blob)
                    else:
                        assert shutil.disk_usage(ROOT).free > size + 128 * 1024**2
                        shutil.copyfile(path, blob)
            assert blob.stat().st_size == size and digest(blob) == identity
            verified[identity] = blob
        rows.append({'source': {'path': path.relative_to(ROOT).as_posix(), 'sha256': identity, 'byte_length': size},
                     'snapshot': {'path': verified[identity].relative_to(ROOT).as_posix(), 'sha256': identity, 'byte_length': size}})
    archive = read(ROOT / 'target/config-publication/full-build/release.json')['archive']
    assert digest(ROOT / archive['path']) == archive['sha256']
    record = dict(status='passed-bounded-manifest-reader-width-checkpoint', files=rows,
                  accepted_evidence=accepted, separate_gap_audits=audits, offline_version=manifest['version'],
                  previous_archive=archive, previous_archive_unchanged=True,
                  new_whole_builder_passed=False, full_parent_acceptance=False, deployed=False,
                  immutable_artifact_rule='Never overwrite linked owned build or sealed package WASM; use new output directories.',
                  scope='Four read-only manifest readers, exact native comparisons and real workbench runs over selected sources only. Existing native binaries are hashed, not newly rebuilt references. Actual OS grants, full workflows/builder, wider matrices, reproducible release and deployment remain open.')
    destination.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(dict(closure=destination.relative_to(ROOT).as_posix(), sha256=digest(destination),
                          files=len(rows), identities=len(verified))))


if __name__ == '__main__':
    main()
