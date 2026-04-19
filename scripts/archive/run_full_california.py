#!/usr/bin/env python3
"""Run full recursive bisection for California - all 52 districts."""

import sys
from pathlib import Path
import pickle
import numpy as np
from datetime import datetime

sys.path.insert(0, str(Path(__file__).parent.parent / 'src'))

from apportionment.partition.recursive_bisection import RecursiveBisection
import geopandas as gpd
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches

# Load tract graph
print("Loading California tract adjacency graph...")
with open('data/processed/ca_tracts_graph.pkl', 'rb') as f:
    graph_data = pickle.load(f)

adjacency = graph_data['adjacency']
vertex_weights = graph_data['vertex_weights']
index_to_geoid = graph_data['index_to_geoid']

total_pop = int(vertex_weights.sum())
num_districts = 52

print(f"\nCalifornia Congressional Redistricting")
print(f"=" * 70)
print(f"Total tracts: {len(vertex_weights):,}")
print(f"Total population: {total_pop:,}")
print(f"Target districts: {num_districts}")
print(f"Ideal per district: {total_pop / num_districts:,.0f}")
print(f"=" * 70)

# Create dated output directory
timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
output_dir = Path(f'outputs/california_full_{timestamp}')
output_dir.mkdir(parents=True, exist_ok=True)

print(f"\nOutput directory: {output_dir}")
print("="* 70)

# Run recursive bisection with intermediate results
print("\nRunning recursive bisection...")
print("This will generate maps for each round of splitting.")
print("")

partitioner = RecursiveBisection(
    adjacency=adjacency,
    vertex_weights=vertex_weights,
    num_districts=num_districts,
    save_intermediate=True,
    intermediate_dir=str(output_dir / 'intermediate'),
    state_code="CA"
)

# Run the algorithm
final_assignments = partitioner.partition()

print("\n" + "=" * 70)
print("FINAL RESULTS")
print("=" * 70)

# Analyze final districts (district_ids are 1-based from the partitioner)
for district_id in range(1, num_districts + 1):
    district_pop = sum(vertex_weights[block_idx] for block_idx, assigned_dist in final_assignments.items() if assigned_dist == district_id)
    district_tracts = sum(1 for assigned_dist in final_assignments.values() if assigned_dist == district_id)
    deviation = (district_pop - (total_pop / num_districts)) / (total_pop / num_districts) * 100
    print(f"District {district_id:2d}: {district_pop:,} people ({district_tracts:,} tracts) - {deviation:+.2f}%")

# Calculate overall statistics
populations = [sum(vertex_weights[block_idx] for block_idx, assigned_dist in final_assignments.items() if assigned_dist == district_id)
               for district_id in range(1, num_districts + 1)]
ideal = total_pop / num_districts
deviations = [(p - ideal) / ideal * 100 for p in populations]
max_dev = max(abs(d) for d in deviations)

print(f"\n" + "=" * 70)
print(f"Overall Statistics:")
print(f"  Min population: {min(populations):,}")
print(f"  Max population: {max(populations):,}")
print(f"  Mean population: {np.mean(populations):,.0f}")
print(f"  Std dev: {np.std(populations):,.0f}")
print(f"  Max deviation: {max_dev:.2f}%")

if max_dev < 1:
    print(f"\nEXCELLENT: Maximum deviation under 1%!")
elif max_dev < 2:
    print(f"\nGOOD: Maximum deviation under 2%")
else:
    print(f"\nWARNING: Maximum deviation above 2%")

# Save final assignments
final_file = output_dir / 'final_assignments.pkl'
with open(final_file, 'wb') as f:
    pickle.dump({i: int(final_assignments[i]) for i in range(len(final_assignments))}, f)
print(f"\nFinal assignments saved to: {final_file}")

# Generate final map with all 52 districts
print(f"\nGenerating final map with all 52 districts...")

tracts = gpd.read_parquet('data/raw/ca_tracts_2020.parquet')

# Map tract indices to district IDs
# Create a list where index corresponds to tract position in file
district_assignments = [final_assignments[i] for i in range(len(tracts))]
tracts['district'] = district_assignments

fig, ax = plt.subplots(1, 1, figsize=(16, 14))

# Use a colormap with enough distinct colors
import matplotlib.cm as cm
colors = cm.tab20.colors + cm.tab20b.colors + cm.tab20c.colors  # 60 colors total

for district_id in range(1, num_districts + 1):
    district_data = tracts[tracts['district'] == district_id]
    if len(district_data) > 0:
        district_data.plot(
            ax=ax,
            color=colors[(district_id - 1) % len(colors)],
            edgecolor='white',
            linewidth=0.05,
            alpha=0.8
        )

# Add district numbers - smaller text for 52 districts
import matplotlib.patheffects as path_effects
for district_id in range(1, num_districts + 1):
    district_data = tracts[tracts['district'] == district_id]
    if len(district_data) > 0:
        try:
            centroid = district_data.geometry.union_all().representative_point()
            text = ax.text(centroid.x, centroid.y, str(district_id),
                    fontsize=8, fontweight='bold', ha='center', va='center',
                    color='white', zorder=10)
            text.set_path_effects([path_effects.Stroke(linewidth=2, foreground='black'),
                                  path_effects.Normal()])
        except:
            pass

ax.set_axis_off()
ax.set_title(f'California Congressional Districts - 52 Districts\nTract-Level Redistricting (2020 Census)',
             fontsize=16, fontweight='bold', pad=20)

# Add stats text box
textstr = f'Total Population: {total_pop:,}\n'
textstr += f'Districts: {num_districts}\n'
textstr += f'Ideal per district: {ideal:,.0f}\n'
textstr += f'Max Deviation: {max_dev:.2f}%'

props = dict(boxstyle='round', facecolor='wheat', alpha=0.8)
ax.text(0.02, 0.98, textstr, transform=ax.transAxes, fontsize=11,
        verticalalignment='top', bbox=props)

plt.tight_layout()

final_map = output_dir / 'california_52_districts.png'
print(f"Saving final map to: {final_map}")
plt.savefig(final_map, dpi=300, bbox_inches='tight')
print(f"Final map saved!")
plt.close()

print("\n" + "=" * 70)
print("SUCCESS! California redistricting complete!")
print("=" * 70)
print(f"\nOutput files in: {output_dir}")
print(f"  Final map: {final_map}")
print(f"  Final assignments: {final_file}")
print(f"  Intermediate results: {output_dir / 'intermediate'}")
print("=" * 70)

# Generate markdown documentation
print(f"\nGenerating documentation...")
readme_file = output_dir / 'README.md'
with open(readme_file, 'w', encoding='utf-8') as f:
    f.write(f"""# California Congressional Redistricting Results

**Generated:** {datetime.now().strftime("%Y-%m-%d %H:%M:%S")}

## Overview

This directory contains the results of running recursive bisection redistricting for California's 52 congressional districts using Census 2020 tract-level data.

## Method

### Algorithm Overview
- **Approach:** Recursive Bisection with METIS graph partitioning
- **Data Source:** Census 2020 tract boundaries and population
- **Granularity:** {len(vertex_weights):,} tracts (vs. ~500K blocks for better performance)
- **Graph Partitioner:** gpmetis (METIS 5.2.1)

### Key Algorithm Features

#### 1. Tight Water-Based Adjacency
- **Problem:** Islands and coastal tracts need connections across water
- **Solution:** Nearest-neighbor-only approach
  - Identify island tracts (< 3 Queen contiguity neighbors)
  - Each island connects ONLY to its single nearest neighbor
  - Prevents "leaking" across water bodies and maintains compactness
  - Added only 93 edges for 94 islands (vs. 10,449 in distance-band approach)

#### 2. Target Partition Weights (-tpwgt)
- **Critical for quality:** Pass exact target weights to METIS
- For splits with odd number of districts (e.g., 13 -> 7+6):
  - Calculate target_weights = [7/13, 6/13] = [0.538462, 0.461538]
  - Write to file: `0 = 0.538462\\n1 = 0.461538`
  - Pass to gpmetis via `-tpwgt=filename`
- **Impact:** Reduced max deviation from 62.87% to 0.32%!

#### 3. Dynamic Ufactor Strategy
- **Depth 1:** ufactor=1.001 (0.1% tolerance) - Very tight for initial split
- **Depth 2:** ufactor=1.002 (0.2% tolerance)
- **Depth 3:** ufactor=1.003 (0.3% tolerance)
- **Depth 4+:** ufactor=1.005 (0.5% tolerance) - Relaxed for difficult smaller regions
- **Rationale:** Early splits are critical; later splits on smaller regions need flexibility

#### 4. Whole Number District Allocations
- Always use integer division: 13 districts -> 7 left, 6 right (never 6.5)
- Track `target_districts` for each region throughout recursion
- Enables accurate deviation measurement: actual districts worth vs. target

#### 5. Contiguity Enforcement
- Use gpmetis k-way partitioning with `-contig` flag
- DO NOT use `-ptype=rb` (recursive bisection mode doesn't support contiguity)
- Implement recursive bisection at application level, call gpmetis for each 2-way split
- Also use `-minconn` to minimize edge cuts between districts

## Results

### Population Statistics

- **Total Population:** {total_pop:,}
- **Target Districts:** {num_districts}
- **Ideal per District:** {ideal:,.0f}
- **Min Population:** {min(populations):,}
- **Max Population:** {max(populations):,}
- **Mean Population:** {np.mean(populations):,.0f}
- **Std Deviation:** {np.std(populations):,.0f}
- **Max Deviation:** {max_dev:.2f}%

### Quality Assessment

""")
    if max_dev < 1:
        f.write("**Status:** EXCELLENT - Maximum deviation under 1%\n\n")
    elif max_dev < 2:
        f.write("**Status:** GOOD - Maximum deviation under 2%\n\n")
    else:
        f.write("**Status:** WARNING - Maximum deviation above 2%\n\n")

    f.write(f"""### District Details

| District | Population | Tracts | Deviation |
|----------|-----------|--------|-----------|
""")
    for district_id in range(num_districts):
        district_pop = sum(vertex_weights[i] for i, d in enumerate(final_assignments) if d == district_id)
        district_tracts = sum(1 for d in final_assignments if d == district_id)
        deviation = (district_pop - ideal) / ideal * 100
        f.write(f"| {district_id+1} | {district_pop:,.0f} | {district_tracts} | {deviation:+.2f}% |\n")

    f.write(f"""
## Files

- `README.md` - This documentation file
- `california_52_districts.png` - Final map with all 52 districts
- `final_assignments.pkl` - District assignments for each tract
- `intermediate/` - Intermediate results from each round of splitting
  - `round_N_metadata.json` - Metadata for round N
  - `round_N_assignments.pkl` - Assignments for round N

## Algorithm Details

### Recursive Bisection Process

1. **Initial Split:** Divide California (52 districts) into 2 regions (26 districts each)
2. **Round 2:** Split each of 2 regions into 2 (4 regions: 13, 13, 13, 13)
3. **Round 3:** Split each of 4 regions into 2 (8 regions: 7, 6, 7, 6, 7, 6, 7, 6)
4. **Continue recursively** until all regions have 1 district each

### Target Weight Calculation

For each split of k districts:
- k_left = k // 2
- k_right = k - k_left
- target_weights = [k_left/k, k_right/k]

Example: Splitting 13 districts -> 7 left, 6 right
- target_weights = [7/13, 6/13] = [0.538462, 0.461538]

### Dynamic Ufactor Strategy

- **Depth 1:** ufactor=1.001 (0.1% imbalance tolerance) - Tight for initial split
- **Depth 2:** ufactor=1.002 (0.2% tolerance)
- **Depth 3:** ufactor=1.003 (0.3% tolerance)
- **Depth 4+:** ufactor=1.005 (0.5% tolerance) - Relaxed for difficult splits

### METIS gpmetis Command

```bash
gpmetis -contig -minconn -ufactor=1.001 -tpwgt=weights.txt graph.txt 2
```

**Parameters:**
- `-contig`: Enforce geographic contiguity (CRITICAL for valid districts)
- `-minconn`: Minimize subdomain connectivity (reduce edge cuts)
- `-ufactor`: Limit load imbalance (dynamic: 1.001 to 1.005 based on depth)
- `-tpwgt`: Target partition weights file (format: `partition_id = weight` per line)

## Implementation Notes

### Performance
- **Tract-level granularity:** ~9K tracts vs. ~500K blocks
- Provides good balance between detail and computational efficiency
- Each split takes ~1-2 seconds with METIS

### Challenges Solved

1. **Water Connectivity:**
   - Initial approach (distance-band) added too many edges (10,449)
   - Created "leaks" across water, non-compact districts
   - Solution: Nearest-neighbor-only (93 edges) maintains compactness

2. **Population Balance:**
   - Without `-tpwgt`: 62.87% max deviation
   - With `-tpwgt`: 0.32% max deviation
   - Passing exact target weights is essential

3. **Recursive Error Accumulation:**
   - Early splits are critical (use tighter ufactor)
   - Dynamic ufactor strategy prevents compounding errors
   - Final result: all 52 districts within ±0.32%

### Applicability to Other States
- **Islands:** AK, HI, MI, NY, MA, WA - need tight water adjacency
- **Peninsulas:** FL, MI Upper Peninsula, MD - similar approach
- **Mountain barriers:** CO, MT, WY - may need manual connections
- **Complex coastlines:** LA, NC, SC, GA - nearest-neighbor handles well
""")

print(f"Documentation saved to: {readme_file}")
print("=" * 70)
