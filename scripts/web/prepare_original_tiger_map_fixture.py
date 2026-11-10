"""Pack or materialize the pinned original California and Texas map inputs."""

import argparse
import hashlib
import json
import lzma
from pathlib import Path, PurePosixPath
import re


DECLARATION = Path(__file__).parent / 'fixtures/original-tiger-state-map-v1.json'


def declaration():
    d = json.loads(DECLARATION.read_bytes())
    assert d['schema_version'] == 'bisect-original-tiger-state-map-fixture-v1'
    assert d['year'] == 2020 and d['dpi'] == 72 and set(d['cases']) == {'CA', 'TX'}
    for state, case in d['cases'].items():
        assert set(case['files']) == {'shp', 'shx', 'dbf', 'assignments'}
        for kind, row in case['files'].items():
            relative = PurePosixPath(row['path'])
            assert not relative.is_absolute() and '..' not in relative.parts
            assert all(re.fullmatch(r'[a-zA-Z0-9_.-]+', part) for part in relative.parts)
            assert relative.parts[0] == ('outputs' if kind == 'assignments' else 'data')
            assert 0 < row['bytes'] <= 64 * 1024 * 1024 and re.fullmatch('[a-f0-9]{64}', row['sha256'])
    return d


def copy_bound(source, destination, expected):
    digest = hashlib.sha256()
    remaining = expected['bytes']
    while remaining:
        block = source.read(min(1024 * 1024, remaining))
        if not block:
            raise ValueError('Map fixture ended before the declared length')
        digest.update(block)
        destination.write(block)
        remaining -= len(block)
    if source.read(1) or digest.hexdigest() != expected['sha256']:
        raise ValueError('Map fixture does not match the declared source bytes')


def unpack_xz(source, destination, expected):
    decoder = lzma.LZMADecompressor(format=lzma.FORMAT_XZ, memlimit=64 * 1024 * 1024)
    digest = hashlib.sha256()
    remaining = expected['bytes']
    while not decoder.eof:
        data = source.read(1024 * 1024) if decoder.needs_input else b''
        if decoder.needs_input and not data:
            raise ValueError('Truncated compressed map fixture')
        block = decoder.decompress(data, max_length=min(1024 * 1024, remaining + 1))
        if len(block) > remaining:
            raise ValueError('Compressed map fixture exceeds its declared length')
        digest.update(block)
        destination.write(block)
        remaining -= len(block)
    if decoder.unused_data or source.read(1) or remaining or digest.hexdigest() != expected['sha256']:
        raise ValueError('Compressed map fixture does not match the declared source bytes')


def file_hash(path):
    with path.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    commands = p.add_subparsers(dest='command', required=True)
    pack = commands.add_parser('pack')
    pack.add_argument('--data-root', type=Path, required=True)
    pack.add_argument('--native-case-root', type=Path, required=True)
    restore = commands.add_parser('restore')
    restore.add_argument('--packed-root', type=Path, required=True)
    for command in (pack, restore):
        command.add_argument('--output', type=Path, required=True)
    args = p.parse_args()
    d = declaration()
    args.output.mkdir()
    rows = []
    for state, case in d['cases'].items():
        for kind, row in case['files'].items():
            relative = Path(row['path'])
            member = f'{state}-{kind}.xz'
            if args.command == 'pack':
                source = (args.native_case_root / state / relative if kind == 'assignments'
                          else args.data_root.joinpath(*relative.parts[1:]))
                dest = args.output / member
                with source.open('rb') as s, lzma.open(dest, 'xb', preset=6) as encoded:
                    copy_bound(s, encoded, row)
                packed = dest
            else:
                source = args.packed_root / member
                dest = args.output / relative
                dest.parent.mkdir(parents=True, exist_ok=True)
                if source.stat().st_size > 64 * 1024 * 1024:
                    raise ValueError('Compressed map fixture exceeds the file limit')
                with source.open('rb') as s, dest.open('xb') as out:
                    unpack_xz(s, out, row)
                packed = source
            rows.append(dict(state=state, kind=kind, member=member, packed_bytes=packed.stat().st_size,
                             packed_sha256=file_hash(packed), **row))
    report = dict(status='passed-original-tiger-fixture-' + args.command,
                  declaration_sha256=hashlib.sha256(DECLARATION.read_bytes()).hexdigest(),
                  files=rows, decoded_bytes=sum(row['bytes'] for row in rows),
                  packed_bytes=sum(row['packed_bytes'] for row in rows))
    with (args.output / 'evidence.json').open('x', encoding='utf8', newline='\n') as f:
        json.dump(report, f, indent=2)
        f.write('\n')
    print(report['status'], report['decoded_bytes'], report['packed_bytes'])


if __name__ == '__main__':
    main()
