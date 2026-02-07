"""
Test Alabama with direct n-way partitioning (7 districts at once).

Instead of recursive bisection (7 -> [3,4] -> ...), partition directly into 7 districts
using tpwgts to specify targets for each final district.
"""

import sys
from pathlib import Path

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

import numpy as np
from apportionment.partition import vra_utils
from apportionment.data.units_loader import load_units
from apportionment.data.adjacency import build_adjacency_queen
from apportionment.partition.metis_executable import partition_graph_with_executable


def test_alabama_nway(year=2020):
    """Test Alabama with direct 7-way partitioning."""
    state_name = "alabama"
    num_districts = 7
    target_mm_districts = 2
    state_minority_pct = 0.369
    mm_target_pct = 0.60

    print(f"\n{'='*70}")
    print(f"Testing Alabama with DIRECT 7-way partitioning")
    print(f"{'='*70}\n")

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
    adjacency = build_adjacency_queen(tracts_with_demo)

    # Calculate target weights for each final district
    print("\nCalculating target weights for 7 districts...")

    pop_per_district = 1.0 / num_districts

    # MM districts: 2 districts at 60% minority
    mm_minority_per_district = pop_per_district * mm_target_pct  # as fraction of state
    mm_minority_fraction = mm_minority_per_district / state_minority_pct  # as fraction of total minority

    # Non-MM districts: split remaining minority
    total_mm_minority = target_mm_districts * mm_minority_per_district
    remaining_minority = state_minority_pct - total_mm_minority
    non_mm_districts = num_districts - target_mm_districts
    non_mm_minority_per_district = remaining_minority / non_mm_districts  # as fraction of state
    non_mm_minority_fraction = non_mm_minority_per_district / state_minority_pct  # as fraction of total minority

    print(f"  Population per district: {pop_per_district:.6f} ({pop_per_district*100:.2f}%)")
    print(f"  MM districts (first {target_mm_districts}):")
    print(f"    Target: {mm_target_pct*100:.1f}% minority")
    print(f"    Minority fraction of total: {mm_minority_fraction:.6f} ({mm_minority_fraction*100:.2f}%)")
    print(f"  Non-MM districts (remaining {non_mm_districts}):")
    print(f"    Target: {non_mm_minority_per_district/pop_per_district*100:.1f}% minority")
    print(f"    Minority fraction of total: {non_mm_minority_fraction:.6f} ({non_mm_minority_fraction*100:.2f}%)")

    # Build target_weights for all 7 partitions
    target_weights = []
    for i in range(num_districts):
        if i < target_mm_districts:
            # MM district
            target_weights.append([pop_per_district, mm_minority_fraction])
        else:
            # Non-MM district
            target_weights.append([pop_per_district, non_mm_minority_fraction])

    print(f"\ntpwgts matrix ({num_districts} partitions x 2 constraints):")
    for i, weights in enumerate(target_weights):
        mm_mark = "[MM]" if i < target_mm_districts else "    "
        print(f"  {mm_mark} Partition {i}: pop={weights[0]:.6f}, minority={weights[1]:.6f}")

    # Run n-way partitioning
    print(f"\nRunning METIS n-way partitioning (nparts={num_districts})...")
    assignments = partition_graph_with_executable(
        adjacency=adjacency,
        vertex_weights=vertex_weights_vra,
        nparts=num_districts,
        target_weights=target_weights,
        ufactor=1.005,
        niter=100,
        debug=True
    )

    # Analyze results
    print("\nAnalyzing results...")
    tracts_with_demo['district'] = assignments

    vra_analysis = vra_utils.analyze_mm_districts(
        tracts_with_demo, assignments, mm_threshold=0.50
    )

    mm_count = vra_analysis['mm_count']
    print(f"\n{'='*70}")
    print(f"RESULTS - Direct 7-way partitioning")
    print(f"{'='*70}")
    print(f"MM Districts Created: {mm_count} / {target_mm_districts} (target)")
    print()

    # Show all districts sorted by minority %
    districts = sorted(vra_analysis['districts'], key=lambda d: d['pct_minority'], reverse=True)
    for dist in districts:
        mm_mark = "[MM]" if dist['is_mm'] else "     "
        pop = dist['total_pop']
        min_pop = dist['minority_pop']
        print(f"{mm_mark} District {dist['district']}: {dist['pct_minority']*100:.1f}% minority "
              f"(pop={pop:,}, minority={min_pop:,})")

    # Compare to recursive bisection result
    print(f"\n{'='*70}")
    print(f"COMPARISON")
    print(f"{'='*70}")
    print(f"Recursive bisection [3,4] with tpwgts: 0 MM districts (max 43%)")
    print(f"Direct 7-way with tpwgts:              {mm_count} MM districts "
          f"(max {districts[0]['pct_minority']*100:.1f}%)")
    print(f"{'='*70}\n")

    return mm_count, districts


if __name__ == '__main__':
    mm_count, districts = test_alabama_nway()

    if mm_count >= 2:
        print("SUCCESS: Achieved 2 MM districts with principled n-way approach!")
    elif mm_count == 1:
        print("PARTIAL: Achieved 1 MM district with n-way approach (better than recursive bisection).")
    else:
        print("RESULT: N-way approach produces same result as recursive bisection.")
