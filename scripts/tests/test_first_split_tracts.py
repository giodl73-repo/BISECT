#!/usr/bin/env python3
"""Test first split using proper tract data and generate map."""

import sys
from pathlib import Path
import pickle

sys.path.insert(0, str(Path(__file__).parent.parent / 'src'))

from apportionment.data.census import load_blocks
from apportionment.partition.metis_wrapper import partition_graph
import geopandas as gpd
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import numpy as np

# Load tract graph
print("Loading tract adjacency graph...")
with open('data/processed/ca_tracts_graph.pkl', 'rb') as f:
    graph_data = pickle.load(f)

adjacency = graph_data['adjacency']
vertex_weights = graph_data['vertex_weights']
index_to_geoid = graph_data['index_to_geoid']

total_pop = int(vertex_weights.sum())
num_districts = 52
ideal_pop = total_pop / num_districts

print(f"\nCalifornia Redistricting - First Split Test")
print(f"=" * 60)
print(f"Total tracts: {len(vertex_weights):,}")
print(f"Total population: {total_pop:,}")
print(f"Target districts: {num_districts}")
print(f"Ideal per district: {ideal_pop:,.0f}")

# First split: 52 -> 26/26
k_left, k_right = 26, 26
target_weights = [0.5, 0.5]

print(f"\nFirst Split:")
print(f"  Left:  {k_left} districts")
print(f"  Right: {k_right} districts")

expected_left = total_pop * 0.5
expected_right = total_pop * 0.5

print(f"  Expected left:  {expected_left:,.0f}")
print(f"  Expected right: {expected_right:,.0f}")

# Run METIS
print(f"\nRunning METIS partition...")

parts = partition_graph(
    adjacency=adjacency,
    vertex_weights=vertex_weights,
    nparts=2,
    target_weights=target_weights,
    recursive=True
)

# Analyze results
left_indices = [i for i, p in enumerate(parts) if p == 0]
right_indices = [i for i, p in enumerate(parts) if p == 1]

left_pop = sum(vertex_weights[i] for i in left_indices)
right_pop = sum(vertex_weights[i] for i in right_indices)

print(f"\nResults:")
print(f"  Left:  {left_pop:,} people ({len(left_indices):,} tracts)")
print(f"    Deviation: {(left_pop - expected_left) / expected_left * 100:+.2f}%")
print(f"  Right: {right_pop:,} people ({len(right_indices):,} tracts)")
print(f"    Deviation: {(right_pop - expected_right) / expected_right * 100:+.2f}%")

left_per_district = left_pop / k_left
right_per_district = right_pop / k_right
left_dev = (left_per_district - ideal_pop) / ideal_pop * 100
right_dev = (right_per_district - ideal_pop) / ideal_pop * 100

print(f"\nPer-district average:")
print(f"  Left:  {left_per_district:,.0f} ({left_dev:+.2f}% from ideal)")
print(f"  Right: {right_per_district:,.0f} ({right_dev:+.2f}% from ideal)")

max_dev = max(abs(left_dev), abs(right_dev))
print(f"  Max deviation: {max_dev:.2f}%")

if max_dev < 1:
    print(f"\nOK: Excellent balance!")
elif max_dev < 2:
    print(f"\nOK: Good balance")
else:
    print(f"\nWARNING: Significant imbalance")

# Generate map
print(f"\nGenerating map...")

# Load tracts for visualization
tracts = gpd.read_parquet('data/raw/ca_tracts_2020.parquet')

# Add partition assignments
tracts['partition'] = parts

# Create map
fig, ax = plt.subplots(1, 1, figsize=(14, 12))

colors = ['#e74c3c', '#3498db']

for part_id in [0, 1]:
    partition_data = tracts[tracts['partition'] == part_id]
    partition_data.plot(
        ax=ax,
        color=colors[part_id],
        edgecolor='white',
        linewidth=0.1,
        alpha=0.75
    )

# Add district numbers (1-based)
for part_id in [0, 1]:
    partition_data = tracts[tracts['partition'] == part_id]
    centroid = partition_data.geometry.unary_union.centroid
    district_num = part_id + 1
    ax.text(centroid.x, centroid.y, str(district_num),
            fontsize=40, fontweight='bold', ha='center', va='center',
            color='white', bbox=dict(boxstyle='round,pad=0.5', facecolor='black', alpha=0.7))

ax.set_axis_off()
ax.set_title('California Round 1: 2 Regions (26 districts each)\nTract-Level Redistricting',
             fontsize=16, fontweight='bold', pad=20)

# Legend
legend_elements = [
    mpatches.Patch(facecolor=colors[0], edgecolor='white',
                  label=f"District 1 (CA0): {left_pop:,} people ({len(left_indices):,} tracts)\n  {k_left} districts x {left_per_district:,.0f} avg"),
    mpatches.Patch(facecolor=colors[1], edgecolor='white',
                  label=f"District 2 (CA1): {right_pop:,} people ({len(right_indices):,} tracts)\n  {k_right} districts x {right_per_district:,.0f} avg")
]

ax.legend(handles=legend_elements, loc='lower right',
         frameon=True, fancybox=True, shadow=True, fontsize=11)

# Add stats text box
textstr = f'Total Population: {total_pop:,}\n'
textstr += f'Target: 52 districts x {ideal_pop:,.0f} people\n'
textstr += f'Max Deviation: {max_dev:.2f}%'

props = dict(boxstyle='round', facecolor='wheat', alpha=0.8)
ax.text(0.02, 0.98, textstr, transform=ax.transAxes, fontsize=10,
        verticalalignment='top', bbox=props)

plt.tight_layout()

# Save
output_dir = Path('outputs/splits')
output_dir.mkdir(parents=True, exist_ok=True)

map_file = output_dir / 'ca_first_split_tracts.png'
print(f"  Saving to: {map_file}")
plt.savefig(map_file, dpi=300, bbox_inches='tight')
print(f"  Map saved!")
plt.close()

# Also save assignments
assignments_file = output_dir / 'ca_first_split_tracts_assignments.pkl'
with open(assignments_file, 'wb') as f:
    pickle.dump({i: int(parts[i]) for i in range(len(parts))}, f)
print(f"  Assignments saved to: {assignments_file}")

print(f"\n" + "=" * 60)
print(f"SUCCESS!")
print(f"=" * 60)
print(f"\nFiles:")
print(f"  Map: {map_file}")
print(f"  Assignments: {assignments_file}")
