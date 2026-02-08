"""
Recursive bisection implementation for Paper 2.

Implements:
1. Predetermined tree structures (e.g., [3,4] for k=7)
2. Adaptive tree selection (locally optimal at each split)

Uses multi-constraint (tpwgts) approach for fair comparison with n-way.
"""

import sys
from pathlib import Path
import pandas as pd
import numpy as np
import time
from collections import defaultdict
from typing import List, Tuple

project_root = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(project_root / 'src'))
sys.path.insert(0, str(project_root))

import geopandas as gpd
from apportionment.data.adjacency import build_adjacency_graph
from apportionment.partition.metis_executable import partition_graph_with_executable


def generate_balanced_trees(k: int) -> List[Tuple[int, int]]:
    """Generate all balanced binary tree structures for k partitions"""
    trees = []
    for left_size in range(1, k):
        right_size = k - left_size
        trees.append((left_size, right_size))
    return trees


def create_target_weights_2d(num_districts, mm_targets, state_minority_pct):
    """Create 2D target weights for multi-constraint partitioning"""
    target_mm_pct = 0.60  # Aim for 60% minority in MM districts

    tpwgts = []
    for i in range(num_districts):
        pop_weight = 1.0 / num_districts

        if i < mm_targets:
            # MM district: aim for 60% minority
            minority_weight = target_mm_pct / num_districts / state_minority_pct
        else:
            # Non-MM district: remaining minority distributed
            remaining = 1.0 - (mm_targets * target_mm_pct / state_minority_pct)
            minority_weight = remaining / (num_districts - mm_targets)

        tpwgts.extend([pop_weight, minority_weight])

    return tpwgts


def partition_subset(tracts_subset, target_k, target_mm, state_minority_pct):
    """
    Partition a subset of tracts into target_k districts.
    Returns district assignments (0 to target_k-1)
    """
    if len(tracts_subset) == 0:
        return np.array([])

    # Build adjacency for this subset
    adjacency_list, _, _, _, _ = build_adjacency_graph(tracts_subset)

    # Create 2D vertex weights
    vertex_weights_2d = np.column_stack([
        tracts_subset['total_pop'].values,
        tracts_subset['minority_vap'].values
    ])

    # Create target weights
    tpwgts = create_target_weights_2d(target_k, target_mm, state_minority_pct)

    # Run METIS
    partition = partition_graph_with_executable(
        adjacency_list,
        vertex_weights_2d,
        nparts=target_k,
        target_weights=tpwgts,
        ubvec=[1.005, 1.50],
        niter=100,
        debug=False
    )

    return partition


def recursive_bisection_predetermined(tracts, tree_structure, target_mm, state_minority_pct):
    """
    Recursive bisection with predetermined tree structure.

    Args:
        tracts: GeoDataFrame with tracts
        tree_structure: tuple like (3, 4) for first split
        target_mm: number of MM districts to create
        state_minority_pct: state-level minority percentage

    Returns:
        np.array: district assignments for each tract
    """
    k = sum(tree_structure) if isinstance(tree_structure, tuple) else tree_structure

    # Base case: k=1, all tracts in one district
    if k == 1:
        return np.zeros(len(tracts), dtype=int)

    # Recursive case: split into two groups
    left_size, right_size = tree_structure

    # Determine MM target distribution
    # Assume MM districts go to left partition (can be made smarter)
    left_mm = min(target_mm, left_size)
    right_mm = max(0, target_mm - left_mm)

    # Do binary split
    print(f"    Splitting {len(tracts)} tracts into [{left_size}, {right_size}] (MM targets: [{left_mm}, {right_mm}])")

    binary_partition = partition_subset(tracts, 2, left_mm, state_minority_pct)

    # Split tracts into left and right groups
    left_mask = (binary_partition == 0)
    right_mask = (binary_partition == 1)

    left_tracts = tracts[left_mask].reset_index(drop=True)
    right_tracts = tracts[right_mask].reset_index(drop=True)

    # Recursively partition each side
    if left_size == 1:
        left_assignments = np.zeros(len(left_tracts), dtype=int)
    else:
        # Need to further split left side - for simplicity, use balanced split
        left_tree = generate_balanced_trees(left_size)[len(generate_balanced_trees(left_size))//2]
        left_assignments = recursive_bisection_predetermined(
            left_tracts, left_tree, left_mm, state_minority_pct
        )

    if right_size == 1:
        right_assignments = np.zeros(len(right_tracts), dtype=int)
    else:
        # Need to further split right side
        right_tree = generate_balanced_trees(right_size)[len(generate_balanced_trees(right_size))//2]
        right_assignments = recursive_bisection_predetermined(
            right_tracts, right_tree, right_mm, state_minority_pct
        )

    # Combine assignments (right side gets offset by left_size)
    final_assignments = np.zeros(len(tracts), dtype=int)

    # Map back to original indices
    final_assignments[left_mask] = left_assignments
    final_assignments[right_mask] = right_assignments + left_size

    return final_assignments


def adaptive_recursive_bisection(tracts, k, target_mm, state_minority_pct):
    """
    Adaptive recursive bisection - chooses tree structure based on results.

    At each split, tries both orderings and chooses the one that achieves
    better minority concentration.
    """
    if k == 1:
        return np.zeros(len(tracts), dtype=int)

    # Try all possible splits and choose best
    best_partition = None
    best_score = -float('inf')
    best_split = None

    for left_size in range(1, k):
        right_size = k - left_size

        # Determine MM target distribution
        left_mm = min(target_mm, left_size)
        right_mm = max(0, target_mm - left_mm)

        print(f"    Trying split [{left_size}, {right_size}] (MM targets: [{left_mm}, {right_mm}])")

        # Do binary split
        binary_partition = partition_subset(tracts, 2, left_mm, state_minority_pct)

        # Evaluate quality of this split
        # Score = max minority percentage achievable
        left_mask = (binary_partition == 0)
        right_mask = (binary_partition == 1)

        left_tracts = tracts[left_mask]
        right_tracts = tracts[right_mask]

        # Compute minority percentages for left and right groups
        left_minority_pct = left_tracts['minority_vap'].sum() / left_tracts['total_pop'].sum()
        right_minority_pct = right_tracts['minority_vap'].sum() / right_tracts['total_pop'].sum()

        # Score = max of the two groups (higher is better)
        score = max(left_minority_pct, right_minority_pct)

        if score > best_score:
            best_score = score
            best_partition = binary_partition
            best_split = (left_size, right_size)

    print(f"    Best split: {best_split} (score: {best_score:.1%})")

    # Use best split and recurse
    left_size, right_size = best_split
    left_mm = min(target_mm, left_size)
    right_mm = max(0, target_mm - left_mm)

    left_mask = (best_partition == 0)
    right_mask = (best_partition == 1)

    left_tracts = tracts[left_mask].reset_index(drop=True)
    right_tracts = tracts[right_mask].reset_index(drop=True)

    # Recursively partition each side
    if left_size == 1:
        left_assignments = np.zeros(len(left_tracts), dtype=int)
    else:
        left_assignments = adaptive_recursive_bisection(
            left_tracts, left_size, left_mm, state_minority_pct
        )

    if right_size == 1:
        right_assignments = np.zeros(len(right_tracts), dtype=int)
    else:
        right_assignments = adaptive_recursive_bisection(
            right_tracts, right_size, right_mm, state_minority_pct
        )

    # Combine assignments
    final_assignments = np.zeros(len(tracts), dtype=int)
    final_assignments[left_mask] = left_assignments
    final_assignments[right_mask] = right_assignments + left_size

    return final_assignments


def analyze_partition(partition, tracts_gdf):
    """Analyze partition results"""
    tracts_gdf['district'] = partition

    # District-level stats
    district_stats = tracts_gdf.groupby('district').agg({
        'total_pop': 'sum',
        'minority_vap': 'sum'
    })

    district_stats['pct_minority'] = (
        district_stats['minority_vap'] / district_stats['total_pop']
    )

    max_minority = district_stats['pct_minority'].max()
    mm_count = (district_stats['pct_minority'] >= 0.50).sum()

    return {
        'max_minority_pct': max_minority,
        'mm_count': mm_count,
        'district_minorities': sorted(district_stats['pct_minority'].values, reverse=True)
    }


def load_tracts_and_demographics(state_name, fips_code, year='2020'):
    """Load census tracts with demographics"""
    data_dir = project_root / 'data' / year
    tiger_dir = data_dir / 'tiger' / 'tracts' / f'tl_{year}_{fips_code}_tract'
    demographics_file = data_dir / 'demographics' / f'{state_name}_demographics_{year}.csv'

    # Load tract shapefile
    shapefiles = list(tiger_dir.glob('*.shp'))
    if not shapefiles:
        raise FileNotFoundError(f"No shapefile found in {tiger_dir}")
    tracts = gpd.read_file(shapefiles[0])

    # Load demographics
    demographics = pd.read_csv(demographics_file)

    # Ensure GEOID types match (zero-pad demographics)
    tracts['GEOID'] = tracts['GEOID'].astype(str)
    demographics['GEOID'] = demographics['GEOID'].astype(str).str.zfill(11)

    # Merge
    tracts = tracts.merge(demographics, on='GEOID', how='inner')

    # Compute minority population
    if 'black_non_hispanic' in tracts.columns:
        tracts['minority_vap'] = tracts['black_non_hispanic']
        tracts['minority_total'] = tracts['total_pop']
    else:
        raise ValueError("Cannot determine minority population")

    tracts['pct_minority'] = (tracts['minority_vap'] / tracts['minority_total']).fillna(0)

    # Add population column for build_adjacency_graph
    if 'population' not in tracts.columns:
        tracts['population'] = tracts['total_pop']

    return tracts


# State configurations with FIPS codes
STATES = {
    'mississippi': {'fips': '28', 'k': 4, 'target_mm': 2, 'minority_pct': 0.461},
    'georgia': {'fips': '13', 'k': 14, 'target_mm': 5, 'minority_pct': 0.424},
    'louisiana': {'fips': '22', 'k': 6, 'target_mm': 2, 'minority_pct': 0.416},
    'alabama': {'fips': '01', 'k': 7, 'target_mm': 2, 'minority_pct': 0.369},
    'south_carolina': {'fips': '45', 'k': 7, 'target_mm': 3, 'minority_pct': 0.351}
}


def main():
    """Run recursive bisection experiments"""
    results = []

    # All 5 VRA states
    test_states = ['mississippi', 'louisiana', 'alabama', 'south_carolina', 'georgia']

    for state_name in test_states:
        config = STATES[state_name]

        print(f"\n{'='*70}")
        print(f"Testing {state_name.upper()}")
        print(f"{'='*70}")

        # Load data
        print("Loading data...")
        tracts = load_tracts_and_demographics(state_name, config['fips'])
        print(f"  Loaded {len(tracts)} tracts")

        # Test predetermined tree: [3,4]
        print(f"\nRunning RECURSIVE BISECTION (predetermined tree [3,4])...")
        start_time = time.time()

        partition_recursive = recursive_bisection_predetermined(
            tracts, (3, 4), config['target_mm'], config['minority_pct']
        )

        runtime_recursive = time.time() - start_time

        analysis_recursive = analyze_partition(partition_recursive, tracts.copy())

        results.append({
            'state': state_name,
            'method': 'recursive_predetermined',
            'tree_structure': '[3,4]',
            'k': config['k'],
            'target_mm': config['target_mm'],
            'state_minority_pct': config['minority_pct'],
            'runtime': runtime_recursive,
            **analysis_recursive
        })

        print(f"\n  RECURSIVE [3,4] RESULTS:")
        print(f"    MM Districts: {analysis_recursive['mm_count']}/{config['target_mm']}")
        print(f"    Max Minority: {analysis_recursive['max_minority_pct']:.1%}")
        top3 = [f"{p:.1%}" for p in analysis_recursive['district_minorities'][:3]]
        print(f"    Top 3 Districts: {top3}")
        print(f"    Runtime: {runtime_recursive:.2f}s")

        # Test adaptive (DISABLED for speed - takes ~50s per state)
        # Set run_adaptive = True to enable
        run_adaptive = False

        if run_adaptive:
            print(f"\nRunning ADAPTIVE RECURSIVE BISECTION...")
            start_time = time.time()

            partition_adaptive = adaptive_recursive_bisection(
                tracts.copy(), config['k'], config['target_mm'], config['minority_pct']
            )

            runtime_adaptive = time.time() - start_time

            analysis_adaptive = analyze_partition(partition_adaptive, tracts.copy())

            results.append({
                'state': state_name,
                'method': 'recursive_adaptive',
                'tree_structure': 'adaptive',
                'k': config['k'],
                'target_mm': config['target_mm'],
                'state_minority_pct': config['minority_pct'],
                'runtime': runtime_adaptive,
                **analysis_adaptive
            })

            print(f"\n  ADAPTIVE RESULTS:")
            print(f"    MM Districts: {analysis_adaptive['mm_count']}/{config['target_mm']}")
            print(f"    Max Minority: {analysis_adaptive['max_minority_pct']:.1%}")
            top3_adaptive = [f"{p:.1%}" for p in analysis_adaptive['district_minorities'][:3]]
            print(f"    Top 3 Districts: {top3_adaptive}")
            print(f"    Runtime: {runtime_adaptive:.2f}s")

    # Save results
    if results:
        results_df = pd.DataFrame(results)
        results_for_csv = results_df.drop(columns=['district_minorities'])

        output_path = project_root / 'research' / 'gerry-nway-vs-recursive' / 'results'
        output_path.mkdir(parents=True, exist_ok=True)
        output_file = output_path / 'recursive_bisection_results.csv'
        results_for_csv.to_csv(output_file, index=False)

        print(f"\n{'='*70}")
        print(f"Results saved to {output_file}")
        print(f"{'='*70}")


if __name__ == '__main__':
    main()
