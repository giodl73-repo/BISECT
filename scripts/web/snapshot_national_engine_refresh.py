"""Freeze an explicitly enumerated national refresh scope with full blob readback."""
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
    parser.add_argument('--root', action='append', default=[])
    parser.add_argument('--file', action='append', default=[])
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    base = SnapshotBlobs(args.base, args.base_sha256)
    validation = json.loads(local(args.validation).read_text(encoding='utf-8'))
    assert validation['status'] == 'passed' and validation['deployed'] is False
    rows = {}
    sources = {}

    def add(name, expected=None, size=None):
        path = local(name)
        identity, length = digest(path), path.stat().st_size
        assert expected is None or identity == expected, str(path)
        assert size is None or length == size, str(path)
        key = path.relative_to(ROOT).as_posix()
        assert key not in rows or rows[key]['sha256'] == identity
        rows[key] = dict(path=key, sha256=identity, byte_length=length, blob='blobs/' + identity)
        sources[identity] = path

    def references(value):
        if isinstance(value, dict):
            if isinstance(value.get('path'), str) and isinstance(value.get('sha256'), str):
                add(value['path'], value['sha256'], value.get('byte_length'))
            else:
                for item in value.values():
                    references(item)
        elif isinstance(value, list):
            for item in value:
                references(item)

    references(validation)
    package = json.loads(local(validation['package_validation']['path']).read_text(encoding='utf-8'))
    references(package['packages'])
    for ref in validation['records']:
        record = json.loads(local(ref['evidence']['path']).read_text(encoding='utf-8'))
        # Comparison pins include both historic and current exact whole replies.
        if ref['kind'] == 'comparison':
            references(record)
    native_map = json.loads(local(validation['native_map']['path']).read_text(encoding='utf-8'))
    references(native_map)
    add(args.validation)
    add(__file__)
    for name in args.file:
        add(name)
    for name in args.root:
        folder = local(name)
        assert folder.is_relative_to(ROOT / 'target') and folder.is_dir()
        for directory, dirs, files in os.walk(folder, followlinks=False):
            dirs[:] = [d for d in dirs if not d.startswith('profile-') and d not in ['profile', 'chrome-profile', 'browser-profile', 'blobs']]
            for filename in files:
                add(Path(directory) / filename)
    unique = {r['sha256']: r for r in rows.values()}
    required = sum(r['byte_length'] for h, r in unique.items() if h not in base.rows)
    assert shutil.disk_usage(ROOT).free > required + 256 * 1024**2
    output = local(args.output)
    assert output.is_relative_to(ROOT / 'target/national-engine-refresh') and not output.exists()
    (output / 'blobs').mkdir(parents=True)
    for identity, ref in unique.items():
        destination = output / 'blobs' / identity
        if identity in base.rows:
            os.link(base.blob(identity, ref['byte_length']), destination)
        else:
            shutil.copyfile(sources[identity], destination)
        assert digest(destination) == identity
    manifest = dict(storage_format='content-addressed-blobs-v1', files=sorted(rows.values(), key=lambda r: r['path']), prior_snapshot=dict(path=base.path.relative_to(ROOT).as_posix(), sha256=base.identity), scope=validation['scope'], deployed=False, acceptance_v1=False)
    path = output / 'snapshot.json'
    path.write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(dict(path=path.relative_to(ROOT).as_posix(), sha256=digest(path), verification=SnapshotBlobs(path).verify(), new_bytes=required)), flush=True)


if __name__ == '__main__':
    main()
