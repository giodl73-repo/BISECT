"""
P1.1: Parameter Generalization - α Ablation Study

Tests method equivalence across different edge weight factors (α) and minority
thresholds (τ) to validate that finding generalizes beyond α=5, τ=0.40.

Experiments:
1. α sweep: α ∈ {1, 2, 3, 4, 5, 7, 10, 20, 50, 100}
2. τ sensitivity: τ ∈ {0.40, 0.45, 0.50} for α=5

Expected result: Variance drops to zero around α_crit ∈ [3,5], validating
theoretical prediction.
"""

import sys
from pathlib import Path
import numpy as np
import pandas as pd
import time

# Add project root to path
project_root = Path(__file__).parents[2]  # Go up 2 levels from gerry-adaptive-bisection to apportionment
sys.path.insert(0, str(project_root / 'src'))

from apportionment.partition.metis_wrapper import partition_graph
from apportionment.partition.recursive_bisection import RecursiveBisection


def load_state_data(state_code: str, year: int = 2020):
    """Load tract data and adjacency for a state."""
    import geopandas as gpd

    data_dir = project_root / 'outputs' / 'data' / str(year)

    # Load tracts
    tracts_file = data_dir / 'units' / f'{state_code}_tracts_{year}.parquet'
    tracts = gpd.read_parquet(tracts_file)

    # Load adjacency
    import pickle
    adj_file = data_dir / 'adjacency' / f'{state_code}_tract_adjacency.pkl'
    with open(adj_file, 'rb') as f:
        adjacency = pickle.load(f)

    return tracts, adjacency


def compute_edge_weights(tracts, adjacency, minority_threshold: float, weight_factor: float):
    """
    Create edge-weighted adjacency list.

    Args:
        tracts: GeoDataFrame with minority_vap and total_pop
        minority_threshold: τ - threshold for minority-dense tract (e.g., 0.40)
        weight_factor: α - weight for minority-minority edges

    Returns:
        edge_weights: Dict mapping (u,v) -> weight
    """
    # Calculate minority percentage per tract
    minority_pct = tracts['minority_vap'] / tracts['total_pop']
    is_minority_dense = minority_pct >= minority_threshold

    edge_weights = {}
    for u, neighbors in adjacency.items():
        for v in neighbors:
            if u < v:  # Only store each edge once
                # Both tracts are minority-dense
                if is_minority_dense.iloc[u] and is_minority_dense.iloc[v]:
                    edge_weights[(u, v)] = weight_factor
                else:
                    edge_weights[(u, v)] = 1.0

    return edge_weights


def run_predetermined_tree(tracts, adjacency, k: int, first_split: tuple,
                          minority_threshold: float, weight_factor: float):
    """Run recursive bisection with predetermined tree structure."""

    edge_weights = compute_edge_weights(tracts, adjacency, minority_threshold, weight_factor)

    rb = RecursiveBisection(
        adjacency_list=adjacency,
        vertex_weights=tracts['total_pop'].values,
        num_districts=k,
        target_population=tracts['total_pop'].sum() / k,
        tolerance=0.005,
        edge_weights=edge_weights
    )

    # Run with specific first split
    partition = rb.partition(first_split=first_split)

    # Calculate metrics
    metrics = calculate_metrics(tracts, partition, k)
    return partition, metrics


def run_nway(tracts, adjacency, k: int, minority_threshold: float, weight_factor: float):
    """Run n-way partitioning."""

    edge_weights = compute_edge_weights(tracts, adjacency, minority_threshold, weight_factor)

    # Convert adjacency dict to list format for METIS
    n = len(tracts)
    adj_list = [[] for _ in range(n)]
    for u, neighbors in adjacency.items():
        adj_list[u] = sorted(neighbors)

    # Create 2D vertex weights (population only for n-way)
    vertex_weights = tracts['total_pop'].values

    # Call METIS with edge weights
    partition = partition_graph(
        adjacency=adj_list,
        vertex_weights=vertex_weights,
        nparts=k,
        edge_weights=edge_weights,
        ufactor=1.005
    )

    metrics = calculate_metrics(tracts, partition, k)
    return partition, metrics


def calculate_metrics(tracts, partition, k: int):
    """Calculate redistricting metrics."""

    district_stats = []
    for i in range(k):
        mask = partition == i
        district_tracts = tracts[mask]

        total_pop = district_tracts['total_pop'].sum()
        minority_vap = district_tracts['minority_vap'].sum()
        minority_pct = minority_vap / total_pop if total_pop > 0 else 0

        district_stats.append({
            'district': i,
            'total_pop': total_pop,
            'minority_vap': minority_vap,
            'minority_pct': minority_pct
        })

    df_stats = pd.DataFrame(district_stats)

    # Aggregate metrics
    max_minority_pct = df_stats['minority_pct'].max()
    mm_count = (df_stats['minority_pct'] >= 0.50).sum()
    district_pcts = df_stats['minority_pct'].tolist()

    return {
        'max_minority_pct': max_minority_pct,
        'mm_count': mm_count,
        'district_pcts': district_pcts
    }


def generate_tree_structures(k: int):
    """Generate all balanced binary tree structures for k districts."""
    if k == 1:
        return [None]

    structures = []
    for left in range(1, k):
        right = k - left
        structures.append((left, right))

    return structures


def run_alpha_ablation(state_code: str, k: int, target_mm: int,
                       alpha_values: list, tau: float = 0.40):
    """
    Run α ablation study for one state.

    Tests all α values with predetermined trees and n-way to measure
    variance in outcomes as function of α.
    """

    print(f"\n{'='*70}")
    print(f"α Ablation Study: {state_code.upper()} (k={k})")
    print(f"{'='*70}")

    # Load data
    print(f"Loading data...")
    tracts, adjacency = load_state_data(state_code)

    results = []

    # Generate tree structures
    tree_structures = generate_tree_structures(k)
    print(f"Testing {len(tree_structures)} tree structures")

    for alpha in alpha_values:
        print(f"\n--- Testing α = {alpha} ---")

        # Test predetermined trees
        for tree_idx, first_split in enumerate(tree_structures):
            start_time = time.time()
            partition, metrics = run_predetermined_tree(
                tracts, adjacency, k, first_split, tau, alpha
            )
            runtime = time.time() - start_time

            results.append({
                'state': state_code,
                'k': k,
                'target_mm': target_mm,
                'method': 'predetermined',
                'tree_structure': str(first_split),
                'weight_factor': alpha,
                'minority_threshold': tau,
                'max_minority_pct': metrics['max_minority_pct'],
                'mm_count': metrics['mm_count'],
                'district_pcts': str(metrics['district_pcts']),
                'runtime': runtime
            })

            if tree_idx == 0:  # Only print first tree
                print(f"  Predetermined {first_split}: {metrics['max_minority_pct']:.1%} max, {metrics['mm_count']} MM")

        # Test n-way
        start_time = time.time()
        partition, metrics = run_nway(tracts, adjacency, k, tau, alpha)
        runtime = time.time() - start_time

        results.append({
            'state': state_code,
            'k': k,
            'target_mm': target_mm,
            'method': 'nway',
            'tree_structure': 'n/a',
            'weight_factor': alpha,
            'minority_threshold': tau,
            'max_minority_pct': metrics['max_minority_pct'],
            'mm_count': metrics['mm_count'],
            'district_pcts': str(metrics['district_pcts']),
            'runtime': runtime
        })

        print(f"  N-way: {metrics['max_minority_pct']:.1%} max, {metrics['mm_count']} MM")

        # Calculate variance across methods
        alpha_results = [r for r in results if r['weight_factor'] == alpha and r['state'] == state_code]
        max_pcts = [r['max_minority_pct'] for r in alpha_results]
        variance = np.var(max_pcts)
        std = np.std(max_pcts)

        print(f"  Variance: {variance:.6f}, Std: {std:.6f}")

    return results


def run_tau_sensitivity(state_code: str, k: int, target_mm: int,
                       alpha: float, tau_values: list):
    """
    Run τ sensitivity analysis for one state.

    Tests different minority thresholds with fixed α=5.
    """

    print(f"\n{'='*70}")
    print(f"τ Sensitivity Study: {state_code.upper()} (k={k}, α={alpha})")
    print(f"{'='*70}")

    # Load data
    print(f"Loading data...")
    tracts, adjacency = load_state_data(state_code)

    results = []
    tree_structures = generate_tree_structures(k)

    for tau in tau_values:
        print(f"\n--- Testing τ = {tau} ---")

        # Test predetermined trees
        for tree_idx, first_split in enumerate(tree_structures):
            start_time = time.time()
            partition, metrics = run_predetermined_tree(
                tracts, adjacency, k, first_split, tau, alpha
            )
            runtime = time.time() - start_time

            results.append({
                'state': state_code,
                'k': k,
                'target_mm': target_mm,
                'method': 'predetermined',
                'tree_structure': str(first_split),
                'weight_factor': alpha,
                'minority_threshold': tau,
                'max_minority_pct': metrics['max_minority_pct'],
                'mm_count': metrics['mm_count'],
                'district_pcts': str(metrics['district_pcts']),
                'runtime': runtime
            })

            if tree_idx == 0:
                print(f"  Predetermined {first_split}: {metrics['max_minority_pct']:.1%} max, {metrics['mm_count']} MM")

        # Test n-way
        start_time = time.time()
        partition, metrics = run_nway(tracts, adjacency, k, tau, alpha)
        runtime = time.time() - start_time

        results.append({
            'state': state_code,
            'k': k,
            'target_mm': target_mm,
            'method': 'nway',
            'tree_structure': 'n/a',
            'weight_factor': alpha,
            'minority_threshold': tau,
            'max_minority_pct': metrics['max_minority_pct'],
            'mm_count': metrics['mm_count'],
            'district_pcts': str(metrics['district_pcts']),
            'runtime': runtime
        })

        print(f"  N-way: {metrics['max_minority_pct']:.1%} max, {metrics['mm_count']} MM")

        # Calculate variance
        tau_results = [r for r in results if r['minority_threshold'] == tau and r['state'] == state_code]
        max_pcts = [r['max_minority_pct'] for r in tau_results]
        variance = np.var(max_pcts)

        print(f"  Variance: {variance:.6f}")

    return results


def main():
    """Run full P1.1 ablation study."""

    # Test states
    states = [
        ('alabama', 7, 2),
        ('georgia', 14, 5),
        ('louisiana', 6, 2),
        ('mississippi', 4, 2),
        ('south_carolina', 7, 3)
    ]

    # Parameter ranges
    alpha_values = [1, 2, 3, 4, 5, 7, 10, 20, 50, 100]
    tau_values = [0.40, 0.45, 0.50]

    print("="*70)
    print("P1.1: Parameter Generalization - α Ablation Study")
    print("="*70)
    print()
    print(f"States: {len(states)}")
    print(f"α values: {alpha_values}")
    print(f"τ values: {tau_values}")
    print(f"Estimated total runs: {len(states) * (len(alpha_values) * 8 + len(tau_values) * 8)}")
    print()

    # Run α ablation for all states
    print("\n" + "="*70)
    print("PART 1: α Ablation Study")
    print("="*70)

    all_results_alpha = []
    for state_code, k, target_mm in states:
        results = run_alpha_ablation(state_code, k, target_mm, alpha_values)
        all_results_alpha.extend(results)

    # Save α ablation results
    df_alpha = pd.DataFrame(all_results_alpha)
    output_file = Path('results') / 'alpha_ablation_study.csv'
    output_file.parent.mkdir(exist_ok=True)
    df_alpha.to_csv(output_file, index=False)
    print(f"\n[OK] Saved α ablation results to: {output_file}")

    # Run τ sensitivity for all states
    print("\n" + "="*70)
    print("PART 2: τ Sensitivity Analysis")
    print("="*70)

    all_results_tau = []
    for state_code, k, target_mm in states:
        results = run_tau_sensitivity(state_code, k, target_mm, alpha=5.0, tau_values=tau_values)
        all_results_tau.extend(results)

    # Save τ sensitivity results
    df_tau = pd.DataFrame(all_results_tau)
    output_file_tau = Path('results') / 'tau_sensitivity_study.csv'
    df_tau.to_csv(output_file_tau, index=False)
    print(f"\n[OK] Saved τ sensitivity results to: {output_file_tau}")

    # Summary statistics
    print("\n" + "="*70)
    print("SUMMARY: Variance by α")
    print("="*70)

    for alpha in alpha_values:
        alpha_data = df_alpha[df_alpha['weight_factor'] == alpha]
        variance_by_state = alpha_data.groupby('state')['max_minority_pct'].agg(['var', 'std'])
        avg_variance = variance_by_state['var'].mean()
        print(f"α = {alpha:5.0f}: Avg variance = {avg_variance:.6f}")

    print("\n" + "="*70)
    print("[OK] P1.1 Ablation Study Complete!")
    print("="*70)
    print()
    print("Next steps:")
    print("1. Run: python create_alpha_visualizations.py")
    print("2. Identify α_crit from variance vs. α plot")
    print("3. Update paper Section 6 with results")


if __name__ == '__main__':
    main()
