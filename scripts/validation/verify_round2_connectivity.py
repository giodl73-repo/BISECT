#!/usr/bin/env python3
"""Verify that all 4 round 2 regions are internally connected."""

import pickle

print("Loading tract graph and round 2 assignments...")
with open('data/processed/ca_tracts_graph.pkl', 'rb') as f:
    graph_data = pickle.load(f)

adjacency = graph_data['adjacency']
vertex_weights = graph_data['vertex_weights']
index_to_geoid = graph_data['index_to_geoid']

with open('outputs/splits/ca_second_split_tracts_assignments.pkl', 'rb') as f:
    assignments = pickle.load(f)

parts = [assignments[i] for i in range(len(adjacency))]

print(f"\nTotal tracts: {len(adjacency)}")
for part_id in range(4):
    count = parts.count(part_id)
    pop = sum(vertex_weights[i] for i, p in enumerate(parts) if p == part_id)
    print(f"District {part_id+1}: {count} tracts, {pop:,} people")

# Check connectivity of each region
def check_connectivity(part_id):
    """Check if a partition is internally connected."""
    part_indices = [i for i, p in enumerate(parts) if p == part_id]

    if not part_indices:
        return True, []

    part_set = set(part_indices)
    visited = set()
    components = []

    def dfs_iterative(start):
        component = []
        stack = [start]
        visited.add(start)

        while stack:
            node = stack.pop()
            component.append(node)

            for neighbor in adjacency[node]:
                if neighbor in part_set and neighbor not in visited:
                    visited.add(neighbor)
                    stack.append(neighbor)

        return component

    for idx in part_indices:
        if idx not in visited:
            component = dfs_iterative(idx)
            components.append(component)

    return len(components) == 1, components

print("\n" + "=" * 70)
print("CONNECTIVITY CHECK:")
print("=" * 70)

region_names = ['CA00', 'CA01', 'CA10', 'CA11']

all_connected = True
for part_id in range(4):
    print(f"\nDistrict {part_id+1} ({region_names[part_id]}):")
    is_connected, components = check_connectivity(part_id)

    if is_connected:
        print(f"  OK: CONNECTED - Single component with {len(components[0])} tracts")
    else:
        print(f"  ERROR: DISCONNECTED - {len(components)} components:")
        for i, comp in enumerate(sorted(components, key=len, reverse=True)):
            comp_pop = sum(vertex_weights[idx] for idx in comp)
            print(f"    Component {i+1}: {len(comp):,} tracts, {comp_pop:,} people")
        all_connected = False

print("\n" + "=" * 70)
if all_connected:
    print("SUCCESS: All 4 districts are internally connected!")
else:
    print("ERROR: One or more districts are disconnected!")
print("=" * 70)
