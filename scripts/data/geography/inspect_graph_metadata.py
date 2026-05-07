#!/usr/bin/env python3
"""Inspect adjacency graph metadata."""

import pickle
import argparse
from pathlib import Path
import networkx as nx


def inspect_graph(state_code, year='2020'):
    """Inspect graph metadata and structure."""
    state_lower = state_code.lower()

    # Load adjacency graph
    if year == '2020':
        graph_file = Path(f'data/adjacency/{state_lower}_adjacency_{year}.pkl')
    else:
        graph_file = Path(f'data/adjacency/{year}/{state_lower}_adjacency_{year}.pkl')

    if not graph_file.exists():
        print(f"Error: Graph file not found: {graph_file}")
        return

    print(f"\n{'='*70}")
    print(f"Inspecting {state_code} ({year} Census)")
    print(f"{'='*70}\n")

    with open(graph_file, 'rb') as f:
        graph_data = pickle.load(f)

    print("Keys in graph data:")
    for key in graph_data.keys():
        print(f"  - {key}")

    print(f"\nMetadata:")
    if 'n_components' in graph_data:
        print(f"  n_components (stored): {graph_data['n_components']}")

    adjacency_list = graph_data['adjacency']
    print(f"  Number of nodes: {len(adjacency_list)}")

    total_edges = sum(len(neighbors) for neighbors in adjacency_list) / 2
    print(f"  Total edges: {int(total_edges)}")

    # Build NetworkX graph
    graph = nx.Graph()
    for node_idx, neighbors in enumerate(adjacency_list):
        if neighbors:  # Only add node if it has neighbors
            for neighbor in neighbors:
                graph.add_edge(node_idx, neighbor)
        else:
            # Isolated node with no neighbors
            graph.add_node(node_idx)

    # Find connected components
    components = list(nx.connected_components(graph))
    components.sort(key=len, reverse=True)

    print(f"  n_components (computed): {len(components)}")
    print(f"  Component sizes: {[len(c) for c in components]}")

    # Check for isolated nodes
    isolated = [node for node in graph.nodes() if graph.degree(node) == 0]
    print(f"  Isolated nodes (degree 0): {len(isolated)}")
    if isolated and len(isolated) <= 10:
        print(f"    Node indices: {isolated}")

    # Check adjacency list for empty entries
    empty_adjacency = [idx for idx, neighbors in enumerate(adjacency_list) if not neighbors]
    print(f"  Empty adjacency entries: {len(empty_adjacency)}")
    if empty_adjacency and len(empty_adjacency) <= 10:
        print(f"    Indices: {empty_adjacency}")


def main():
    """Inspect graph metadata."""
    parser = argparse.ArgumentParser(description='Inspect adjacency graph metadata')
    parser.add_argument('--state', type=str, required=True,
                        help='State code (e.g., WA)')
    parser.add_argument('--year', type=str, default='2020',
                        choices=['2020', '2010', '2000'],
                        help='Census year (default: 2020)')
    args = parser.parse_args()

    inspect_graph(args.state, args.year)


if __name__ == '__main__':
    main()
