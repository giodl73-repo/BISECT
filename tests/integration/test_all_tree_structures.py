"""
Test all possible binary tree structures for recursive bisection.

For 7 districts, there are multiple ways to recursively split:
- 7 -> [1, 6], [2, 5], [3, 4], [4, 3], [5, 2], [6, 1]

Each creates different intermediate groupings and may produce different results.
"""

import pytest
import numpy as np
from src.apportionment.partition.vra_targets import build_binary_tree_structure


class TestAllTreeStructures:
    """Enumerate all possible tree structures for N districts."""

    def test_enumerate_7_district_trees(self):
        """
        Enumerate all binary tree structures for 7 districts.
        """
        print("\n=== All possible tree structures for 7 districts ===\n")

        # All possible first splits for 7 districts
        splits = [
            (1, 6),
            (2, 5),
            (3, 4),
            (4, 3),
            (5, 2),
            (6, 1),
        ]

        for left, right in splits:
            print(f"7 -> [{left}, {right}]")
            tree = build_binary_tree_structure_custom(7, first_split=(left, right))
            print_tree_structure(tree, indent=2)
            print()

    def test_all_splits_for_2_mm_districts(self):
        """
        Test which tree structure is best for 2 MM districts.

        Goal: Concentrate 2 MM districts (60% minority each) within 7 total.
        State: 36.9% minority overall.

        Different splits may work better:
        - [2, 5]: Put both MM districts in left branch (2 districts)
        - [3, 4]: Put both MM districts in left branch (3 districts)
        - [4, 3]: Put both MM districts in right branch (3 districts)
        etc.
        """
        from src.apportionment.partition.vra_targets import (
            calculate_final_district_targets,
            assign_districts_to_tree,
            calculate_node_targets
        )

        splits = [(1, 6), (2, 5), (3, 4), (4, 3), (5, 2), (6, 1)]

        print("\n=== VRA targets for different tree structures ===\n")

        for left, right in splits:
            # Build tree with this split
            tree = build_binary_tree_structure_custom(7, first_split=(left, right))

            # Calculate district targets (2 MM at 60%)
            district_targets = calculate_final_district_targets(
                num_districts=7,
                target_mm_districts=2,
                state_minority_pct=0.369,
                mm_target_pct=0.60
            )

            # Assign districts to tree (MM districts clustered left)
            tree = assign_districts_to_tree(tree, district_targets, mm_clustering='left')

            # Calculate targets
            tree = calculate_node_targets(tree, 0.369)

            # Show root split targets
            root_targets = tree['target_weights']
            left_pop, left_min = root_targets[0]
            right_pop, right_min = root_targets[1]

            print(f"7 -> [{left}, {right}]:")
            print(f"  Left:  {left_pop:.3f} pop, {left_min:.3f} minority")
            print(f"  Right: {right_pop:.3f} pop, {right_min:.3f} minority")

            # Calculate how concentrated minorities need to be in left branch
            if left_pop > 0:
                left_minority_pct_needed = (left_min * 0.369) / (left_pop * (1.0/7))
                print(f"  Left branch needs: {left_minority_pct_needed*100:.1f}% minority concentration")

            print()

    def test_optimal_tree_for_alabama(self):
        """
        Determine optimal tree structure for Alabama (2 MM districts).

        Test criterion: Which split requires the LEAST concentration?
        The easier the target, the more likely METIS can achieve it.
        """
        from src.apportionment.partition.vra_targets import (
            calculate_final_district_targets,
            assign_districts_to_tree,
            calculate_node_targets
        )

        splits = [(1, 6), (2, 5), (3, 4), (4, 3), (5, 2), (6, 1)]

        print("\n=== Optimal tree for Alabama (2 MM districts) ===\n")

        results = []
        for left, right in splits:
            tree = build_binary_tree_structure_custom(7, first_split=(left, right))
            district_targets = calculate_final_district_targets(7, 2, 0.369, 0.60)
            tree = assign_districts_to_tree(tree, district_targets, mm_clustering='left')
            tree = calculate_node_targets(tree, 0.369)

            root_targets = tree['target_weights']
            left_pop, left_min = root_targets[0]

            # Calculate concentration difficulty
            # Higher minority target with same population = harder to achieve
            difficulty = left_min / left_pop if left_pop > 0 else 0

            results.append({
                'split': (left, right),
                'left_pop': left_pop,
                'left_min': left_min,
                'difficulty': difficulty
            })

        # Sort by difficulty (easiest first)
        results.sort(key=lambda x: x['difficulty'])

        print("Tree structures ranked by feasibility (easiest first):\n")
        for i, r in enumerate(results, 1):
            left, right = r['split']
            print(f"{i}. 7 -> [{left}, {right}]:")
            print(f"   Target: {r['left_min']*100:.1f}% minorities in {r['left_pop']*100:.1f}% population")
            print(f"   Concentration needed: {r['difficulty']:.3f}")
            print()

        print(f"RECOMMENDATION: Try 7 -> {results[0]['split']} (easiest target)")


def build_binary_tree_structure_custom(num_districts: int, first_split=None) -> dict:
    """
    Build binary tree with custom first split.

    Parameters
    ----------
    num_districts : int
        Total number of districts
    first_split : tuple, optional
        (left_count, right_count) for first split
        If None, uses default (num_districts // 2, remainder)
    """
    def split_recursive(n: int, depth: int = 0, path: str = "", custom_split=None) -> dict:
        if n == 1:
            return {
                'districts': 1,
                'depth': depth,
                'path': path,
                'is_leaf': True
            }

        # Use custom split at root level if provided
        if depth == 0 and custom_split is not None:
            left_n, right_n = custom_split
        else:
            # Default: split as evenly as possible
            left_n = n // 2
            right_n = n - left_n

        return {
            'districts': n,
            'depth': depth,
            'path': path,
            'is_leaf': False,
            'left_count': left_n,
            'right_count': right_n,
            'left': split_recursive(left_n, depth + 1, path + "0"),
            'right': split_recursive(right_n, depth + 1, path + "1")
        }

    return split_recursive(num_districts, custom_split=first_split)


def print_tree_structure(tree: dict, indent: int = 0):
    """Print tree structure."""
    prefix = " " * indent

    if tree['is_leaf']:
        print(f"{prefix}Leaf: 1 district")
    else:
        print(f"{prefix}{tree['districts']} -> [{tree['left_count']}, {tree['right_count']}]")
        if 'left' in tree:
            print_tree_structure(tree['left'], indent + 2)
        if 'right' in tree:
            print_tree_structure(tree['right'], indent + 2)


class TestTreeStructureForAlabama:
    """Test different tree structures on Alabama-like geography."""

    def test_simple_alabama_all_trees(self):
        """
        Test all tree structures on simplified Alabama geography.

        14 tracts: 7 high-minority (60%), 7 low-minority (20%)
        Overall: 40% minority
        Target: 1 MM district at 60%+
        """
        from src.apportionment.partition.metis_executable import partition_graph_with_executable

        # Simplified Alabama geography
        adjacency = [
            [1, 2], [0, 2, 3], [0, 1, 3, 4], [1, 2, 4, 5], [2, 3, 5, 6],
            [3, 4, 6, 7], [4, 5, 7],  # Bridge at 6-7
            [5, 6, 8, 9], [7, 9, 10], [7, 8, 10, 11], [8, 9, 11, 12],
            [9, 10, 12, 13], [10, 11, 13], [11, 12]
        ]

        vertex_weights = np.array([
            [1000, 600], [1000, 600], [1000, 600], [1000, 600],
            [1000, 600], [1000, 600], [1000, 600],
            [1000, 200], [1000, 200], [1000, 200], [1000, 200],
            [1000, 200], [1000, 200], [1000, 200],
        ])

        print("\n=== Testing tree structures on simplified Alabama ===\n")

        # NOTE: For 2 districts, there's only one tree: 2 -> [1, 1]
        # But this demonstrates the concept for when we scale up

        target_weights = [[0.5, 0.75], [0.5, 0.25]]

        parts = partition_graph_with_executable(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            nparts=2,
            target_weights=target_weights,
            ufactor=1.005,
            niter=100,
            debug=True
        )

        p0_tracts = [i for i, p in enumerate(parts) if p == 0]
        p1_tracts = [i for i, p in enumerate(parts) if p == 1]

        def calc_stats(tracts):
            total_pop = sum(vertex_weights[i, 0] for i in tracts)
            total_minority = sum(vertex_weights[i, 1] for i in tracts)
            return total_minority / total_pop

        p0_pct = calc_stats(p0_tracts)
        p1_pct = calc_stats(p1_tracts)

        print(f"Result: {max(p0_pct, p1_pct)*100:.1f}% minority (target: 60%)")
        print(f"MM district created: {max(p0_pct, p1_pct) > 0.5}")
