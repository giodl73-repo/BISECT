"""Flatten verified local evidence inheritance using immutable same-volume hard links."""
import argparse
import json
import os
from pathlib import Path
import time
from evidence_blob_store import ROOT, SnapshotBlobs, digest, local


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--snapshot', required=True)
    parser.add_argument('--sha256', required=True)
    parser.add_argument('--output')
    args = parser.parse_args()
    source = SnapshotBlobs(args.snapshot, args.sha256)
    print('Verifying original inherited snapshot', flush=True)
    original = source.verify()
    destination = local(args.output or 'target/national-matrix/evidence-rollup-'+str(time.time_ns()))
    assert destination.is_relative_to(ROOT/'target') and not destination.exists(), 'Choose a new owned target directory'
    (destination/'blobs').mkdir(parents=True)
    for identity, row in source.rows.items():
        blob = source.blob(identity, row['byte_length'])
        assert not blob.is_symlink() and not blob.is_junction()
        # A hard link owns a directory entry to the bytes independently of the
        # old path. No existing blob, package, manifest or attribute is rewritten.
        os.link(blob, destination/'blobs'/identity)
    rows = [dict(path=row['path'],sha256=row['sha256'],byte_length=row['byte_length'],blob='blobs/'+row['sha256']) for row in source.data['files']]
    manifest = destination/'snapshot.json'
    manifest.write_text(json.dumps(dict(schema_version='bisect-evidence-snapshot-rollup-v1',storage_format='content-addressed-blobs-v1',source_snapshot=dict(path=source.path.relative_to(ROOT).as_posix(),sha256=args.sha256),storage_note='Same-volume hard links to verified immutable blobs; own complete blob paths, no inherited lookup required. Historical paths/bytes remain unchanged.',files=rows),indent=2),encoding='utf-8')
    identity = digest(manifest)
    print('Verifying independent rollup blob paths', flush=True)
    checked = SnapshotBlobs(manifest, identity).verify()
    assert checked == original
    assert digest(source.path) == args.sha256
    print(json.dumps(dict(path=manifest.relative_to(ROOT).as_posix(),sha256=identity,byte_length=manifest.stat().st_size,verification=checked)),flush=True)


if __name__ == '__main__':
    main()
