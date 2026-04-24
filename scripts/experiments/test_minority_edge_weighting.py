"""
Test minority-edge weighting approach for VRA compliance.

Instead of multi-constraint optimization, this approach:
1. Identifies edges between high-minority tracts
2. Assigns higher weights to these edges (making them harder to cut)
3. Uses single-objective edge-cut minimization
4. Tests if Alabama can achieve VRA compliance this way
"""

import sys
from pathlib import Path

# Add paths
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root / 'src'))
sys.path.insert(0, str(project_root))

import geopandas as gpd
import pandas as pd
import numpy as np
from apportionment.data.adjacency import build_adjacency_graph
from apportionment.partition.metis_executable import partition_graph_with_executable
from apportionment.partition.vra_utils import load_tract_demographics, analyze_mm_districts, create_vra_vertex_weights
from scripts.utils import get_tract_file

def create_minority_edge_weights(adj_matrix, tracts_with_demo, minority_threshold=0.50, weight_factor=100):
    """
    Create edge weight dictionary for minority-minority edges.

    Args:
        adj_matrix: scipy sparse adjacency matrix
        tracts_with_demo: DataFrame with tract data including pct_minority column
        minority_threshold: Threshold for considering tract as minority (default 0.50)
        weight_factor: Weight for minority-minority edges (default 100)

    Returns:
        Dictionary mapping (i, j) tuples to edge weights
    """
    # Identify minority tracts
    is_minority = tracts_with_demo['pct_minority'] >= minority_threshold

    print(f"Tracts with >={minority_threshold*100}% minority: {is_minority.sum()} of {len(tracts_with_demo)}")

    # Create edge weights
    edge_weights = {}
    minority_edge_count = 0

    # Convert to COO format for iteration
    adj_coo = adj_matrix.tocoo()

    for i, j in zip(adj_coo.row, adj_coo.col):
        if i < j:  # Only process each edge once
            if is_minority.iloc[i] and is_minority.iloc[j]:
                # Both endpoints are minority - increase weight
                edge_weights[(i, j)] = weight_factor
                minority_edge_count += 1
            else:
                # At least one endpoint is not minority - normal weight
                edge_weights[(i, j)] = 1.0

    print(f"Weighted {minority_edge_count} minority-minority edges by factor {weight_factor}")
    print(f"Total edges: {len(edge_weights)}")

    return edge_weights

def test_alabama_weighted_edges(weight_factor=100, minority_threshold=0.50, tree_structure='nway'):
    """
    Test Alabama VRA compliance with minority-edge weighting.

    Args:
        weight_factor: Multiplier for minority-minority edges
        minority_threshold: Threshold for minority tract classification
        tree_structure: 'nway', 'recursive', or specific split like [3,4]
    """
    state = 'alabama'
    num_districts = 7
    target_mm_districts = 2

    print(f"\n{'='*80}")
    print(f"Testing {state.upper()} with minority-edge weighting")
    print(f"Weight factor: {weight_factor}, Threshold: {minority_threshold*100}%")
    print(f"Target: {target_mm_districts} MM districts (>50% minority)")
    print(f"{'='*80}\n")

    # Load tract data
    year = '2020'
    version = 'v1'
    state_code = 'AL'  # Alabama state code

    tracts_file = str(get_tract_file(state_code, year, version))
    tracts_gdf = gpd.read_parquet(tracts_file)
    print(f"Loaded {len(tracts_gdf)} tracts from {tracts_file}")

    # Load demographics and create VRA vertex weights
    demographics = load_tract_demographics(state, year)
    vertex_weights_vra, tracts_with_demo = create_vra_vertex_weights(tracts_gdf, demographics)

    # Calculate state-wide minority percentage
    total_pop = tracts_with_demo['total_pop'].sum()
    minority_pop = (tracts_with_demo['total_pop'] * tracts_with_demo['pct_minority']).sum()
    print(f"State-wide minority: {minority_pop / total_pop:.1%}")

    # Build adjacency graph
    adjacency_list, _, _, _, _ = build_adjacency_graph(tracts_with_demo)
    print(f"Built adjacency graph with {len(adjacency_list)} vertices")

    # Create adjacency matrix from list (needed for edge weight calculation)
    n_tracts = len(adjacency_list)
    from scipy.sparse import lil_matrix
    adj_matrix = lil_matrix((n_tracts, n_tracts), dtype=int)
    for i, neighbors in enumerate(adjacency_list):
        for j in neighbors:
            adj_matrix[i, j] = 1
    adj_matrix = adj_matrix.tocsr()

    # Create edge weights for minority-minority edges
    edge_weights = create_minority_edge_weights(adj_matrix, tracts_with_demo, minority_threshold, weight_factor)

    # Use population-only vertex weights (single-objective optimization)
    vertex_weights = tracts_with_demo['total_pop'].values

    # Run METIS partition
    print(f"\nRunning direct k-way partition into {num_districts} districts...")
    partition = partition_graph_with_executable(
        adjacency_list,
        vertex_weights,
        nparts=num_districts,
        ufactor=1.005,  # 0.5% imbalance tolerance (METIS ufactor format)
        edge_weights=edge_weights,
        niter=100,
        debug=False
    )

    # Analyze results
    analysis = analyze_mm_districts(
        tracts_with_demo,
        partition,
        mm_threshold=0.50
    )
    mm_count = analysis['mm_count']
    district_demographics = {d['district']: {'minority_pct': d['pct_minority']}
                            for d in analysis['districts']}

    # Print results
    print(f"\n{'='*80}")
    print(f"RESULTS: {state.upper()}")
    print(f"{'='*80}")
    print(f"MM districts achieved: {mm_count} of {target_mm_districts} target")
    print(f"Success: {'YES' if mm_count >= target_mm_districts else 'NO'}")
    print(f"\nDistrict breakdown:")

    # Sort by minority percentage
    sorted_districts = sorted(
        district_demographics.items(),
        key=lambda x: x[1]['minority_pct'],
        reverse=True
    )

    for district_id, stats in sorted_districts:
        minority_pct = stats['minority_pct']
        is_mm = 'MM' if minority_pct >= 0.50 else ''
        print(f"  District {district_id}: {minority_pct:.1%} minority {is_mm}")

    max_minority = max(d['minority_pct'] for d in district_demographics.values())
    print(f"\nMaximum minority concentration: {max_minority:.1%}")

    if max_minority < 0.50:
        shortfall = 0.50 - max_minority
        print(f"Shortfall from 50% threshold: {shortfall:.1%} ({shortfall*100:.1f} percentage points)")

    return mm_count, max_minority, district_demographics

if __name__ == '__main__':
    # Test with different weight factors
    print("\n" + "="*80)
    print("MINORITY-EDGE WEIGHTING EXPERIMENTS")
    print("="*80)

    # Baseline: No weighting (weight_factor=1)
    print("\n--- BASELINE: No edge weighting (factor=1) ---")
    mm_baseline, max_baseline, _ = test_alabama_weighted_edges(
        weight_factor=1,
        minority_threshold=0.50,
        tree_structure='nway'
    )

    # Moderate weighting
    print("\n--- EXPERIMENT 1: Moderate weighting (factor=10) ---")
    mm_10x, max_10x, _ = test_alabama_weighted_edges(
        weight_factor=10,
        minority_threshold=0.50,
        tree_structure='nway'
    )

    # Strong weighting
    print("\n--- EXPERIMENT 2: Strong weighting (factor=100) ---")
    mm_100x, max_100x, _ = test_alabama_weighted_edges(
        weight_factor=100,
        minority_threshold=0.50,
        tree_structure='nway'
    )

    # Very strong weighting
    print("\n--- EXPERIMENT 3: Very strong weighting (factor=1000) ---")
    mm_1000x, max_1000x, _ = test_alabama_weighted_edges(
        weight_factor=1000,
        minority_threshold=0.50,
        tree_structure='nway'
    )

    # Lower threshold (45% minority tracts)
    print("\n--- EXPERIMENT 4: Lower threshold (45% minority, factor=100) ---")
    mm_45, max_45, _ = test_alabama_weighted_edges(
        weight_factor=100,
        minority_threshold=0.45,
        tree_structure='nway'
    )

    # Summary
    print("\n" + "="*80)
    print("SUMMARY: Alabama Minority-Edge Weighting Results")
    print("="*80)
    print(f"{'Approach':<40} {'MM Districts':<15} {'Max Minority':<15}")
    print("-" * 80)
    print(f"{'Baseline (no weighting)':<40} {mm_baseline:<15} {max_baseline:.1%}")
    print(f"{'10x weighting (>50% tracts)':<40} {mm_10x:<15} {max_10x:.1%}")
    print(f"{'100x weighting (>50% tracts)':<40} {mm_100x:<15} {max_100x:.1%}")
    print(f"{'1000x weighting (>50% tracts)':<40} {mm_1000x:<15} {max_1000x:.1%}")
    print(f"{'100x weighting (>45% tracts)':<40} {mm_45:<15} {max_45:.1%}")
    print("="*80)

    # Compare to multi-constraint approach
    print("\nComparison to multi-constraint optimization:")
    print("  Multi-constraint (tpwgts): 0 MM districts (47.3% max)")
    print("  Multi-constraint (ubvec):  0 MM districts (49.6% max)")
    print(f"  Edge weighting (best):     {max(mm_baseline, mm_10x, mm_100x, mm_1000x, mm_45)} MM districts ({max(max_baseline, max_10x, max_100x, max_1000x, max_45):.1%} max)")
