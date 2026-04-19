#!/usr/bin/env python3
"""
Test redistricting algorithm by rounds.

Run specific rounds and generate maps for each intermediate result.

Example usage:
    python scripts/test_rounds.py --state CA --num-districts 52 --round 1
    python scripts/test_rounds.py --state CA --num-districts 52 --rounds 1-3
    python scripts/test_rounds.py --state CA --num-districts 52 --all
"""

import argparse
import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent / 'src'))

from apportionment.data.census import load_blocks
from apportionment.data.adjacency import load_adjacency_graph
from apportionment.partition.recursive_bisection import RecursiveBisection
from apportionment.visualization.maps import visualize_districts
import json


def visualize_round(
    round_num: int,
    blocks_gdf,
    intermediate_dir: str,
    output_dir: str,
    state: str
):
    """Visualize a specific round's results."""
    intermediate_path = Path(intermediate_dir)

    # Find the round file
    round_files = list(intermediate_path.glob(f"round_{round_num}_*_metadata.json"))

    if not round_files:
        print(f"Warning: No results found for round {round_num}")
        return False

    metadata_file = round_files[0]

    # Load metadata
    with open(metadata_file, 'r') as f:
        metadata = json.load(f)

    num_regions = metadata['num_regions']
    print(f"\n{'=' * 60}")
    print(f"Round {round_num}: {num_regions} regions")
    print(f"{'=' * 60}")

    # Load assignments
    assignments_file = metadata_file.parent / f"round_{round_num}_{num_regions}_regions_assignments.json"
    with open(assignments_file, 'r') as f:
        assignments = json.load(f)

    # Convert block indices back to GEOIDs
    # Assignments are stored as {block_idx: region_id}
    # We need {GEOID: region_id}
    # This requires index_to_geoid mapping

    print("\nRegion Statistics:")
    for region in metadata['regions']:
        print(f"  Region {region['region_id']} ({region['name']}):")
        print(f"    Population: {region['population']:,}")
        print(f"    Blocks: {region['num_blocks']:,}")

    # Generate map
    print(f"\nGenerating map...")
    output_path = Path(output_dir)
    output_path.mkdir(parents=True, exist_ok=True)

    map_file = output_path / f"{state.lower()}_round_{round_num}_{num_regions}_regions.png"

    # Note: This requires converting assignments to GEOID format
    # For now, just report that visualization is needed
    print(f"  Map would be saved to: {map_file}")
    print(f"  (Map generation requires GEOID conversion - to be implemented)")

    return True


def main():
    parser = argparse.ArgumentParser(
        description='Test redistricting by rounds with intermediate visualization'
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
        '--round',
        type=int,
        help='Specific round to test (1, 2, 3, etc.)'
    )
    parser.add_argument(
        '--rounds',
        type=str,
        help='Range of rounds to test (e.g., 1-3)'
    )
    parser.add_argument(
        '--all',
        action='store_true',
        help='Run all rounds'
    )
    parser.add_argument(
        '--visualize-only',
        action='store_true',
        help='Only visualize existing results (don\'t run algorithm)'
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
        '--intermediate-dir',
        type=str,
        default='data/intermediate',
        help='Directory for intermediate results (default: ./data/intermediate)'
    )
    parser.add_argument(
        '--output-dir',
        type=str,
        default='outputs/rounds',
        help='Output directory for maps (default: ./outputs/rounds)'
    )

    args = parser.parse_args()

    # Determine which rounds to run/visualize
    rounds_to_process = []

    if args.round:
        rounds_to_process = [args.round]
    elif args.rounds:
        start, end = map(int, args.rounds.split('-'))
        rounds_to_process = list(range(start, end + 1))
    elif args.all:
        # Calculate max rounds based on num_districts
        import math
        max_rounds = math.ceil(math.log2(args.num_districts))
        rounds_to_process = list(range(1, max_rounds + 1))
    else:
        print("Error: Must specify --round, --rounds, or --all")
        return 1

    print(f"\n{'=' * 60}")
    print(f"Redistricting Round Testing")
    print(f"{'=' * 60}\n")
    print(f"State: {args.state}")
    print(f"Districts: {args.num_districts}")
    print(f"Rounds to process: {rounds_to_process}")

    # Load blocks
    blocks_file = Path(args.blocks_dir) / f"{args.state.lower()}_blocks_2020.parquet"
    if not blocks_file.exists():
        print(f"Error: Block data not found at {blocks_file}")
        return 1

    blocks_gdf = load_blocks(str(blocks_file))

    if not args.visualize_only:
        # Run the algorithm with intermediate saving
        print("\nRunning redistricting algorithm...")

        # Load adjacency graph
        graph_file = Path(args.graph_dir) / f"{args.state.lower()}_blocks_graph.pkl"
        if not graph_file.exists():
            print(f"Error: Adjacency graph not found at {graph_file}")
            print(f"Run: python scripts/build_adjacency.py --state {args.state} --year 2020")
            return 1

        adjacency, vertex_weights, index_to_geoid, geoid_to_index = load_adjacency_graph(str(graph_file))

        # Run recursive bisection with intermediate saving
        rb = RecursiveBisection(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            num_districts=args.num_districts,
            save_intermediate=True,
            intermediate_dir=args.intermediate_dir
        )

        assignments = rb.partition()

        print("\nAlgorithm complete! Intermediate results saved.")

    # Visualize requested rounds
    print(f"\n{'=' * 60}")
    print(f"Visualizing Rounds")
    print(f"{'=' * 60}")

    for round_num in rounds_to_process:
        visualize_round(
            round_num,
            blocks_gdf,
            args.intermediate_dir,
            args.output_dir,
            args.state
        )

    print(f"\n{'=' * 60}")
    print(f"Round Testing Complete")
    print(f"{'=' * 60}\n")

    return 0


if __name__ == '__main__':
    sys.exit(main())
