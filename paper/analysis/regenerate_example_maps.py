#!/usr/bin/env python3
"""
Regenerate example state maps with proper district count labels.
"""

import sys
import pickle
from pathlib import Path
import geopandas as gpd
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches

def load_state_data(state_name, state_code):
    """Load tract geometries and intermediate assignments."""

    # Load tract geometries
    tracts_file = Path('data/raw') / f'{state_code}_tracts_2020.parquet'
    if not tracts_file.exists():
        print(f"ERROR: {tracts_file} not found")
        return None, None

    print(f"Loading {state_name} tract geometries...")
    tracts_gdf = gpd.read_parquet(tracts_file)

    # Load intermediate round data
    state_dir = Path('outputs/us_2020_v1/states') / state_name.lower().replace(' ', '_')
    intermediate_dir = state_dir / 'intermediate'

    if not intermediate_dir.exists():
        print(f"ERROR: {intermediate_dir} not found")
        return None, None

    # Find all round files
    round_files = sorted(intermediate_dir.glob('round_*_assignments.json'))

    return tracts_gdf, round_files

def load_future_districts_from_metadata(metadata_file):
    """
    Load target_districts for each region from metadata file.
    """
    import json

    with open(metadata_file, 'r') as f:
        metadata = json.load(f)

    # Build dict of region_id -> target_districts
    future_districts = {}
    for region in metadata['regions']:
        # region_id in metadata might be 1-indexed, but in assignments it's 0-indexed
        # Actually let me check the actual data
        future_districts[region['region_id']] = region['target_districts']

    return future_districts

def create_round_map(tracts_gdf, assignments_file, output_file, round_num, total_districts, state_name=''):
    """Create a map for one round with proper labels."""

    import json

    # Load assignments - simple dict of {tract_index: region_id}
    with open(assignments_file, 'r') as f:
        assignments_data = json.load(f)

    # Convert to int keys
    assignments = {int(k): v for k, v in assignments_data.items()}

    # Count number of regions
    num_regions = len(set(assignments.values()))

    # Load future district counts from metadata file
    metadata_file = assignments_file.parent / assignments_file.name.replace('_assignments.json', '_metadata.json')
    future_districts = load_future_districts_from_metadata(metadata_file)

    # Merge assignments into GeoDataFrame
    tracts_gdf = tracts_gdf.copy()
    tracts_gdf['region'] = tracts_gdf.index.map(assignments)

    # Filter out tracts without assignments
    tracts_gdf = tracts_gdf[tracts_gdf['region'].notna()]

    # Create figure - Minnesota gets taller aspect ratio
    if 'minnesota' in state_name.lower():
        fig, ax = plt.subplots(figsize=(10, 11))
    else:
        fig, ax = plt.subplots(figsize=(10, 8))

    # Plot tracts colored by region
    tracts_gdf.plot(column='region', ax=ax, cmap='tab20',
                    edgecolor='white', linewidth=0.3, legend=False)

    # Add region labels with district counts
    # Determine font size based on state and round
    if 'alabama' in state_name.lower():
        fontsize = 16  # Smaller for Alabama
    elif 'minnesota' in state_name.lower():
        if round_num <= 2:
            fontsize = 24  # Bigger for Minnesota rounds 1-2
        else:
            fontsize = 20  # Normal size for Minnesota round 3
    else:
        fontsize = 20  # Default

    for region_id in sorted(tracts_gdf['region'].unique()):
        region_tracts = tracts_gdf[tracts_gdf['region'] == region_id]
        centroid = region_tracts.geometry.unary_union.centroid

        # Label format: "region_id (future_districts)"
        # Note: assignments are 0-indexed, metadata is 1-indexed
        metadata_region_id = int(region_id) + 1
        label = f"{metadata_region_id} ({future_districts[metadata_region_id]})"

        ax.annotate(label, xy=(centroid.x, centroid.y),
                   fontsize=fontsize, fontweight='bold', ha='center',
                   color='black')

    # Title
    if num_regions == total_districts:
        title = f'Round {round_num}: {num_regions} Districts (Final)'
    else:
        title = f'Round {round_num}: {num_regions} Regions'

    ax.set_title(title, fontsize=16, fontweight='bold', pad=20)
    ax.axis('off')

    plt.tight_layout()
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"  Saved: {output_file}")
    plt.close()

def regenerate_state_maps(state_name, state_code, total_districts, output_prefix):
    """Regenerate all round maps for a state."""

    print(f"\n{'='*70}")
    print(f"Regenerating maps for {state_name} ({total_districts} districts)")
    print(f"{'='*70}")

    # Load data
    tracts_gdf, round_files = load_state_data(state_name, state_code)

    if tracts_gdf is None or round_files is None:
        return False

    # Generate maps for each round (skip round 0 which is just initial state)
    output_dir = Path('paper/figures')
    output_dir.mkdir(parents=True, exist_ok=True)

    for i, round_file in enumerate(round_files):
        round_num = i + 1
        output_file = output_dir / f'{output_prefix}_round_{i}.png'

        create_round_map(tracts_gdf, round_file, output_file, round_num, total_districts, state_name)

    print(f"\nCompleted {len(round_files)} maps for {state_name}")
    return True

def main():
    """Main function."""

    print("="*70)
    print("REGENERATING EXAMPLE STATE MAPS")
    print("="*70)

    # Regenerate Alabama (7 districts, odd)
    success1 = regenerate_state_maps('Alabama', 'al', 7, 'odd_example')

    # Regenerate Minnesota (8 districts, even)
    success2 = regenerate_state_maps('Minnesota', 'mn', 8, 'example_state')

    if success1 and success2:
        print("\n" + "="*70)
        print("ALL MAPS REGENERATED SUCCESSFULLY")
        print("="*70)
        return 0
    else:
        return 1

if __name__ == '__main__':
    sys.exit(main())
