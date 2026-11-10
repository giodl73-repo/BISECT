"""Losslessly retain a small byte delta between two owned national project archives."""

import argparse
import hashlib
import json
from pathlib import Path


BLOCK = 1024 * 1024
MAX_BYTES = 128 * BLOCK
MAX_EDITS = 4096


def require(condition, message):
    if not condition:
        raise ValueError(message)


def identity(value):
    return (type(value) is str and len(value) == 64
            and all(c in '0123456789abcdef' for c in value))


def create(original, saved):
    size = original.stat().st_size
    require(0 < size <= MAX_BYTES and size == saved.stat().st_size, 'Project size differs')
    digests = hashlib.sha256(), hashlib.sha256()
    edits = []
    with original.open('rb') as source, saved.open('rb') as target:
        for pos in range(0, size, BLOCK):
            count = min(BLOCK, size-pos)
            a, b = source.read(count), target.read(count)
            require(len(a) == len(b) == count, 'Project changed while comparing')
            digests[0].update(a)
            digests[1].update(b)
            if a != b:
                for i, (old, new) in enumerate(zip(a, b)):
                    if old != new:
                        require(len(edits) < MAX_EDITS, 'Projects need a full byte witness')
                        edits.append([pos+i, old, new])
        require(not source.read(1) and not target.read(1), 'Project grew while comparing')
    return dict(schema='bisect-national-project-delta-v1', bytes=size,
                original_sha256=digests[0].hexdigest(), saved_sha256=digests[1].hexdigest(), edits=edits)


def verify(original, delta, output=None):
    require(isinstance(delta, dict) and set(delta) == {'schema','bytes','original_sha256','saved_sha256','edits'}
            and delta['schema'] == 'bisect-national-project-delta-v1', 'Invalid project delta')
    size, edits = delta['bytes'], delta['edits']
    require(type(size) is int and 0 < size <= MAX_BYTES and size == original.stat().st_size, 'Invalid project size')
    require(all(identity(delta[k]) for k in ('original_sha256','saved_sha256')), 'Invalid project digest')
    require(type(edits) is list and len(edits) <= MAX_EDITS, 'Too many project edits')
    previous = -1
    for edit in edits:
        require(type(edit) is list and len(edit) == 3 and all(type(x) is int for x in edit), 'Invalid edit')
        offset, old, new = edit
        require(previous < offset < size and 0 <= old <= 255 and 0 <= new <= 255 and old != new, 'Invalid edit position or bytes')
        previous = offset
    digests = hashlib.sha256(), hashlib.sha256()
    index = 0
    with original.open('rb') as source:
        for pos in range(0, size, BLOCK):
            count = min(BLOCK, size-pos)
            block = source.read(count)
            require(len(block) == count, 'Original project changed while decoding')
            digests[0].update(block)
            if index < len(edits) and edits[index][0] < pos+count:
                block = bytearray(block)
                while index < len(edits) and edits[index][0] < pos+count:
                    offset, old, new = edits[index]
                    require(block[offset-pos] == old, 'Original edit preimage differs')
                    block[offset-pos] = new
                    index += 1
            digests[1].update(block)
            if output is not None:
                output.write(block)
        require(index == len(edits) and not source.read(1), 'Incomplete project decode')
    require([h.hexdigest() for h in digests] == [delta['original_sha256'],delta['saved_sha256']], 'Project digest differs')


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('mode', choices=('pack','verify','restore'))
    p.add_argument('--original',type=Path,required=True)
    p.add_argument('--delta',type=Path,required=True)
    p.add_argument('--saved',type=Path)
    p.add_argument('--output',type=Path,help='Fresh destination directory for restore')
    a = p.parse_args()
    if a.mode == 'pack':
        require(a.saved is not None,'Pass the captured --saved project')
        delta = create(a.original,a.saved)
        verify(a.original,delta)
        with a.delta.open('x',encoding='utf8',newline='\n') as f:
            json.dump(delta,f,separators=(',',':'))
            f.write('\n')
    else:
        with a.delta.open('rb') as f:
            raw = f.read(64*1024+1)
        require(len(raw) <= 64*1024,'Oversized project delta')
        delta = json.loads(raw)
        if a.mode == 'restore':
            require(a.output is not None,'Pass a fresh --output directory')
            a.output.mkdir()
            pending = a.output/'saved.bisect-workbench.partial'
            with pending.open('xb') as f:
                verify(a.original,delta,f)
            pending.rename(a.output/'saved.bisect-workbench')
        else:
            verify(a.original,delta)
    print(json.dumps(dict(status='passed-exact-national-project-delta',bytes=delta['bytes'],
                          saved_sha256=delta['saved_sha256'],edits=len(delta['edits']),executed_files=0)))
