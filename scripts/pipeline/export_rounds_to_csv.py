#!/usr/bin/env python3
"""Export all rounds to a single CSV for debugging."""

import json
from pathlib import Path
import pandas as pd
import sys

# Get intermediate directory from command line or use latest
if len(sys.argv) > 1:
    intermediate_dir = Path(sys.argv[1])
else:
    # Find latest california_full directory
    outputs_dir = Path('outputs')
    ca_dirs = sorted(outputs_dir.glob('california_full_*'), reverse=True)
    if ca_dirs:
        intermediate_dir = ca_dirs[0] / 'intermediate'
        print(f"Using latest run: {ca_dirs[0].name}")
    else:
        raise ValueError("No california_full_* directories found in outputs/")

# Find all round metadata files
round_files = sorted(intermediate_dir.glob('round_*_metadata.json'))

print(f"Found {len(round_files)} rounds")
print("=" * 70)

all_rows = []

for metadata_file in round_files:
    # Load metadata
    with open(metadata_file, 'r') as f:
        metadata = json.load(f)

    depth = metadata['depth']
    num_regions = metadata['num_regions']
    total_districts = metadata.get('total_districts', 52)
    regions = metadata.get('regions', [])

    print(f"Round {depth}: {num_regions} regions")

    for region in regions:
        region_id = region['region_id']
        name = region['name']
        population = region['population']
        num_blocks = region['num_blocks']
        target_districts = region.get('target_districts', None)  # NEW: Get actual target districts

        # Parse hierarchical name to get parent info
        # e.g., "CA0" -> parent="CA", child_index=0
        # e.g., "CA01" -> parent="CA0", child_index=1
        # e.g., "CA101" -> parent="CA10", child_index=1

        if len(name) <= 2:  # Root level like "CA"
            parent_name = None
            child_index = None
            parent_round = None
        else:
            # Extract parent by removing last character
            parent_name = name[:-1]
            child_index = int(name[-1])  # 0 or 1
            parent_round = depth - 1

        # Calculate how close the population is to the target number of districts
        ideal_pop_per_district = metadata['total_population'] / total_districts

        if target_districts and target_districts > 0:
            # Calculate target population for this region
            target_population = ideal_pop_per_district * target_districts

            # Calculate actual "districts worth" of population
            actual_districts_worth = population / ideal_pop_per_district

            # Deviation: how many districts off are we?
            district_deviation = actual_districts_worth - target_districts
            deviation_pct = (district_deviation / target_districts * 100) if target_districts > 0 else 0

            # Also calculate per-district average
            per_district_pop = population / target_districts
        else:
            # Fallback if target_districts not available
            target_population = None
            actual_districts_worth = None
            district_deviation = None
            districts_per_region = total_districts / num_regions
            per_district_pop = population / districts_per_region
            deviation_pct = 0

        row = {
            'round': depth,
            'num_regions_in_round': num_regions,
            'region_id': region_id,
            'region_name': name,
            'parent_round': parent_round,
            'parent_name': parent_name,
            'child_index': child_index,  # 0=left, 1=right
            'population': population,
            'num_tracts': num_blocks,
            'target_districts': target_districts,  # ACTUAL district count (e.g., 6 or 7, not 6.5)
            'target_population': target_population,  # Ideal pop for this many districts
            'actual_districts_worth': actual_districts_worth,  # How many districts worth of pop do we actually have?
            'district_deviation': district_deviation,  # Difference in districts (e.g., -0.044 districts)
            'deviation_pct': deviation_pct,  # Percent deviation from target districts
            'pop_per_district': per_district_pop,
            'ideal_pop_per_district': ideal_pop_per_district
        }

        all_rows.append(row)

# Create DataFrame
df = pd.DataFrame(all_rows)

# Sort by round, then region_name
df = df.sort_values(['round', 'region_name'])

# Save to CSV in data/ subdirectory
data_dir = intermediate_dir.parent / 'data'
data_dir.mkdir(parents=True, exist_ok=True)
output_file = data_dir / 'rounds_hierarchy.csv'
df.to_csv(output_file, index=False, float_format='%.2f')

print("\n" + "=" * 70)
print(f"Exported {len(df)} rows to: {output_file}")
print("=" * 70)

# Print summary statistics by round
print("\nSummary by Round:")
print("-" * 70)
summary = df.groupby('round').agg({
    'num_regions_in_round': 'first',
    'deviation_pct': ['min', 'max', 'mean'],
    'population': ['min', 'max']
}).round(2)
print(summary)

# Also create a detailed parent-child mapping
print("\n" + "=" * 70)
print("Sample parent-child relationships:")
print("-" * 70)
sample = df[['round', 'region_name', 'parent_name', 'child_index', 'population', 'deviation_pct']].head(20)
print(sample.to_string(index=False))

print("\n" + "=" * 70)
print(f"Full data saved to: {output_file}")
print("=" * 70)
