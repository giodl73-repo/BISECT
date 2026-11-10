"""Freeze the naming-preview scope, reusing only verified immutable blobs."""
import argparse
import json
import os
from pathlib import Path
import shutil
from evidence_blob_store import ROOT, SnapshotBlobs, digest, local


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base', required=True)
    parser.add_argument('--base-sha256', required=True)
    parser.add_argument('--validation', required=True)
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    base = SnapshotBlobs(args.base, args.base_sha256)
    validation = json.loads(local(args.validation).read_text(encoding='utf-8'))
    assert validation['status'] == 'passed-naming-preview' and validation['deployed'] is False
    rows, sources = {}, {}

    def add(name, expected=None, size=None):
        path = local(name)
        identity, length = digest(path), path.stat().st_size
        assert expected is None or identity == expected, str(path)
        assert size is None or size == length, str(path)
        key = path.relative_to(ROOT).as_posix()
        assert key not in rows or rows[key]['sha256'] == identity
        rows[key] = dict(path=key, sha256=identity, byte_length=length, blob='blobs/' + identity)
        sources[identity] = path

    def refs(value):
        if isinstance(value, dict):
            if isinstance(value.get('path'), str) and isinstance(value.get('sha256'), str):
                add(value['path'], value['sha256'], value.get('byte_length'))
            else:
                for nested in value.values():
                    refs(nested)
        elif isinstance(value, list):
            for nested in value:
                refs(nested)

    refs(validation)
    for profile, group in validation['profiles'].items():
        stage = json.loads(local(group['stage']['path']).read_text(encoding='utf-8'))
        assert stage['profile'] == profile and stage['status'] == 'passed-stage-build'
        refs(stage['package_files'])
    # Preserve historical source pins as records, never rebind them to current
    # files. The earlier foundation ZIP retains its exact original source bytes.
    for directory, dirs, files in os.walk(ROOT / 'target/state-identity', followlinks=False):
        dirs[:] = [d for d in dirs if d != 'blobs' and not d.startswith('profile-')]
        for name in files:
            add(Path(directory) / name)
    for name in [
        'crates/bisect-core/src/state_catalog.rs', 'crates/bisect-cli/src/main.rs',
        'crates/bisect-report/src/manifest.rs', 'crates/bisect-report/src/manifest_schema.rs', 'crates/bisect-report/src/lib.rs',
        'crates/bisect-wasm/src/state_identity.rs', 'crates/bisect-wasm/src/run_target.rs', 'crates/bisect-wasm/src/toolkit.rs', 'crates/bisect-wasm/src/lib.rs',
        'web/lab/native-state-identity-preview.js', 'web/lab/state-command-files.js', 'web/lab/wasm-worker.js',
        'scripts/web/prepare_state_identity_native_reference.py', 'scripts/web/stage_state_identity_preview.py',
        'scripts/web/test_state_identity_inspection.mjs', 'scripts/web/test_state_identity_targets.mjs',
        'scripts/web/test_state_identity_preview_browser.mjs', 'scripts/web/validate_state_identity_preview.py',
        'scripts/web/snapshot_state_identity_preview.py', 'scripts/web/check_cffi_legacy_gates.ps1', 'scripts/web/build_wasm_lab.ps1',
        'scripts/web/fixtures/native-state-source.json', 'docs/specs/wasm-native-state-identity.md',
        'docs/specs/wasm-desktop-parity-plan.md', 'docs/specs/wasm-practitioner-actions.json',
        'docs/specs/wasm-capability-inventory.json', 'docs/specs/wasm-engine-option-inventory.json',
        'signals/roles/check/wasm-state-identity-output-preview-roles-check-2026-10-26.md',
        'signals/roles/check/wasm-native-state-identity-roles-check-2026-10-26.md', 'README.md', 'Cargo.lock',
    ]:
        add(name)
    unique = {row['sha256']: row for row in rows.values()}
    required = sum(row['byte_length'] for identity, row in unique.items() if identity not in base.rows)
    assert shutil.disk_usage(ROOT).free > required + 256 * 1024**2
    output = local(args.output)
    assert output.is_relative_to(ROOT / 'target/state-identity') and not output.exists()
    (output / 'blobs').mkdir(parents=True)
    for identity, row in unique.items():
        destination = output / 'blobs' / identity
        if identity in base.rows:
            os.link(base.blob(identity, row['byte_length']), destination)
        else:
            shutil.copyfile(sources[identity], destination)
        assert digest(destination) == identity
    manifest = dict(storage_format='content-addressed-blobs-v1', files=sorted(rows.values(), key=lambda r: r['path']), prior_snapshot=dict(path=base.path.relative_to(ROOT).as_posix(), sha256=base.identity), scope=validation['scope'], deployed=False, acceptance_v1=False)
    path = output / 'snapshot.json'
    path.write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(dict(path=path.relative_to(ROOT).as_posix(), sha256=digest(path), verification=SnapshotBlobs(path).verify(), new_bytes=required)), flush=True)


if __name__ == '__main__':
    main()
