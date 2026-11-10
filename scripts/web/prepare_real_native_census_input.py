"""Check original sources and encode documented RADJ independently of Rust."""
import hashlib
import json
import struct
import sys
from pathlib import Path
kind = sys.argv[1]
prefixes = {'county': 'target/real-census-preparation/county-national/tl_2020_us_county', 'block-group': 'target/authentic-bg/RI/tiger/tl_2020_44_bg', 'block': 'target/real-census-preparation/RI/block/tl_2020_44_tabblock20'}
assert kind in prefixes
root = Path('target/real-native-census-input/RI') / kind
root.mkdir(parents=True, exist_ok=True)
prepared = json.loads((Path('target/real-census-preparation/RI') / kind / 'prepared.json').read_text(encoding='utf-8'))
sha = lambda b: hashlib.sha256(b).hexdigest()
assert sha(prepared['graph_json'].encode()) == prepared['graph_sha256']
assert sha(prepared['geometry_json'].encode()) == prepared['geometry_sha256']
sources = {k: Path(prefixes[kind] + '.' + k) for k in ['shp', 'dbf', 'shx', 'prj']}
sources.update(geography=Path('data/2020/redistricting/rhode_island/rigeo2020.pl'), population=Path('data/2020/redistricting/rhode_island/ri000012020.pl'))
proof = {}
for key, path in sources.items():
    data = path.read_bytes()
    expected = prepared['sources'][key] if key == 'prj' else prepared['sources'][key + '_decoded']
    assert sha(data) == expected['sha256' if key == 'prj' else 'decoded_sha256'], (kind, key)
    assert len(data) == expected['byte_length' if key == 'prj' else 'decoded_byte_length']
    proof[key] = {'path': str(path), 'bytes': len(data), 'sha256': sha(data)}
g = json.loads(prepared['graph_json'])
assert g['unit_kind'] == kind and g['state'] == 'RI' and g['year'] == '2020'
assert sum(g['population']) == 1097379
n = len(g['geoids'])
body = bytearray(b'RADJ' + struct.pack('<III', 3, n, len(g['edges'])))
body += struct.pack('<' + 'q' * n, *g['population'])
for neighbors in g['adjacency']:
    body += struct.pack('<I', len(neighbors)) + struct.pack('<' + 'I' * len(neighbors), *neighbors)
body += struct.pack('<I', len(g['edges']))
for u, v, w in g['edges']:
    body += struct.pack('<IId', u, v, w)
body += struct.pack('<I', 1)
body += struct.pack('<' + 'd' * n, *g['exterior_perimeters'])
body += struct.pack('<' + 'd' * n, *g['areas'])
(root / 'original.adj.bin').write_bytes(body)
(root / 'original_geoids.json').write_text(json.dumps(dict(enumerate(g['geoids']))), encoding='utf-8')
(root / 'original_centroids.json').write_text(json.dumps({'centroids': g['centroids']}), encoding='utf-8')
(root / 'original-source-evidence.json').write_text(json.dumps({'source_files': proof, 'graph_sha256': prepared['graph_sha256'], 'geometry_sha256': prepared['geometry_sha256'], 'radj_sha256': sha(body)}, indent=2), encoding='utf-8')
print(f'{kind}: original source/derived graph hashes verified; independently encoded {n} native binary units.')
