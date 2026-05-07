#!/usr/bin/env python3
"""Connect disconnected components in tract adjacency graph."""

import pickle
import numpy as np
import geopandas as gpd
from pathlib import Path
from scipy.spatial import cKDTree

print("Loading tract data and graph...")
tracts = gpd.read_parquet('data/raw/ca_tracts_2020.parquet')
with open('data/processed/ca_tracts_graph.pkl', 'rb') as f:
    graph_data = pickle.load(f)

adjacency = graph_data['adjacency']
vertex_weights = graph_data['vertex_weights']
index_to_geoid = graph_data['index_to_geoid']

print(f"Total tracts: {len(adjacency)}")

# Find connected components using iterative DFS
visited = [False] * len(adjacency)
components = []

def dfs_iterative(start_node):
    component = []
    stack = [start_node]
    visited[start_node] = True

    while stack:
        node = stack.pop()
        component.append(node)

        for neighbor in adjacency[node]:
            if not visited[neighbor]:
                visited[neighbor] = True
                stack.append(neighbor)

    return component

for i in range(len(adjacency)):
    if not visited[i]:
        component = dfs_iterative(i)
        components.append(component)

print(f"\nFound {len(components)} connected components:")
for i, comp in enumerate(sorted(components, key=len, reverse=True)):
    comp_pop = sum(vertex_weights[idx] for idx in comp)
    print(f"  Component {i+1}: {len(comp):,} tracts, {comp_pop:,} people")

if len(components) == 1:
    print("\nGraph is already fully connected!")
    exit(0)

# Get centroids for all tracts
centroids = tracts.geometry.centroid
centroid_coords = np.array([[p.x, p.y] for p in centroids])

# Connect components by finding nearest neighbors between them
print(f"\nConnecting {len(components)} components...")

# Sort components by size (largest first)
components_sorted = sorted(enumerate(components), key=lambda x: len(x[1]), reverse=True)

# Connect each smaller component to the main component
main_comp_idx, main_comp = components_sorted[0]
print(f"\nMain component: {len(main_comp):,} tracts")

new_edges = []

for comp_idx, comp in components_sorted[1:]:
    print(f"\nConnecting component with {len(comp):,} tracts...")

    # Build KD-tree for main component
    main_coords = centroid_coords[main_comp]
    tree = cKDTree(main_coords)

    # For each tract in the small component, find nearest in main component
    min_dist = float('inf')
    best_pair = None

    for tract_idx in comp:
        point = centroid_coords[tract_idx]
        dist, nearest_idx = tree.query(point)

        if dist < min_dist:
            min_dist = dist
            best_pair = (tract_idx, main_comp[nearest_idx])

    if best_pair:
        idx1, idx2 = best_pair
        geoid1 = index_to_geoid[idx1]
        geoid2 = index_to_geoid[idx2]

        print(f"  Connecting tract {geoid1} <-> {geoid2}")
        print(f"  Distance: {min_dist:.2f} degrees")

        # Add bidirectional edge
        adjacency[idx1].append(idx2)
        adjacency[idx1].sort()
        adjacency[idx2].append(idx1)
        adjacency[idx2].sort()

        new_edges.append((idx1, idx2))

        # Add this component to main component for next iteration
        main_comp.extend(comp)

print(f"\nAdded {len(new_edges)} new edges")

# Verify connectivity
visited = [False] * len(adjacency)
def dfs_count_iterative(start_node):
    stack = [start_node]
    visited[start_node] = True
    count = 0

    while stack:
        node = stack.pop()
        count += 1

        for neighbor in adjacency[node]:
            if not visited[neighbor]:
                visited[neighbor] = True
                stack.append(neighbor)

    return count

connected_count = dfs_count_iterative(0)
print(f"\nVerification: {connected_count:,} / {len(adjacency):,} tracts reachable from tract 0")

if connected_count == len(adjacency):
    print("SUCCESS: Graph is now fully connected!")
else:
    print(f"WARNING: Still have disconnected tracts!")

# Save updated graph
print("\nSaving updated graph...")
graph_data['adjacency'] = adjacency

output_file = Path('data/processed/ca_tracts_graph.pkl')
with open(output_file, 'wb') as f:
    pickle.dump(graph_data, f)

print(f"Updated graph saved to: {output_file}")

# Save edge list for reference
edges_file = Path('data/processed/ca_tracts_manual_edges.txt')
with open(edges_file, 'w') as f:
    f.write("# Manually added edges to connect disconnected components\n")
    for idx1, idx2 in new_edges:
        geoid1 = index_to_geoid[idx1]
        geoid2 = index_to_geoid[idx2]
        f.write(f"{geoid1},{geoid2}\n")

print(f"Manual edges saved to: {edges_file}")
print("\nDone!")
