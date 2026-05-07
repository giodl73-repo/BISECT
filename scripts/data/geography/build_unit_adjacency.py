#!/usr/bin/env python3
"""
Build spatial adjacency graph from Census geographic unit geometries.

Supports tracts, block groups, and blocks.

Example usage:
    python scripts/build_unit_adjacency.py --state CA --year 2020 --resolution tract
    python scripts/build_unit_adjacency.py --state IA --year 2020 --resolution block_group --compute-boundary-lengths
    python scripts/build_unit_adjacency.py --state VT --year 2020 --resolution block
"""

import argparse
import pickle
import sys
from pathlib import Path
import warnings
import os

# Suppress warnings that can disrupt hierarchical display
warnings.filterwarnings('ignore')

# Suppress pandas warnings
os.environ['PYTHONWARNINGS'] = 'ignore'

# Add project root to path (use cwd which should be project root)
project_root = Path.cwd()
sys.path.insert(0, str(project_root / 'src'))
sys.path.insert(0, str(project_root))

import geopandas as gpd
import pandas as pd
from apportionment.data.adjacency import build_adjacency_graph

# Additional pandas-specific warning suppression
pd.options.mode.chained_assignment = None  # Suppress SettingWithCopyWarning


def build_unit_adjacency(
    units_file: str,
    resolution: str,
    water_distance_km: float = 1.0,
    compute_boundary_lengths: bool = False,
    minimum_boundary_length: float = 0.0,
    output_dir: str = 'data/adjacency',
    year: int = 2020
):
    """
    Build unit-level adjacency graph using the library's build_adjacency_graph function.

    Parameters
    ----------
    units_file : str
        Path to units parquet file (tracts, block_groups, or blocks)
    resolution : str
        Geographic resolution ('tract', 'block_group', or 'block')
    water_distance_km : float
        Water distance threshold (km)
    compute_boundary_lengths : bool
        Whether to compute boundary lengths for edge-weighted partitioning
    minimum_boundary_length : float
        Minimum shared boundary length (meters) to consider units adjacent
    output_dir : str
        Output directory
    year : int
        Census year
    """
    print(f"\nLoading {resolution}s from {units_file}...")
    units_gdf = gpd.read_parquet(units_file)
    print(f"Loaded {len(units_gdf):,} {resolution}s")

    # Extract state from filename for output
    state = Path(units_file).stem.split('_')[0]
    output_path = Path(output_dir)
    output_path.mkdir(parents=True, exist_ok=True)
    output_file = output_path / f"{state}_{resolution}_adjacency_{year}.pkl"

    # Use library function to build adjacency graph
    adjacency, vertex_weights, index_to_geoid, geoid_to_index, edge_weights = build_adjacency_graph(
        blocks_gdf=units_gdf,  # Despite name, works with any resolution
        water_distance_km=water_distance_km,
        include_water_adjacency=True,
        compute_boundary_lengths=compute_boundary_lengths,
        minimum_boundary_length=minimum_boundary_length,
        output_path=str(output_file)
    )

    print(f"\nGraph saved to: {output_file}")


def main():
    parser = argparse.ArgumentParser(description='Build geographic unit adjacency graph')
    parser.add_argument('--state', type=str, required=True, help='State code (e.g., CA, IA)')
    parser.add_argument('--year', type=int, default=2020, help='Census year (default: 2020)')
    parser.add_argument('--resolution', type=str, required=True, choices=['tract', 'block_group', 'block'],
                        help='Geographic resolution (tract, block_group, or block)')
    parser.add_argument('--version', type=str, help='Version identifier (for compatibility with parent scripts, not used directly)')
    parser.add_argument('--water-distance', type=float, default=1.0, help='Water distance threshold in km (default: 1.0)')
    parser.add_argument('--compute-boundary-lengths', action='store_true',
                        help='Compute boundary lengths for edge-weighted partitioning (slower but enables edge-weighted mode)')
    parser.add_argument('--minimum-boundary-length', type=float, default=0.0,
                        help='Minimum shared boundary length (meters) to consider units adjacent. Filters tiny corner touches. (default: 0, recommended: 10-25)')
    parser.add_argument('--input-dir', type=str, default='data/raw', help='Input directory (default: data/raw)')
    parser.add_argument('--output-dir', type=str, default='data/adjacency', help='Output directory (default: data/adjacency)')

    args = parser.parse_args()

    # Construct filename based on resolution
    units_file = Path(args.input_dir) / f"{args.state.lower()}_{args.resolution}s_{args.year}.parquet"

    if not units_file.exists():
        print(f"Error: {args.resolution.capitalize()}s file not found: {units_file}")
        print(f"Run: python scripts/data/merge_units_with_geometries.py --state {args.state} --year {args.year} --resolution {args.resolution}")
        return 1

    try:
        build_unit_adjacency(
            str(units_file),
            args.resolution,
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
