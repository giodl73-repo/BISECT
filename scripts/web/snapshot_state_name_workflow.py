"""Freeze the selected-source State naming workflow and its exact regression run."""
import argparse
import json
import os
from pathlib import Path
import shutil
from evidence_blob_store import ROOT, SnapshotBlobs, digest, local


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['base', 'base-sha256', 'validation', 'output']:
        parser.add_argument('--' + name, required=True)
    args = parser.parse_args()
    base = SnapshotBlobs(args.base, args.base_sha256)
    validation = json.loads(local(args.validation).read_text(encoding='utf-8'))
    assert validation['status'] == 'passed-selected-source-workflow'
    assert validation['deployed'] is False and validation['acceptance_v1'] is False
    rows, sources, directories = {}, {}, set()

    def add(name, expected=None, size=None):
        path = local(name)
        identity, length = digest(path), path.stat().st_size
        assert expected is None or identity == expected, str(path)
        assert size is None or length == size, str(path)
        key = path.relative_to(ROOT).as_posix()
        assert key not in rows or rows[key]['sha256'] == identity, key
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

    def roots(value):
        if isinstance(value, dict):
            for key, nested in value.items():
                if key == 'root' and isinstance(nested, str):
                    folder = local(nested)
                    assert folder.is_relative_to(ROOT / 'target') and folder.is_dir()
                    directories.add(folder)
                else:
                    roots(nested)
        elif isinstance(value, list):
            for nested in value:
                roots(nested)

    add(args.validation)
    refs(validation)
    for group in validation['profiles'].values():
        for key in ['stage', 'browser', 'ignored', 'registered', 'registered_node']:
            record = json.loads(local(group[key]['path']).read_text(encoding='utf-8-sig'))
            refs(record)
            if key == 'registered':
                roots(record)
            elif key in ['ignored', 'registered_node']:
                native = json.loads(local(record['native_reference']['path']).read_text(encoding='utf-8-sig'))
                refs(native)
                directories.add(local(record['native_reference']['path']).parent)
        engine = ROOT / 'target/cffi-legacy-gates' / (group['registered']['path'].split('/')[-2] + '.wasm')
        add(engine, group['engine']['sha256'])
    directories.update([ROOT / 'target/state-name-workflow', ROOT / 'target/cffi-legacy-gates'])
    output = local(args.output)
    assert output.is_relative_to(ROOT / 'target/state-name-workflow') and not output.exists()
    for folder in sorted(directories):
        for directory, dirs, files in os.walk(folder, followlinks=False):
            # Browser profiles are ephemeral credentials/permissions, never evidence.
            dirs[:] = [d for d in dirs if d != 'blobs' and not d.startswith('profile-')
                       and d not in ['profile', 'chrome-profile', 'browser-profile']]
            for name in files:
                add(Path(directory) / name)
    for folder in ['web/lab', 'scripts/web']:
        for path in (ROOT / folder).rglob('*'):
            if path.is_file() and path.suffix in ['.js', '.mjs', '.py', '.ps1', '.html', '.json', '.css']:
                add(path)
    for name in [
        'crates/bisect-core/src/state_catalog.rs', 'crates/bisect-cli/src/main.rs',
        'crates/bisect-report/src/manifest_schema.rs', 'crates/bisect-report/src/manifest.rs',
        'crates/bisect-wasm/src/toolkit.rs', 'crates/bisect-wasm/src/state_identity.rs',
        'crates/bisect-wasm/src/run_target.rs', 'crates/bisect-wasm/src/plan_audit_text.rs',
        'docs/specs/wasm-native-state-name-workflow.md', 'docs/specs/wasm-desktop-parity-plan.md',
        'docs/specs/wasm-capability-inventory.json', 'docs/specs/wasm-engine-option-inventory.json',
        'docs/specs/wasm-practitioner-actions.json', 'design/pitfalls/README.md',
        'design/pitfalls/pitfalls-pipeline.md', 'README.md', 'Cargo.lock',
        'signals/roles/check/wasm-state-name-workflow-roles-check-2026-10-27.md',
    ]:
        add(name)
    unique = {r['sha256']: r for r in rows.values()}
    required = sum(r['byte_length'] for identity, r in unique.items() if identity not in base.rows)
    assert shutil.disk_usage(ROOT).free > required + 512 * 1024**2, 'Insufficient snapshot reserve'
    (output / 'blobs').mkdir(parents=True)
    for identity, row in unique.items():
        destination = output / 'blobs' / identity
        if identity in base.rows:
            os.link(base.blob(identity, row['byte_length']), destination)
        else:
            shutil.copyfile(sources[identity], destination)
        assert destination.stat().st_size == row['byte_length'] and digest(destination) == identity
    manifest = dict(storage_format='content-addressed-blobs-v1',
                    prior_snapshot=dict(path=base.path.relative_to(ROOT).as_posix(), sha256=base.identity),
                    files=sorted(rows.values(), key=lambda r: r['path']), scope=validation['scope'],
                    deployed=False, acceptance_v1=False)
    path = output / 'snapshot.json'
    path.write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(dict(path=path.relative_to(ROOT).as_posix(), sha256=digest(path),
                         verification=SnapshotBlobs(path).verify(), new_bytes=required)), flush=True)


if __name__ == '__main__':
    main()
