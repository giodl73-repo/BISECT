"""Verify all retained original-source preparation receipts, including Run failures."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path('target/original-all-state-wasm')
campaign = json.loads((root / 'campaign.json').read_text(encoding='utf-8'))
manifest = json.loads(Path('data/manifest.json').read_text(encoding='utf-8'))
states = sorted(s for s in manifest['states'] if s != 'DC')
assert len(states) == campaign['completed'] == len(campaign['requested']) == 50
assert sorted(campaign['requested']) == states
assert sum(manifest['states'][s]['districts']['2020'] for s in states) == 435

def file_hash(path):
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(block)
    return digest.hexdigest()

units = population = compressed = 0
for state in states:
    receipt = json.loads((root / state / 'receipt.json').read_text(encoding='utf-8'))
    assert receipt['state'] == state and receipt['native_source_graph_parity'], state
    assert receipt['status'] in ['verified', 'failed']
    slug = manifest['states'][state]['name'].lower().replace(' ', '_')
    fips = manifest['states'][state]['fips']
    stem = Path(f'data/2020/tiger/tracts/tl_2020_{fips}_tract/tl_2020_{fips}_tract')
    original = {f'tiger.{extension}': stem.with_suffix('.' + extension) for extension in ['shp', 'dbf', 'shx']}
    original['projection'] = stem.with_suffix('.prj')
    original['geography'] = Path(f'data/2020/redistricting/{slug}/{state.lower()}geo2020.pl')
    original['population'] = Path(f'data/2020/redistricting/{slug}/{state.lower()}000012020.pl')
    assert set(original) == set(receipt['source_sha256'])
    for role, path in original.items():
        assert file_hash(path) == receipt['source_sha256'][role], (state, role)
    encoded_graph = (root / state / 'engine-graph.json.gz').read_bytes()
    encoded_geometry = (root / state / 'display-geometry.geojson.gz').read_bytes()
    graph_bytes, geometry_bytes = gzip.decompress(encoded_graph), gzip.decompress(encoded_geometry)
    assert hashlib.sha256(graph_bytes).hexdigest() == receipt['graph_sha256']
    assert hashlib.sha256(geometry_bytes).hexdigest() == receipt['geometry_sha256']
    graph = json.loads(graph_bytes)
    assert graph['state'] == state and graph['year'] == '2020' and graph['unit_kind'] == 'tract'
    ids = graph['geoids']
    assert len(ids) == len(set(ids)) == len(graph['population']) == len(graph['adjacency']) == receipt['units']
    assert all(len(unit) == 11 and unit.startswith(fips) and unit.isdigit() for unit in ids)
    assert all(isinstance(p, int) and p >= 0 for p in graph['population'])
    assert sum(graph['population']) == int(receipt['population'])
    units += len(ids)
    population += sum(graph['population'])
    compressed += len(encoded_graph) + len(encoded_geometry)
print(f'All original state sources: 50 native source graph oracles, {units} units, population {population}, {compressed} compressed derived bytes; original and derived hashes/identity/populations passed.')
(root / 'source-campaign-evidence.json').write_text(json.dumps({
    'states': 50, 'units': units, 'population': str(population), 'compressed_input_bytes': compressed,
    'current_original_hashes_match': True, 'current_derived_hashes_match': True,
    'native_source_graph_parity': True, 'scope': 'Complete original 2020 state tract preparation receipts and retained inputs, including native generation refusals. This does not establish successful generation or browser capacity for every state/option.'
}, indent=2) + '\n', encoding='utf-8')
