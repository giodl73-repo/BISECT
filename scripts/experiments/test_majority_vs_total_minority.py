"""
Compare majority minority vs total minority approaches.

For each state, run edge-weighting with:
1. Majority minority (largest single minority group)
2. Total minority (all non-white coalition)

Shows when coalition districts help vs hurt.
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

    # Ensure GEOID types match
    tracts['GEOID'] = tracts['GEOID'].astype(str)
    demographics['GEOID'] = demographics['GEOID'].astype(str).str.zfill(11)

    # Merge
    tracts = tracts.merge(demographics, on='GEOID', how='inner')

    print(f"  Loaded {len(tracts)} tracts")

    # Identify available minority groups
    minority_groups = {}
    if 'black_non_hispanic' in tracts.columns:
        minority_groups['black'] = tracts['black_non_hispanic'].sum()
    if 'hispanic' in tracts.columns:
        minority_groups['hispanic'] = tracts['hispanic'].sum()
    if 'asian_non_hispanic' in tracts.columns:
        minority_groups['asian'] = tracts['asian_non_hispanic'].sum()

    # Find majority minority group (largest)
    if minority_groups:
        majority_minority = max(minority_groups, key=minority_groups.get)
        print(f"  Minority groups: {', '.join(f'{k}={v:,.0f}' for k,v in minority_groups.items())}")
        print(f"  Majority minority: {majority_minority}")
    else:
        majority_minority = None

    # Compute total minority
    if 'white_non_hispanic' in tracts.columns and 'total_pop' in tracts.columns:
        total_minority = tracts['total_pop'].sum() - tracts['white_non_hispanic'].sum()
    else:
        total_minority = None

    # Add population column
    if 'population' not in tracts.columns:
        tracts['population'] = tracts['total_pop']

    return tracts, majority_minority, minority_groups, total_minority


def partition_with_edge_weighting(tracts, k, minority_column, minority_name, minority_threshold, weight_factor):
    """Partition using edge-weighting with specified minority definition"""
    print(f"\n  Testing: {minority_name}")
    print(f"  Building adjacency graph...")
    adjacency_list, tract_indices, _, _, _ = build_adjacency_graph(tracts)

    # Set minority column
    tracts['minority_vap'] = tracts[minority_column]
    tracts['pct_minority'] = (tracts['minority_vap'] / tracts['total_pop']).fillna(0)

    # Compute state-level minority percentage
    state_minority_pct = tracts['minority_vap'].sum() / tracts['total_pop'].sum()
    print(f"  State {minority_name} %: {state_minority_pct:.1%}")

    # Classify tracts as minority or not
    minority_mask = tracts['pct_minority'] >= minority_threshold
    minority_count = minority_mask.sum()
    print(f"  Minority tracts (>={minority_threshold:.0%}): {minority_count}/{len(tracts)}")

    # Weight edges - EXPLICITLY set ALL edge weights (not just minority-minority)
    edge_weights = {}
    minority_edges = 0
    total_edges = 0

    for node_idx, neighbors in enumerate(adjacency_list):
        is_minority = minority_mask.iloc[node_idx]
        for neighbor_idx in neighbors:
            if neighbor_idx > node_idx:
                total_edges += 1
                neighbor_is_minority = minority_mask.iloc[neighbor_idx]
                edge_key = (node_idx, neighbor_idx)

                if is_minority and neighbor_is_minority:
                    # Both endpoints are minority - HIGH weight (expensive to cut)
                    edge_weights[edge_key] = weight_factor
                    minority_edges += 1
                else:
                    # At least one endpoint is not minority - NORMAL weight
                    # CRITICAL: Explicitly set to 1.0 (don't rely on defaults)
                    edge_weights[edge_key] = 1.0

    print(f"  Total edges: {total_edges}")
    print(f"  Minority-minority edges: {minority_edges} ({minority_edges/total_edges*100:.1f}%)")
    print(f"  Edge weights dictionary size: {len(edge_weights)}")
    print(f"  Weight factor: {weight_factor}x")

    # Show sample edge weights
    if edge_weights:
        sample = list(edge_weights.items())[:3]
        print(f"  Sample edge weights: {sample}")

    # Run METIS (SINGLE-OBJECTIVE optimization)
    vertex_weights_1d = tracts['total_pop'].values
    print(f"  Running METIS (k={k})...")

    partition = partition_graph_with_executable(
        adjacency_list,
        vertex_weights_1d,
        nparts=k,
        ufactor=1.005,  # Single-constraint imbalance (NOT ubvec for multi-constraint!)
        niter=100,
        debug=False,
        edge_weights=edge_weights
    )

    return partition, state_minority_pct, tracts


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
    'mississippi': {'fips': '28', 'k': 4, 'target_mm': 2},
    'louisiana': {'fips': '22', 'k': 6, 'target_mm': 2},
    'alabama': {'fips': '01', 'k': 7, 'target_mm': 2},
    'south_carolina': {'fips': '45', 'k': 7, 'target_mm': 3},
    'georgia': {'fips': '13', 'k': 14, 'target_mm': 5},
    'california': {'fips': '06', 'k': 52, 'target_mm': None},
    'texas': {'fips': '48', 'k': 38, 'target_mm': None},
}


def main():
    """Compare majority minority vs total minority"""
    results = []

    # Parameters
    weight_factor = 5
    minority_threshold = 0.4

    # Test with one state first (user requested)
    test_state = 'alabama'  # Can change to any state

    config = STATES[test_state]

    print(f"\n{'='*70}")
    print(f"Testing {test_state.upper()}: Majority Minority vs Total Minority")
    print(f"{'='*70}")

    # Load data
    print("\nLoading data...")
    tracts, majority_minority, minority_groups, total_minority_pop = load_tracts_and_demographics(
        test_state, config['fips']
    )

    # Determine target MM districts
    if config['target_mm'] is None:
        # Use proportionality based on majority minority
        if majority_minority:
            majority_col = f"{majority_minority}_non_hispanic" if majority_minority != 'hispanic' else 'hispanic'
            state_pct = tracts[majority_col].sum() / tracts['total_pop'].sum()
            target_mm = int(state_pct * config['k'])
        else:
            target_mm = 0
    else:
        target_mm = config['target_mm']

    print(f"\n  Target MM districts: {target_mm}/{config['k']}")

    # Test 1: Majority minority approach
    print(f"\n{'='*70}")
    print(f"APPROACH 1: MAJORITY MINORITY ({majority_minority.upper()})")
    print(f"{'='*70}")

    majority_col = f"{majority_minority}_non_hispanic" if majority_minority != 'hispanic' else 'hispanic'

    start_time = time.time()
    partition_majority, state_pct_majority, tracts_majority = partition_with_edge_weighting(
        tracts.copy(), config['k'], majority_col,
        f"majority_minority_{majority_minority}",
        minority_threshold, weight_factor
    )
    runtime_majority = time.time() - start_time

    analysis_majority = analyze_partition(partition_majority, tracts_majority, target_mm)

    results.append({
        'state': test_state,
        'approach': 'majority_minority',
        'minority_group': majority_minority,
        'k': config['k'],
        'target_mm': target_mm,
        'state_minority_pct': state_pct_majority,
        'weight_factor': weight_factor,
        'minority_threshold': minority_threshold,
        'runtime': runtime_majority,
        **{k: v for k, v in analysis_majority.items() if k != 'district_minorities'}
    })

    print(f"\n  RESULTS:")
    status = "SUCCESS" if analysis_majority['success'] else "FAIL"
    print(f"    MM Districts: {analysis_majority['mm_count']}/{target_mm} ({status})")
    print(f"    Max Minority: {analysis_majority['max_minority_pct']:.1%}")
    top5 = [f"{p:.1%}" for p in analysis_majority['district_minorities'][:5]]
    print(f"    Top 5 Districts: {', '.join(top5)}")
    print(f"    Runtime: {runtime_majority:.2f}s")

    # Test 2: Total minority (coalition) approach
    print(f"\n{'='*70}")
    print(f"APPROACH 2: TOTAL MINORITY (ALL NON-WHITE COALITION)")
    print(f"{'='*70}")

    # Create total minority column
    tracts_total = tracts.copy()
    tracts_total['total_minority'] = tracts_total['total_pop'] - tracts_total['white_non_hispanic']

    start_time = time.time()
    partition_total, state_pct_total, tracts_total_result = partition_with_edge_weighting(
        tracts_total, config['k'], 'total_minority',
        'total_minority_coalition',
        minority_threshold, weight_factor
    )
    runtime_total = time.time() - start_time

    analysis_total = analyze_partition(partition_total, tracts_total_result, target_mm)

    results.append({
        'state': test_state,
        'approach': 'total_minority',
        'minority_group': 'all_non_white',
        'k': config['k'],
        'target_mm': target_mm,
        'state_minority_pct': state_pct_total,
        'weight_factor': weight_factor,
        'minority_threshold': minority_threshold,
        'runtime': runtime_total,
        **{k: v for k, v in analysis_total.items() if k != 'district_minorities'}
    })

    print(f"\n  RESULTS:")
    status = "SUCCESS" if analysis_total['success'] else "FAIL"
    print(f"    MM Districts: {analysis_total['mm_count']}/{target_mm} ({status})")
    print(f"    Max Minority: {analysis_total['max_minority_pct']:.1%}")
    top5 = [f"{p:.1%}" for p in analysis_total['district_minorities'][:5]]
    print(f"    Top 5 Districts: {', '.join(top5)}")
    print(f"    Runtime: {runtime_total:.2f}s")

    # Comparison
    print(f"\n{'='*70}")
    print(f"COMPARISON")
    print(f"{'='*70}")

    print(f"\n  State minority %:")
    print(f"    Majority minority ({majority_minority}): {state_pct_majority:.1%}")
    print(f"    Total minority (coalition):  {state_pct_total:.1%}")
    print(f"    Difference: {(state_pct_total - state_pct_majority)*100:+.1f} percentage points")

    print(f"\n  MM Districts achieved:")
    print(f"    Majority minority: {analysis_majority['mm_count']}/{target_mm} ({analysis_majority['mm_count']/target_mm*100:.0f}%)")
    print(f"    Total minority:    {analysis_total['mm_count']}/{target_mm} ({analysis_total['mm_count']/target_mm*100:.0f}%)")

    delta_mm = analysis_total['mm_count'] - analysis_majority['mm_count']
    print(f"    Difference: {delta_mm:+d} MM districts")

    print(f"\n  Max minority concentration:")
    print(f"    Majority minority: {analysis_majority['max_minority_pct']:.1%}")
    print(f"    Total minority:    {analysis_total['max_minority_pct']:.1%}")
    print(f"    Difference: {(analysis_total['max_minority_pct'] - analysis_majority['max_minority_pct'])*100:+.1f} percentage points")

    if delta_mm > 0:
        print(f"\n  FINDING: Coalition approach HELPS (creates {delta_mm} more MM districts)")
    elif delta_mm < 0:
        print(f"\n  FINDING: Coalition approach HURTS (loses {-delta_mm} MM districts)")
    else:
        print(f"\n  FINDING: Coalition approach has NO EFFECT (same number of MM districts)")

    # Save results
    results_df = pd.DataFrame(results)
    output_path = project_root / 'research' / 'gerry-nway-vs-recursive' / 'results'
    output_path.mkdir(parents=True, exist_ok=True)
    output_file = output_path / f'{test_state}_majority_vs_total_comparison.csv'
    results_df.to_csv(output_file, index=False)

    print(f"\n{'='*70}")
    print(f"Results saved to {output_file}")
    print(f"{'='*70}")


if __name__ == '__main__':
    main()
