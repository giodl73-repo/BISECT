#!/usr/bin/env python3
"""
Cross-Census Validation Framework for Redistricting Algorithms

Implements the slice-based validation methodology from:
"Slice-Based Cross-Census Validation for Congressional Redistricting Algorithms"

This script:
1. Loads census tracts for 2000, 2010, 2020
2. Creates persistent centroids (population-weighted)
3. Partitions states into K=5 geographic slices using k-means
4. Runs METIS 10 times per state-slice-year
5. Computes compactness metrics (Polsby-Popper, Reock)
6. Performs variance decomposition (geographic vs temporal)
"""

import sys
import os
import warnings
from pathlib import Path
import pickle
import numpy as np
import pandas as pd
from datetime import datetime
import argparse
from typing import Dict, List, Tuple
from collections import defaultdict
import json

# Suppress warnings
warnings.filterwarnings('ignore')

# Add project paths
sys.path.insert(0, str(Path(__file__).parent.parent.parent / 'src'))
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

# Imports
from scripts.utils import get_state_config, get_tract_file, get_adjacency_file
from apportionment.partition.recursive_bisection import RecursiveBisection
import geopandas as gpd
from sklearn.cluster import KMeans
from scipy.spatial.distance import cdist


def compute_population_weighted_centroid(tracts_gdf: gpd.GeoDataFrame) -> Tuple[float, float]:
    """Compute population-weighted centroid for a set of tracts."""
    total_pop = tracts_gdf['POP100'].sum()
    if total_pop == 0:
        # Fallback to geometric centroid
        centroid = tracts_gdf.unary_union.centroid
        return (centroid.x, centroid.y)

    weighted_x = (tracts_gdf.geometry.centroid.x * tracts_gdf['POP100']).sum() / total_pop
    weighted_y = (tracts_gdf.geometry.centroid.y * tracts_gdf['POP100']).sum() / total_pop
    return (weighted_x, weighted_y)


def create_persistent_centroids(tracts_dict: Dict[str, gpd.GeoDataFrame],
                                state_code: str) -> pd.DataFrame:
    """
    Create persistent tract centroids by matching tracts across census years.

    Returns DataFrame with columns: tract_id, centroid_x, centroid_y, year_2000, year_2010, year_2020
    """
    print(f"  Creating persistent centroids for {state_code}...")

    # Start with 2020 tracts as base (most recent)
    base_tracts = tracts_dict['2020'].copy()
    base_tracts['centroid_x'] = base_tracts.geometry.centroid.x
    base_tracts['centroid_y'] = base_tracts.geometry.centroid.y

    # For each 2020 tract, find nearest tract in 2010 and 2000 (within 5km threshold)
    persistent_centroids = []

    for idx, tract_2020 in base_tracts.iterrows():
        centroid_2020 = np.array([[tract_2020['centroid_x'], tract_2020['centroid_y']]])

        # Find 2010 match
        if '2010' in tracts_dict:
            tracts_2010 = tracts_dict['2010']
            tracts_2010['centroid_x'] = tracts_2010.geometry.centroid.x
            tracts_2010['centroid_y'] = tracts_2010.geometry.centroid.y
            centroids_2010 = tracts_2010[['centroid_x', 'centroid_y']].values
            distances = cdist(centroid_2020, centroids_2010, metric='euclidean').flatten()
            nearest_2010_idx = np.argmin(distances)
            if distances[nearest_2010_idx] < 5000:  # 5km threshold in meters
                match_2010 = tracts_2010.iloc[nearest_2010_idx]
                centroid_2010 = (match_2010['centroid_x'], match_2010['centroid_y'])
            else:
                centroid_2010 = None
        else:
            centroid_2010 = None

        # Find 2000 match
        if '2000' in tracts_dict:
            tracts_2000 = tracts_dict['2000']
            tracts_2000['centroid_x'] = tracts_2000.geometry.centroid.x
            tracts_2000['centroid_y'] = tracts_2000.geometry.centroid.y
            centroids_2000 = tracts_2000[['centroid_x', 'centroid_y']].values
            distances = cdist(centroid_2020, centroids_2000, metric='euclidean').flatten()
            nearest_2000_idx = np.argmin(distances)
            if distances[nearest_2000_idx] < 5000:
                match_2000 = tracts_2000.iloc[nearest_2000_idx]
                centroid_2000 = (match_2000['centroid_x'], match_2000['centroid_y'])
            else:
                centroid_2000 = None
        else:
            centroid_2000 = None

        # Compute persistent centroid as mean of available centroids
        available_centroids = [centroid_2020[0]]
        if centroid_2010 is not None:
            available_centroids.append(centroid_2010)
        if centroid_2000 is not None:
            available_centroids.append(centroid_2000)

        persistent_x = np.mean([c[0] if isinstance(c, tuple) else c[0] for c in available_centroids])
        persistent_y = np.mean([c[1] if isinstance(c, tuple) else c[1] for c in available_centroids])

        persistent_centroids.append({
            'tract_id': tract_2020['GEOID'],
            'persistent_x': persistent_x,
            'persistent_y': persistent_y,
            'has_2000': centroid_2000 is not None,
            'has_2010': centroid_2010 is not None,
            'has_2020': True
        })

    return pd.DataFrame(persistent_centroids)


def create_slices(persistent_centroids: pd.DataFrame, K: int = 5) -> np.ndarray:
    """
    Partition tracts into K geographic slices using k-means clustering.

    Returns: Array of slice assignments (0 to K-1)
    """
    print(f"    Creating {K} slices using k-means...")

    # Extract coordinates
    X = persistent_centroids[['persistent_x', 'persistent_y']].values

    # K-means clustering
    kmeans = KMeans(n_clusters=K, random_state=42, n_init=10)
    slice_assignments = kmeans.fit_predict(X)

    return slice_assignments


def run_metis_ensemble(tracts_gdf: gpd.GeoDataFrame, graph_file: str,
                       num_districts: int, n_runs: int = 10) -> Dict:
    """
    Run METIS 10 times and collect statistics.

    Returns dict with edge_cuts, pp_scores, reock_scores, runtimes
    """
    results = {
        'edge_cuts': [],
        'pp_scores': [],
        'reock_scores': [],
        'runtimes': []
    }

    for run_idx in range(n_runs):
        try:
            # Run METIS with different seed
            rb = RecursiveBisection(
                tracts=tracts_gdf,
                graph_file=graph_file,
                num_districts=num_districts,
                ufactor=1,  # 0.1% max imbalance (strict)
                niter=20,
                seed=42 + run_idx  # Different seed per run
            )

            districts_gdf = rb.partition()

            # Compute metrics
            edge_cut = rb.get_total_boundary_length()
            pp_scores = [compute_polsby_popper(districts_gdf[districts_gdf['district'] == d])
                        for d in districts_gdf['district'].unique()]
            reock_scores = [compute_reock(districts_gdf[districts_gdf['district'] == d])
                          for d in districts_gdf['district'].unique()]

            results['edge_cuts'].append(edge_cut)
            results['pp_scores'].append(np.mean(pp_scores))
            results['reock_scores'].append(np.mean(reock_scores))
            results['runtimes'].append(rb.runtime)

        except Exception as e:
            print(f"      WARNING: METIS run {run_idx+1} failed: {e}")
            continue

    return results


def compute_polsby_popper(district_gdf: gpd.GeoDataFrame) -> float:
    """Compute Polsby-Popper compactness score."""
    geom = district_gdf.unary_union
    area = geom.area
    perimeter = geom.length
    if perimeter == 0:
        return 0.0
    return (4 * np.pi * area) / (perimeter ** 2)


def compute_reock(district_gdf: gpd.GeoDataFrame) -> float:
    """Compute Reock compactness score."""
    geom = district_gdf.unary_union
    area = geom.area
    # Minimum bounding circle approximation using convex hull
    convex_hull = geom.convex_hull
    mbc_area = convex_hull.area
    if mbc_area == 0:
        return 0.0
    return area / mbc_area


def process_state_slice(state_code: str, slice_idx: int, year: str,
                       slice_tracts: gpd.GeoDataFrame, state_config: dict,
                       version: str) -> Dict:
    """Process a single state-slice-year combination."""

    print(f"    Processing {state_code} slice {slice_idx+1} for {year}...")

    config = state_config[state_code]
    num_districts_total = config['districts']

    # Determine number of districts for this slice (proportional to population)
    # For simplicity, assign at least 1 district per slice
    slice_population = slice_tracts['POP100'].sum()
    num_districts_slice = max(1, int(round(num_districts_total *
                                           (slice_population / slice_tracts['POP100'].sum()))))

    # Get graph file
    graph_file = str(get_adjacency_file(state_code, year, version))

    if not Path(graph_file).exists():
        print(f"      WARNING: Graph file not found: {graph_file}")
        return None

    # Run METIS ensemble (10 runs)
    ensemble_results = run_metis_ensemble(slice_tracts, graph_file,
                                          num_districts_slice, n_runs=10)

    if not ensemble_results['edge_cuts']:
        print(f"      WARNING: No successful METIS runs for {state_code} slice {slice_idx+1}")
        return None

    # Select median run by edge-cut
    median_idx = np.argsort(ensemble_results['edge_cuts'])[len(ensemble_results['edge_cuts'])//2]

    return {
        'state': state_code,
        'slice': slice_idx,
        'year': year,
        'num_districts': num_districts_slice,
        'population': int(slice_population),
        'num_tracts': len(slice_tracts),
        'edge_cut_mean': np.mean(ensemble_results['edge_cuts']),
        'edge_cut_std': np.std(ensemble_results['edge_cuts']),
        'edge_cut_median': ensemble_results['edge_cuts'][median_idx],
        'pp_mean': np.mean(ensemble_results['pp_scores']),
        'pp_std': np.std(ensemble_results['pp_scores']),
        'pp_median': ensemble_results['pp_scores'][median_idx],
        'reock_mean': np.mean(ensemble_results['reock_scores']),
        'reock_std': np.std(ensemble_results['reock_scores']),
        'reock_median': ensemble_results['reock_scores'][median_idx],
        'runtime_mean': np.mean(ensemble_results['runtimes']),
        'cv_edge_cut': np.std(ensemble_results['edge_cuts']) / np.mean(ensemble_results['edge_cuts']) if np.mean(ensemble_results['edge_cuts']) > 0 else 0
    }


def compute_variance_decomposition(results_df: pd.DataFrame) -> Dict:
    """
    Compute variance decomposition: geographic vs temporal.

    Returns dict with variance components and ratio.
    """
    print("\nComputing variance decomposition...")

    # Geographic variance: variance across slices within each state-year
    geographic_variances = []
    for (state, year), group in results_df.groupby(['state', 'year']):
        if len(group) >= 2:
            geographic_variances.append(group['pp_median'].var())

    var_geographic = np.mean(geographic_variances) if geographic_variances else 0

    # Temporal variance: variance across years within each state-slice
    temporal_variances = []
    for (state, slice_idx), group in results_df.groupby(['state', 'slice']):
        if len(group) >= 2:
            temporal_variances.append(group['pp_median'].var())

    var_temporal = np.mean(temporal_variances) if temporal_variances else 0

    ratio = var_geographic / var_temporal if var_temporal > 0 else float('inf')

    return {
        'variance_geographic': var_geographic,
        'variance_temporal': var_temporal,
        'std_geographic': np.sqrt(var_geographic),
        'std_temporal': np.sqrt(var_temporal),
        'variance_ratio': ratio
    }


def main():
    parser = argparse.ArgumentParser(description='Run cross-census validation experiment')
    parser.add_argument('--states', nargs='+', default=None,
                       help='States to process (default: all 50)')
    parser.add_argument('--years', nargs='+', default=['2000', '2010', '2020'],
                       help='Census years to include')
    parser.add_argument('--slices', type=int, default=5,
                       help='Number of geographic slices per state (K)')
    parser.add_argument('--version', default='v1',
                       help='Data version to use')
    parser.add_argument('--output', default='research/gerry-cross-census-validation/results',
                       help='Output directory for results')

    args = parser.parse_args()

    # Load state configurations
    from scripts.config_2020 import STATE_CONFIG_2020 as config_2020
    from scripts.config_2010 import STATE_CONFIG_2010 as config_2010
    from scripts.config_2000 import STATE_CONFIG_2000 as config_2000

    state_configs = {
        '2020': config_2020,
        '2010': config_2010,
        '2000': config_2000
    }

    # Determine states to process
    if args.states:
        states_to_process = [s.upper() for s in args.states]
    else:
        states_to_process = list(config_2020.keys())

    print(f"Cross-Census Validation Experiment")
    print(f"=" * 60)
    print(f"States: {len(states_to_process)}")
    print(f"Years: {args.years}")
    print(f"Slices per state: {args.slices}")
    print(f"Version: {args.version}")
    print(f"=" * 60)

    # Create output directory
    output_dir = Path(args.output)
    output_dir.mkdir(parents=True, exist_ok=True)

    # Collect all results
    all_results = []

    # Process each state
    for state_idx, state_code in enumerate(states_to_process, 1):
        print(f"\n[{state_idx}/{len(states_to_process)}] Processing {state_code}...")

        try:
            # Load tracts for all years
            tracts_dict = {}
            for year in args.years:
                tract_file = get_tract_file(state_code, year, args.version)
                if tract_file.exists():
                    tracts_dict[year] = gpd.read_file(tract_file)
                    print(f"  Loaded {len(tracts_dict[year])} tracts for {year}")
                else:
                    print(f"  WARNING: Tract file not found for {year}: {tract_file}")

            if not tracts_dict:
                print(f"  SKIPPING: No tract data found for {state_code}")
                continue

            # Create persistent centroids
            persistent_centroids = create_persistent_centroids(tracts_dict, state_code)

            # Create geographic slices
            slice_assignments = create_slices(persistent_centroids, K=args.slices)

            # Process each slice-year combination
            for slice_idx in range(args.slices):
                slice_tract_ids = persistent_centroids[slice_assignments == slice_idx]['tract_id'].values

                for year in args.years:
                    if year not in tracts_dict:
                        continue

                    # Get tracts in this slice for this year
                    slice_tracts = tracts_dict[year][tracts_dict[year]['GEOID'].isin(slice_tract_ids)]

                    if len(slice_tracts) == 0:
                        print(f"      WARNING: No tracts in slice {slice_idx+1} for {year}")
                        continue

                    # Process this slice-year combination
                    result = process_state_slice(
                        state_code, slice_idx, year, slice_tracts,
                        state_configs[year], args.version
                    )

                    if result:
                        all_results.append(result)

        except Exception as e:
            print(f"  ERROR processing {state_code}: {e}")
            import traceback
            traceback.print_exc()
            continue

    # Convert to DataFrame
    results_df = pd.DataFrame(all_results)

    # Save results
    results_file = output_dir / 'cross_census_validation_results.csv'
    results_df.to_csv(results_file, index=False)
    print(f"\nResults saved to: {results_file}")

    # Compute variance decomposition
    if len(results_df) > 0:
        variance_decomp = compute_variance_decomposition(results_df)

        print("\n" + "=" * 60)
        print("VARIANCE DECOMPOSITION")
        print("=" * 60)
        print(f"Geographic variance:  {variance_decomp['variance_geographic']:.6f}")
        print(f"Temporal variance:    {variance_decomp['variance_temporal']:.6f}")
        print(f"Geographic std:       {variance_decomp['std_geographic']:.3f}")
        print(f"Temporal std:         {variance_decomp['std_temporal']:.3f}")
        print(f"Variance ratio:       {variance_decomp['variance_ratio']:.2f}×")
        print("=" * 60)

        # Save variance decomposition
        variance_file = output_dir / 'variance_decomposition.json'
        with open(variance_file, 'w') as f:
            json.dump(variance_decomp, f, indent=2)
        print(f"Variance decomposition saved to: {variance_file}")

    print("\nExperiment complete!")


if __name__ == '__main__':
    main()
