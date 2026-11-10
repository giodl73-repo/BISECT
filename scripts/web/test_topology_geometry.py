"""Independent GEOS and original binary-SHP checks for CA shared-boundary output."""
import hashlib
import json
from pathlib import Path
import struct
from shapely.geometry import shape
from shapely.strtree import STRtree

root = Path('target/original-state-acquisition/CA')
data = (root / 'topology-geometry.json').read_bytes()
collection = json.loads(data)
features = collection['features']
assert len(features) == 9129
geometries = [shape(feature['geometry']) for feature in features]
assert all(geometry.is_valid and not geometry.is_empty for geometry in geometries)
tree = STRtree(geometries)
pairs = 0
for i, geometry in enumerate(geometries):
    for j in tree.query(geometry, predicate='intersects'):
        if int(j) >= i:
            continue
        assert geometry.intersection(geometries[int(j)]).area == 0, (i, int(j))
        pairs += 1

dbf = (root / 'parts/tl_2020_06_tract.dbf').read_bytes()
records = struct.unpack_from('<I', dbf, 4)[0]
header, width = struct.unpack_from('<HH', dbf, 8)
field_offset = 1
geoid = None
for position in range(32, header - 1, 32):
    field = dbf[position:position + 32]
    if field[0] == 13:
        break
    name = field[:11].split(b'\0')[0].decode('ascii')
    if name == 'GEOID':
        geoid = field_offset, field[16]
    field_offset += field[16]
assert geoid is not None
ids = [dbf[header + i * width + geoid[0]:header + i * width + geoid[0] + geoid[1]].decode('ascii').strip() for i in range(records)]
by_id = {feature['properties']['geoid']: feature for feature in features}
shp = (root / 'parts/tl_2020_06_tract.shp').read_bytes()
offset = 100
source_positions = output_positions = 0
for unit in ids:
    length = struct.unpack_from('>I', shp, offset + 4)[0] * 2
    start = offset + 8
    assert struct.unpack_from('<I', shp, start)[0] == 5
    parts, count = struct.unpack_from('<II', shp, start + 36)
    points = start + 44 + 4 * parts
    originals = {struct.unpack_from('<dd', shp, points + i * 16) for i in range(count)}
    polygons = by_id[unit]['geometry']['coordinates']
    assert sum(len(polygon) for polygon in polygons) == parts
    for polygon in polygons:
        for ring in polygon:
            assert len(ring) >= 4 and ring[0] == ring[-1]
            assert all(tuple(point) in originals for point in ring)
            output_positions += len(ring)
    source_positions += count
    offset = start + length
assert offset == len(shp)
assert output_positions < source_positions
evidence = {'features':len(features), 'source_positions':source_positions, 'output_positions':output_positions, 'geometry_bytes':len(data), 'sha256':hashlib.sha256(data).hexdigest(), 'geos_valid_all_units':True, 'intersecting_pairs_checked':pairs, 'cross_unit_overlap_area':0, 'retained_vertices_from_original_binary_shp':True, 'all_source_rings_retained':True}
(root / 'topology-geometry-evidence.json').write_text(json.dumps(evidence, indent=2), encoding='utf-8')
print(json.dumps(evidence))
