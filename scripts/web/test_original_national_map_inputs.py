"""Independently check every actual plan in the full national source package."""
import base64
import gzip
import hashlib
import json
from pathlib import Path

root = Path('target/original-national-map-stream')
manifest = json.loads(Path('data/manifest.json').read_text(encoding='utf-8'))
package = json.loads((root / 'package.json').read_text(encoding='utf-8'))
states = sorted(s for s in manifest['states'] if s != 'DC')
assert len(states) == 50 and sorted(package['states']) == states
rows = []
for state in states:
    original = Path('target/original-all-state-wasm') / state
    receipt = json.loads((original / 'receipt.json').read_text(encoding='utf-8'))
    graph_bytes = gzip.decompress((original / 'engine-graph.json.gz').read_bytes())
    assert hashlib.sha256(graph_bytes).hexdigest() == receipt['graph_sha256']
    graph = json.loads(graph_bytes)
    job = package['states'][state]
    plan_bytes = base64.b64decode(job['plan']['base64'], validate=True)
    plan = json.loads(plan_bytes)['plan']
    compressed = base64.b64decode(job['geometry']['base64'], validate=True)
    assert job['geometry_format'] == 'gzip'
    assert compressed == (original / 'display-geometry.geojson.gz').read_bytes()
    assert hashlib.sha256(gzip.decompress(compressed)).hexdigest() == receipt['geometry_sha256']
    ids, populations, adjacency = graph['geoids'], graph['population'], graph['adjacency']
    assert graph['state'] == state and graph['year'] == '2020'
    assert plan['units']['state'] == state and str(plan['units']['year']) == '2020'
    assert plan['units']['unit_ids'] == ids
    assert len(ids) == len(set(ids)) == receipt['units'] == len(plan['assignment'])
    assert all(isinstance(p, int) and p >= 0 for p in populations)
    k = manifest['states'][state]['districts']['2020']
    assignments = plan['assignment']
    assert plan['k'] == k and set(assignments) == set(range(k))
    assert all(type(label) is int for label in assignments)
    total = sum(populations)
    assert total == int(receipt['population'])
    district_populations = []
    for district in range(k):
        members = {i for i, label in enumerate(assignments) if label == district}
        assert members
        population = sum(populations[i] for i in members)
        assert abs(population * k - total) * 200 <= total, (state, district, '0.5 percent target')
        pending = [next(iter(members))]
        visited = set(pending)
        while pending:
            for neighbor in adjacency[pending.pop()]:
                if neighbor in members and neighbor not in visited:
                    visited.add(neighbor)
                    pending.append(neighbor)
        assert visited == members, (state, district, 'disconnected induced graph')
        district_populations.append(population)
    rows.append({'state': state, 'units': len(ids), 'population': str(total),
                 'districts': k, 'district_populations': district_populations,
                 'plan_sha256': hashlib.sha256(plan_bytes).hexdigest(),
                 'geometry_sha256': receipt['geometry_sha256']})
assert sum(r['units'] for r in rows) == 84208
assert sum(r['districts'] for r in rows) == 435
assert sum(int(r['population']) for r in rows) == 330759736
(root / 'independent-input-evidence.json').write_text(json.dumps({
    'states': rows,
    'scope': 'All actual national-package plans, exact retained original prepared geometry, integer populations, allocation and induced adjacency connectivity. Does not certify legal geographic contiguity, source authenticity or optimality.'
}, indent=2) + '\n', encoding='utf-8')
print('Independent national inputs: 50 states, 84208 tracts, 435 districts; exact retained geometry/plan identities, integer population targets and induced-graph contiguity passed.')
