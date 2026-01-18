#!/usr/bin/env python3
"""
Build spatial adjacency graph from Census tract geometries.

Example usage:
    python scripts/build_tract_adjacency.py --state CA --year 2020
    python scripts/build_tract_adjacency.py --state IA --year 2020 --compute-boundary-lengths
"""

import argparse
import pickle
import sys
from pathlib import Path

# Add project root to path (use cwd which should be project root)
project_root = Path.cwd()
sys.path.insert(0, str(project_root / 'src'))
sys.path.insert(0, str(project_root))

import geopandas as gpd
from apportionment.data.adjacency import build_adjacency_graph


def build_tract_adjacency(
    tracts_file: str,
    water_distance_km: float = 1.0,
    compute_boundary_lengths: bool = False,
    minimum_boundary_length: float = 0.0,
    output_dir: str = 'data/adjacency',
    year: int = 2020
):
    """
    Build tract-level adjacency graph using the library's build_adjacency_graph function.

    Parameters
    ----------
    tracts_file : str
        Path to tracts parquet file
    water_distance_km : float
        Water distance threshold (km)
    compute_boundary_lengths : bool
        Whether to compute boundary lengths for edge-weighted partitioning
    minimum_boundary_length : float
        Minimum shared boundary length (meters) to consider tracts adjacent
    output_dir : str
        Output directory
    year : int
        Census year
    """
    print(f"\nLoading tracts from {tracts_file}...")
    tracts_gdf = gpd.read_parquet(tracts_file)
    print(f"Loaded {len(tracts_gdf):,} tracts")

    # Extract state from filename for output
    state = Path(tracts_file).stem.split('_')[0]
    output_path = Path(output_dir)
    output_path.mkdir(parents=True, exist_ok=True)
    output_file = output_path / f"{state}_adjacency_{year}.pkl"

    # Use library function to build adjacency graph
    adjacency, vertex_weights, index_to_geoid, geoid_to_index, edge_weights = build_adjacency_graph(
        blocks_gdf=tracts_gdf,
        water_distance_km=water_distance_km,
        include_water_adjacency=True,
        compute_boundary_lengths=compute_boundary_lengths,
        minimum_boundary_length=minimum_boundary_length,
        output_path=str(output_file)
    )

    print(f"\nGraph saved to: {output_file}")


def main():
    parser = argparse.ArgumentParser(description='Build tract adjacency graph')
    parser.add_argument('--state', type=str, required=True, help='State code (e.g., CA, IA)')
    parser.add_argument('--year', type=int, default=2020, help='Census year (default: 2020)')
    parser.add_argument('--water-distance', type=float, default=1.0, help='Water distance threshold in km (default: 1.0)')
    parser.add_argument('--compute-boundary-lengths', action='store_true',
                        help='Compute boundary lengths for edge-weighted partitioning (slower but enables edge-weighted mode)')
    parser.add_argument('--minimum-boundary-length', type=float, default=0.0,
                        help='Minimum shared boundary length (meters) to consider tracts adjacent. Filters tiny corner touches. (default: 0, recommended: 10-25)')
    parser.add_argument('--input-dir', type=str, default='data/raw', help='Input directory (default: data/raw)')
    parser.add_argument('--output-dir', type=str, default='data/adjacency', help='Output directory (default: data/adjacency)')

    args = parser.parse_args()

    tracts_file = Path(args.input_dir) / f"{args.state.lower()}_tracts_{args.year}.parquet"

    if not tracts_file.exists():
        print(f"Error: Tracts file not found: {tracts_file}")
        print(f"Run: python scripts/download_tracts.py --state {args.state} --year {args.year}")
        return 1

    try:
        build_tract_adjacency(
            str(tracts_file),
            args.water_distance,
            args.compute_boundary_lengths,
            args.minimum_boundary_length,
            args.output_dir,
            args.year
        )
        return 0
    except Exception as e:
        print(f"\nError: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == '__main__':
    sys.exit(main())
