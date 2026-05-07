#!/usr/bin/env python3
"""
Run recursive bifurcation redistricting algorithm.

Example usage:
    python scripts/redistrict_ca.py --state CA --num-districts 52
    python scripts/redistrict_ca.py --state CA --num-districts 5 --county 075  # SF County
"""

import argparse
import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent / 'src'))

from apportionment.data.census import load_blocks
from apportionment.data.adjacency import load_adjacency_graph
from apportionment.data.io import save_results
from apportionment.partition.recursive_bisection import RecursiveBisection
from apportionment.partition.metis_wrapper import check_metis_installation, print_installation_instructions


def main():
    parser = argparse.ArgumentParser(
        description='Run recursive bifurcation redistricting'
    )
    parser.add_argument(
        '--state',
        type=str,
        required=True,
        help='State code (e.g., CA for California)'
    )
    parser.add_argument(
        '--num-districts',
        type=int,
        required=True,
        help='Number of districts to create'
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
        '--graph-dir',
        type=str,
        default='data/adjacency',
        help='Directory containing adjacency graph (default: ./data/adjacency)'
    )
    parser.add_argument(
        '--blocks-dir',
        type=str,
        default='data/raw',
        help='Directory containing block data (default: ./data/raw)'
    )
    parser.add_argument(
        '--output-dir',
        type=str,
        default='data/results',
        help='Output directory for results (default: ./data/results)'
    )
    parser.add_argument(
        '--yes', '-y',
        action='store_true',
        help='Skip confirmation prompts'
    )

    args = parser.parse_args()

    print(f"\n{'=' * 60}")
    print(f"Recursive Bifurcation Redistricting")
    print(f"{'=' * 60}\n")

    # Check METIS installation
    metis_installed, metis_method = check_metis_installation()
    if not metis_installed:
        print("WARNING: METIS is not installed!")
        print("Falling back to NetworkX native algorithms (slower, lower quality)")
        if not args.yes:
            print_installation_instructions()
            response = input("\nContinue anyway? (y/n): ")
            if response.lower() != 'y':
                return 1
        else:
            print("Continuing with NetworkX fallback...\n")
    else:
        print(f"Using METIS method: {metis_method}\n")

    # Load adjacency graph
    county_suffix = f"_{args.county}" if args.county else ""
    graph_file = Path(args.graph_dir) / f"{args.state.lower()}_blocks_graph{county_suffix}.pkl"

    if not graph_file.exists():
        print(f"Error: Adjacency graph not found at {graph_file}")
        print(f"Run build_adjacency.py first.")
        return 1

    adjacency, vertex_weights, index_to_geoid, geoid_to_index = load_adjacency_graph(str(graph_file))

    # Load blocks for result saving
    blocks_file = Path(args.blocks_dir) / f"{args.state.lower()}_blocks_{args.year}{county_suffix}.parquet"
    if not blocks_file.exists():
        print(f"Error: Block data not found at {blocks_file}")
        return 1

    blocks_gdf = load_blocks(str(blocks_file))

    # Run recursive bisection
    print("\n" + "=" * 60)
    print("Starting Recursive Bisection Algorithm")
    print("=" * 60)

    try:
        rb = RecursiveBisection(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            num_districts=args.num_districts
        )

        # Run partitioning
        assignments = rb.partition()

        print("\n" + "=" * 60)
        print("Partitioning Complete!")
        print("=" * 60)

        # Convert index-based assignments to GEOID-based
        geoid_assignments = {
            index_to_geoid[idx]: district_id
            for idx, district_id in assignments.items()
        }

        # Save results
        save_results(
            assignments=geoid_assignments,
            blocks_gdf=blocks_gdf,
            output_dir=args.output_dir,
            state=args.state,
            num_districts=args.num_districts
        )

        print("\n" + "=" * 60)
        print("SUCCESS!")
        print("=" * 60)
        print(f"\nResults saved to: {args.output_dir}/{args.state.lower()}_{args.num_districts}_districts/")
        print("\nNext steps:")
        print(f"  python scripts/visualize_districts.py --state {args.state} --num-districts {args.num_districts}")

    except Exception as e:
        print(f"\nError during redistricting: {e}")
        import traceback
        traceback.print_exc()
        return 1

    return 0


if __name__ == '__main__':
    sys.exit(main())
