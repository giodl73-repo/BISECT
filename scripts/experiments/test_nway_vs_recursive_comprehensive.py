"""
Comprehensive comparison of n-way vs recursive bisection for VRA compliance.
Uses multi-constraint (tpwgts) approach for BOTH methods to isolate impact of
partitioning strategy (n-way vs recursive tree structure).

This script fills data gaps for Paper 2: N-Way vs Recursive Bisection.
"""

import sys
from pathlib import Path
import pandas as pd
import numpy as np
from scipy.sparse import coo_matrix
from collections import defaultdict
import time

project_root = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(project_root / 'src'))
sys.path.insert(0, str(project_root))

from apportionment.partition.metis_executable import partition_graph_with_executable
from apportionment.data.adjacency import load_adjacency_matrix

# State configurations
STATES = {
    'mississippi': {
        'code': 28,
        'name': 'Mississippi',
        'k': 4,
        'target_mm': 2,
        'state_minority_pct': 0.461,
        'trees': [[3, 1], [2, 2], [1, 3]]  # All balanced binary trees for k=4
    },
    'georgia': {
        'code': 13,
        'name': 'Georgia',
        'k': 14,
        'target_mm': 5,
        'state_minority_pct': 0.424,
        'trees': [[7, 7]]  # Example tree for k=14 (can add more)
    },
    'louisiana': {
        'code': 22,
        'name': 'Louisiana',
        'k': 6,
        'target_mm': 2,
        'state_minority_pct': 0.416,
        'trees': [[5, 1], [4, 2], [3, 3], [2, 4], [1, 5]]
    },
    'alabama': {
        'code': 1,
        'name': 'Alabama',
        'k': 7,
        'target_mm': 2,
        'state_minority_pct': 0.369,
        'trees': [[6, 1], [5, 2], [4, 3], [3, 4], [2, 5], [1, 6]]
    },
    'south_carolina': {
        'code': 45,
        'name': 'South Carolina',
        'k': 7,
        'target_mm': 3,
        'state_minority_pct': 0.351,
        'trees': [[6, 1], [5, 2], [4, 3], [3, 4], [2, 5], [1, 6]]
    }
}


def load_state_data(state_code, year=2020, version='v1'):
    """Load census tracts, demographics, adjacency for a state"""
    from scripts.utils.path_utils import get_tract_file, get_demographics_file, get_adjacency_file

    print(f"  Loading census tracts...")
    tracts = pd.read_csv(get_tract_file(state_code, year, version))

    print(f"  Loading demographics...")
    demographics = pd.read_csv(get_demographics_file(state_code, year, version))

    print(f"  Loading adjacency matrix...")
    adjacency = load_adjacency_matrix(get_adjacency_file(state_code, year, version))

    tracts_with_demo = tracts.merge(demographics, on='GEOID', how='inner')

    # Create VRA-compatible demographic columns
    if 'minority_vap' not in tracts_with_demo.columns:
        # Assume we have black_vap or need to compute it
        if 'black_vap' in tracts_with_demo.columns:
            tracts_with_demo['minority_vap'] = tracts_with_demo['black_vap']
        else:
            # Fallback: use total_vap * pct_minority if available
            print("  WARNING: No minority_vap column, attempting to compute...")
            if 'pct_minority' in tracts_with_demo.columns:
                tracts_with_demo['minority_vap'] = (
                    tracts_with_demo['total_vap'] * tracts_with_demo['pct_minority']
                )
            else:
                raise ValueError("Cannot determine minority_vap from available columns")

    # Compute percentage minority if not present
    if 'pct_minority' not in tracts_with_demo.columns:
        tracts_with_demo['pct_minority'] = (
            tracts_with_demo['minority_vap'] / tracts_with_demo['total_vap']
        ).fillna(0)

    return tracts_with_demo, adjacency


def create_vra_vertex_weights(tracts_with_demo):
    """Create 2D vertex weights for multi-constraint optimization"""
    return np.column_stack([
        tracts_with_demo['total_pop'].values,
        tracts_with_demo['minority_vap'].values
    ])


def create_target_weights(num_districts, mm_targets, state_minority_pct):
    """
    Create target weights (tpwgts) for multi-constraint partitioning.

    For MM districts: aim for 60% minority (1.5x state average)
    For non-MM districts: aim for remaining minority distributed
    """
    target_mm_pct = 0.60  # VRA target for MM districts

    tpwgts = []
    for i in range(num_districts):
        if i < mm_targets:
            # MM district target
            pop_weight = 1.0 / num_districts
            minority_weight = target_mm_pct / num_districts / state_minority_pct
        else:
            # Non-MM district target
            pop_weight = 1.0 / num_districts
            # Remaining minority distributed across non-MM districts
            remaining_minority = 1.0 - (mm_targets * target_mm_pct / state_minority_pct)
            minority_weight = remaining_minority / (num_districts - mm_targets)

        tpwgts.extend([pop_weight, minority_weight])

    return tpwgts


def build_adjacency_list(adjacency_matrix):
    """Convert sparse adjacency matrix to list format for METIS"""
    adjacency_list = {}
    adj_coo = adjacency_matrix.tocoo()

    for i, j in zip(adj_coo.row, adj_coo.col):
        if i not in adjacency_list:
            adjacency_list[i] = []
        if j not in adjacency_list:
            adjacency_list[j] = []
        adjacency_list[i].append(j)
        adjacency_list[j].append(i)

    return adjacency_list


def run_nway_partitioning(tracts_with_demo, adjacency, num_districts, target_mm, state_minority_pct):
    """N-way partitioning with multi-constraint (tpwgts) approach"""
    print(f"  Running n-way partitioning (k={num_districts})...")

    vertex_weights = create_vra_vertex_weights(tracts_with_demo)
    tpwgts = create_target_weights(num_districts, target_mm, state_minority_pct)
    adjacency_list = build_adjacency_list(adjacency)

    start_time = time.time()
    partition = partition_graph_with_executable(
        adjacency_list,
        vertex_weights,
        nparts=num_districts,
        target_weights=tpwgts,
        ubvec=[1.005, 1.50],  # Tight population, loose minority
        niter=100,
        debug=False
    )
    runtime = time.time() - start_time

    return partition, runtime


def run_recursive_bisection(tracts_with_demo, adjacency, num_districts, target_mm,
                           state_minority_pct, tree_structure=None):
    """
    Recursive bisection with multi-constraint (tpwgts) approach.

    tree_structure: e.g., [3, 4] means first split into 3 and 4 districts
    If None, uses adaptive tree selection (tries both orderings at each split)
    """
    print(f"  Running recursive bisection (tree={tree_structure})...")

    # For now, implement simplified version that does one split
    # Full recursive implementation would need to be more complex

    # This is a placeholder - would need full recursive tree traversal
    # For initial testing, we can call METIS with recursive mode

    vertex_weights = create_vra_vertex_weights(tracts_with_demo)
    adjacency_list = build_adjacency_list(adjacency)

    # METIS doesn't directly support tree structure specification
    # We'd need to implement our own recursive splitting logic
    # For now, return None to indicate this needs implementation

    print(f"    [PLACEHOLDER - Full recursive bisection needs implementation]")
    return None, 0.0


def analyze_partition(partition, tracts_with_demo, adjacency, target_mm_count):
    """Compute metrics for a partition"""
    # District-level minority percentages
    district_stats = defaultdict(lambda: {'pop': 0, 'minority_vap': 0})

    for idx, district in enumerate(partition):
        row = tracts_with_demo.iloc[idx]
        district_stats[district]['pop'] += row['total_pop']
        district_stats[district]['minority_vap'] += row['minority_vap']

    minority_pcts = []
    for d in district_stats.values():
        pct = d['minority_vap'] / d['pop'] if d['pop'] > 0 else 0
        minority_pcts.append(pct)

    max_minority_pct = max(minority_pcts)
    mm_count = sum(1 for pct in minority_pcts if pct >= 0.50)

    # Edge cut
    edge_cut = 0
    adj_coo = adjacency.tocoo()
    for i, j in zip(adj_coo.row, adj_coo.col):
        if i < j and partition[i] != partition[j]:
            edge_cut += 1

    # Average internal edges per district
    internal_edges = defaultdict(int)
    for i, j in zip(adj_coo.row, adj_coo.col):
        if i < j and partition[i] == partition[j]:
            internal_edges[partition[i]] += 1
    avg_internal = np.mean(list(internal_edges.values())) if internal_edges else 0

    success = (mm_count >= target_mm_count)

    return {
        'max_minority_pct': max_minority_pct,
        'mm_count': mm_count,
        'edge_cut': edge_cut,
        'avg_internal_edges': avg_internal,
        'success': success,
        'district_minorities': sorted(minority_pcts, reverse=True)
    }


def main():
    results = []

    # TEST: Run only Mississippi first
    test_states = {k: v for k, v in STATES.items() if k == 'mississippi'}

    for state_name, config in test_states.items():
        print(f"\n{'='*70}")
        print(f"Testing {config['name'].upper()}")
        print(f"{'='*70}")
        print(f"Districts: {config['k']}, Target MM: {config['target_mm']}, "
              f"State Minority: {config['state_minority_pct']:.1%}")

        # Load data
        print(f"\nLoading data for {config['name']}...")
        try:
            tracts_with_demo, adjacency = load_state_data(config['code'])
            print(f"  Loaded {len(tracts_with_demo)} tracts")
        except Exception as e:
            print(f"  ERROR loading data: {e}")
            continue

        # N-way partitioning
        try:
            partition_nway, runtime_nway = run_nway_partitioning(
                tracts_with_demo, adjacency, config['k'],
                config['target_mm'], config['state_minority_pct']
            )

            metrics_nway = analyze_partition(
                partition_nway, tracts_with_demo, adjacency, config['target_mm']
            )

            results.append({
                'state': state_name,
                'state_name': config['name'],
                'method': 'nway',
                'tree_structure': 'N/A',
                'k': config['k'],
                'target_mm': config['target_mm'],
                'state_minority_pct': config['state_minority_pct'],
                'runtime': runtime_nway,
                **metrics_nway
            })

            print(f"\n  N-WAY RESULTS:")
            print(f"    MM Districts: {metrics_nway['mm_count']}/{config['target_mm']}")
            print(f"    Max Minority: {metrics_nway['max_minority_pct']:.1%}")
            print(f"    Edge Cut: {metrics_nway['edge_cut']}")
            print(f"    Runtime: {runtime_nway:.2f}s")
            print(f"    Top 3 Districts: {[f'{p:.1%}' for p in metrics_nway['district_minorities'][:3]]}")

        except Exception as e:
            print(f"  ERROR running n-way: {e}")
            import traceback
            traceback.print_exc()

        # Recursive bisection - predetermined trees
        print(f"\n  Testing {len(config['trees'])} predetermined tree structures...")
        for tree in config['trees']:
            try:
                partition_recursive, runtime_recursive = run_recursive_bisection(
                    tracts_with_demo, adjacency, config['k'],
                    config['target_mm'], config['state_minority_pct'],
                    tree_structure=tree
                )

                if partition_recursive is None:
                    print(f"    Tree {tree}: [Not yet implemented]")
                    continue

                metrics_recursive = analyze_partition(
                    partition_recursive, tracts_with_demo, adjacency, config['target_mm']
                )

                results.append({
                    'state': state_name,
                    'state_name': config['name'],
                    'method': 'recursive_predetermined',
                    'tree_structure': str(tree),
                    'k': config['k'],
                    'target_mm': config['target_mm'],
                    'state_minority_pct': config['state_minority_pct'],
                    'runtime': runtime_recursive,
                    **metrics_recursive
                })

                print(f"    Tree {tree}: {metrics_recursive['mm_count']}/{config['target_mm']} MM, "
                      f"{metrics_recursive['max_minority_pct']:.1%} max, "
                      f"edge cut {metrics_recursive['edge_cut']}")

            except Exception as e:
                print(f"    Tree {tree}: ERROR - {e}")

        # Adaptive recursive bisection
        print(f"\n  Testing adaptive tree selection...")
        try:
            partition_adaptive, runtime_adaptive = run_recursive_bisection(
                tracts_with_demo, adjacency, config['k'],
                config['target_mm'], config['state_minority_pct'],
                tree_structure=None  # None signals adaptive
            )

            if partition_adaptive is None:
                print(f"    Adaptive: [Not yet implemented]")
            else:
                metrics_adaptive = analyze_partition(
                    partition_adaptive, tracts_with_demo, adjacency, config['target_mm']
                )

                results.append({
                    'state': state_name,
                    'state_name': config['name'],
                    'method': 'recursive_adaptive',
                    'tree_structure': 'adaptive',
                    'k': config['k'],
                    'target_mm': config['target_mm'],
                    'state_minority_pct': config['state_minority_pct'],
                    'runtime': runtime_adaptive,
                    **metrics_adaptive
                })

                print(f"    Adaptive: {metrics_adaptive['mm_count']}/{config['target_mm']} MM, "
                      f"{metrics_adaptive['max_minority_pct']:.1%} max, "
                      f"edge cut {metrics_adaptive['edge_cut']}")

        except Exception as e:
            print(f"    Adaptive: ERROR - {e}")

    # Save results
    if results:
        results_df = pd.DataFrame(results)
        output_path = project_root / 'research' / 'gerry-nway-vs-recursive' / 'results'
        output_path.mkdir(parents=True, exist_ok=True)
        output_file = output_path / 'nway_vs_recursive_comparison.csv'
        results_df.to_csv(output_file, index=False)

        print(f"\n{'='*70}")
        print(f"Results saved to {output_file}")
        print(f"{'='*70}")

        # Summary table
        print("\nSUMMARY BY STATE:")
        summary = results_df.groupby(['state_name', 'method']).agg({
            'mm_count': 'mean',
            'max_minority_pct': 'mean',
            'edge_cut': 'mean',
            'runtime': 'mean'
        }).round(3)
        print(summary.to_string())

        # Comparison: N-way vs Recursive (best predetermined)
        print("\n\nN-WAY VS RECURSIVE (BEST PREDETERMINED):")
        for state in results_df['state_name'].unique():
            state_results = results_df[results_df['state_name'] == state]
            nway = state_results[state_results['method'] == 'nway'].iloc[0] if len(state_results[state_results['method'] == 'nway']) > 0 else None
            recursive = state_results[state_results['method'] == 'recursive_predetermined']

            if nway is not None and len(recursive) > 0:
                best_recursive = recursive.loc[recursive['max_minority_pct'].idxmax()]
                gap = (nway['max_minority_pct'] - best_recursive['max_minority_pct']) * 100

                print(f"\n{state}:")
                print(f"  N-way:              {nway['max_minority_pct']:.1%} ({nway['mm_count']}/{nway['target_mm']} MM)")
                print(f"  Recursive (best):   {best_recursive['max_minority_pct']:.1%} ({best_recursive['mm_count']}/{best_recursive['target_mm']} MM)")
                print(f"  Gap:                +{gap:.1f} percentage points")

    else:
        print("\n[NO RESULTS GENERATED]")


if __name__ == '__main__':
    main()
