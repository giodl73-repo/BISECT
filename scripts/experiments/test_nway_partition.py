"""
Test n-way partitioning (direct multi-way split without recursive bisection).

This script tests whether METIS can achieve VRA targets using direct n-way partitioning
with tpwgts only (no ubvec, no recursive bisection).
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


def main():
    parser = argparse.ArgumentParser(description='Test direct n-way VRA partitioning')
    parser.add_argument('--state', type=str, required=True, help='State code (e.g., AL)')
    parser.add_argument('--num-districts', type=int, required=True, help='Number of districts')
    parser.add_argument('--target-mm-districts', type=int, required=True, help='Target MM districts')
    parser.add_argument('--year', type=int, default=2020, help='Census year')
    parser.add_argument('--mm-target-pct', type=float, default=0.60, help='Target minority % for MM districts')
    parser.add_argument('--ubvec', type=float, default=None, help='Minority constraint imbalance tolerance (e.g., 1000 for VRA)')

    args = parser.parse_args()

    # Map state code to name
    state_map = {'AL': 'alabama', 'GA': 'georgia', 'LA': 'louisiana', 'MS': 'mississippi', 'SC': 'south_carolina'}
    state_name = state_map.get(args.state)
    if not state_name:
        print(f"ERROR: Unknown state code: {args.state}")
        return 1

    # Get state minority percentage (would normally come from demographics)
    state_minority_pcts = {'alabama': 0.369, 'georgia': 0.424, 'louisiana': 0.416, 'mississippi': 0.461, 'south_carolina': 0.351}
    state_minority_pct = state_minority_pcts[state_name]

    print(f"\n{'='*70}")
    print(f"Testing {state_name.title()} with DIRECT {args.num_districts}-way partitioning")
    print(f"{'='*70}")
    print(f"  State: {args.state} ({state_name.title()})")
    print(f"  Districts: {args.num_districts}")
    print(f"  Target MM districts: {args.target_mm_districts}")
    print(f"  State minority %: {state_minority_pct*100:.1f}%")
    print(f"  MM target %: {args.mm_target_pct*100:.1f}%")
    print(f"  Year: {args.year}")
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

    # Calculate target weights for each final district
    print("\nCalculating target weights...")

    pop_per_district = 1.0 / args.num_districts

    # MM districts: target_mm_districts at mm_target_pct minority
    mm_minority_per_district = pop_per_district * args.mm_target_pct  # as fraction of state
    mm_minority_fraction = mm_minority_per_district / state_minority_pct  # as fraction of total minority

    # Non-MM districts: split remaining minority
    total_mm_minority = args.target_mm_districts * mm_minority_per_district
    remaining_minority = state_minority_pct - total_mm_minority
    non_mm_districts = args.num_districts - args.target_mm_districts
    non_mm_minority_per_district = remaining_minority / non_mm_districts if non_mm_districts > 0 else 0
    non_mm_minority_fraction = non_mm_minority_per_district / state_minority_pct if state_minority_pct > 0 else 0

    print(f"  Population per district: {pop_per_district:.6f} ({pop_per_district*100:.2f}%)")
    print(f"  MM districts (first {args.target_mm_districts}):")
    print(f"    Target: {args.mm_target_pct*100:.1f}% minority")
    print(f"    Minority fraction of total: {mm_minority_fraction:.6f} ({mm_minority_fraction*100:.2f}%)")
    if non_mm_districts > 0:
        non_mm_pct = (non_mm_minority_per_district / pop_per_district) * 100 if pop_per_district > 0 else 0
        print(f"  Non-MM districts (remaining {non_mm_districts}):")
        print(f"    Target: {non_mm_pct:.1f}% minority")
        print(f"    Minority fraction of total: {non_mm_minority_fraction:.6f} ({non_mm_minority_fraction*100:.2f}%)")

    # Build target_weights for all partitions
    target_weights = []
    for i in range(args.num_districts):
        if i < args.target_mm_districts:
            # MM district
            target_weights.append([pop_per_district, mm_minority_fraction])
        else:
            # Non-MM district
            target_weights.append([pop_per_district, non_mm_minority_fraction])

    print(f"\ntpwgts matrix ({args.num_districts} partitions x 2 constraints):")
    for i, weights in enumerate(target_weights):
        mm_mark = "[MM]" if i < args.target_mm_districts else "    "
        print(f"  {mm_mark} Partition {i}: pop={weights[0]:.6f}, minority={weights[1]:.6f}")

    # Run n-way partitioning
    print(f"\nRunning METIS n-way partitioning (nparts={args.num_districts})...")
    if args.ubvec:
        print(f"  Using tpwgts + ubvec={args.ubvec}")
        # Build ubvec: [ufactor for pop, ubvec for minority]
        ubvec = [1.005, args.ubvec]
    else:
        print(f"  Using tpwgts only (no ubvec)")
        ubvec = None

    assignments = partition_graph_with_executable(
        adjacency=adjacency_list,
        vertex_weights=vertex_weights_vra,
        nparts=args.num_districts,
        target_weights=target_weights,
        ufactor=1.005,
        ubvec=ubvec,
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
    print(f"RESULTS - Direct {args.num_districts}-way partitioning")
    print(f"{'='*70}")
    print(f"MM Districts Created: {mm_count} / {args.target_mm_districts} (target)")
    print()

    # Show all districts sorted by minority %
    districts = sorted(vra_analysis['districts'], key=lambda d: d['pct_minority'], reverse=True)
    for dist in districts:
        mm_mark = "[MM]" if dist['is_mm'] else "     "
        print(f"{mm_mark} District {dist['district']}: {dist['pct_minority']*100:.1f}% minority")

    print(f"\n{'='*70}")
    if mm_count >= args.target_mm_districts:
        print(f"SUCCESS: Achieved {args.target_mm_districts} MM districts with principled n-way approach!")
    elif mm_count > 0:
        print(f"PARTIAL: Achieved {mm_count} MM district(s), target was {args.target_mm_districts}")
    else:
        print(f"NO MM DISTRICTS: N-way approach could not create MM districts with tpwgts only")
    print(f"{'='*70}\n")

    return 0


if __name__ == '__main__':
    sys.exit(main())
