"""Inventory actual 2020 TIGER polygon records without materializing GeoJSON."""
import hashlib
import json
import struct
from pathlib import Path

manifest = json.loads(Path('data/manifest.json').read_text(encoding='utf-8'))
rows = []
for code, state in sorted(manifest['states'].items()):
    if code == 'DC':
        continue
    name = f"tl_2020_{state['fips']}_tract"
    stem = Path('data/2020/tiger/tracts') / name / name
    sizes = {ext: stem.with_suffix('.' + ext).stat().st_size for ext in ['shp', 'dbf', 'shx']}
    records = positions = 0
    with stem.with_suffix('.shp').open('rb') as stream:
        header = stream.read(100)
        assert len(header) == 100 and struct.unpack_from('>I', header)[0] == 9994
        assert struct.unpack_from('>I', header, 24)[0] * 2 == sizes['shp']
        while record_header := stream.read(8):
            assert len(record_header) == 8
            length = struct.unpack('>II', record_header)[1] * 2
            body = stream.read(min(44, length))
            kind = struct.unpack_from('<I', body)[0]
            assert kind in [0, 5, 15]
            if kind:
                assert len(body) == 44
                positions += struct.unpack_from('<I', body, 40)[0]
            stream.seek(length - len(body), 1)
            records += 1
        assert stream.tell() == sizes['shp']
    hashes = {}
    for part in sizes:
        digest = hashlib.sha256()
        with stem.with_suffix('.' + part).open('rb') as stream:
            for block in iter(lambda: stream.read(1024 * 1024), b''):
                digest.update(block)
        hashes[part] = digest.hexdigest()
    rows.append(dict(state=code, units=records, positions=positions, sizes=sizes,
                     sha256=hashes, over_large_positions=positions > 2_000_000,
                     over_source_size=max(sizes.values()) > 64 * 1024 * 1024))
assert len(rows) == 50
output = Path('target/original-native-directory')
output.mkdir(exist_ok=True)
(output / 'source-inventory.json').write_text(json.dumps(rows, indent=2) + '\n', encoding='utf-8')
print(json.dumps(dict(states=len(rows), units=sum(r['units'] for r in rows),
                     positions=sum(r['positions'] for r in rows),
                     source_bytes=sum(sum(r['sizes'].values()) for r in rows),
                     above_strict_large=[r['state'] for r in rows if r['over_large_positions']],
                     above_selected_part_limit=[r['state'] for r in rows if r['over_source_size']])))
