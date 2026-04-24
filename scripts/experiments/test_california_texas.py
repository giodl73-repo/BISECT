"""
Test California and Texas with edge-weighting approach (alpha=5 @ 40%).

Large states with dispersed minority populations:
- California: 52 districts, ~20% Latino VAP
- Texas: 38 districts, ~28% Latino VAP
"""

import sys
from pathlib import Path
import pandas as pd
import numpy as np
import time

project_root = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(project_root / 'src'))
sys.path.insert(0, str(project_root))

import geopandas as gpd
from apportionment.data.adjacency import build_adjacency_graph
from apportionment.partition.metis_executable import partition_graph_with_executable


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

    print(f"  Loaded {len(tracts)} tracts")

    # Compute TOTAL minority population (all non-white)
    # This is the proper VRA approach - protects ALL minority groups
    if 'white_non_hispanic' in tracts.columns and 'total_pop' in tracts.columns:
        tracts['minority_vap'] = tracts['total_pop'] - tracts['white_non_hispanic']
        minority_name = 'total_minority (all non-white)'
    else:
        raise ValueError("Cannot compute total minority population - missing required columns")

    print(f"  Using {minority_name} as minority VAP")

    tracts['minority_total'] = tracts['total_pop']
    tracts['pct_minority'] = (tracts['minority_vap'] / tracts['minority_total']).fillna(0)

    # Add population column for build_adjacency_graph
    if 'population' not in tracts.columns:
        tracts['population'] = tracts['total_pop']

    # Compute state-level minority percentage
    state_minority_pct = tracts['minority_vap'].sum() / tracts['total_pop'].sum()
    print(f"  State minority VAP: {state_minority_pct:.1%}")

    return tracts, state_minority_pct


def partition_with_edge_weighting(tracts, k, minority_threshold, weight_factor):
    """
    Partition using edge-weighting approach.

    Args:
        tracts: GeoDataFrame with tracts
        k: number of districts
        minority_threshold: threshold for minority tract (e.g., 0.4)
        weight_factor: alpha (e.g., 5)
    """
    print(f"  Building adjacency graph...")
    adjacency_list, tract_indices, _, _, _ = build_adjacency_graph(tracts)

    # Classify tracts as minority or not
    minority_mask = tracts['pct_minority'] >= minority_threshold
    minority_count = minority_mask.sum()
    print(f"  Minority tracts (>={minority_threshold:.0%}): {minority_count}/{len(tracts)}")

    # Weight edges based on minority adjacency
    print(f"  Weighting edges (alpha={weight_factor})...")
    edge_weights = {}  # Dictionary mapping (i, j) -> weight
    minority_edges = 0

    for node_idx, neighbors in enumerate(adjacency_list):
        is_minority = minority_mask.iloc[node_idx]

        for neighbor_idx in neighbors:
            # Only process each edge once (use canonical ordering)
            if neighbor_idx > node_idx:
                neighbor_is_minority = minority_mask.iloc[neighbor_idx]

                # If both tracts are minority, apply weight factor
                if is_minority and neighbor_is_minority:
                    edge_key = (node_idx, neighbor_idx)
                    edge_weights[edge_key] = weight_factor
                    minority_edges += 1
                # Otherwise use default weight of 1 (no need to store)

    print(f"  Total minority-minority edges: {minority_edges}")

    # 1D vertex weights (population only)
    vertex_weights_1d = tracts['total_pop'].values

    # Run METIS with edge weights
    print(f"  Running METIS (k={k})...")
    partition = partition_graph_with_executable(
        adjacency_list,  # Keep as simple list of neighbor indices
        vertex_weights_1d,
        nparts=k,
        ubvec=[1.005],
        niter=100,
        debug=False,
        edge_weights=edge_weights  # Pass as separate dictionary
    )

    return partition


def analyze_partition(partition, tracts_gdf, target_mm):
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
    success = (mm_count >= target_mm)

    return {
        'max_minority_pct': max_minority,
        'mm_count': mm_count,
        'target_mm': target_mm,
        'success': success,
        'district_minorities': sorted(district_stats['pct_minority'].values, reverse=True)
    }


# State configurations
STATES = {
    'california': {
        'fips': '06',
        'k': 52,
        'target_mm': None,  # Will estimate based on state minority %
        'notes': 'Large Latino population, dispersed across state'
    },
    'texas': {
        'fips': '48',
        'k': 38,
        'target_mm': None,  # Will estimate based on state minority %
        'notes': 'Large Latino population, concentrated in South Texas and urban areas'
    }
}


def main():
    """Test California and Texas with edge-weighting"""
    results = []

    # Edge-weighting parameters
    weight_factor = 5
    minority_threshold = 0.4

    for state_name in ['california', 'texas']:
        config = STATES[state_name]

        print(f"\n{'='*70}")
        print(f"Testing {state_name.upper()}")
        print(f"{'='*70}")
        print(f"Notes: {config['notes']}")

        # Load data
        print("\nLoading data...")
        tracts, state_minority_pct = load_tracts_and_demographics(state_name, config['fips'])

        # Estimate target MM districts using Gingles proportionality
        # target_mm = floor(state_minority_pct * k)
        target_mm = int(state_minority_pct * config['k'])
        print(f"  Target MM districts (proportional): {target_mm}/{config['k']}")

        # Run edge-weighting
        print(f"\nRunning EDGE-WEIGHTING (alpha={weight_factor}, threshold={minority_threshold:.0%})...")
        start_time = time.time()

        partition = partition_with_edge_weighting(
            tracts, config['k'], minority_threshold, weight_factor
        )

        runtime = time.time() - start_time

        analysis = analyze_partition(partition, tracts.copy(), target_mm)

        results.append({
            'state': state_name,
            'method': 'edge_weighted',
            'k': config['k'],
            'target_mm': target_mm,
            'state_minority_pct': state_minority_pct,
            'weight_factor': weight_factor,
            'minority_threshold': minority_threshold,
            'runtime': runtime,
            **{k: v for k, v in analysis.items() if k != 'district_minorities'}
        })

        print(f"\n  RESULTS:")
        print(f"    MM Districts: {analysis['mm_count']}/{target_mm} ({'SUCCESS' if analysis['success'] else 'FAIL'})")
        print(f"    Max Minority: {analysis['max_minority_pct']:.1%}")
        top5 = [f"{p:.1%}" for p in analysis['district_minorities'][:5]]
        print(f"    Top 5 Districts: {', '.join(top5)}")
        print(f"    Runtime: {runtime:.2f}s")

    # Save results
    if results:
        results_df = pd.DataFrame(results)

        output_path = project_root / 'research' / 'gerry-nway-vs-recursive' / 'results'
        output_path.mkdir(parents=True, exist_ok=True)
        output_file = output_path / 'california_texas_results.csv'
        results_df.to_csv(output_file, index=False)

        print(f"\n{'='*70}")
        print(f"Results saved to {output_file}")
        print(f"{'='*70}")

        # Summary
        print(f"\nSUMMARY:")
        for _, row in results_df.iterrows():
            status = "SUCCESS" if row['success'] else "FAIL"
            print(f"  {row['state'].upper():12s}: {row['mm_count']}/{row['target_mm']} MM ({status}), "
                  f"max={row['max_minority_pct']:.1%}, runtime={row['runtime']:.1f}s")


if __name__ == '__main__':
    main()
