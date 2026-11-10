"""Independently check a saved national Laboratory run against its retained graphs."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path


def digest(data):
    return hashlib.sha256(data).hexdigest()


def readback(assets, project_file):
    project_bytes = project_file.read_bytes()
    project = json.loads(project_bytes)
    catalog = json.loads((assets / 'catalog.json').read_bytes())
    experiment = project['experiment']
    config = experiment['config']
    assert config['mode'] == 'national' and config['year'] == '2020'
    assert experiment['status'] == 'completed' and len(experiment['states']) == 50
    assert config['seed'] == '18446744073709551615'
    assert set(config['states']) == {state['code'] for state in experiment['states']} == set(project['assignments'])
    totals = {'units': 0, 'districts': 0, 'components': 0}
    failures = {}
    for state in experiment['states']:
        code, metrics = state['code'], state['metrics']
        scope = code + ':2020'
        reference = catalog['graphs'][scope]['graph_ref']
        graph_bytes = (assets / reference).read_bytes()
        assert digest(graph_bytes) == catalog['assets'][reference] == project['inputs'][code]['prepared_graph_sha256']
        graph = json.loads(graph_bytes)
        ids, population, adjacency = (graph[key] for key in ('geoids', 'population', 'adjacency'))
        assignment = project['assignments'][code]
        assert set(assignment) == set(ids) and len(ids) == len(population) == len(adjacency)
        assert metrics['recorded_options']['seed'] == config['seed']
        assert metrics['engine_provenance']['wasm_sha256'] == catalog['wasm_sha256']
        districts = [assignment[geoid] for geoid in ids]
        units = Counter(districts)
        pops = Counter()
        for district, pop in zip(districts, population):
            pops[district] += pop
        seen = set()
        components = Counter()
        for start, district in enumerate(districts):
            if start in seen:
                continue
            stack = [start]
            seen.add(start)
            components[district] += 1
            while stack:
                unit = stack.pop()
                for neighbor in adjacency[unit]:
                    if districts[neighbor] == district and neighbor not in seen:
                        seen.add(neighbor)
                        stack.append(neighbor)
        expected = set(range(1, metrics['district_count'] + 1))
        assert set(units) == set(components) == expected
        assert len(seen) == len(ids) == metrics['units'] and sum(population) == metrics['population']
        for row in metrics['districts']:
            d = row['district']
            assert (row['units'], row['population'], row['components']) == (units[d], pops[d], components[d])
        assert metrics['contiguous'] == all(c == 1 for c in components.values())
        ideal = sum(population) / len(expected)
        deviation = max(abs(pop - ideal) / ideal * 100 for pop in pops.values())
        assert abs(deviation - metrics['max_deviation_percent']) < 1e-8
        assert metrics['balance_passed'] == (deviation <= config['balance_tolerance'])
        if not metrics['balance_passed']:
            failures[code] = deviation
        totals['units'] += len(ids)
        totals['districts'] += len(expected)
        totals['components'] += sum(components.values())
    assert totals['units'] == 84208 and totals['districts'] == 435
    return {'status': 'passed-independent-retained-national-assignment-readback', 'totals': totals,
            'balance_tolerance_percent': config['balance_tolerance'], 'balance_misses': failures,
            'project_sha256': digest(project_bytes), 'engine_sha256': catalog['wasm_sha256']}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('assets', type=Path)
    parser.add_argument('project', type=Path)
    parser.add_argument('receipt', type=Path)
    args = parser.parse_args()
    with args.receipt.open('x', encoding='utf-8') as output:
        json.dump(readback(args.assets, args.project), output, indent=2)
