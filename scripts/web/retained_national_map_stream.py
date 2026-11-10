"""Pack or restore the exact retained national map request stream, without running it."""

import argparse
import hashlib
import json
import lzma
from pathlib import Path


SNAPSHOT = Path('target/parity-current-matrix-20261101/verification-manifest.json')
SNAPSHOT_SHA = 'd005a4c140a507744a755b30fb433268d700bcbb75a5747dc7d6f689e62909bb'
REQUESTS = 'target/national-matrix/map-1793562621520/requests.jsonl'
BLOCK = 1024 * 1024


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    with path.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def authority(snapshot=SNAPSHOT):
    with Path(snapshot).open('rb') as f:
        data = f.read(BLOCK+1)
    require(len(data) <= BLOCK and hashlib.sha256(data).hexdigest() == SNAPSHOT_SHA, 'Changed snapshot identity')
    record, = (r for r in json.loads(data)['artifacts'] if r['path'] == REQUESTS)
    require(record['byte_length'] == 324026471, 'Unexpected national stream size')
    return record


def verify(packed, record, output=None):
    require(0 < packed.stat().st_size <= 128*BLOCK, 'Encoded request stream exceeds its size limit')
    decoder = lzma.LZMADecompressor(format=lzma.FORMAT_XZ, memlimit=64*BLOCK)
    remaining = record['byte_length']
    identity = hashlib.sha256()
    with packed.open('rb') as f:
        while not decoder.eof:
            block = f.read(BLOCK) if decoder.needs_input else b''
            require(not decoder.needs_input or block, 'Truncated request stream')
            out = decoder.decompress(block, max_length=min(BLOCK,remaining+1))
            require(len(out) <= remaining, 'Request stream exceeds its declared length')
            remaining -= len(out)
            identity.update(out)
            if output is not None:
                output.write(out)
        require(not decoder.unused_data and not f.read(1), 'Trailing request stream bytes')
    require(remaining == 0 and identity.hexdigest() == record['sha256'], 'Decoded request stream differs')


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('mode', choices=('pack','verify','restore'))
    p.add_argument('--packed', type=Path, required=True)
    p.add_argument('--snapshot', type=Path, default=SNAPSHOT, help='Exact retained snapshot manifest')
    p.add_argument('--output', type=Path, help='Fresh destination directory for restore')
    args = p.parse_args()
    r = authority(args.snapshot)
    if args.mode == 'pack':
        source = Path(r['path'])
        require(source.stat().st_size == r['byte_length'] and digest(source) == r['sha256'], 'Changed original request stream')
        with lzma.open(args.packed,'xb',preset=3) as dst, source.open('rb') as f:
            while block := f.read(BLOCK):
                dst.write(block)
    if args.mode == 'restore':
        require(args.output is not None, 'Pass a fresh --output directory')
        args.output.mkdir()
        partial = args.output/'requests.jsonl.partial'
        with partial.open('xb') as dst:
            verify(args.packed,r,dst)
        partial.rename(args.output/'requests.jsonl')
    else:
        verify(args.packed,r)
    print(json.dumps(dict(status='passed-exact-retained-national-map-stream', packed_sha256=digest(args.packed),
                          packed_bytes=args.packed.stat().st_size, decoded_sha256=r['sha256'],
                          decoded_bytes=r['byte_length'],executed_files=0)))
