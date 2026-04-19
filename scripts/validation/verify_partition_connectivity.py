#!/usr/bin/env python3
"""Verify that each partition region is internally connected."""

import pickle
from pathlib import Path

print("Loading tract graph and partition assignments...")
with open('data/processed/ca_tracts_graph.pkl', 'rb') as f:
    graph_data = pickle.load(f)

adjacency = graph_data['adjacency']
vertex_weights = graph_data['vertex_weights']
index_to_geoid = graph_data['index_to_geoid']

with open('outputs/splits/ca_first_split_tracts_assignments.pkl', 'rb') as f:
    assignments = pickle.load(f)

# Get indices for each partition
parts = [assignments[i] for i in range(len(adjacency))]

print(f"\nTotal tracts: {len(adjacency)}")
print(f"Partition 0 (CA0): {parts.count(0)} tracts")
print(f"Partition 1 (CA1): {parts.count(1)} tracts")

# Check connectivity of each partition
def check_partition_connectivity(part_id):
    """Check if a partition is internally connected."""
    # Get all indices in this partition
    part_indices = [i for i, p in enumerate(parts) if p == part_id]

    if not part_indices:
        return True, []

    # Build set for fast lookup
    part_set = set(part_indices)

    # Find connected components within this partition
    visited = set()
    components = []

    def dfs_iterative(start):
        component = []
        stack = [start]
        visited.add(start)

        while stack:
            node = stack.pop()
            component.append(node)

            # Only follow edges to nodes in the same partition
            for neighbor in adjacency[node]:
                if neighbor in part_set and neighbor not in visited:
                    visited.add(neighbor)
                    stack.append(neighbor)

        return component

    for idx in part_indices:
        if idx not in visited:
            component = dfs_iterative(idx)
            components.append(component)

    is_connected = len(components) == 1
    return is_connected, components

print("\n" + "=" * 70)
print("CONNECTIVITY CHECK:")
print("=" * 70)

for part_id in [0, 1]:
    print(f"\nPartition {part_id} (CA{part_id}):")
    is_connected, components = check_partition_connectivity(part_id)

    if is_connected:
        print(f"  OK: CONNECTED - Single component with {len(components[0])} tracts")
    else:
        print(f"  ERROR: DISCONNECTED - {len(components)} components:")
        for i, comp in enumerate(sorted(components, key=len, reverse=True)):
            comp_pop = sum(vertex_weights[idx] for idx in comp)
            print(f"    Component {i+1}: {len(comp):,} tracts, {comp_pop:,} people")
            if len(comp) <= 5:
                # Show GEOIDs for small components
                geoids = [index_to_geoid[idx] for idx in comp]
                print(f"      GEOIDs: {', '.join(geoids)}")

print("\n" + "=" * 70)

# Overall summary
all_connected = all(check_partition_connectivity(p)[0] for p in [0, 1])
if all_connected:
    print("SUCCESS: Both partitions are internally connected!")
else:
    print("ERROR: One or more partitions are disconnected!")
    print("\nThis likely means the graph partitioner (NetworkX) did not enforce")
    print("the contiguity constraint properly. We need to use gpmetis with the")
    print("-contig flag, or fix the NetworkX partition post-hoc.")

print("=" * 70)
