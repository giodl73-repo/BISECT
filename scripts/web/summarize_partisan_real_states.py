"""Independently count population/connectivity outcomes of the completed gate."""
import argparse
import hashlib
import json
import math
from collections import Counter
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('catalog', nargs='?', type=Path, default=Path('dist/wasm-lab-build'))
args = parser.parse_args()
gate = json.loads(Path('target/partisan-real-state-verification.json').read_text())
manifest = json.loads((args.catalog / 'catalog.json').read_text())
searches = ['single', 'multi', 'percentile', 'convergence', 'bisection-ensemble']
assert len(manifest['graphs']) == 50
assert gate['real_state_native_comparisons'] == gate['real_state_project_export_roundtrips'] == 250
digest = hashlib.sha256((args.catalog / 'bisect_wasm.wasm').read_bytes()).hexdigest()
outcomes = []
for key, entry in manifest['graphs'].items():
    code, year = key.split(':')
    graph = json.loads((args.catalog / entry['graph_ref']).read_text())
    for search in searches:
        project = json.loads(Path(f'target/partisan-{code}-{search}.bisect').read_text())
        config = project['experiment']['config']
        assert config['states'] == [code] and config['year'] == year and config['search'] == search
        assert config['weights'] == 'partisan' and config['metis_objective'] == 'volume' and config['metis_trials'] == 3
        state = project['experiment']['states'][0]
        assert state['status'] == 'completed'
        metrics = state['metrics']
        assert metrics['engine_provenance']['wasm_sha256'] == digest
        labels = [project['assignments'][code][geoid] for geoid in graph['geoids']]
        k = metrics['district_count']
        assert set(labels) == set(range(1, k + 1))
        sizes, populations = Counter(labels), Counter()
        for label, population in zip(labels, graph['population']):
            populations[label] += population
        components = Counter()
        seen = set()
        for start, label in enumerate(labels):
            if start in seen:
                continue
            components[label] += 1
            seen.add(start)
            stack = [start]
            while stack:
                u = stack.pop()
                for v in graph['adjacency'][u]:
                    if v not in seen and labels[v] == label:
                        seen.add(v)
                        stack.append(v)
        total = sum(graph['population'])
        ideal = total / k
        max_deviation = max(abs(populations[d] / ideal - 1) * 100 for d in populations)
        tolerance = config['balance_tolerance']
        balanced = max_deviation <= tolerance + 1e-9
        contiguous = all(components[d] == 1 for d in populations)
        assert metrics['population'] == total
        assert math.isclose(metrics['max_deviation_percent'], max_deviation, abs_tol=1e-9)
        assert metrics['balance_passed'] == metrics['within_requested_tolerance'] == balanced
        assert metrics['contiguous'] == contiguous
        for district in metrics['district_metrics']:
            label = district['district']
            assert district['population'] == populations[label]
            assert district['units'] == sizes[label]
            assert district['components'] == components[label]
        outcomes.append({'state': code, 'search': search, 'units': len(labels), 'districts': k,
                         'balanced': balanced, 'contiguous': contiguous,
                         'max_deviation_percent': max_deviation, 'tolerance_percent': tolerance})
result = {'wasm_sha256': digest, 'states': 50, 'cases': len(outcomes),
          'exact_native_wasm_cases': gate['real_state_native_comparisons'],
          'independent_population_and_connectivity_checks': len(outcomes),
          'balance_misses': sum(not o['balanced'] for o in outcomes),
          'disconnected_plans': sum(not o['contiguous'] for o in outcomes),
          'partisan_data': 'synthetic stripe shares, not election results',
          'scope': '2020 tract graphs, congressional counts, five standard-bisection searches, volume objective, three METIS trials, seed 42, 10 percent tolerance',
          'outcomes': outcomes}
Path('target/partisan-real-state-outcomes.json').write_text(json.dumps(result, indent=2) + '\n')
gate['balance_misses'] = result['balance_misses']
gate['disconnected'] = result['disconnected_plans']
gate['constraint_counts_source'] = 'independently verified from all 250 saved plans by scripts/web/summarize_partisan_real_states.py'
Path('target/partisan-real-state-verification.json').write_text(json.dumps(gate, indent=2) + '\n')
print(f"{result['cases']} real-state comparisons: independent population/connectivity checks passed; "
      f"{result['balance_misses']} balance misses, {result['disconnected_plans']} disconnected plans.")
