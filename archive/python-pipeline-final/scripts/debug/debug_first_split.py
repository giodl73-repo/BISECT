#!/usr/bin/env python3
"""
Debug the first split of California redistricting to visualize what's happening.
Tests splitting CA into 2 partitions for 26 districts each.
"""

import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent / 'src'))

from apportionment.data.adjacency import load_adjacency_graph
from apportionment.partition.metis_wrapper import partition_graph
import numpy as np


def main():
    # Load California graph
    graph_file = Path('data/processed/ca_blocks_graph.pkl')

    if not graph_file.exists():
        print(f"Error: {graph_file} not found")
        print("Run: python scripts/build_adjacency.py --state CA --year 2020")
        return 1

    print("Loading California adjacency graph...")
    adjacency, vertex_weights, index_to_geoid, geoid_to_index = load_adjacency_graph(str(graph_file))

    n_blocks = len(vertex_weights)
    total_population = int(vertex_weights.sum())
    num_districts = 52
    ideal_district_pop = total_population / num_districts

    print(f"\n{'=' * 60}")
    print(f"California Redistricting - First Split Debug")
    print(f"{'=' * 60}")
    print(f"Total blocks: {n_blocks:,}")
    print(f"Total population: {total_population:,}")
    print(f"Target districts: {num_districts}")
    print(f"Ideal district population: {ideal_district_pop:,.0f}")

    # First split: 52 -> 26/26
    k = 52
    k_left = k // 2  # 26
    k_right = k - k_left  # 26

    print(f"\n{'=' * 60}")
    print(f"First Split Configuration")
    print(f"{'=' * 60}")
    print(f"Splitting into 2 partitions to create {k} districts")
    print(f"  Left partition:  {k_left} districts")
    print(f"  Right partition: {k_right} districts")

    # Target weights for METIS
    target_weights = [k_left / k, k_right / k]
    print(f"\nTarget weights passed to METIS:")
    print(f"  Left:  {target_weights[0]:.4f} ({target_weights[0]*100:.2f}%)")
    print(f"  Right: {target_weights[1]:.4f} ({target_weights[1]*100:.2f}%)")

    # Expected populations
    expected_left_pop = total_population * target_weights[0]
    expected_right_pop = total_population * target_weights[1]

    print(f"\nExpected populations:")
    print(f"  Left:  {expected_left_pop:,.0f}")
    print(f"    -> {k_left} districts × {expected_left_pop/k_left:,.0f} per district")
    print(f"  Right: {expected_right_pop:,.0f}")
    print(f"    -> {k_right} districts × {expected_right_pop/k_right:,.0f} per district")

    # Perform the first split using METIS
    print(f"\n{'=' * 60}")
    print(f"Calling METIS gpmetis to perform split...")
    print(f"{'=' * 60}")

    try:
        parts = partition_graph(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            nparts=2,
            target_weights=target_weights,
            recursive=True
        )

        # Analyze the split
        left_blocks = [i for i, p in enumerate(parts) if p == 0]
        right_blocks = [i for i, p in enumerate(parts) if p == 1]

        left_pop = sum(vertex_weights[i] for i in left_blocks)
        right_pop = sum(vertex_weights[i] for i in right_blocks)

        print(f"\n{'=' * 60}")
        print(f"METIS Split Results")
        print(f"{'=' * 60}")

        print(f"\nLeft partition (part 0):")
        print(f"  Blocks: {len(left_blocks):,}")
        print(f"  Population: {left_pop:,}")
        print(f"  Target: {expected_left_pop:,.0f}")
        print(f"  Difference: {left_pop - expected_left_pop:+,.0f} ({(left_pop - expected_left_pop) / expected_left_pop * 100:+.2f}%)")
        print(f"  Per district: {left_pop / k_left:,.0f} (ideal: {ideal_district_pop:,.0f})")

        print(f"\nRight partition (part 1):")
        print(f"  Blocks: {len(right_blocks):,}")
        print(f"  Population: {right_pop:,}")
        print(f"  Target: {expected_right_pop:,.0f}")
        print(f"  Difference: {right_pop - expected_right_pop:+,.0f} ({(right_pop - expected_right_pop) / expected_right_pop * 100:+.2f}%)")
        print(f"  Per district: {right_pop / k_right:,.0f} (ideal: {ideal_district_pop:,.0f})")

        print(f"\n{'=' * 60}")
        print(f"Split Balance Analysis")
        print(f"{'=' * 60}")

        # Calculate what each partition's per-district population should be
        left_per_district = left_pop / k_left
        right_per_district = right_pop / k_right

        left_deviation = (left_per_district - ideal_district_pop) / ideal_district_pop * 100
        right_deviation = (right_per_district - ideal_district_pop) / ideal_district_pop * 100

        print(f"\nIf we continue splitting these partitions...")
        print(f"Average population per final district:")
        print(f"  Left side:  {left_per_district:,.0f} ({left_deviation:+.2f}% from ideal)")
        print(f"  Right side: {right_per_district:,.0f} ({right_deviation:+.2f}% from ideal)")

        # Overall balance
        max_deviation = max(abs(left_deviation), abs(right_deviation))

        print(f"\nMaximum deviation: {max_deviation:.2f}%")

        if max_deviation > 2:
            print(f"\n⚠️  WARNING: Split is imbalanced!")
            print(f"   This imbalance will compound through recursive splits.")
            print(f"   Final districts may exceed acceptable population deviation.")
        elif max_deviation > 1:
            print(f"\n⚠️  Moderate imbalance detected.")
            print(f"   Acceptable but could be better.")
        else:
            print(f"\nOK Split looks well-balanced!")
            print(f"  Population is evenly distributed between partitions.")

        # Edge cut statistics
        print(f"\n{'=' * 60}")
        print(f"Graph Cut Statistics")
        print(f"{'=' * 60}")

        # Count edges that cross the partition boundary
        cut_edges = 0
        for i in range(n_blocks):
            part_i = parts[i]
            for neighbor in adjacency[i]:
                if parts[neighbor] != part_i:
                    cut_edges += 1

        cut_edges = cut_edges // 2  # Each edge counted twice

        print(f"Edges cut: {cut_edges:,}")
        print(f"  (Fewer cut edges = more compact districts)")

    except Exception as e:
        print(f"\n❌ Error during split: {e}")
        import traceback
        traceback.print_exc()
        return 1

    print(f"\n{'=' * 60}")
    print(f"First Split Analysis Complete")
    print(f"{'=' * 60}\n")

    return 0


if __name__ == '__main__':
    sys.exit(main())
