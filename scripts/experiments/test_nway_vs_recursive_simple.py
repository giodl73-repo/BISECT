"""
Simple n-way vs recursive comparison script.
Focuses on getting n-way results first (recursive is more complex).

Uses multi-constraint (tpwgts) approach similar to Paper 1's edge-weighting,
but without edge weights to isolate the partitioning strategy effect.
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
from scipy.sparse import lil_matrix
from apportionment.data.adjacency import build_adjacency_graph
from apportionment.partition.metis_executable import partition_graph_with_executable
from scripts.utils import get_tract_file

# State configurations (using state abbreviations to match file names)
STATES = {
    'ms': {'name': 'Mississippi', 'k': 4, 'target_mm': 2, 'minority_pct': 0.461},
    'ga': {'name': 'Georgia', 'k': 14, 'target_mm': 5, 'minority_pct': 0.424},
    'la': {'name': 'Louisiana', 'k': 6, 'target_mm': 2, 'minority_pct': 0.416},
    'al': {'name': 'Alabama', 'k': 7, 'target_mm': 2, 'minority_pct': 0.369},
    'sc': {'name': 'South Carolina', 'k': 7, 'target_mm': 3, 'minority_pct': 0.351}
}


def load_tracts(state_name, year='2020', version='v1'):
    """Load tracts with demographics"""
    tract_file = get_tract_file(state_name, year, version)
    tracts = gpd.read_file(tract_file)

    # Ensure we have necessary columns
    required_cols = ['GEOID', 'geometry', 'total_pop', 'total_vap']
    for col in required_cols:
        if col not in tracts.columns:
            raise ValueError(f"Missing required column: {col}")

    # Compute minority VAP if not present
    if 'minority_vap' not in tracts.columns:
        if 'black_vap' in tracts.columns:
            tracts['minority_vap'] = tracts['black_vap']
        else:
            raise ValueError("Cannot determine minority_vap")

    # Compute percentage
    tracts['pct_minority'] = (tracts['minority_vap'] / tracts['total_vap']).fillna(0)

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


def run_nway_multiconstraint(state_abbr, config):
    """Run n-way partitioning with multi-constraint (tpwgts)"""
    print(f"\n{'='*70}")
    print(f"Testing {config['name'].upper()}")
    print(f"{'='*70}")

    # Load tracts
    print("Loading tracts...")
    tracts = load_tracts(state_abbr)
    print(f"  Loaded {len(tracts)} tracts")

    # Build adjacency
    print("Building adjacency graph...")
    adjacency_list, vertex_weights, _, _, _ = build_adjacency_graph(tracts)
    print(f"  Built adjacency: {len(adjacency_list)} nodes")

    # Create 2D vertex weights for multi-constraint
    vertex_weights_2d = np.column_stack([
        tracts['total_pop'].values,
        tracts['minority_vap'].values
    ])

    # Create target weights
    tpwgts = create_target_weights_2d(
        config['k'], config['target_mm'], config['minority_pct']
    )

    # Run n-way partitioning
    print(f"Running n-way partitioning (k={config['k']})...")
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
    results['state'] = state_abbr
    results['state_name'] = config['name']
    results['k'] = config['k']
    results['target_mm'] = config['target_mm']
    results['state_minority_pct'] = config['minority_pct']

    print(f"\nRESULTS:")
    print(f"  MM Districts: {results['mm_count']}/{config['target_mm']}")
    print(f"  Max Minority: {results['max_minority_pct']:.1%}")
    print(f"  Top 3 Districts: {[f'{p:.1%}' for p in results['district_minorities'][:3]]}")
    print(f"  Runtime: {runtime:.2f}s")

    return results


def main():
    """Run n-way comparison for all states"""
    results = []

    # Test with Mississippi first
    test_states = ['ms']  # Start small

    for state_abbr in test_states:
        try:
            result = run_nway_multiconstraint(state_abbr, STATES[state_abbr])
            results.append(result)
        except Exception as e:
            print(f"\nERROR processing {state_abbr}: {e}")
            import traceback
            traceback.print_exc()

    # Save results
    if results:
        results_df = pd.DataFrame(results)
        output_path = project_root / 'research' / 'gerry-nway-vs-recursive' / 'results'
        output_path.mkdir(parents=True, exist_ok=True)
        output_file = output_path / 'nway_multiconstraint_results.csv'
        results_df.to_csv(output_file, index=False)

        print(f"\n{'='*70}")
        print(f"Results saved to {output_file}")
        print(f"{'='*70}")

        print("\nSUMMARY:")
        print(results_df[['state', 'k', 'target_mm', 'mm_count', 'max_minority_pct']].to_string(index=False))

    else:
        print("\n[NO RESULTS GENERATED]")


if __name__ == '__main__':
    main()
