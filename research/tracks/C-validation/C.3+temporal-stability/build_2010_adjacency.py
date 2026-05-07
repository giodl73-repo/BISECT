#!/usr/bin/env python3
"""
Build adjacency matrices for 2010 census tracts.

Simplified script specifically for temporal stability analysis.
"""

import sys
from pathlib import Path
import geopandas as gpd
from scipy import sparse
import numpy as np
from datetime import datetime

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from src.apportionment.data.adjacency import build_adjacency_graph

# State FIPS mapping
STATE_TO_FIPS = {
    'alabama': '01',
    'georgia': '13',
    'louisiana': '22',
    'mississippi': '28',
    'south_carolina': '45'
}

STATES = ['alabama', 'georgia', 'louisiana', 'mississippi', 'south_carolina']


def build_tract_adjacency(state: str):
    """Build adjacency matrix for a state's 2010 census tracts."""
    print(f"\n{'='*70}")
    print(f"Building adjacency for {state.upper()}")
    print(f"{'='*70}")

    fips = STATE_TO_FIPS[state]

    # Load tract geometries (use absolute path from project root)
    project_root = Path(__file__).parent.parent.parent
    tracts_dir = project_root / f'data/2010/tiger/tracts/tl_2010_{fips}_tract10'
    print(f"Loading tracts from: {tracts_dir}")

    if not tracts_dir.exists():
        print(f"ERROR: Directory not found: {tracts_dir}")
        return False

    tracts = gpd.read_file(tracts_dir)
    print(f"  Loaded {len(tracts)} tracts")

    # Rename GEOID10 to GEOID (2010 shapefiles use different naming)
    if 'GEOID10' in tracts.columns and 'GEOID' not in tracts.columns:
        tracts = tracts.rename(columns={'GEOID10': 'GEOID'})
        print(f"  Renamed GEOID10 -> GEOID")

    # Build adjacency graph using the library function
    print("  Computing adjacency (Queen contiguity)...")
    try:
        # Use build_adjacency_graph from the library
        adjacency = build_adjacency_graph(
            tracts,
            water_distance_km=1.0,  # For coastal adjacency
            include_water_adjacency=True
        )

        print(f"  Adjacency matrix: {adjacency.shape}")
        print(f"  Number of edges: {adjacency.nnz}")
        print(f"  Average neighbors: {adjacency.nnz / len(tracts):.2f}")

        # Save adjacency matrix (use absolute path)
        output_dir = project_root / 'outputs/data/2010/adjacency'
        output_dir.mkdir(parents=True, exist_ok=True)

        output_file = output_dir / f'{state}_adjacency.npz'
        sparse.save_npz(output_file, adjacency)

        print(f"  Saved: {output_file}")
        print(f"  [OK] {state} completed")

        return True

    except Exception as e:
        print(f"  ERROR: {e}")
        import traceback
        traceback.print_exc()
        return False


def main():
    """Build adjacency matrices for all 5 states."""
    print()
    print("="*70)
    print("2010 TRACT ADJACENCY MATRIX BUILDER")
    print("="*70)
    print(f"Started: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"States: {', '.join(STATES)}")
    print()

    results = {}

    for state in STATES:
        success = build_tract_adjacency(state)
        results[state] = 'SUCCESS' if success else 'FAILED'

    # Summary
    print()
    print("="*70)
    print("SUMMARY")
    print("="*70)

    for state, status in results.items():
        symbol = '[OK]' if status == 'SUCCESS' else '[FAIL]'
        print(f"  {symbol} {state}")

    successful = sum(1 for s in results.values() if s == 'SUCCESS')
    print()
    print(f"Completed: {successful}/{len(STATES)} states")
    print(f"Finished: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")

    if successful == len(STATES):
        print()
        print("All adjacency matrices built successfully!")
        print("Next step: Run experiments")
        print("  cd research/gerry-temporal-stability")
        print("  python run_all_experiments.py")
    else:
        print()
        print("Some states failed. Check errors above.")


if __name__ == '__main__':
    main()
