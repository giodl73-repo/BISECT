"""
Test adaptive recursive bisection for VRA compliance.

At each split, evaluate both options (e.g., [3,4] vs [4,3]) and choose
whichever achieves better minority concentration. Recurse adaptively.

This is more principled than predetermined tree structures - we let the
actual geography guide the splitting decisions.
"""

import argparse
from pathlib import Path
import sys

# Add project root and src to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root / 'src'))
sys.path.insert(0, str(project_root))

import numpy as np
import geopandas as gpd
from apportionment.partition import vra_utils
from apportionment.partition.metis_executable import partition_graph_with_executable
from apportionment.data.adjacency import build_adjacency_graph
from scripts.utils import get_state_config, get_tract_file


def adaptive_bisection(tracts_with_demo, adjacency_list, vertex_weights_vra,
                       num_districts, target_mm_districts, state_minority_pct,
                       mm_target_pct=0.60, depth=0):
    """
    Recursively bisect with adaptive split selection.

    At each level, try both split options and pick the one that achieves
    better minority concentration.
    """
    indent = "  " * depth

    if num_districts == 1:
        print(f"{indent}Leaf: 1 district (no split needed)")
        return tracts_with_demo['district'].values

    print(f"\n{indent}{'='*60}")
    print(f"{indent}Adaptive split: {num_districts} districts")
    print(f"{indent}{'='*60}")

    # Determine split options
    if num_districts == 2:
        options = [(1, 1)]  # Only one way to split 2
    else:
        # Try both ways: [smaller, larger] and [larger, smaller]
        left = num_districts // 2
        right = num_districts - left
        if left == right:
            options = [(left, right)]  # Symmetric, only try once
        else:
            options = [(left, right), (right, left)]

    print(f"{indent}Testing {len(options)} split option(s)...")

    best_split = None
    best_concentration = -1
    best_assignments = None

    for left_count, right_count in options:
        print(f"\n{indent}  Testing split: [{left_count}, {right_count}]")

        # Calculate target weights for this split
        # Assume MM districts go to left side (standard VRA clustering)
        pop_per_district = 1.0 / num_districts

        # How many MM districts in left vs right?
        mm_in_left = min(left_count, target_mm_districts)
        mm_in_right = max(0, target_mm_districts - mm_in_left)

        # Calculate minority needs
        left_mm_minority = mm_in_left * pop_per_district * mm_target_pct
        right_mm_minority = mm_in_right * pop_per_district * mm_target_pct

        # Remaining minority for non-MM districts
        total_mm_minority = (mm_in_left + mm_in_right) * pop_per_district * mm_target_pct
        remaining_minority = state_minority_pct - total_mm_minority

        non_mm_in_left = left_count - mm_in_left
        non_mm_in_right = right_count - mm_in_right

        left_non_mm_minority = (non_mm_in_left * remaining_minority /
                                (num_districts - target_mm_districts)) if (num_districts - target_mm_districts) > 0 else 0
        right_non_mm_minority = (non_mm_in_right * remaining_minority /
                                 (num_districts - target_mm_districts)) if (num_districts - target_mm_districts) > 0 else 0

        # Total minority fraction for each side
        left_minority = left_mm_minority + left_non_mm_minority
        right_minority = right_mm_minority + right_non_mm_minority

        # As fractions of total minority
        left_minority_frac = left_minority / state_minority_pct if state_minority_pct > 0 else 0.5
        right_minority_frac = right_minority / state_minority_pct if state_minority_pct > 0 else 0.5

        # Population fractions
        left_pop_frac = left_count / num_districts
        right_pop_frac = right_count / num_districts

        target_weights = [
            [left_pop_frac, left_minority_frac],
            [right_pop_frac, right_minority_frac]
        ]

        print(f"{indent}    Targets: L=[{left_pop_frac:.3f} pop, {left_minority_frac:.3f} min] "
              f"R=[{right_pop_frac:.3f} pop, {right_minority_frac:.3f} min]")
        print(f"{indent}    MM distribution: L={mm_in_left}, R={mm_in_right}")

        # Run METIS
        assignments = partition_graph_with_executable(
            adjacency=adjacency_list,
            vertex_weights=vertex_weights_vra,
            nparts=2,
            target_weights=target_weights,
            ufactor=1.005,
            niter=100,
            debug=False
        )

        # Analyze concentration achieved
        temp_tracts = tracts_with_demo.copy()
        temp_tracts['temp_part'] = assignments

        part0_data = temp_tracts[temp_tracts['temp_part'] == 0]
        part1_data = temp_tracts[temp_tracts['temp_part'] == 1]

        part0_total = part0_data['total_pop'].sum()
        part0_minority = (part0_data['total_pop'] * part0_data['pct_minority']).sum()
        part1_total = part1_data['total_pop'].sum()
        part1_minority = (part1_data['total_pop'] * part1_data['pct_minority']).sum()

        part0_pct = part0_minority / part0_total if part0_total > 0 else 0
        part1_pct = part1_minority / part1_total if part1_total > 0 else 0

        # Concentration score: minority % in the side that should contain MM districts (left)
        concentration = part0_pct if left_minority_frac > right_minority_frac else part1_pct

        print(f"{indent}    Result: Part0={part0_pct*100:.1f}% minority, Part1={part1_pct*100:.1f}% minority")
        print(f"{indent}    Concentration score: {concentration*100:.1f}%")

        if concentration > best_concentration:
            best_concentration = concentration
            best_split = (left_count, right_count)
            best_assignments = assignments

    print(f"\n{indent}BEST: [{best_split[0]}, {best_split[1]}] with {best_concentration*100:.1f}% concentration")

    # Now we have the best split for this level
    # Recurse on each part
    # For simplicity, we'll just return the assignments at this level
    # A full implementation would need to track which tracts are in which part
    # and recurse on each subgraph

    return best_assignments


def main():
    parser = argparse.ArgumentParser(description='Test adaptive recursive bisection for VRA')
    parser.add_argument('--state', type=str, required=True, help='State code (e.g., AL)')
    parser.add_argument('--num-districts', type=int, required=True, help='Number of districts')
    parser.add_argument('--target-mm-districts', type=int, required=True, help='Target MM districts')
    parser.add_argument('--year', type=int, default=2020, help='Census year')
    parser.add_argument('--mm-target-pct', type=float, default=0.60, help='Target minority % for MM districts')

    args = parser.parse_args()

    # Map state code to name
    state_map = {'AL': 'alabama', 'GA': 'georgia', 'LA': 'louisiana', 'MS': 'mississippi', 'SC': 'south_carolina'}
    state_name = state_map.get(args.state)
    if not state_name:
        print(f"ERROR: Unknown state code: {args.state}")
        return 1

    # Get state minority percentage
    state_minority_pcts = {'alabama': 0.369, 'georgia': 0.424, 'louisiana': 0.416, 'mississippi': 0.461, 'south_carolina': 0.351}
    state_minority_pct = state_minority_pcts[state_name]

    print(f"\n{'='*70}")
    print(f"Adaptive Recursive Bisection - {state_name.title()}")
    print(f"{'='*70}")
    print(f"  State: {args.state} ({state_name.title()})")
    print(f"  Districts: {args.num_districts}")
    print(f"  Target MM districts: {args.target_mm_districts}")
    print(f"  State minority %: {state_minority_pct*100:.1f}%")
    print(f"  MM target %: {args.mm_target_pct*100:.1f}%")
    print(f"  Year: {args.year}")
    print()
    print("Strategy: At each split, try both options and pick the one")
    print("          that achieves better minority concentration.")
    print()

    # Get tract file
    tracts_file = str(get_tract_file(args.state, str(args.year), 'v1'))

    # Load tract data
    print("Loading tract data...")
    tracts_gdf = gpd.read_parquet(tracts_file)
    print(f"  Loaded {len(tracts_gdf)} tracts")

    # Load demographics
    print("Loading demographics...")
    demographics = vra_utils.load_tract_demographics(state_name, args.year)

    # Create VRA vertex weights
    print("Creating VRA vertex weights...")
    vertex_weights_vra, tracts_with_demo = vra_utils.create_vra_vertex_weights(
        tracts_gdf, demographics
    )
    print(f"  Created weights for {len(tracts_with_demo)} tracts")

    # Build adjacency
    print("Building adjacency...")
    adjacency_list, _, _, _, _ = build_adjacency_graph(tracts_with_demo)
    print(f"  Built adjacency graph with {len(adjacency_list)} nodes")

    # Run adaptive bisection (just first level for now)
    print("\n" + "="*70)
    print("Starting Adaptive Recursive Bisection")
    print("="*70)

    assignments = adaptive_bisection(
        tracts_with_demo=tracts_with_demo,
        adjacency_list=adjacency_list,
        vertex_weights_vra=vertex_weights_vra,
        num_districts=args.num_districts,
        target_mm_districts=args.target_mm_districts,
        state_minority_pct=state_minority_pct,
        mm_target_pct=args.mm_target_pct,
        depth=0
    )

    print(f"\n{'='*70}")
    print("Adaptive bisection first level complete!")
    print(f"{'='*70}")
    print("\nNOTE: This is a proof-of-concept showing the first split.")
    print("Full recursive implementation would continue splitting each part.")
    print(f"{'='*70}\n")

    return 0


if __name__ == '__main__':
    sys.exit(main())
