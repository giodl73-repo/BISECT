"""
Comprehensive Compactness-VRA Tradeoff Experiments

Research Question: Do non-MM districts suffer compactness loss when we optimize for VRA compliance?

Experiments:
1. Baseline: Uniform METIS (no VRA optimization)
2. Edge-Weighted Sweep: 11 weight factors × 4 thresholds
3. District-Level Analysis: Compare MM vs non-MM district compactness

Metrics:
- State-level: Edge cut (unweighted, weighted)
- District-level: Polsby-Popper, Reock, Convex Hull, internal edges
- Breakdown: MM districts vs non-MM districts
"""

import sys
from pathlib import Path
import pandas as pd
import numpy as np
from collections import defaultdict

# Add paths
project_root = Path(__file__).parent.parent.parent.parent
sys.path.insert(0, str(project_root / 'src'))
sys.path.insert(0, str(project_root))

import geopandas as gpd
from scipy.sparse import lil_matrix
from shapely.ops import unary_union
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


def compute_polsby_popper(district_geometry):
    """
    Polsby-Popper compactness: 4π * Area / Perimeter²

    Range: [0, 1], where 1 = perfect circle
    Higher values indicate more compact districts
    """
    try:
        area = district_geometry.area
        perimeter = district_geometry.length

        if perimeter == 0 or area == 0:
            return 0.0

        pp = (4 * np.pi * area) / (perimeter ** 2)
        return min(pp, 1.0)  # Cap at 1.0 due to numerical precision
    except Exception as e:
        print(f"  Warning: Polsby-Popper calculation failed: {e}")
        return 0.0


def compute_reock(district_geometry):
    """
    Reock compactness: Area / Area of minimum bounding circle

    Range: [0, 1], where 1 = perfect circle
    Uses minimum rotated rectangle as proxy for bounding circle (computationally efficient)
    """
    try:
        area = district_geometry.area

        # Use minimum rotated rectangle as proxy for bounding circle
        # (True minimum bounding circle is computationally expensive)
        min_rect = district_geometry.minimum_rotated_rectangle
        rect_area = min_rect.area

        if rect_area == 0 or area == 0:
            return 0.0

        reock = area / rect_area
        return min(reock, 1.0)
    except Exception as e:
        print(f"  Warning: Reock calculation failed: {e}")
        return 0.0


def compute_convex_hull_ratio(district_geometry):
    """
    Convex Hull Ratio: Area / Area of convex hull

    Range: [0, 1], where 1 = perfectly convex shape
    Measures how much the district deviates from convexity
    """
    try:
        area = district_geometry.area
        convex_hull = district_geometry.convex_hull
        hull_area = convex_hull.area

        if hull_area == 0 or area == 0:
            return 0.0

        ch_ratio = area / hull_area
        return min(ch_ratio, 1.0)
    except Exception as e:
        print(f"  Warning: Convex Hull calculation failed: {e}")
        return 0.0


def compute_district_compactness(tracts_gdf, partition, district_id):
    """
    Compute all compactness metrics for a single district.

    Returns dict with:
    - polsby_popper: Geometric compactness (circle-based)
    - reock: Bounding circle ratio
    - convex_hull_ratio: Convexity measure
    - num_tracts: Number of tracts in district
    """
    # Get tracts in this district
    district_tracts = tracts_gdf[partition == district_id]

    if len(district_tracts) == 0:
        return {
            'polsby_popper': 0.0,
            'reock': 0.0,
            'convex_hull_ratio': 0.0,
            'num_tracts': 0
        }

    # Union geometries to create district boundary
    district_geom = unary_union(district_tracts.geometry)

    return {
        'polsby_popper': compute_polsby_popper(district_geom),
        'reock': compute_reock(district_geom),
        'convex_hull_ratio': compute_convex_hull_ratio(district_geom),
        'num_tracts': len(district_tracts)
    }


def compute_internal_edges_per_district(partition, adjacency_list):
    """
    Calculate number of internal edges for each district.

    Internal edges = edges where both endpoints are in the same district
    Higher internal edges = more connected/compact district
    """
    n_districts = len(set(partition))
    internal_edges = {d: 0 for d in range(n_districts)}

    for i, neighbors in enumerate(adjacency_list):
        district_i = partition[i]
        for j in neighbors:
            if i < j and partition[j] == district_i:  # Same district
                internal_edges[district_i] += 1

    return internal_edges


def create_minority_edge_weights(adj_matrix, tracts_with_demo, minority_threshold=0.50, weight_factor=100):
    """
    Create edge weight dictionary for minority-minority edges.

    Edges between minority tracts get higher weights (expensive to cut).
    METIS minimizes total edge cut weight, so it keeps minority communities together.
    """
    # Identify minority tracts
    is_minority = tracts_with_demo['pct_minority'] >= minority_threshold

    edge_weights = {}
    minority_edge_count = 0

    # Convert to COO format for iteration
    adj_coo = adj_matrix.tocoo()

    for i, j in zip(adj_coo.row, adj_coo.col):
        if i < j:  # Only process each edge once
            if is_minority.iloc[i] and is_minority.iloc[j]:
                # Both endpoints are minority - HIGH weight
                edge_weights[(i, j)] = weight_factor
                minority_edge_count += 1
            else:
                # At least one endpoint is not minority - NORMAL weight
                edge_weights[(i, j)] = 1.0

    return edge_weights, minority_edge_count, is_minority.sum()


def calculate_edge_cut(partition, adjacency_list, edge_weights=None):
    """
    Calculate edge cut metrics.

    Returns:
    - edge_cut_unweighted: Number of edges cut
    - edge_cut_weighted: Sum of weights of edges cut (if edge_weights provided)
    """
    edge_cut_unweighted = 0
    edge_cut_weighted = 0.0

    for i, neighbors in enumerate(adjacency_list):
        district_i = partition[i]

        for j in neighbors:
            if i < j:  # Count each edge once
                district_j = partition[j]

                if district_i != district_j:
                    edge_cut_unweighted += 1

                    if edge_weights:
                        edge_key = (min(i, j), max(i, j))
                        weight = edge_weights.get(edge_key, 1.0)
                        edge_cut_weighted += weight

    return edge_cut_unweighted, edge_cut_weighted


def run_baseline_experiment(state_code, year='2020', version='V1', verbose=True):
    """
    Experiment 1: Baseline compactness (no VRA optimization).

    Run uniform METIS with:
    - Population balance only (1D vertex weights)
    - No edge weights
    - No target weights

    This establishes ground truth compactness without VRA goals.
    """
    config = VRA_STATES[state_code]
    state_name = config['name']
    num_districts = config['districts']

    if verbose:
        print(f"\n=== BASELINE: {state_code} ({state_name.title()}) ===")

    # Load data
    tracts_file = str(get_tract_file(state_code, year, version))
    tracts_gdf = gpd.read_parquet(tracts_file)

    demographics = load_tract_demographics(state_name, year)
    vertex_weights_vra, tracts_with_demo = create_vra_vertex_weights(tracts_gdf, demographics)

    # Build adjacency graph
    adjacency_list, _, _, _, _ = build_adjacency_graph(tracts_with_demo)

    # BASELINE: Population-only vertex weights (1D)
    vertex_weights = tracts_with_demo['total_pop'].values

    if verbose:
        print(f"  Tracts: {len(tracts_with_demo)}, Districts: {num_districts}")
        print(f"  Running uniform METIS (no edge weights)...")

    # Run METIS with NO edge weights
    partition = partition_graph_with_executable(
        adjacency_list,
        vertex_weights,
        nparts=num_districts,
        ufactor=1.005,
        niter=100,
        debug=False
    )

    # Analyze VRA compliance
    analysis = analyze_mm_districts(tracts_with_demo, partition, mm_threshold=0.50)
    mm_count = analysis['mm_count']
    max_minority = max(d['pct_minority'] for d in analysis['districts'])

    # Calculate state-level edge cut
    edge_cut_unweighted, _ = calculate_edge_cut(partition, adjacency_list)

    # Calculate internal edges per district
    internal_edges = compute_internal_edges_per_district(partition, adjacency_list)

    # Calculate district-level compactness
    district_results = []

    for district_id in range(num_districts):
        # Geometric compactness
        compactness = compute_district_compactness(tracts_with_demo, partition, district_id)

        # VRA metrics
        district_info = analysis['districts'][district_id]
        minority_pct = district_info['pct_minority']
        is_mm = minority_pct >= 0.50

        district_results.append({
            'state': state_code,
            'method': 'baseline',
            'weight_factor': 1.0,
            'minority_threshold': None,
            'district_id': district_id,
            'minority_pct': minority_pct,
            'is_mm': is_mm,
            'polsby_popper': compactness['polsby_popper'],
            'reock': compactness['reock'],
            'convex_hull_ratio': compactness['convex_hull_ratio'],
            'internal_edges': internal_edges[district_id],
            'num_tracts': compactness['num_tracts']
        })

    if verbose:
        print(f"  Edge cut: {edge_cut_unweighted}")
        print(f"  MM districts: {mm_count}/{config['target_mm']} (max minority: {max_minority:.1%})")
        print(f"  Avg Polsby-Popper: {np.mean([d['polsby_popper'] for d in district_results]):.3f}")

    # State-level summary
    state_result = {
        'state': state_code,
        'method': 'baseline',
        'weight_factor': 1.0,
        'minority_threshold': None,
        'mm_count': mm_count,
        'target_mm': config['target_mm'],
        'max_minority_pct': max_minority,
        'edge_cut_unweighted': edge_cut_unweighted,
        'avg_polsby_popper': np.mean([d['polsby_popper'] for d in district_results]),
        'avg_reock': np.mean([d['reock'] for d in district_results]),
        'avg_convex_hull_ratio': np.mean([d['convex_hull_ratio'] for d in district_results]),
        'avg_internal_edges': np.mean(list(internal_edges.values()))
    }

    return state_result, district_results


def run_edge_weighted_experiment(state_code, weight_factor, minority_threshold,
                                  year='2020', version='V1', verbose=True):
    """
    Experiment 3: Edge-weighted optimization with specific parameters.

    Tests single configuration:
    - weight_factor: How much heavier are minority-minority edges (e.g., 5x, 10x, 100x)
    - minority_threshold: What % defines a minority tract (e.g., 40%, 50%)

    Returns state-level and district-level results with MM vs non-MM breakdown.
    """
    config = VRA_STATES[state_code]
    state_name = config['name']
    num_districts = config['districts']

    if verbose:
        print(f"\n=== EDGE-WEIGHTED: {state_code} | {weight_factor}x @ {minority_threshold*100}% ===")

    # Load data
    tracts_file = str(get_tract_file(state_code, year, version))
    tracts_gdf = gpd.read_parquet(tracts_file)

    demographics = load_tract_demographics(state_name, year)
    vertex_weights_vra, tracts_with_demo = create_vra_vertex_weights(tracts_gdf, demographics)

    # Build adjacency graph
    adjacency_list, _, _, _, _ = build_adjacency_graph(tracts_with_demo)

    # Create adjacency matrix for edge weights
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

    # Population-only vertex weights (1D)
    vertex_weights = tracts_with_demo['total_pop'].values

    if verbose:
        print(f"  Minority tracts: {minority_tracts}/{n_tracts}, Minority edges: {minority_edges}")
        print(f"  Running METIS with edge weights...")

    # Run METIS with edge weights
    partition = partition_graph_with_executable(
        adjacency_list,
        vertex_weights,
        nparts=num_districts,
        ufactor=1.005,
        edge_weights=edge_weights,
        niter=100,
        debug=False
    )

    # Analyze VRA compliance
    analysis = analyze_mm_districts(tracts_with_demo, partition, mm_threshold=0.50)
    mm_count = analysis['mm_count']
    max_minority = max(d['pct_minority'] for d in analysis['districts'])

    # Calculate state-level edge cut
    edge_cut_unweighted, edge_cut_weighted = calculate_edge_cut(partition, adjacency_list, edge_weights)

    # Calculate internal edges per district
    internal_edges = compute_internal_edges_per_district(partition, adjacency_list)

    # Calculate district-level compactness
    district_results = []

    for district_id in range(num_districts):
        # Geometric compactness
        compactness = compute_district_compactness(tracts_with_demo, partition, district_id)

        # VRA metrics
        district_info = analysis['districts'][district_id]
        minority_pct = district_info['pct_minority']
        is_mm = minority_pct >= 0.50

        district_results.append({
            'state': state_code,
            'method': 'edge_weighted',
            'weight_factor': weight_factor,
            'minority_threshold': minority_threshold,
            'district_id': district_id,
            'minority_pct': minority_pct,
            'is_mm': is_mm,
            'polsby_popper': compactness['polsby_popper'],
            'reock': compactness['reock'],
            'convex_hull_ratio': compactness['convex_hull_ratio'],
            'internal_edges': internal_edges[district_id],
            'num_tracts': compactness['num_tracts']
        })

    # Calculate breakdown: MM vs non-MM districts
    mm_districts = [d for d in district_results if d['is_mm']]
    non_mm_districts = [d for d in district_results if not d['is_mm']]

    if verbose:
        print(f"  Edge cut: {edge_cut_unweighted} (weighted: {edge_cut_weighted:.0f})")
        print(f"  MM districts: {mm_count}/{config['target_mm']} (max minority: {max_minority:.1%})")

        if mm_districts:
            print(f"  MM districts avg PP: {np.mean([d['polsby_popper'] for d in mm_districts]):.3f}")
        if non_mm_districts:
            print(f"  Non-MM districts avg PP: {np.mean([d['polsby_popper'] for d in non_mm_districts]):.3f}")

    # State-level summary
    state_result = {
        'state': state_code,
        'method': 'edge_weighted',
        'weight_factor': weight_factor,
        'minority_threshold': minority_threshold,
        'mm_count': mm_count,
        'target_mm': config['target_mm'],
        'success': mm_count >= config['target_mm'],
        'max_minority_pct': max_minority,
        'minority_tracts': minority_tracts,
        'minority_edges': minority_edges,
        'edge_cut_unweighted': edge_cut_unweighted,
        'edge_cut_weighted': edge_cut_weighted,
        'avg_polsby_popper': np.mean([d['polsby_popper'] for d in district_results]),
        'avg_reock': np.mean([d['reock'] for d in district_results]),
        'avg_convex_hull_ratio': np.mean([d['convex_hull_ratio'] for d in district_results]),
        'avg_internal_edges': np.mean(list(internal_edges.values())),
        # Breakdown by district type
        'mm_avg_polsby_popper': np.mean([d['polsby_popper'] for d in mm_districts]) if mm_districts else None,
        'non_mm_avg_polsby_popper': np.mean([d['polsby_popper'] for d in non_mm_districts]) if non_mm_districts else None,
        'mm_avg_internal_edges': np.mean([d['internal_edges'] for d in mm_districts]) if mm_districts else None,
        'non_mm_avg_internal_edges': np.mean([d['internal_edges'] for d in non_mm_districts]) if non_mm_districts else None
    }

    return state_result, district_results


def run_full_parameter_sweep(states=['AL'], weight_factors=[1, 5, 10, 50, 100],
                              thresholds=[0.40, 0.45, 0.50, 0.55],
                              year='2020', version='V1'):
    """
    Run comprehensive parameter sweep across states, weight factors, and thresholds.

    Args:
        states: List of state codes (default: ['AL'] for testing)
        weight_factors: List of weight multiplication factors
        thresholds: List of minority thresholds (as decimals)

    Returns:
        state_results_df: State-level aggregated metrics
        district_results_df: District-level detailed metrics
    """
    print("=" * 80)
    print("COMPACTNESS-VRA TRADEOFF EXPERIMENTS")
    print("=" * 80)

    state_results = []
    district_results = []

    # Run baseline for each state
    print("\n" + "=" * 80)
    print("PHASE 1: BASELINE (No VRA Optimization)")
    print("=" * 80)

    for state_code in states:
        state_res, dist_res = run_baseline_experiment(state_code, year, version)
        state_results.append(state_res)
        district_results.extend(dist_res)

    # Run edge-weighted sweep
    print("\n" + "=" * 80)
    print("PHASE 2: EDGE-WEIGHTED PARAMETER SWEEP")
    print("=" * 80)

    total_configs = len(states) * len(weight_factors) * len(thresholds)
    config_num = 0

    for state_code in states:
        for weight_factor in weight_factors:
            for threshold in thresholds:
                config_num += 1
                print(f"\n[{config_num}/{total_configs}]", end=' ')

                state_res, dist_res = run_edge_weighted_experiment(
                    state_code, weight_factor, threshold, year, version
                )
                state_results.append(state_res)
                district_results.extend(dist_res)

    # Convert to DataFrames
    state_df = pd.DataFrame(state_results)
    district_df = pd.DataFrame(district_results)

    return state_df, district_df


if __name__ == '__main__':
    # Run all 5 VRA states
    print("Running all 5 VRA states...")

    state_df, district_df = run_full_parameter_sweep(
        states=['AL', 'GA', 'LA', 'MS', 'SC'],  # All VRA states
        weight_factors=[1, 5, 10, 50, 100],  # Full range
        thresholds=[0.40, 0.45, 0.50, 0.55],  # Full range
        year='2020',
        version='V1'  # Uppercase V1 to match actual directory structure
    )

    # Save results
    results_dir = Path(__file__).parent.parent / 'results'
    results_dir.mkdir(exist_ok=True)

    state_df.to_csv(results_dir / 'compactness_state_level.csv', index=False)
    district_df.to_csv(results_dir / 'compactness_district_level.csv', index=False)

    print("\n" + "=" * 80)
    print("RESULTS SAVED")
    print("=" * 80)
    print(f"State-level: {results_dir / 'compactness_state_level.csv'}")
    print(f"District-level: {results_dir / 'compactness_district_level.csv'}")

    # Quick summary
    print("\n" + "=" * 80)
    print("QUICK SUMMARY: MM vs Non-MM District Compactness")
    print("=" * 80)

    # Compare baseline vs best edge-weighted
    baseline = state_df[state_df['method'] == 'baseline'].iloc[0]
    successful = state_df[(state_df['method'] == 'edge_weighted') & (state_df['success'] == True)]

    if len(successful) > 0:
        best = successful.loc[successful['edge_cut_unweighted'].idxmin()]

        print(f"\nBaseline (no VRA optimization):")
        print(f"  Edge cut: {baseline['edge_cut_unweighted']}")
        print(f"  Avg Polsby-Popper: {baseline['avg_polsby_popper']:.3f}")
        print(f"  MM districts: {baseline['mm_count']}")

        print(f"\nBest edge-weighted (VRA success):")
        print(f"  Weight factor: {best['weight_factor']}x @ {best['minority_threshold']*100}%")
        print(f"  Edge cut: {best['edge_cut_unweighted']} ({(best['edge_cut_unweighted']/baseline['edge_cut_unweighted']-1)*100:+.1f}%)")
        print(f"  Avg Polsby-Popper: {best['avg_polsby_popper']:.3f} ({(best['avg_polsby_popper']/baseline['avg_polsby_popper']-1)*100:+.1f}%)")
        print(f"  MM districts: {best['mm_count']}/{best['target_mm']}")

        if best['mm_avg_polsby_popper'] and best['non_mm_avg_polsby_popper']:
            print(f"\n  MM districts PP: {best['mm_avg_polsby_popper']:.3f}")
            print(f"  Non-MM districts PP: {best['non_mm_avg_polsby_popper']:.3f}")
            print(f"  Gap: {(best['non_mm_avg_polsby_popper'] - best['mm_avg_polsby_popper']):.3f} "
                  f"({((best['non_mm_avg_polsby_popper']/best['mm_avg_polsby_popper']-1)*100):+.1f}%)")
            print(f"\n  -> Non-MM districts are {((best['non_mm_avg_polsby_popper']/best['mm_avg_polsby_popper']-1)*100):.1f}% more compact")
    else:
        print("\nNo successful VRA configurations found!")
