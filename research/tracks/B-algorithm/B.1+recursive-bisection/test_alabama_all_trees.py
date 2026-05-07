"""
Test Alabama with all possible tree structures.

For 7 districts, try all 6 possible first splits:
- 7 -> [1, 6], [2, 5], [3, 4], [4, 3], [5, 2], [6, 1]
"""

import sys
from pathlib import Path

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

import numpy as np
from apportionment.partition.vra_targets import create_vra_target_tree
from apportionment.partition.recursive_bisection import RecursiveBisection
from apportionment.partition import vra_utils
from apportionment.data.units_loader import load_units

def test_alabama_with_tree(first_split, year=2020):
    """Test Alabama with a specific tree structure."""
    state_name = "alabama"
    num_districts = 7
    target_mm_districts = 2

    print(f"\n{'='*70}")
    print(f"Testing 7 -> {list(first_split)}")
    print(f"{'='*70}")

    # Load tract data
    print("Loading tract data...")
    tracts_gdf = load_units(state_name, year, 'tract')

    # Load demographics
    print("Loading demographics...")
    demographics = vra_utils.load_tract_demographics(state_name, year)

    # Create VRA vertex weights
    vertex_weights_vra, tracts_with_demo = vra_utils.create_vra_vertex_weights(
        tracts_gdf, demographics
    )

    # Build adjacency
    print("Building adjacency...")
    from apportionment.data.adjacency import build_adjacency_queen
    adjacency = build_adjacency_queen(tracts_with_demo)

    # Create VRA target tree with custom first split
    print(f"Creating VRA target tree with first split: {first_split}...")
    vra_target_tree = create_vra_target_tree(
        num_districts=num_districts,
        target_mm_districts=target_mm_districts,
        state_minority_pct=0.369,
        mm_target_pct=0.60,
        first_split=first_split
    )

    # Show root targets
    root_targets = vra_target_tree['target_weights']
    left_pop, left_min = root_targets[0]
    right_pop, right_min = root_targets[1]
    print(f"Root split targets:")
    print(f"  Left ({first_split[0]} districts):  {left_pop:.3f} pop, {left_min:.3f} minority")
    print(f"  Right ({first_split[1]} districts): {right_pop:.3f} pop, {right_min:.3f} minority")
    print(f"  Concentration needed: {left_min / left_pop:.3f}")

    # Run recursive bisection
    print("Running recursive bisection...")
    partitioner = RecursiveBisection(
        adjacency=adjacency,
        vertex_weights=vertex_weights_vra,
        num_districts=num_districts,
        state_code="AL",
        vra_mode=True,
        vra_target_weights=vra_target_tree,
        debug=False
    )

    assignments = partitioner.partition()

    # Analyze results
    print("\nAnalyzing results...")
    vra_analysis = vra_utils.analyze_mm_districts(
        tracts_with_demo, assignments, mm_threshold=0.50
    )

    mm_count = vra_analysis['mm_count']
    print(f"\n{'='*70}")
    print(f"RESULTS for 7 -> {list(first_split)}")
    print(f"{'='*70}")
    print(f"MM Districts Created: {mm_count} / 2 (target)")

    # Show all districts sorted by minority %
    districts = sorted(vra_analysis['districts'], key=lambda d: d['pct_minority'], reverse=True)
    for dist in districts:
        mm_mark = "[MM]" if dist['is_mm'] else "     "
        print(f"{mm_mark} District {dist['district']}: {dist['pct_minority']*100:.1f}% minority")

    return mm_count, districts[0]['pct_minority']


def main():
    """Test all tree structures."""
    # All possible first splits for 7 districts
    splits = [
        (6, 1),  # Optimal (easiest)
        (5, 2),
        (4, 3),
        (3, 4),  # Current default
        (2, 5),
        (1, 6),  # Hardest
    ]

    results = []

    for split in splits:
        try:
            mm_count, max_minority = test_alabama_with_tree(split)
            results.append({
                'split': split,
                'mm_count': mm_count,
                'max_minority': max_minority,
                'success': mm_count >= 2
            })
        except Exception as e:
            print(f"\nERROR with split {split}: {e}")
            results.append({
                'split': split,
                'mm_count': 0,
                'max_minority': 0,
                'success': False,
                'error': str(e)
            })

    # Summary
    print(f"\n\n{'='*70}")
    print("SUMMARY: All Tree Structures")
    print(f"{'='*70}\n")

    print(f"{'Split':<15} {'MM Count':<12} {'Max Minority':<15} {'Success'}")
    print(f"{'-'*70}")

    for r in results:
        split_str = f"7 -> {list(r['split'])}"
        mm_str = f"{r['mm_count']} / 2"
        max_str = f"{r['max_minority']*100:.1f}%"
        success_str = "YES" if r['success'] else "NO"
        print(f"{split_str:<15} {mm_str:<12} {max_str:<15} {success_str}")

    print(f"\n{'='*70}")
    best = max(results, key=lambda r: (r['mm_count'], r['max_minority']))
    print(f"BEST: 7 -> {list(best['split'])} with {best['mm_count']} MM districts")
    print(f"{'='*70}\n")


if __name__ == '__main__':
    main()
