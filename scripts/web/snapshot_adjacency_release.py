"""Freeze GitHub adjacency release client, provider and native-consumer evidence."""
import argparse
import json
import os
from pathlib import Path
import shutil
from evidence_blob_store import ROOT, SnapshotBlobs, digest, local


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['base', 'base-sha256', 'validation', 'source-before', 'output']:
        parser.add_argument('--' + name, required=True)
    parser.add_argument('--supplement', action='append', default=[])
    args = parser.parse_args()
    base = SnapshotBlobs(args.base, args.base_sha256)
    print('Verify and freeze scoped package/workflow evidence', flush=True)
    rows = {}
    sources = {}

    def add(name, expected=None, size=None):
        path = local(name)
        identity = digest(path)
        length = path.stat().st_size
        assert expected is None or expected == identity, 'Current evidence changed: ' + str(path)
        assert size is None or size == length
        logical = path.relative_to(ROOT).as_posix()
        if logical in rows and rows[logical]['sha256'] != identity:
            old = rows.pop(logical)
            raise ValueError('Conflicting current scope identity: ' + logical)
        rows[logical] = dict(path=logical, sha256=identity, byte_length=length, blob=identity,
                             storage='local')
        sources[identity] = path

    def references(value):
        if isinstance(value, dict):
            if isinstance(value.get('path'), str) and isinstance(value.get('sha256'), str):
                add(value['path'], value['sha256'], value.get('byte_length'))
            else:
                for nested in value.values():
                    references(nested)
        elif isinstance(value, list):
            for nested in value:
                references(nested)

    validation = json.loads(local(args.validation).read_text(encoding='utf-8'))
    assert validation['status'] == 'passed' and validation['deployed'] is False
    assert all(r['exit_code'] == 0 for r in validation['checks'])
    references(validation)
    for folder in [local(args.validation).parent, local(args.source_before), *map(local,args.supplement)]:
        assert folder.is_relative_to(ROOT / 'target')
        evidence = folder / 'evidence.json'
        if evidence.is_file():
            references(json.loads(evidence.read_text(encoding='utf-8')))
        for path in folder.rglob('*'):
            if path.is_file() and not any(p.startswith('profile-') for p in path.relative_to(folder).parts):
                add(path)
    for name in ['scripts/web/snapshot_adjacency_release.py', 'scripts/web/test_adjacency_release_private_browser.mjs',
                 'docs/specs/wasm-adjacency-release-acquisition.md',
                 'signals/roles/check/wasm-adjacency-release-acquisition-roles-check-2026-10-26.md',
                 'design/pitfalls/README.md', 'design/pitfalls/pitfalls-pipeline.md']:
        add(name)
    unique = {r['sha256']: r for r in rows.values() if r['storage'] == 'local'}
    required = sum(r['byte_length'] for r in unique.values() if r['sha256'] not in base.rows)
    assert shutil.disk_usage(ROOT).free > required + 128 * 1024**2, 'Insufficient snapshot reserve'
    output = local(args.output)
    assert output.is_relative_to(ROOT / 'target/adjacency-release') and not output.exists()
    (output / 'blobs').mkdir(parents=True)
    for identity, ref in unique.items():
        destination = output / 'blobs' / identity
        if identity in base.rows:
            # Link only a digest-verified immutable blob, never mutable inputs.
            os.link(base.blob(identity, ref['byte_length']), destination)
        else:
            shutil.copyfile(sources[identity], destination)
        assert destination.stat().st_size == ref['byte_length'] and digest(destination) == identity
    manifest = dict(storage_format='content-addressed-blobs-v1',
                    prior_snapshot=dict(path=base.path.relative_to(ROOT).as_posix(), sha256=base.identity),
                    files=sorted([{**r, 'blob': 'blobs/' + r['sha256']} for r in rows.values()], key=lambda r: r['path']),
                    scope='GitHub release input client; controlled browser credential/redirect proof and native-authenticated real RI provider proof are separate realms; authorized real private-browser metadata succeeds but redirected assets fail MissingAllowOriginHeader; provider acceptance remains open.',
                    acceptance_v1=False, deployed=False)
    destination = output / 'snapshot.json'
    destination.write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(dict(path=destination.relative_to(ROOT).as_posix(), sha256=digest(destination),
                         verification=SnapshotBlobs(destination).verify(), new_bytes=required)), flush=True)


if __name__ == '__main__':
    main()
