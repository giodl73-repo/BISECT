"""
Comprehensive minority-edge weighting study across all VRA states.

This is SINGLE-OBJECTIVE optimization with weighted edges:
- NO tpwgts (no target weights)
- NO ubvec (no multi-constraint relaxation)
- ONLY edge_weights (weighted graph edges)

Ablation study tests:
1. Weight factors: 1x (baseline), 5x, 10x, 50x, 100x, 500x, 1000x
2. Minority thresholds: 40%, 45%, 50%, 55%
3. All 5 VRA states: AL, GA, LA, MS, SC
"""

import sys
from pathlib import Path
import pandas as pd
import numpy as np

# Add paths
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root / 'src'))
sys.path.insert(0, str(project_root))

import geopandas as gpd
from scipy.sparse import lil_matrix
from apportionment.data.adjacency import build_adjacency_graph
from apportionment.partition.metis_executable import partition_graph_with_executable
from apportionment.partition.vra_utils import load_tract_demographics, analyze_mm_districts, create_vra_vertex_weights
from scripts.utils import get_tract_file

# VRA state configurations
VRA_STATES = {
    'AL': {'name': 'alabama', 'districts': 7, 'target_mm': 2, 'minority_pct': 0.369},
    'GA': {'name': 'georgia', 'districts': 14, 'target_mm': 5, 'minority_pct': 0.424},
    'LA': {'name': 'louisiana', 'districts': 6, 'target_mm': 2, 'minority_pct': 0.416},
    'MS': {'name': 'mississippi', 'districts': 4, 'target_mm': 2, 'minority_pct': 0.461},
    'SC': {'name': 'south_carolina', 'districts': 7, 'target_mm': 3, 'minority_pct': 0.351},
}

def create_minority_edge_weights(adj_matrix, tracts_with_demo, minority_threshold=0.50, weight_factor=100):
    """
    Create edge weight dictionary for minority-minority edges.

    This is the KEY INNOVATION: edges between minority tracts get higher weights,
    making them "expensive" to cut. METIS minimizes total edge cut weight, so it
    will try to keep minority communities together.
    """
    # Identify minority tracts
    is_minority = tracts_with_demo['pct_minority'] >= minority_threshold

    # Create edge weights
    edge_weights = {}
    minority_edge_count = 0

    # Convert to COO format for iteration
    adj_coo = adj_matrix.tocoo()

    for i, j in zip(adj_coo.row, adj_coo.col):
        if i < j:  # Only process each edge once
            if is_minority.iloc[i] and is_minority.iloc[j]:
                # Both endpoints are minority - HIGH weight (expensive to cut)
                edge_weights[(i, j)] = weight_factor
                minority_edge_count += 1
            else:
                # At least one endpoint is not minority - NORMAL weight
                edge_weights[(i, j)] = 1.0

    return edge_weights, minority_edge_count, is_minority.sum()

def calculate_compactness_metrics(partition, adjacency_list, edge_weights):
    """
    Calculate compactness metrics for partitioning.

    Returns:
        - edge_cut_unweighted: Number of edges cut (traditional metric)
        - edge_cut_weighted: Sum of weights of edges cut (METIS objective)
        - avg_neighbors_per_district: Average internal edges per district
    """
    n_vertices = len(partition)
    n_districts = len(set(partition))

    # Calculate edge cuts
    edge_cut_unweighted = 0
    edge_cut_weighted = 0.0

    # Count internal edges per district
    internal_edges = {d: 0 for d in range(n_districts)}

    for i, neighbors in enumerate(adjacency_list):
        district_i = partition[i]

        for j in neighbors:
            if i < j:  # Count each edge once
                district_j = partition[j]

                edge_key = (min(i, j), max(i, j))
                weight = edge_weights.get(edge_key, 1.0)

                if district_i != district_j:
                    # Edge is cut
                    edge_cut_unweighted += 1
                    edge_cut_weighted += weight
                else:
                    # Internal edge
                    internal_edges[district_i] += 1

    avg_internal_edges = np.mean(list(internal_edges.values()))

    return {
        'edge_cut_unweighted': edge_cut_unweighted,
        'edge_cut_weighted': edge_cut_weighted,
        'avg_internal_edges': avg_internal_edges
    }

def test_single_configuration(state_code, weight_factor, minority_threshold, verbose=False):
    """
    Test a single configuration: state + weight factor + threshold.

    CRITICAL: This uses SINGLE-OBJECTIVE optimization:
    - vertex_weights is 1D (population only)
    - No target_weights parameter (no tpwgts)
    - No ubvec parameter (no multi-constraint)
    - Only edge_weights parameter (weighted edges)
    """
    config = VRA_STATES[state_code]
    state_name = config['name']
    num_districts = config['districts']
    target_mm = config['target_mm']

    # Load data
    year = '2020'
    version = 'v1'

    tracts_file = str(get_tract_file(state_code, year, version))
    tracts_gdf = gpd.read_parquet(tracts_file)

    demographics = load_tract_demographics(state_name, year)
    vertex_weights_vra, tracts_with_demo = create_vra_vertex_weights(tracts_gdf, demographics)

    # Build adjacency graph
    adjacency_list, _, _, _, _ = build_adjacency_graph(tracts_with_demo)

    # Create adjacency matrix for edge weight calculation
    n_tracts = len(adjacency_list)
    adj_matrix = lil_matrix((n_tracts, n_tracts), dtype=int)
    for i, neighbors in enumerate(adjacency_list):
        for j in neighbors:
            adj_matrix[i, j] = 1
    adj_matrix = adj_matrix.tocsr()

    # Create minority-edge weights
    edge_weights, minority_edges, minority_tracts = create_minority_edge_weights(
        adj_matrix, tracts_with_demo, minority_threshold, weight_factor
    )

    # SINGLE-OBJECTIVE vertex weights (population only, 1D array)
    vertex_weights = tracts_with_demo['total_pop'].values

    if verbose:
        print(f"  Tracts: {n_tracts}, Minority tracts (>={minority_threshold*100}%): {minority_tracts}")
        print(f"  Minority-minority edges: {minority_edges}, Weight factor: {weight_factor}x")

    # Run METIS with SINGLE-OBJECTIVE optimization
    # NO tpwgts, NO ubvec, ONLY edge_weights
    partition = partition_graph_with_executable(
        adjacency_list,
        vertex_weights,  # 1D array - single objective
        nparts=num_districts,
        ufactor=1.005,
        edge_weights=edge_weights,  # ONLY parameter controlling minority preservation
        niter=100,
        debug=False
    )

    # Analyze VRA compliance
    analysis = analyze_mm_districts(tracts_with_demo, partition, mm_threshold=0.50)
    mm_count = analysis['mm_count']
    max_minority = max(d['pct_minority'] for d in analysis['districts'])

    # Calculate compactness metrics
    compactness = calculate_compactness_metrics(partition, adjacency_list, edge_weights)

    return {
        'state': state_code,
        'weight_factor': weight_factor,
        'minority_threshold': minority_threshold,
        'mm_count': mm_count,
        'target_mm': target_mm,
        'success': mm_count >= target_mm,
        'max_minority_pct': max_minority,
        'minority_tracts': minority_tracts,
        'minority_edges': minority_edges,
        'total_tracts': n_tracts,
        'edge_cut_unweighted': compactness['edge_cut_unweighted'],
        'edge_cut_weighted': compactness['edge_cut_weighted'],
        'avg_internal_edges': compactness['avg_internal_edges']
    }

def run_ablation_study():
    """
    Systematic ablation study: vary weight factors and thresholds.
    """
    # Ablation parameters
    weight_factors = [1, 5, 10, 50, 100, 500, 1000]
    minority_thresholds = [0.40, 0.45, 0.50, 0.55]

    results = []

    print("="*80)
    print("ABLATION STUDY: Minority-Edge Weighting")
    print("="*80)
    print("Single-objective edge-cut minimization with weighted edges")
    print("NO tpwgts, NO ubvec, ONLY edge_weights")
    print("="*80)

    total_tests = len(VRA_STATES) * len(weight_factors) * len(minority_thresholds)
    test_num = 0

    for state_code in VRA_STATES.keys():
        config = VRA_STATES[state_code]
        print(f"\n{'='*80}")
        print(f"{config['name'].upper()} ({state_code})")
        print(f"Districts: {config['districts']}, Target MM: {config['target_mm']}, State minority: {config['minority_pct']*100:.1f}%")
        print(f"{'='*80}")

        for threshold in minority_thresholds:
            print(f"\n--- Minority threshold: {threshold*100:.0f}% ---")

            for weight_factor in weight_factors:
                test_num += 1
                print(f"[{test_num}/{total_tests}] Weight factor: {weight_factor}x... ", end='', flush=True)

                try:
                    result = test_single_configuration(
                        state_code,
                        weight_factor,
                        threshold,
                        verbose=False
                    )
                    results.append(result)

                    success_str = "SUCCESS" if result['success'] else "FAIL"
                    print(f"{result['mm_count']}/{result['target_mm']} MM ({result['max_minority_pct']*100:.1f}% max), "
                          f"EdgeCut: {result['edge_cut_unweighted']} - {success_str}")

                except Exception as e:
                    print(f"ERROR: {e}")
                    results.append({
                        'state': state_code,
                        'weight_factor': weight_factor,
                        'minority_threshold': threshold,
                        'mm_count': None,
                        'target_mm': config['target_mm'],
                        'success': False,
                        'max_minority_pct': None,
                        'error': str(e)
                    })

    return pd.DataFrame(results)

def analyze_results(df):
    """
    Analyze ablation study results.
    """
    print("\n" + "="*80)
    print("ABLATION STUDY RESULTS")
    print("="*80)

    # Success rate by state
    print("\n1. SUCCESS RATE BY STATE")
    print("-" * 80)
    for state_code in VRA_STATES.keys():
        state_df = df[df['state'] == state_code]
        success_count = state_df['success'].sum()
        total_count = len(state_df)
        success_rate = success_count / total_count * 100 if total_count > 0 else 0

        config = VRA_STATES[state_code]
        print(f"{state_code} ({config['minority_pct']*100:.1f}% minority): "
              f"{success_count}/{total_count} configurations successful ({success_rate:.1f}%)")

        # Best configuration for this state
        success_df = state_df[state_df['success'] == True]
        if len(success_df) > 0:
            best = success_df.loc[success_df['max_minority_pct'].idxmax()]
            print(f"  Best: weight={best['weight_factor']}x, threshold={best['minority_threshold']*100:.0f}%, "
                  f"MM={best['mm_count']}/{best['target_mm']} ({best['max_minority_pct']*100:.1f}% max)")

    # Success rate by weight factor
    print("\n2. SUCCESS RATE BY WEIGHT FACTOR")
    print("-" * 80)
    for weight in sorted(df['weight_factor'].unique()):
        weight_df = df[df['weight_factor'] == weight]
        success_count = weight_df['success'].sum()
        total_count = len(weight_df)
        success_rate = success_count / total_count * 100 if total_count > 0 else 0
        print(f"Weight {weight}x: {success_count}/{total_count} successful ({success_rate:.1f}%)")

    # Success rate by threshold
    print("\n3. SUCCESS RATE BY MINORITY THRESHOLD")
    print("-" * 80)
    for threshold in sorted(df['minority_threshold'].unique()):
        threshold_df = df[df['minority_threshold'] == threshold]
        success_count = threshold_df['success'].sum()
        total_count = len(threshold_df)
        success_rate = success_count / total_count * 100 if total_count > 0 else 0
        print(f"Threshold {threshold*100:.0f}%: {success_count}/{total_count} successful ({success_rate:.1f}%)")

    # Optimal configurations
    print("\n4. OPTIMAL CONFIGURATION PER STATE")
    print("-" * 80)
    for state_code in VRA_STATES.keys():
        state_df = df[df['state'] == state_code]
        success_df = state_df[state_df['success'] == True]

        if len(success_df) > 0:
            # Find configuration with lowest weight factor (most principled)
            optimal = success_df.loc[success_df['weight_factor'].idxmin()]
            print(f"{state_code}: weight={optimal['weight_factor']}x, "
                  f"threshold={optimal['minority_threshold']*100:.0f}%, "
                  f"MM={optimal['mm_count']}/{optimal['target_mm']} "
                  f"({optimal['max_minority_pct']*100:.1f}% max)")
        else:
            print(f"{state_code}: NO SUCCESSFUL CONFIGURATION")

    # Compactness analysis
    print("\n5. COMPACTNESS METRICS (Edge Cut)")
    print("-" * 80)
    print("Lower edge cut = more compact districts")
    print("\nBy weight factor (averaged across states):")
    for weight in sorted(df['weight_factor'].unique()):
        weight_df = df[df['weight_factor'] == weight]
        avg_cut_unweighted = weight_df['edge_cut_unweighted'].mean()
        avg_cut_weighted = weight_df['edge_cut_weighted'].mean()
        print(f"Weight {weight:4d}x: Unweighted cut={avg_cut_unweighted:.0f}, "
              f"Weighted cut={avg_cut_weighted:.0f}")

    print("\nBy state (baseline weight=1x vs best successful config):")
    for state_code in VRA_STATES.keys():
        state_df = df[df['state'] == state_code]
        baseline = state_df[state_df['weight_factor'] == 1].iloc[0]
        success_df = state_df[state_df['success'] == True]

        if len(success_df) > 0:
            # Best = minimum edge cut among successful configs
            best = success_df.loc[success_df['edge_cut_unweighted'].idxmin()]
            cut_increase = best['edge_cut_unweighted'] - baseline['edge_cut_unweighted']
            pct_increase = cut_increase / baseline['edge_cut_unweighted'] * 100
            print(f"{state_code}: Baseline cut={baseline['edge_cut_unweighted']:.0f}, "
                  f"Best success cut={best['edge_cut_unweighted']:.0f} "
                  f"(+{cut_increase:.0f}, +{pct_increase:.1f}%)")
        else:
            print(f"{state_code}: Baseline cut={baseline['edge_cut_unweighted']:.0f}, "
                  f"NO SUCCESSFUL CONFIG")

    # Comparison to multi-constraint (from previous tests)
    print("\n6. COMPARISON TO MULTI-CONSTRAINT OPTIMIZATION")
    print("-" * 80)
    multi_constraint_results = {
        'AL': {'mm_count': 0, 'max_pct': 49.6},
        'GA': {'mm_count': 5, 'max_pct': 76.7},
        'LA': {'mm_count': 1, 'max_pct': 58.8},
        'MS': {'mm_count': 2, 'max_pct': 53.3},
        'SC': {'mm_count': 0, 'max_pct': 47.2},
    }

    for state_code in VRA_STATES.keys():
        state_df = df[df['state'] == state_code]
        success_df = state_df[state_df['success'] == True]

        mc = multi_constraint_results[state_code]
        config = VRA_STATES[state_code]

        if len(success_df) > 0:
            best = success_df.loc[success_df['max_minority_pct'].idxmax()]
            improvement = best['mm_count'] - mc['mm_count']
            print(f"{state_code}: Multi-constraint {mc['mm_count']}/{config['target_mm']} MM "
                  f"vs Edge-weighting {best['mm_count']}/{config['target_mm']} MM "
                  f"(+{improvement} districts)")
        else:
            print(f"{state_code}: Multi-constraint {mc['mm_count']}/{config['target_mm']} MM "
                  f"vs Edge-weighting FAIL")

if __name__ == '__main__':
    # Run comprehensive ablation study
    results_df = run_ablation_study()

    # Save results
    output_dir = project_root / 'research' / 'gerry-vra-compliance' / 'results'
    output_dir.mkdir(parents=True, exist_ok=True)

    results_file = output_dir / 'edge_weighting_ablation_study.csv'
    results_df.to_csv(results_file, index=False)
    print(f"\nResults saved to: {results_file}")

    # Analyze results
    analyze_results(results_df)

    # Generate summary tables
    print("\n7. DETAILED RESULTS: MM DISTRICTS ACHIEVED")
    print("-" * 80)
    mm_summary = results_df.pivot_table(
        index=['state', 'minority_threshold'],
        columns='weight_factor',
        values='mm_count',
        aggfunc='first'
    )
    print(mm_summary)

    print("\n8. DETAILED RESULTS: EDGE CUT (COMPACTNESS)")
    print("-" * 80)
    print("Lower values = more compact")
    compactness_summary = results_df.pivot_table(
        index=['state', 'minority_threshold'],
        columns='weight_factor',
        values='edge_cut_unweighted',
        aggfunc='first'
    )
    print(compactness_summary)

    print("\n9. TRADEOFF: VRA vs COMPACTNESS")
    print("-" * 80)
    print("For each successful configuration, show MM districts and edge cut")
    success_df = results_df[results_df['success'] == True].copy()
    if len(success_df) > 0:
        success_df['config'] = (success_df['weight_factor'].astype(str) + 'x @ ' +
                               (success_df['minority_threshold'] * 100).astype(int).astype(str) + '%')
        tradeoff = success_df.groupby(['state', 'config']).agg({
            'mm_count': 'first',
            'target_mm': 'first',
            'edge_cut_unweighted': 'first',
            'max_minority_pct': 'first'
        }).reset_index()
        print(tradeoff.to_string(index=False))
    else:
        print("No successful configurations found")
