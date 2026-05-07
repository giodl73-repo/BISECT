#!/usr/bin/env python3
"""
Compute tract adjacency graphs with boundary lengths for 2000 Census data.

This script processes all 50 states to create adjacency graphs needed for
edge-weighted recursive bisection redistricting. For each state, it:
1. Loads tract geometries from parquet files
2. Computes Queen contiguity (land-based adjacency)
3. Adds water-based adjacency for coastal tracts
4. Computes shared boundary lengths for edge weighting
5. Saves adjacency graph as pickle file

IMPORTANT: Requires NHGIS 2000 tract shapefiles to have been merged first.
Run these scripts before this one:
  python scripts/data/census/parse_pl94171_tracts_2000.py
  python scripts/data/geography/download_tiger_tracts_2000.py  # Manual NHGIS download
  python scripts/data/merge_tracts_with_geometries_2000.py

Output adjacency files are compatible with the edge-weighted redistricting pipeline.
"""

import argparse
import sys
from pathlib import Path
import geopandas as gpd
from tqdm import tqdm

# Add src to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from src.apportionment.data.adjacency import build_adjacency_graph

# State FIPS codes
STATE_FIPS = {
    'AL': '01', 'AK': '02', 'AZ': '04', 'AR': '05', 'CA': '06',
    'CO': '08', 'CT': '09', 'DE': '10', 'FL': '12', 'GA': '13',
    'HI': '15', 'ID': '16', 'IL': '17', 'IN': '18', 'IA': '19',
    'KS': '20', 'KY': '21', 'LA': '22', 'ME': '23', 'MD': '24',
    'MA': '25', 'MI': '26', 'MN': '27', 'MS': '28', 'MO': '29',
    'MT': '30', 'NE': '31', 'NV': '32', 'NH': '33', 'NJ': '34',
    'NM': '35', 'NY': '36', 'NC': '37', 'ND': '38', 'OH': '39',
    'OK': '40', 'OR': '41', 'PA': '42', 'RI': '44', 'SC': '45',
    'SD': '46', 'TN': '47', 'TX': '48', 'UT': '49', 'VT': '50',
    'VA': '51', 'WA': '53', 'WV': '54', 'WI': '55', 'WY': '56'
}


def compute_state_adjacency(state_code, tracts_dir, output_dir, water_distance_km=5.0):
    """Compute adjacency graph with boundary lengths for one state."""

    state_lower = state_code.lower()

    # Input file
    tracts_file = tracts_dir / f"{state_lower}_tracts_2000.parquet"

    if not tracts_file.exists():
        print(f"  ERROR: Tract file not found: {tracts_file}")
        print(f"  Have you run merge_tracts_with_geometries_2000.py?")
        return False

    # Output file
    output_file = output_dir / f"{state_lower}_adjacency_2000.pkl"

    # Skip if already exists
    if output_file.exists():
        print(f"  Adjacency graph already exists, skipping")
        return True

    try:
        # Load tract geometries
        print(f"  Loading {tracts_file}...")
        tracts_gdf = gpd.read_parquet(tracts_file)

        # Ensure required columns exist
        required_cols = ['GEOID', 'geometry', 'population']
        if not all(col in tracts_gdf.columns for col in required_cols):
            print(f"  ERROR: Missing required columns: {required_cols}")
            print(f"  Available columns: {list(tracts_gdf.columns)}")
            return False

        # Add AWATER column if missing (needed for water adjacency detection)
        if 'AREAWATR' in tracts_gdf.columns and 'AWATER' not in tracts_gdf.columns:
            tracts_gdf['AWATER'] = tracts_gdf['AREAWATR']
        elif 'AWATER' not in tracts_gdf.columns:
            # If no water area data, set to 0 (disable water adjacency)
            tracts_gdf['AWATER'] = 0

        print(f"  Building adjacency graph for {len(tracts_gdf):,} tracts...")
        print(f"  Computing boundary lengths for edge-weighted partitioning...")

        # Build adjacency graph with boundary lengths
        adjacency, vertex_weights, index_to_geoid, geoid_to_index, edge_weights = build_adjacency_graph(
            tracts_gdf,
            water_distance_km=water_distance_km,
            include_water_adjacency=True,
            compute_boundary_lengths=True,  # Enable edge-weighted mode
            output_path=str(output_file)
        )

        # Verify edge weights were computed
        if edge_weights is None or len(edge_weights) == 0:
            print(f"  WARNING: No edge weights computed")
            return False

        print(f"  SUCCESS: Saved adjacency graph with {len(edge_weights):,} edge weights")
        return True

    except Exception as e:
        print(f"  ERROR: {e}")
        import traceback
        traceback.print_exc()
        return False


def main():
    parser = argparse.ArgumentParser(
        description='Compute tract adjacency graphs with boundary lengths for 2000 Census',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Prerequisites:
  1. Parse 2000 census data:
     python scripts/data/census/parse_pl94171_tracts_2000.py

  2. Download NHGIS shapefiles (manual):
     python scripts/data/geography/download_tiger_tracts_2000.py
     (Follow instructions to download from https://www.nhgis.org/)

  3. Merge data with geometries:
     python scripts/data/merge_tracts_with_geometries_2000.py

  4. Then run this script to compute adjacencies

Examples:
  # Process single state
  python compute_tract_adjacencies_2000.py --states CA

  # Process all 50 states
  python compute_tract_adjacencies_2000.py

  # Adjust water distance threshold
  python compute_tract_adjacencies_2000.py --water-distance 3.0
        """
    )
    parser.add_argument(
        '--tracts-dir',
        type=Path,
        default=Path('data/tracts/2000'),
        help='Directory containing tract parquet files'
    )
    parser.add_argument(
        '--output-dir',
        type=Path,
        default=Path('data/adjacency/2000'),
        help='Output directory for adjacency pickle files'
    )
    parser.add_argument(
        '--water-distance',
        type=float,
        default=5.0,
        help='Water distance threshold in km (default: 5.0)'
    )
    parser.add_argument(
        '--states',
        type=str,
        nargs='+',
        help='Process specific states (e.g., AL CA TX). If not specified, processes all states.'
    )

    args = parser.parse_args()

    # Check input directory exists
    if not args.tracts_dir.exists():
        print(f"ERROR: Tracts directory not found: {args.tracts_dir}")
        print()
        print("You need to complete the 2000 tract data pipeline first:")
        print()
        print("1. Parse census data:")
        print("   python scripts/data/census/parse_pl94171_tracts_2000.py")
        print()
        print("2. Download NHGIS shapefiles (manual):")
        print("   python scripts/data/geography/download_tiger_tracts_2000.py")
        print("   (Follow instructions for https://www.nhgis.org/)")
        print()
        print("3. Merge data with geometries:")
        print("   python scripts/data/merge_tracts_with_geometries_2000.py")
        print()
        print("4. Then run this script")
        return 1

    # Create output directory
    args.output_dir.mkdir(parents=True, exist_ok=True)

    # Determine which states to process
    if args.states:
        states_to_process = [s.upper() for s in args.states]
    else:
        states_to_process = sorted(STATE_FIPS.keys())

    print("=" * 70)
    print("COMPUTING TRACT ADJACENCY GRAPHS WITH BOUNDARY LENGTHS (2000)")
    print("=" * 70)
    print(f"Processing {len(states_to_process)} states...")
    print(f"Tracts directory: {args.tracts_dir}")
    print(f"Output directory: {args.output_dir}")
    print(f"Water distance threshold: {args.water_distance} km")
    print()
    print("Mode: Edge-weighted (computing boundary lengths for METIS)")
    print("This enables edge-weighted recursive bisection for optimal compactness.")
    print()

    # Process each state
    success_count = 0
    failed_states = []

    for state_code in tqdm(states_to_process, desc="Processing states"):
        print(f"\n{'=' * 70}")
        print(f"{state_code}:")
        print(f"{'=' * 70}")

        if compute_state_adjacency(state_code, args.tracts_dir, args.output_dir, args.water_distance):
            success_count += 1
        else:
            failed_states.append(state_code)

    # Summary
    print("\n" + "=" * 70)
    print("SUMMARY")
    print("=" * 70)
    print(f"Processed {success_count}/{len(states_to_process)} states successfully")
    if failed_states:
        print(f"Failed states: {', '.join(failed_states)}")
    print("=" * 70)
    print()
    if success_count > 0:
        print("Adjacency graphs are ready for edge-weighted redistricting!")
        print("Run: python scripts/pipeline/run_complete_redistricting.py --year 2000 --version v1")

    return 0 if success_count > 0 else 1


if __name__ == '__main__':
    sys.exit(main())
