"""
N-way vs recursive comparison using data from C:/src/apportionment/data/2020/

This script:
1. Loads tract geometries from data/2020/tiger/tracts/
2. Loads demographics from data/2020/demographics/
3. Runs n-way partitioning with multi-constraint (tpwgts)
4. Compares results across 5 VRA states

For Paper 2: N-Way vs Recursive Bisection
"""

import sys
from pathlib import Path
import pandas as pd
import numpy as np
import time
from collections import defaultdict

project_root = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(project_root / 'src'))
sys.path.insert(0, str(project_root))

import geopandas as gpd
from apportionment.data.adjacency import build_adjacency_graph
from apportionment.partition.metis_executable import partition_graph_with_executable

# State configurations with FIPS codes
STATES = {
    'mississippi': {'fips': '28', 'k': 4, 'target_mm': 2, 'minority_pct': 0.461},
    'georgia': {'fips': '13', 'k': 14, 'target_mm': 5, 'minority_pct': 0.424},
    'louisiana': {'fips': '22', 'k': 6, 'target_mm': 2, 'minority_pct': 0.416},
    'alabama': {'fips': '01', 'k': 7, 'target_mm': 2, 'minority_pct': 0.369},
    'south_carolina': {'fips': '45', 'k': 7, 'target_mm': 3, 'minority_pct': 0.351}
}


def load_tracts_and_demographics(state_name, fips_code, year='2020'):
    """
    Load census tracts with demographics from data directory.

    Returns GeoDataFrame with geometry, population, and demographics.
    """
    # Paths
    data_dir = project_root / 'data' / year
    tiger_dir = data_dir / 'tiger' / 'tracts' / f'tl_{year}_{fips_code}_tract'
    demographics_file = data_dir / 'demographics' / f'{state_name}_demographics_{year}.csv'

    print(f"  Loading tracts from {tiger_dir}")

    # Load tract shapefile
    shapefiles = list(tiger_dir.glob('*.shp'))
    if not shapefiles:
        raise FileNotFoundError(f"No shapefile found in {tiger_dir}")

    tracts = gpd.read_file(shapefiles[0])
    print(f"    Loaded {len(tracts)} tracts")

    # Load demographics
    print(f"  Loading demographics from {demographics_file}")
    demographics = pd.read_csv(demographics_file)
    print(f"    Loaded demographics for {len(demographics)} tracts")

    # Ensure GEOID types match (convert both to string with zero-padding)
    tracts['GEOID'] = tracts['GEOID'].astype(str)
    # Demographics GEOIDs may be integers - convert and zero-pad to 11 digits
    demographics['GEOID'] = demographics['GEOID'].astype(str).str.zfill(11)

    # Merge
    tracts = tracts.merge(demographics, on='GEOID', how='inner')
    print(f"    Merged: {len(tracts)} tracts with demographics")

    # Ensure required columns
    required = ['GEOID', 'geometry', 'total_pop']
    missing = [c for c in required if c not in tracts.columns]
    if missing:
        raise ValueError(f"Missing required columns: {missing}")

    # Compute minority population/VAP
    # Prefer VAP if available, else use total population
    if 'black_vap' in tracts.columns and 'total_vap' in tracts.columns:
        tracts['minority_vap'] = tracts['black_vap']
        tracts['minority_total'] = tracts['total_vap']
    elif 'black_non_hispanic' in tracts.columns:
        # Use total population as proxy for VAP (rough approximation)
        # Typically VAP is ~75-80% of total pop, but for comparison purposes this is OK
        tracts['minority_vap'] = tracts['black_non_hispanic']
        tracts['minority_total'] = tracts['total_pop']
    else:
        raise ValueError("Cannot determine minority population from available columns")

    tracts['pct_minority'] = (tracts['minority_vap'] / tracts['minority_total']).fillna(0)

    # Rename total_pop to population for build_adjacency_graph compatibility
    if 'population' not in tracts.columns:
        tracts['population'] = tracts['total_pop']

    return tracts


def create_target_weights_2d(num_districts, mm_targets, state_minority_pct):
    """
    Create 2D target weights for multi-constraint partitioning.
    Format: [pop_weight_1, minority_weight_1, pop_weight_2, minority_weight_2, ...]
    """
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


def run_nway_multiconstraint(state_name, config):
    """Run n-way partitioning with multi-constraint (tpwgts)"""
    print(f"\n{'='*70}")
    print(f"Testing {state_name.upper()}")
    print(f"{'='*70}")

    # Load tracts with demographics
    tracts = load_tracts_and_demographics(state_name, config['fips'])

    # Build adjacency
    print("  Building adjacency graph...")
    adjacency_list, vertex_weights, _, _, _ = build_adjacency_graph(tracts)
    print(f"    Built adjacency: {len(adjacency_list)} nodes")

    # Create 2D vertex weights for multi-constraint
    # Use minority_vap (which we computed from either VAP or total pop)
    vertex_weights_2d = np.column_stack([
        tracts['total_pop'].values,
        tracts['minority_vap'].values
    ])

    # Create target weights
    tpwgts = create_target_weights_2d(
        config['k'], config['target_mm'], config['minority_pct']
    )

    # Run n-way partitioning
    print(f"  Running n-way partitioning (k={config['k']})...")
    start_time = time.time()

    partition = partition_graph_with_executable(
        adjacency_list,
        vertex_weights_2d,
        nparts=config['k'],
        target_weights=tpwgts,
        ubvec=[1.005, 1.50],  # Tight population, loose minority
        niter=100,
        debug=False
    )

    runtime = time.time() - start_time

    # Analyze results
    results = analyze_partition(partition, tracts)
    results['runtime'] = runtime
    results['state'] = state_name
    results['k'] = config['k']
    results['target_mm'] = config['target_mm']
    results['state_minority_pct'] = config['minority_pct']
    results['method'] = 'nway_multiconstraint'

    print(f"\n  RESULTS:")
    print(f"    MM Districts: {results['mm_count']}/{config['target_mm']}")
    print(f"    Max Minority: {results['max_minority_pct']:.1%}")
    print(f"    Top 3 Districts: {[f'{p:.1%}' for p in results['district_minorities'][:3]]}")
    print(f"    Runtime: {runtime:.2f}s")
    print(f"    Success: {'YES' if results['mm_count'] >= config['target_mm'] else 'NO'}")

    return results


def main():
    """Run n-way comparison for all states"""
    results = []

    # All 5 VRA states
    test_states = ['mississippi', 'louisiana', 'alabama', 'south_carolina', 'georgia']

    for state_name in test_states:
        try:
            result = run_nway_multiconstraint(state_name, STATES[state_name])
            results.append(result)
        except Exception as e:
            print(f"\n{'='*70}")
            print(f"ERROR processing {state_name}: {e}")
            print(f"{'='*70}")
            import traceback
            traceback.print_exc()
            continue

    # Save results
    if results:
        results_df = pd.DataFrame(results)

        # Drop district_minorities (list column) for CSV
        results_for_csv = results_df.drop(columns=['district_minorities'])

        output_path = project_root / 'research' / 'gerry-nway-vs-recursive' / 'results'
        output_path.mkdir(parents=True, exist_ok=True)
        output_file = output_path / 'nway_multiconstraint_results.csv'
        results_for_csv.to_csv(output_file, index=False)

        print(f"\n{'='*70}")
        print(f"Results saved to {output_file}")
        print(f"{'='*70}")

        print("\nSUMMARY:")
        summary_cols = ['state', 'k', 'target_mm', 'mm_count', 'max_minority_pct', 'runtime']
        print(results_df[summary_cols].to_string(index=False))

        # Success rate
        success_count = sum(r['mm_count'] >= r['target_mm'] for r in results)
        print(f"\nSuccess Rate: {success_count}/{len(results)} ({100*success_count/len(results):.0f}%)")

    else:
        print("\n[NO RESULTS GENERATED]")


if __name__ == '__main__':
    main()
