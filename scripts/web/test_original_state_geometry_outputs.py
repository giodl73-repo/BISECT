"""Independently check complete fixed-geometry workflows against retained inputs."""
import argparse
import base64
import gzip
import hashlib
import json
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('--states', default='AK')
parser.add_argument('--structure', choices=['apportion-regions', 'standard-bisect'])
args = parser.parse_args()
manifest = json.loads(Path('data/manifest.json').read_text(encoding='utf-8'))
districts_checked = 0
for state in args.states.split(','):
    assert state in manifest['states'] and state != 'DC', state
    original = Path('target/original-all-state-wasm') / state
    fixed = Path('target/original-alaska-fixed') if state == 'AK' else Path('target/original-state-geometry-fixed') / state
    if args.structure:
        fixed = Path('target/original-state-algorithm') / args.structure / state
    receipt = json.loads((original / 'receipt.json').read_text(encoding='utf-8'))
    evidence = json.loads((fixed / 'evidence.json').read_text(encoding='utf-8'))
    result = json.loads((fixed / 'result.json').read_text(encoding='utf-8'))
    assert receipt['native_source_graph_parity']
    assert evidence['state'] == state and evidence['full_native_wasm_workflow_match'] and evidence['native_yaml_assignment_parity']
    raw_graph = gzip.decompress((original / 'engine-graph.json.gz').read_bytes())
    raw_geometry = gzip.decompress((original / 'display-geometry.geojson.gz').read_bytes())
    assert hashlib.sha256(raw_graph).hexdigest() == receipt['graph_sha256'] == evidence['source_graph_sha256']
    assert hashlib.sha256(raw_geometry).hexdigest() == receipt['geometry_sha256'] == evidence['geometry_sha256']
    graph = json.loads(raw_graph)
    geometry_ids = [f['properties'].get('GEOID', f['properties'].get('geoid')) for f in json.loads(raw_geometry)['features']]
    ids, population, adjacency = graph['geoids'], graph['population'], graph['adjacency']
    assert len(set(ids)) == len(ids) == receipt['units'] == len(population) == len(adjacency)
    assert len(set(geometry_ids)) == len(geometry_ids) == len(ids) and set(geometry_ids) == set(ids)
    run = result['configured_run']['result']
    assignments = run['assignments']
    k = manifest['states'][state]['districts']['2020']
    assert set(assignments) == set(ids) and set(assignments.values()) == set(range(1, k + 1))
    total = sum(population)
    assert total == int(receipt['population']) == int(run['metrics']['population'])
    metrics = {d['district']: d for d in run['metrics']['district_metrics']}
    for district in range(1, k + 1):
        members = {i for i, unit in enumerate(ids) if assignments[unit] == district}
        assert members
        district_population = sum(population[i] for i in members)
        assert district_population == int(metrics[district]['population'])
        assert len(members) == metrics[district]['units']
        assert abs(district_population * k - total) * 200 <= total, (state, district, '0.5-percent population target')
        pending = [next(iter(members))]
        visited = set(pending)
        while pending:
            for neighbor in adjacency[pending.pop()]:
                if neighbor in members and neighbor not in visited:
                    visited.add(neighbor)
                    pending.append(neighbor)
        assert visited == members, (state, district, 'induced graph is disconnected')
        districts_checked += 1
    stages = {s['name']: s['status'] for s in result['workflow']['stages']}
    assert stages['map:districts'] == stages['report'] == 'completed'
    assert result['workflow']['file_sha256'] == evidence['files']
    assert set(result['workflow']['files_base64']) == set(evidence['files'])
    for name, encoded in result['workflow']['files_base64'].items():
        assert hashlib.sha256(base64.b64decode(encoded, validate=True)).hexdigest() == evidence['files'][name]
print(f'Independent fixed-geometry outputs: {districts_checked} districts, exact populations/identity coverage and connected induced graphs passed.')
