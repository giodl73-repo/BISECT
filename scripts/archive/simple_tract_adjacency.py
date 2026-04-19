#!/usr/bin/env python3
"""Simple fast tract adjacency builder."""

import sys
from pathlib import Path
import pickle
import numpy as np
import geopandas as gpd

print("Loading tracts...")
tracts = gpd.read_parquet('data/raw/ca_tracts_2020.parquet')
print(f"Loaded {len(tracts):,} tracts")

print("\nComputing adjacency (Queen contiguity)...")
from libpysal.weights import Queen

weights = Queen.from_dataframe(tracts, use_index=False, silence_warnings=True)

print(f"  Nodes: {weights.n}")
print(f"  Edges: {weights.s0 / 2:.0f}")
print(f"  Average neighbors: {weights.mean_neighbors:.1f}")
print(f"  Components: {weights.n_components}")

# Convert to adjacency list
adjacency = [sorted(list(weights.neighbors.get(i, []))) for i in range(len(tracts))]

# Get vertex weights
vertex_weights = tracts['population'].values.astype(np.int32)

# Create mappings
geoid_to_index = {geoid: idx for idx, geoid in enumerate(tracts['GEOID'])}
index_to_geoid = {idx: geoid for geoid, idx in geoid_to_index.items()}

# Save
output_dir = Path('data/adjacency')
output_dir.mkdir(parents=True, exist_ok=True)

output_file = output_dir / 'ca_tracts_graph.pkl'

print(f"\nSaving to {output_file}...")

data = {
    'adjacency': adjacency,
    'vertex_weights': vertex_weights,
    'index_to_geoid': index_to_geoid,
    'geoid_to_index': geoid_to_index
}

with open(output_file, 'wb') as f:
    pickle.dump(data, f)

file_size = output_file.stat().st_size / 1e6
print(f"Saved! ({file_size:.1f} MB)")
print(f"\nDone! Graph saved to: {output_file}")
