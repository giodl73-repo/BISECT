#!/usr/bin/env python3
"""
Build spatial adjacency graph from Census block geometries.

Includes water-based adjacency adaptation.

Example usage:
    python scripts/build_adjacency.py --state CA --year 2020
    python scripts/build_adjacency.py --state CA --year 2020 --water-distance 2.0
    python scripts/build_adjacency.py --state CA --year 2020 --no-water-adjacency
"""

import argparse
import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent / 'src'))

from apportionment.data.census import load_blocks, filter_populated_blocks
from apportionment.data.adjacency import build_adjacency_graph


def main():
    parser = argparse.ArgumentParser(
        description='Build spatial adjacency graph with water-based adaptation'
    )
    parser.add_argument(
        '--state',
        type=str,
        required=True,
        help='State code (e.g., CA for California)'
    )
    parser.add_argument(
        '--year',
        type=int,
        default=2020,
        help='Census year (default: 2020)'
    )
    parser.add_argument(
        '--county',
        type=str,
        default=None,
        help='County FIPS code (optional)'
    )
    parser.add_argument(
        '--water-distance',
        type=float,
        default=1.0,
        help='Water distance threshold in km (default: 1.0)'
    )
    parser.add_argument(
        '--no-water-adjacency',
        action='store_true',
        help='Disable water-based adjacency adaptation'
    )
    parser.add_argument(
        '--filter-unpopulated',
        action='store_true',
        help='Filter out blocks with zero population'
    )
    parser.add_argument(
        '--input-dir',
        type=str,
        default='data/raw',
        help='Input directory for block data (default: ./data/raw)'
    )
    parser.add_argument(
        '--output-dir',
        type=str,
        default='data/adjacency',
        help='Output directory for adjacency graph (default: ./data/adjacency)'
    )

    args = parser.parse_args()

    print(f"\n{'=' * 60}")
    print(f"Spatial Adjacency Graph Construction")
    print(f"{'=' * 60}\n")

    # Load blocks
    input_dir = Path(args.input_dir)
    county_suffix = f"_{args.county}" if args.county else ""
    input_file = input_dir / f"{args.state.lower()}_blocks_{args.year}{county_suffix}.parquet"

    if not input_file.exists():
        print(f"Error: Block data not found at {input_file}")
        print(f"Run download_data.py first.")
        return 1

    blocks_gdf = load_blocks(str(input_file))

    # Filter unpopulated blocks if requested
    if args.filter_unpopulated:
        print("\nFiltering unpopulated blocks...")
        blocks_gdf = filter_populated_blocks(blocks_gdf, min_population=1)

    # Build adjacency graph
    output_dir = Path(args.output_dir)
    output_file = output_dir / f"{args.state.lower()}_blocks_graph{county_suffix}.pkl"

    try:
        adjacency, vertex_weights, index_to_geoid, geoid_to_index = build_adjacency_graph(
            blocks_gdf=blocks_gdf,
            water_distance_km=args.water_distance,
            include_water_adjacency=not args.no_water_adjacency,
            output_path=str(output_file)
        )

        print(f"\n{'=' * 60}")
        print(f"Adjacency Graph Complete!")
        print(f"{'=' * 60}\n")
        print(f"Graph saved to: {output_file}")

    except Exception as e:
        print(f"\nError building adjacency graph: {e}")
        import traceback
        traceback.print_exc()
        return 1

    return 0


if __name__ == '__main__':
    sys.exit(main())
