#!/usr/bin/env python3
"""Check adjacency graph connectivity for all states.

This script verifies that each state's adjacency graph forms a single
connected component. Disconnected graphs will cause METIS to fail or
produce invalid redistricting results.
"""

import argparse
import pickle
import sys
from pathlib import Path
import pandas as pd
import networkx as nx

# All 50 states
ALL_STATES = [
    'AL', 'AK', 'AZ', 'AR', 'CA', 'CO', 'CT', 'DE', 'FL', 'GA',
    'HI', 'ID', 'IL', 'IN', 'IA', 'KS', 'KY', 'LA', 'ME', 'MD',
    'MA', 'MI', 'MN', 'MS', 'MO', 'MT', 'NE', 'NV', 'NH', 'NJ',
    'NM', 'NY', 'NC', 'ND', 'OH', 'OK', 'OR', 'PA', 'RI', 'SC',
    'SD', 'TN', 'TX', 'UT', 'VT', 'VA', 'WA', 'WV', 'WI', 'WY'
]


def check_state_connectivity(state_code, year='2020', verbose=False):
    """Check if a state's adjacency graph is fully connected.

    Returns:
        tuple: (is_connected, num_components, component_sizes, disconnected_tracts)
    """
    state_lower = state_code.lower()

    # Load adjacency graph (unified directory structure)
    graph_file = Path(f'data/adjacency/{year}/{state_lower}_adjacency_{year}.pkl')

    if not graph_file.exists():
        return None, None, None, None

    with open(graph_file, 'rb') as f:
        graph_data = pickle.load(f)

    # Extract connectivity info from graph data
    num_components = graph_data.get('n_components', 1)
    is_connected = (num_components == 1)

    # Build NetworkX graph to find component sizes
    adjacency_list = graph_data['adjacency']
    graph = nx.Graph()
    for node_idx, neighbors in enumerate(adjacency_list):
        for neighbor in neighbors:
            graph.add_edge(node_idx, neighbor)

    components = list(nx.connected_components(graph))
    components.sort(key=len, reverse=True)
    component_sizes = [len(c) for c in components]

    disconnected_tracts = []

    if not is_connected and verbose:
        # Load tract data to identify disconnected tracts
        tracts_file = f'data/tracts/{year}/{state_lower}_tracts_{year}.parquet'

        tracts_path = Path(tracts_file)
        if tracts_path.exists():
            tracts = pd.read_parquet(tracts_file)
            index_to_geoid = graph_data['index_to_geoid']

            # Get details for disconnected components (skip main component)
            for comp_idx, comp in enumerate(components[1:], 2):
                for idx in comp:
                    geoid = index_to_geoid.get(idx, 'Unknown')
                    tract_row = tracts[tracts['GEOID'] == geoid]
                    if not tract_row.empty:
                        tract = tract_row.iloc[0]
                        disconnected_tracts.append({
                            'component': comp_idx,
                            'geoid': geoid,
                            'county': tract.get('COUNTY', 'N/A'),
                            'name': tract.get('NAME', 'N/A'),
                            'lat': tract.get('lat', 'N/A'),
                            'lon': tract.get('lon', 'N/A'),
                            'population': tract.get('POP100', 0)
                        })

    return is_connected, num_components, component_sizes, disconnected_tracts


def main():
    parser = argparse.ArgumentParser(
        description='Check adjacency graph connectivity for all states'
    )
    parser.add_argument('--year', type=str, default='2020',
                       choices=['2000', '2010', '2020'],
                       help='Census year (default: 2020)')
    parser.add_argument('--verbose', action='store_true',
                       help='Show detailed information about disconnected tracts')
    parser.add_argument('--state', type=str,
                       help='Check specific state only (e.g., CA)')

    args = parser.parse_args()

    # Determine which states to check
    states_to_check = [args.state.upper()] if args.state else ALL_STATES

    print(f"\n{'='*70}")
    print(f"Checking Adjacency Graph Connectivity - {args.year} Census")
    print(f"{'='*70}\n")

    connected_states = []
    disconnected_states = []
    missing_states = []

    for state in states_to_check:
        result = check_state_connectivity(state, args.year, args.verbose)

        if result[0] is None:
            missing_states.append(state)
            print(f"  [MISSING] {state} - adjacency graph not found")
        elif result[0]:
            connected_states.append(state)
            print(f"  [OK] {state} - fully connected ({result[2][0]} tracts)")
        else:
            disconnected_states.append((state, result))
            is_connected, num_components, component_sizes, disconnected_tracts = result
            print(f"  [FAIL] {state} - {num_components} components: {component_sizes}")

            if args.verbose and disconnected_tracts:
                print(f"         Disconnected tracts:")
                for tract in disconnected_tracts:
                    print(f"           Component {tract['component']}: "
                          f"GEOID {tract['geoid']}, {tract['county']}, "
                          f"Pop {tract['population']}")

    # Summary
    print(f"\n{'='*70}")
    print(f"CONNECTIVITY SUMMARY")
    print(f"{'='*70}")
    print(f"Connected: {len(connected_states)}/{len(states_to_check)}")

    if disconnected_states:
        print(f"\nDISCONNECTED STATES ({len(disconnected_states)}):")
        for state, (_, num_comp, sizes, _) in disconnected_states:
            print(f"  {state}: {num_comp} components - {sizes}")

        print(f"\nThese states need adjacency graphs rebuilt with proper")
        print(f"water connections to join all components into one.")
        print(f"\nTo rebuild with different water distance threshold:")
        print(f"  python scripts/data/geography/build_all_adjacency_graphs.py \\")
        print(f"         --year {args.year} --water-distance 5.0 --reset")

    if missing_states:
        print(f"\nMISSING GRAPHS ({len(missing_states)}):")
        for state in missing_states:
            print(f"  {state}")

    print(f"{'='*70}\n")

    return 0 if not disconnected_states else 1


if __name__ == '__main__':
    sys.exit(main())
