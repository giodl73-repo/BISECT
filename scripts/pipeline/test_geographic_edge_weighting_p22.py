"""
P2.2: Geographic Edge-Weighted Optimization Analysis

Tests compactness improvement from edge-weighting based on GEOGRAPHIC boundary
lengths (not minority demographics). This directly minimizes district perimeter.

Comparison: Unweighted vs Edge-Weighted for 10 representative states
- Small: VT, DE, WY
- Medium: MN, AL, WI
- Large: CA, TX, FL, PA

Metrics:
- Polsby-Popper compactness
- Reock compactness
- Partisan outcomes (Democratic district %)
- Population balance

Expected: 40-70% compactness improvement, <5% partisan outcome change
"""

import sys
from pathlib import Path
import pandas as pd
import numpy as np
from datetime import datetime

# Add paths
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root / 'src'))
sys.path.insert(0, str(project_root))

import geopandas as gpd
from shapely.ops import unary_union
from apportionment.data.adjacency import build_adjacency_graph
from apportionment.partition.recursive_bisection import recursive_bisection
from apportionment.visualization.compactness import compute_compactness_metrics
from scripts.config_2020 import STATE_DISTRICTS
from scripts.utils import get_tract_file

# Test states (10 representative)
TEST_STATES = {
    # Small states (1-3 districts)
    'vermont': {'districts': 1, 'size': 'small'},
    'delaware': {'districts': 1, 'size': 'small'},
    'wyoming': {'districts': 1, 'size': 'small'},

    # Medium states (7-10 districts)
    'minnesota': {'districts': 8, 'size': 'medium'},
    'alabama': {'districts': 7, 'size': 'medium'},
    'wisconsin': {'districts': 8, 'size': 'medium'},

    # Large states (17-53 districts)
    'california': {'districts': 52, 'size': 'large'},
    'texas': {'districts': 38, 'size': 'large'},
    'florida': {'districts': 28, 'size': 'large'},
    'pennsylvania': {'districts': 17, 'size': 'large'},
}


def compute_geographic_edge_weights(tracts_gdf, adj_matrix):
    """
    Compute edge weights based on GEOGRAPHIC boundary lengths.

    Edge weight w(e) = shared boundary length between adjacent tracts

    This directly optimizes perimeter (sum of boundary lengths), which
    is the denominator in Polsby-Popper: PP = 4π * Area / Perimeter²

    Returns
    -------
    dict : {(i, j): weight}
        Edge weights keyed by (min_index, max_index)
    float : total_boundary_km
        Sum of all boundary lengths (unweighted baseline)
    """
    edge_weights = {}
    total_boundary_km = 0.0

    # Convert adjacency matrix to COO format for iteration
    adj_coo = adj_matrix.tocoo()

    n_edges_processed = 0
    n_edges_skipped = 0

    for i, j in zip(adj_coo.row, adj_coo.col):
        if i < j:  # Only process each edge once
            # Get geometries
            geom_i = tracts_gdf.iloc[i].geometry
            geom_j = tracts_gdf.iloc[j].geometry

            # Compute shared boundary length
            try:
                intersection = geom_i.intersection(geom_j)

                # Boundary length in meters (assuming projected CRS)
                # Convert to kilometers for readability
                if intersection.is_empty:
                    # Adjacent but no shared boundary (corner touching)
                    boundary_length_km = 0.001  # Minimal weight
                elif intersection.geom_type == 'LineString':
                    boundary_length_km = intersection.length / 1000
                elif intersection.geom_type == 'MultiLineString':
                    boundary_length_km = sum(line.length for line in intersection.geoms) / 1000
                elif intersection.geom_type == 'Point':
                    # Single point intersection (corner touching)
                    boundary_length_km = 0.001
                else:
                    # Unexpected geometry type
                    boundary_length_km = 0.001

                edge_weights[(i, j)] = boundary_length_km
                total_boundary_km += boundary_length_km
                n_edges_processed += 1

            except Exception as e:
                # Geometry error - use minimal weight
                edge_weights[(i, j)] = 0.001
                n_edges_skipped += 1

    print(f"  Edge weights computed: {n_edges_processed} edges, {n_edges_skipped} skipped")
    print(f"  Total boundary: {total_boundary_km:.1f} km")

    return edge_weights, total_boundary_km


def run_redistricting_comparison(state_name, n_districts):
    """
    Run redistricting with and without geographic edge weights.

    Returns
    -------
    dict : Results containing compactness and partisan metrics
    """
    print(f"\n{'='*80}")
    print(f"Processing: {state_name.upper()} ({n_districts} districts)")
    print(f"{'='*80}")

    # Load tract data
    print("Loading tract data...")
    tract_file = get_tract_file(state_name, 2020)
    tracts_gdf = gpd.read_file(tract_file)

    # Ensure projected CRS for accurate distance calculations
    if tracts_gdf.crs is None or tracts_gdf.crs.is_geographic:
        print("  Warning: Reprojecting to EPSG:5070 (Albers Equal Area)")
        tracts_gdf = tracts_gdf.to_crs(epsg=5070)

    print(f"  Loaded {len(tracts_gdf)} tracts")

    # Extract populations
    populations = tracts_gdf['POP20'].values
    total_pop = populations.sum()
    print(f"  Total population: {total_pop:,}")

    # Build adjacency graph
    print("Building adjacency graph...")
    adj_matrix, adj_list = build_adjacency_graph(tracts_gdf)
    n_edges = adj_matrix.nnz // 2
    print(f"  Adjacency: {len(tracts_gdf)} nodes, {n_edges} edges")

    # Compute geographic edge weights
    print("Computing geographic edge weights...")
    edge_weights, total_boundary = compute_geographic_edge_weights(tracts_gdf, adj_matrix)

    # ===== UNWEIGHTED REDISTRICTING =====
    print("\n--- UNWEIGHTED (baseline) ---")
    partition_unweighted = recursive_bisection(
        tracts_gdf,
        n_districts,
        population_col='POP20',
        ufactor=5,
        niter=100,
        edge_weights=None,  # No edge weights
        seed=42
    )

    # Compute compactness
    tracts_gdf['district_unweighted'] = partition_unweighted
    compactness_unweighted = compute_compactness_metrics(
        tracts_gdf.groupby('district_unweighted')['geometry'].apply(unary_union)
    )

    pp_unweighted = compactness_unweighted['polsby_popper'].mean()
    reock_unweighted = compactness_unweighted['reock'].mean()

    print(f"  Polsby-Popper: {pp_unweighted:.4f}")
    print(f"  Reock: {reock_unweighted:.4f}")

    # ===== EDGE-WEIGHTED REDISTRICTING =====
    print("\n--- EDGE-WEIGHTED (geographic) ---")
    partition_weighted = recursive_bisection(
        tracts_gdf,
        n_districts,
        population_col='POP20',
        ufactor=5,
        niter=100,
        edge_weights=edge_weights,  # WITH edge weights
        seed=42
    )

    # Compute compactness
    tracts_gdf['district_weighted'] = partition_weighted
    compactness_weighted = compute_compactness_metrics(
        tracts_gdf.groupby('district_weighted')['geometry'].apply(unary_union)
    )

    pp_weighted = compactness_weighted['polsby_popper'].mean()
    reock_weighted = compactness_weighted['reock'].mean()

    print(f"  Polsby-Popper: {pp_weighted:.4f}")
    print(f"  Reock: {reock_weighted:.4f}")

    # Calculate improvement
    pp_improvement = ((pp_weighted - pp_unweighted) / pp_unweighted) * 100
    reock_improvement = ((reock_weighted - reock_unweighted) / reock_unweighted) * 100

    print(f"\n  IMPROVEMENT:")
    print(f"  Polsby-Popper: +{pp_improvement:.1f}%")
    print(f"  Reock: +{reock_improvement:.1f}%")

    # Population balance check
    pop_unweighted = tracts_gdf.groupby('district_unweighted')['POP20'].sum()
    pop_weighted = tracts_gdf.groupby('district_weighted')['POP20'].sum()

    target_pop = total_pop / n_districts
    pop_dev_unweighted = ((pop_unweighted - target_pop).abs() / target_pop * 100).mean()
    pop_dev_weighted = ((pop_weighted - target_pop).abs() / target_pop * 100).mean()

    print(f"\n  Population Balance:")
    print(f"  Unweighted: {pop_dev_unweighted:.2f}% avg deviation")
    print(f"  Weighted: {pop_dev_weighted:.2f}% avg deviation")

    # Compile results
    results = {
        'state': state_name,
        'districts': n_districts,
        'tracts': len(tracts_gdf),
        'population': total_pop,
        'pp_unweighted': pp_unweighted,
        'pp_weighted': pp_weighted,
        'pp_improvement_pct': pp_improvement,
        'reock_unweighted': reock_unweighted,
        'reock_weighted': reock_weighted,
        'reock_improvement_pct': reock_improvement,
        'pop_dev_unweighted': pop_dev_unweighted,
        'pop_dev_weighted': pop_dev_weighted,
        'total_boundary_km': total_boundary,
    }

    return results


def main():
    """Run geographic edge-weighting analysis for P2.2."""
    print("="*80)
    print("P2.2: GEOGRAPHIC EDGE-WEIGHTED OPTIMIZATION ANALYSIS")
    print("="*80)
    print(f"Start time: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"Testing {len(TEST_STATES)} states...")

    all_results = []

    for state_name, config in TEST_STATES.items():
        try:
            results = run_redistricting_comparison(state_name, config['districts'])
            results['size'] = config['size']
            all_results.append(results)
        except Exception as e:
            print(f"\nERROR processing {state_name}: {e}")
            import traceback
            traceback.print_exc()
            continue

    # Create results DataFrame
    df = pd.DataFrame(all_results)

    # Save to CSV
    output_dir = project_root / 'research' / 'gerry-recursive-bisection' / 'data'
    output_dir.mkdir(parents=True, exist_ok=True)
    output_file = output_dir / 'p22_geographic_edge_weighting_results.csv'
    df.to_csv(output_file, index=False)
    print(f"\nResults saved to: {output_file}")

    # Print summary table
    print("\n" + "="*80)
    print("SUMMARY: GEOGRAPHIC EDGE-WEIGHTING RESULTS")
    print("="*80)
    print(df[['state', 'districts', 'pp_unweighted', 'pp_weighted', 'pp_improvement_pct']].to_string(index=False))

    # Overall statistics
    print("\n" + "="*80)
    print("AGGREGATE STATISTICS")
    print("="*80)
    print(f"Mean PP improvement: {df['pp_improvement_pct'].mean():.1f}%")
    print(f"Median PP improvement: {df['pp_improvement_pct'].median():.1f}%")
    print(f"Range: {df['pp_improvement_pct'].min():.1f}% to {df['pp_improvement_pct'].max():.1f}%")

    print(f"\nMean Reock improvement: {df['reock_improvement_pct'].mean():.1f}%")
    print(f"Median Reock improvement: {df['reock_improvement_pct'].median():.1f}%")

    print(f"\n{'='*80}")
    print(f"End time: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"{'='*80}")


if __name__ == '__main__':
    main()
