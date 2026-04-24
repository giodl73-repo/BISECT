"""
Comprehensive VRA testing across all methods and states.

Tests 4 approaches on 5 VRA states:
1. N-way + tpwgts only
2. N-way + ubvec=1000
3. Adaptive recursive + tpwgts only
4. Adaptive recursive + ubvec=1000
"""

import argparse
import sys
from pathlib import Path
import time

# Add paths
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root / 'src'))
sys.path.insert(0, str(project_root))

import geopandas as gpd
import numpy as np
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


def load_state_data(state_code, year):
    """Load tract data and build adjacency for a state."""
    config = VRA_STATES[state_code]
    state_name = config['name']

    tracts_file = str(get_tract_file(state_code, str(year), 'v1'))
    tracts_gdf = gpd.read_parquet(tracts_file)
    demographics = vra_utils.load_tract_demographics(state_name, year)
    vertex_weights_vra, tracts_with_demo = vra_utils.create_vra_vertex_weights(
        tracts_gdf, demographics
    )
    adjacency_list, _, _, _, _ = build_adjacency_graph(tracts_with_demo)

    return tracts_with_demo, vertex_weights_vra, adjacency_list


def calculate_target_weights(num_districts, target_mm, state_minority_pct, mm_target_pct=0.60):
    """Calculate tpwgts matrix for n-way partitioning."""
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

    return target_weights


def test_nway(tracts_with_demo, vertex_weights_vra, adjacency_list, config, ubvec=None):
    """Test n-way partitioning."""
    target_weights = calculate_target_weights(
        config['districts'], config['target_mm'], config['minority_pct']
    )

    ubvec_list = [1.005, ubvec] if ubvec else None

    assignments = partition_graph_with_executable(
        adjacency=adjacency_list,
        vertex_weights=vertex_weights_vra,
        nparts=config['districts'],
        target_weights=target_weights,
        ufactor=1.005,
        ubvec=ubvec_list,
        niter=100,
        debug=False
    )

    tracts_with_demo['district'] = assignments
    vra_analysis = vra_utils.analyze_mm_districts(
        tracts_with_demo, assignments, mm_threshold=0.50
    )

    districts = sorted(vra_analysis['districts'], key=lambda d: d['pct_minority'], reverse=True)

    return {
        'mm_count': vra_analysis['mm_count'],
        'max_minority': districts[0]['pct_minority'],
        'districts': districts
    }


def test_adaptive(tracts_with_demo, vertex_weights_vra, adjacency_list, config, ubvec=None):
    """Test adaptive recursive bisection."""
    from scripts.pipeline.test_adaptive_recursive_full import AdaptiveRecursiveBisection

    ubvec_list = [1.005, ubvec] if ubvec else None

    partitioner = AdaptiveRecursiveBisection(
        tracts_gdf=tracts_with_demo,
        vertex_weights=vertex_weights_vra,
        adjacency_list=adjacency_list,
        num_districts=config['districts'],
        target_mm_districts=config['target_mm'],
        state_minority_pct=config['minority_pct'],
        mm_target_pct=0.60,
        ubvec=ubvec_list,
        debug=False
    )

    assignments = partitioner.partition()

    tracts_with_demo['district'] = assignments
    vra_analysis = vra_utils.analyze_mm_districts(
        tracts_with_demo, assignments, mm_threshold=0.50
    )

    districts = sorted(vra_analysis['districts'], key=lambda d: d['pct_minority'], reverse=True)

    return {
        'mm_count': vra_analysis['mm_count'],
        'max_minority': districts[0]['pct_minority'],
        'districts': districts
    }


def test_state_comprehensive(state_code, year=2020):
    """Run all 4 tests on a single state."""
    config = VRA_STATES[state_code]

    print(f"\n{'='*70}")
    print(f"{config['name'].title()} ({state_code})")
    print(f"{'='*70}")
    print(f"Districts: {config['districts']}, Target MM: {config['target_mm']}, "
          f"Minority: {config['minority_pct']*100:.1f}%")

    # Load data once
    print("Loading data...", end=" ", flush=True)
    tracts_with_demo, vertex_weights_vra, adjacency_list = load_state_data(state_code, year)
    print("done")

    results = {}

    # Test 1: N-way + tpwgts only
    print("  Testing n-way + tpwgts only...", end=" ", flush=True)
    tracts_copy = tracts_with_demo.copy()
    results['nway_tpwgts'] = test_nway(tracts_copy, vertex_weights_vra, adjacency_list, config, ubvec=None)
    print(f"{results['nway_tpwgts']['mm_count']}/{config['target_mm']} MM, "
          f"{results['nway_tpwgts']['max_minority']*100:.1f}%")

    # Test 2: N-way + ubvec
    print("  Testing n-way + ubvec=1000...", end=" ", flush=True)
    tracts_copy = tracts_with_demo.copy()
    results['nway_ubvec'] = test_nway(tracts_copy, vertex_weights_vra, adjacency_list, config, ubvec=1000)
    print(f"{results['nway_ubvec']['mm_count']}/{config['target_mm']} MM, "
          f"{results['nway_ubvec']['max_minority']*100:.1f}%")

    # Test 3: Adaptive + tpwgts only
    print("  Testing adaptive + tpwgts only...", end=" ", flush=True)
    tracts_copy = tracts_with_demo.copy()
    results['adaptive_tpwgts'] = test_adaptive(tracts_copy, vertex_weights_vra, adjacency_list, config, ubvec=None)
    print(f"{results['adaptive_tpwgts']['mm_count']}/{config['target_mm']} MM, "
          f"{results['adaptive_tpwgts']['max_minority']*100:.1f}%")

    # Test 4: Adaptive + ubvec
    print("  Testing adaptive + ubvec=1000...", end=" ", flush=True)
    tracts_copy = tracts_with_demo.copy()
    results['adaptive_ubvec'] = test_adaptive(tracts_copy, vertex_weights_vra, adjacency_list, config, ubvec=1000)
    print(f"{results['adaptive_ubvec']['mm_count']}/{config['target_mm']} MM, "
          f"{results['adaptive_ubvec']['max_minority']*100:.1f}%")

    return {
        'state': state_code,
        'config': config,
        'results': results
    }


def main():
    parser = argparse.ArgumentParser(description='Comprehensive VRA testing')
    parser.add_argument('--states', type=str, default='AL,GA,LA,MS,SC', help='Comma-separated state codes')
    parser.add_argument('--year', type=int, default=2020, help='Census year')

    args = parser.parse_args()

    states = args.states.split(',')

    print(f"\n{'='*70}")
    print(f"COMPREHENSIVE VRA TESTING")
    print(f"{'='*70}")
    print(f"States: {', '.join(states)}")
    print(f"Methods: N-way + Adaptive, each with tpwgts only and ubvec=1000")
    print(f"{'='*70}")

    all_results = []
    for state in states:
        if state not in VRA_STATES:
            print(f"\nSkipping {state} (not configured)")
            continue

        result = test_state_comprehensive(state, args.year)
        all_results.append(result)

    # Summary table
    print(f"\n\n{'='*70}")
    print(f"SUMMARY TABLE")
    print(f"{'='*70}\n")

    # Header
    print(f"{'State':<6} {'Target':<8} ", end="")
    print(f"{'N-way':<12} {'N-way+ubvec':<12} ", end="")
    print(f"{'Adaptive':<12} {'Adaptive+ubvec':<12}")
    print(f"{'-'*70}")

    # Rows
    for r in all_results:
        state = r['state']
        config = r['config']
        res = r['results']

        target_str = f"{config['target_mm']} MM"

        nway_t = f"{res['nway_tpwgts']['mm_count']}MM/{res['nway_tpwgts']['max_minority']*100:.1f}%"
        nway_u = f"{res['nway_ubvec']['mm_count']}MM/{res['nway_ubvec']['max_minority']*100:.1f}%"
        adap_t = f"{res['adaptive_tpwgts']['mm_count']}MM/{res['adaptive_tpwgts']['max_minority']*100:.1f}%"
        adap_u = f"{res['adaptive_ubvec']['mm_count']}MM/{res['adaptive_ubvec']['max_minority']*100:.1f}%"

        print(f"{state:<6} {target_str:<8} {nway_t:<12} {nway_u:<12} {adap_t:<12} {adap_u:<12}")

    # Success counts
    print(f"\n{'-'*70}")
    print("Success = Achieved target MM districts")
    print(f"{'-'*70}")

    for method_name, method_key in [
        ('N-way tpwgts', 'nway_tpwgts'),
        ('N-way ubvec', 'nway_ubvec'),
        ('Adaptive tpwgts', 'adaptive_tpwgts'),
        ('Adaptive ubvec', 'adaptive_ubvec')
    ]:
        success = sum(1 for r in all_results
                     if r['results'][method_key]['mm_count'] >= r['config']['target_mm'])
        print(f"{method_name:<20}: {success}/{len(all_results)} states")

    print(f"\n{'='*70}\n")


if __name__ == '__main__':
    sys.exit(main())
