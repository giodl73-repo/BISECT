#!/usr/bin/env python3
"""Audit existing county-weight and DFS evidence; does not generate new maps."""

import argparse
import csv
import hashlib
import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
INPUTS = [
    'docs/experiments/nrs-v0.3-complete-tree-dfs-census-2020/node-results.csv',
    'docs/experiments/nrs-v0.3-national-bakeoff-2020/state-summary.csv',
    'configs/nrs_v0_3/standard_profile_2020.json',
    'configs/official_proposal.yml',
    'crates/bisect-cli/src/exact_cmd.rs',
    'crates/bisect-cli/src/edge_weights.rs',
    'docs/specs/2026-09-09-districting-rule-development-roadmap.md',
]


def summarize(nodes, states):
    state_ids = [row['state'] for row in states]
    if len(states) != 50 or len(set(state_ids)) != 50:
        raise ValueError('expected 50 unique State summary rows')
    expected = {row['state']: int(row['districts']) - 1 for row in states}
    if any(value < 0 for value in expected.values()) or sum(expected.values()) != 385:
        raise ValueError('invalid district schedule')
    keys = [(row['state'], row['path']) for row in nodes]
    if len(nodes) != 385 or len(set(keys)) != len(keys):
        raise ValueError('expected 385 unique State/path nodes')
    counts = Counter(row['state'] for row in nodes)
    if dict(counts) != {state: n for state, n in expected.items() if n}:
        raise ValueError('State node coverage differs from district schedule')
    for row in nodes:
        if row['path'] != 'root' and (not row['path'] or any(ch not in '01' for ch in row['path'])):
            raise ValueError('invalid binary node path')
        for field in ('assignment_match', 'objective_match'):
            if row[field] != 'true':
                raise ValueError(f'non-preserving row: {field}')
        for field in ('nrs_v0_2_fallback_activated', 'nrs_v0_3_fallback_activated'):
            if row[field] not in ('true', 'false'):
                raise ValueError(f'invalid boolean: {field}')
        a, b, c = (int(row[field]) for field in (
            'minimum_deviation_candidates', 'minimum_deviation_cut_candidates',
            'minimum_deviation_cut_partitions'))
        if not 1 <= c <= b <= a:
            raise ValueError('inconsistent candidate counts')
    fields = ('minimum_deviation_candidates', 'minimum_deviation_cut_candidates',
              'minimum_deviation_cut_partitions')
    return {
        'states': len(states), 'multi_district_states': len(counts),
        'nodes': len(nodes),
        'candidate_distributions': {
            field: dict(sorted(Counter(int(row[field]) for row in nodes).items()))
            for field in fields
        },
        'nodes_with_multiple_pre_boundary_candidates': sum(
            int(row[fields[0]]) > 1 for row in nodes),
        'nodes_where_boundary_scoring_reduced_candidate_count': sum(
            int(row[fields[0]]) > int(row[fields[1]]) for row in nodes),
        'nodes_with_multiple_post_boundary_physical_partitions': sum(
            int(row[fields[2]]) > 1 for row in nodes),
        'fallback_activations': {field: sum(row[field] == 'true' for row in nodes)
            for field in ('nrs_v0_2_fallback_activated', 'nrs_v0_3_fallback_activated')},
        'pre_boundary_physical_partition_count': None,
        'next_measurement': 'Enumerate distinct physical minimum-deviation partitions before boundary scoring; then measure weight response and repair effects.',
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    def rows(path):
        with (ROOT / path).open(encoding='utf-8', newline='') as handle:
            return list(csv.DictReader(handle))
    report = {
        'schema_version': 'county-preservation-readiness-v1',
        'status': 'diagnostic-complete-sweep-not-run',
        'source_hashes': {path: hashlib.sha256((ROOT / path).read_bytes()).hexdigest()
                          for path in INPUTS},
        'analyzer_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        'diagnostics': summarize(rows(INPUTS[0]), rows(INPUTS[1])),
        'claim_boundary': 'Existing 2020 census diagnostic only. Candidate multiplicity does not prove county-weight response, physical diversity before boundary scoring, or county-split improvement. Profiles are distinct; source files are bound for inspection, not treated as equivalent runs.',
    }
    payload = json.dumps(report, indent=2, sort_keys=True) + '\n'
    args.out.parent.mkdir(parents=True, exist_ok=True)
    with args.out.open('x', encoding='utf-8', newline='\n') as handle:
        handle.write(payload)
    print(payload)


if __name__ == '__main__':
    main()
