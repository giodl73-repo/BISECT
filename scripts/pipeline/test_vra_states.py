"""
Test VRA compliance across multiple deficit states.

Tests n-way partitioning approach on states with VRA concerns.
"""

import argparse
import sys
from pathlib import Path

# Add paths
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root / 'src'))
sys.path.insert(0, str(project_root))

import geopandas as gpd
from apportionment.partition import vra_utils
from apportionment.partition.metis_executable import partition_graph_with_executable
from apportionment.data.adjacency import build_adjacency_graph
from scripts.utils import get_tract_file

# VRA state configurations
VRA_STATES = {
    'AL': {'name': 'alabama', 'districts': 7, 'target_mm': 2, 'minority_pct': 0.369},
    'GA': {'name': 'georgia', 'districts': 14, 'target_mm': 5, 'minority_pct': 0.424},
    'LA': {'name': 'louisiana', 'districts': 6, 'target_mm': 2, 'minority_pct': 0.416},
    'MS': {'name': 'mississippi', 'districts': 4, 'target_mm': 2, 'minority_pct': 0.461},
    'SC': {'name': 'south_carolina', 'districts': 7, 'target_mm': 3, 'minority_pct': 0.351},
}


def test_state(state_code, year=2020, ubvec=None, mm_target_pct=0.60):
    """Test n-way partitioning on a single state."""
    config = VRA_STATES[state_code]
    state_name = config['name']
    num_districts = config['districts']
    target_mm = config['target_mm']
    state_minority_pct = config['minority_pct']

    print(f"\n{'='*70}")
    print(f"{state_name.title()} ({state_code})")
    print(f"{'='*70}")
    print(f"Districts: {num_districts}, Target MM: {target_mm}, Minority: {state_minority_pct*100:.1f}%")

    try:
        # Load data
        tracts_file = str(get_tract_file(state_code, str(year), 'v1'))
        tracts_gdf = gpd.read_parquet(tracts_file)
        demographics = vra_utils.load_tract_demographics(state_name, year)
        vertex_weights_vra, tracts_with_demo = vra_utils.create_vra_vertex_weights(
            tracts_gdf, demographics
        )
        adjacency_list, _, _, _, _ = build_adjacency_graph(tracts_with_demo)

        # Calculate target weights
        pop_per_district = 1.0 / num_districts
        mm_minority_per_district = pop_per_district * mm_target_pct
        mm_minority_fraction = mm_minority_per_district / state_minority_pct

        total_mm_minority = target_mm * mm_minority_per_district
        remaining_minority = state_minority_pct - total_mm_minority
        non_mm_districts = num_districts - target_mm
        non_mm_minority_per_district = remaining_minority / non_mm_districts if non_mm_districts > 0 else 0
        non_mm_minority_fraction = non_mm_minority_per_district / state_minority_pct if state_minority_pct > 0 else 0

        target_weights = []
        for i in range(num_districts):
            if i < target_mm:
                target_weights.append([pop_per_district, mm_minority_fraction])
            else:
                target_weights.append([pop_per_district, non_mm_minority_fraction])

        # Run METIS
        ubvec_list = [1.005, ubvec] if ubvec else None
        assignments = partition_graph_with_executable(
            adjacency=adjacency_list,
            vertex_weights=vertex_weights_vra,
            nparts=num_districts,
            target_weights=target_weights,
            ufactor=1.005,
            ubvec=ubvec_list,
            niter=100,
            debug=False
        )

        # Analyze
        tracts_with_demo['district'] = assignments
        vra_analysis = vra_utils.analyze_mm_districts(
            tracts_with_demo, assignments, mm_threshold=0.50
        )

        mm_count = vra_analysis['mm_count']
        districts = sorted(vra_analysis['districts'], key=lambda d: d['pct_minority'], reverse=True)

        print(f"Result: {mm_count} / {target_mm} MM districts")
        for i, dist in enumerate(districts[:3]):  # Show top 3
            mm_mark = "[MM]" if dist['is_mm'] else "     "
            print(f"  {mm_mark} District {dist['district']}: {dist['pct_minority']*100:.1f}% minority")

        return {
            'state': state_code,
            'mm_count': mm_count,
            'target_mm': target_mm,
            'max_minority': districts[0]['pct_minority'],
            'success': mm_count >= target_mm
        }

    except Exception as e:
        print(f"ERROR: {e}")
        return {
            'state': state_code,
            'mm_count': 0,
            'target_mm': target_mm,
            'max_minority': 0,
            'success': False,
            'error': str(e)
        }


def main():
    parser = argparse.ArgumentParser(description='Test VRA compliance across multiple states')
    parser.add_argument('--states', type=str, default='AL,GA,LA,MS,SC', help='Comma-separated state codes')
    parser.add_argument('--year', type=int, default=2020, help='Census year')
    parser.add_argument('--ubvec', type=float, default=None, help='Minority constraint imbalance (e.g., 1000)')
    parser.add_argument('--mm-target-pct', type=float, default=0.60, help='Target minority % for MM districts')

    args = parser.parse_args()

    states = args.states.split(',')
    mode = f"ubvec={args.ubvec}" if args.ubvec else "tpwgts only"

    print(f"\n{'='*70}")
    print(f"VRA Compliance Testing - N-Way Partitioning")
    print(f"{'='*70}")
    print(f"Mode: {mode}")
    print(f"States: {', '.join(states)}")
    print(f"{'='*70}")

    results = []
    for state in states:
        if state not in VRA_STATES:
            print(f"\nSkipping {state} (not in VRA_STATES)")
            continue

        result = test_state(state, args.year, args.ubvec, args.mm_target_pct)
        results.append(result)

    # Summary
    print(f"\n\n{'='*70}")
    print(f"SUMMARY - {mode}")
    print(f"{'='*70}\n")
    print(f"{'State':<8} {'Result':<15} {'Max Minority':<15} {'Success'}")
    print(f"{'-'*70}")

    for r in results:
        result_str = f"{r['mm_count']} / {r['target_mm']}"
        max_str = f"{r['max_minority']*100:.1f}%"
        success_str = "YES" if r['success'] else "NO"
        print(f"{r['state']:<8} {result_str:<15} {max_str:<15} {success_str}")

    success_count = sum(1 for r in results if r['success'])
    print(f"\n{success_count} / {len(results)} states achieved VRA targets\n")


if __name__ == '__main__':
    sys.exit(main())
