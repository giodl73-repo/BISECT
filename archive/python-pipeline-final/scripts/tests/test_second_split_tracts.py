#!/usr/bin/env python3
"""Test second split: 2 regions -> 4 regions."""

import sys
from pathlib import Path
import pickle

sys.path.insert(0, str(Path(__file__).parent.parent / 'src'))

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

# Load first split results
print("Loading first split assignments...")
with open('outputs/splits/ca_first_split_tracts_assignments.pkl', 'rb') as f:
    round1_assignments = pickle.load(f)

round1_parts = [round1_assignments[i] for i in range(len(vertex_weights))]

total_pop = int(vertex_weights.sum())
num_districts = 52
ideal_pop = total_pop / num_districts

print(f"\nCalifornia Redistricting - Second Split Test")
print(f"=" * 60)
print(f"Total tracts: {len(vertex_weights):,}")
print(f"Total population: {total_pop:,}")
print(f"Target districts: {num_districts}")
print(f"Ideal per district: {ideal_pop:,.0f}")

# Second split: Split each of the 2 regions into 2
# CA0 (26 districts) -> CA00 (13 districts), CA01 (13 districts)
# CA1 (26 districts) -> CA10 (13 districts), CA11 (13 districts)

results = {}

for parent_id in [0, 1]:
    parent_name = f"CA{parent_id}"
    print(f"\n{'=' * 60}")
    print(f"Splitting {parent_name} into {parent_name}0 and {parent_name}1")
    print(f"{'=' * 60}")

    # Get indices in this parent region
    parent_indices = [i for i, p in enumerate(round1_parts) if p == parent_id]
    parent_pop = sum(vertex_weights[i] for i in parent_indices)

    print(f"Parent {parent_name}: {parent_pop:,} people ({len(parent_indices):,} tracts)")

    # For 26 districts, split into 13/13
    k_left, k_right = 13, 13
    expected_left = parent_pop * 0.5
    expected_right = parent_pop * 0.5

    print(f"  Target split: {k_left}/{k_right} districts")
    print(f"  Expected left:  {expected_left:,.0f}")
    print(f"  Expected right: {expected_right:,.0f}")

    # Build subgraph for this parent region
    # Create mapping from global indices to subgraph indices
    global_to_sub = {global_idx: sub_idx for sub_idx, global_idx in enumerate(parent_indices)}

    # Build subgraph adjacency
    sub_adjacency = []
    for global_idx in parent_indices:
        neighbors = adjacency[global_idx]
        # Only include neighbors that are in the same parent region
        sub_neighbors = [global_to_sub[n] for n in neighbors if n in global_to_sub]
        sub_adjacency.append(sorted(sub_neighbors))

    # Get vertex weights for subgraph
    sub_weights = np.array([vertex_weights[i] for i in parent_indices], dtype=np.int32)

    print(f"\nRunning METIS partition for {parent_name}...")

    # Partition subgraph
    sub_parts = partition_graph(
        adjacency=sub_adjacency,
        vertex_weights=sub_weights,
        nparts=2,
        target_weights=[0.5, 0.5],
        recursive=True
    )

    # Map back to global indices
    left_global = [parent_indices[i] for i, p in enumerate(sub_parts) if p == 0]
    right_global = [parent_indices[i] for i, p in enumerate(sub_parts) if p == 1]

    left_pop = sum(vertex_weights[i] for i in left_global)
    right_pop = sum(vertex_weights[i] for i in right_global)

    left_per_district = left_pop / k_left
    right_per_district = right_pop / k_right
    left_dev = (left_per_district - ideal_pop) / ideal_pop * 100
    right_dev = (right_per_district - ideal_pop) / ideal_pop * 100

    print(f"\nResults:")
    print(f"  {parent_name}0: {left_pop:,} people ({len(left_global):,} tracts)")
    print(f"    Per district: {left_per_district:,.0f} ({left_dev:+.2f}% from ideal)")
    print(f"  {parent_name}1: {right_pop:,} people ({len(right_global):,} tracts)")
    print(f"    Per district: {right_per_district:,.0f} ({right_dev:+.2f}% from ideal)")

    results[f"{parent_name}0"] = {
        'indices': left_global,
        'pop': left_pop,
        'k': k_left,
        'per_district': left_per_district,
        'deviation': left_dev
    }
    results[f"{parent_name}1"] = {
        'indices': right_global,
        'pop': right_pop,
        'k': k_right,
        'per_district': right_per_district,
        'deviation': right_dev
    }

# Create final assignments mapping
# CA00=0, CA01=1, CA10=2, CA11=3 (for 1-based districts: 1, 2, 3, 4)
round2_parts = np.zeros(len(vertex_weights), dtype=np.int32)
for idx in results['CA00']['indices']:
    round2_parts[idx] = 0
for idx in results['CA01']['indices']:
    round2_parts[idx] = 1
for idx in results['CA10']['indices']:
    round2_parts[idx] = 2
for idx in results['CA11']['indices']:
    round2_parts[idx] = 3

print(f"\n{'=' * 60}")
print(f"ROUND 2 SUMMARY")
print(f"{'=' * 60}")

max_dev = max(abs(r['deviation']) for r in results.values())
print(f"Max deviation across all 4 regions: {max_dev:.2f}%")

if max_dev < 1:
    print(f"OK: Excellent balance!")
elif max_dev < 2:
    print(f"OK: Good balance")
else:
    print(f"WARNING: Significant imbalance")

# Generate map
print(f"\nGenerating map...")

# Load tracts for visualization
tracts = gpd.read_parquet('data/raw/ca_tracts_2020.parquet')

# Add partition assignments
tracts['partition'] = round2_parts

# Create map
fig, ax = plt.subplots(1, 1, figsize=(14, 12))

colors = ['#e74c3c', '#f39c12', '#3498db', '#9b59b6']

for part_id in [0, 1, 2, 3]:
    partition_data = tracts[tracts['partition'] == part_id]
    partition_data.plot(
        ax=ax,
        color=colors[part_id],
        edgecolor='white',
        linewidth=0.1,
        alpha=0.75
    )

# Add district numbers (1-based)
for part_id in [0, 1, 2, 3]:
    partition_data = tracts[tracts['partition'] == part_id]
    centroid = partition_data.geometry.unary_union.centroid
    district_num = part_id + 1
    ax.text(centroid.x, centroid.y, str(district_num),
            fontsize=40, fontweight='bold', ha='center', va='center',
            color='white', bbox=dict(boxstyle='round,pad=0.5', facecolor='black', alpha=0.7))

ax.set_axis_off()
ax.set_title('California Round 2: 4 Regions (13 districts each)\nTract-Level Redistricting',
             fontsize=16, fontweight='bold', pad=20)

# Legend
region_names = ['CA00', 'CA01', 'CA10', 'CA11']
legend_elements = []
for i, name in enumerate(region_names):
    r = results[name]
    legend_elements.append(
        mpatches.Patch(facecolor=colors[i], edgecolor='white',
                      label=f"District {i+1} ({name}): {r['pop']:,} people ({len(r['indices']):,} tracts)\n  {r['k']} districts x {r['per_district']:,.0f} avg")
    )

ax.legend(handles=legend_elements, loc='lower right',
         frameon=True, fancybox=True, shadow=True, fontsize=9)

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

map_file = output_dir / 'ca_second_split_tracts.png'
print(f"  Saving to: {map_file}")
plt.savefig(map_file, dpi=300, bbox_inches='tight')
print(f"  Map saved!")
plt.close()

# Also save assignments
assignments_file = output_dir / 'ca_second_split_tracts_assignments.pkl'
with open(assignments_file, 'wb') as f:
    pickle.dump({i: int(round2_parts[i]) for i in range(len(round2_parts))}, f)
print(f"  Assignments saved to: {assignments_file}")

print(f"\n{'=' * 60}")
print(f"SUCCESS!")
print(f"{'=' * 60}")
print(f"\nFiles:")
print(f"  Map: {map_file}")
print(f"  Assignments: {assignments_file}")
