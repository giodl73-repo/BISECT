"""
Multi-Constraint Partitioning Experiments for Paper 4 - FIXED VERSION
Compares multi-constraint optimization vs edge-weighting for VRA compliance

CRITICAL FIX: Original version did NOT pass target_weights to METIS, causing it to
balance minorities equally across all districts instead of concentrating into MM districts.

This version CORRECTLY implements multi-constraint with asymmetric targets.
"""

import sys
from pathlib import Path
import numpy as np
import pandas as pd
import geopandas as gpd
from typing import Dict, List, Tuple
import time
import json

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent / 'src'))

from apportionment.data.adjacency import build_adjacency_graph
from apportionment.partition.metis_executable import partition_graph_with_executable

# Test states (same as Paper 1/2/3)
STATES = {
    'AL': {'fips': '01', 'name': 'Alabama', 'k': 7, 'target_mm': 2},
    'GA': {'fips': '13', 'name': 'Georgia', 'k': 14, 'target_mm': 5},
    'LA': {'fips': '22', 'name': 'Louisiana', 'k': 6, 'target_mm': 2},
    'MS': {'fips': '28', 'name': 'Mississippi', 'k': 4, 'target_mm': 2},
    'SC': {'fips': '45', 'name': 'South Carolina', 'k': 7, 'target_mm': 3},
}

# Test configurations: vary minority tolerance (ubvec[1])
UBVEC_CONFIGS = [
    [1.005, 1.30],  # Tight minority tolerance
    [1.005, 1.50],  # Medium (Paper 1 default)
    [1.005, 2.00],  # Loose
    [1.005, 5.00],  # Very loose
]

MINORITY_THRESHOLD = 0.40  # Same as edge-weighting optimal
METIS_SEED = 42
YEAR = '2020'


def load_state_data(state_code: str, fips_code: str, state_name: str) -> gpd.GeoDataFrame:
    """Load census tracts with demographics for a state"""

    # Load tracts (in subdirectory)
    tracts_dir = Path(f'data/{YEAR}/tiger/tracts')
    tract_subdir = tracts_dir / f'tl_{YEAR}_{fips_code}_tract'
    tracts_file = tract_subdir / f'tl_{YEAR}_{fips_code}_tract.shp'

    if not tracts_file.exists():
        raise FileNotFoundError(f"Tracts shapefile not found: {tracts_file}")

    tracts = gpd.read_file(tracts_file)

    # Load demographics (uses state name, not FIPS)
    state_name_lower = state_name.lower().replace(' ', '_')
    demographics_file = Path(f'data/{YEAR}/demographics/{state_name_lower}_demographics_{YEAR}.csv')

    if not demographics_file.exists():
        raise FileNotFoundError(f"Demographics file not found: {demographics_file}")

    # Read GEOID as string to preserve leading zeros
    demographics = pd.read_csv(demographics_file, dtype={'GEOID': str})

    # Ensure GEOID types match (convert both to string)
    tracts['GEOID'] = tracts['GEOID'].astype(str)

    # Merge
    tracts = tracts.merge(demographics, on='GEOID', how='inner')

    # Calculate minority metrics
    tracts['minority_vap'] = tracts['total_pop'] - tracts['white_non_hispanic']
    tracts['pct_minority'] = (tracts['minority_vap'] / tracts['total_pop']).fillna(0)

    # Ensure required columns for build_adjacency_graph
    if 'pop' not in tracts.columns:
        tracts['pop'] = tracts['total_pop']
    if 'population' not in tracts.columns:
        tracts['population'] = tracts['total_pop']

    print(f"  Loaded {len(tracts)} tracts")

    if len(tracts) == 0:
        raise ValueError(f"No tracts loaded after merge - check GEOID format compatibility")

    print(f"  Total population: {tracts['total_pop'].sum():,.0f}")
    print(f"  Total minority VAP: {tracts['minority_vap'].sum():,.0f} ({tracts['minority_vap'].sum()/tracts['total_pop'].sum()*100:.1f}%)")

    return tracts


def create_target_weights(k: int, target_mm: int, total_pop: float, total_minority: float) -> np.ndarray:
    """
    Create 2D target weights for multi-constraint partitioning

    Strategy: Concentrate minorities into target_mm districts to create MM districts

    CORRECTED LOGIC:
    - MM districts target: 60% minority (to ensure >50% after imbalance)
    - Non-MM districts: Remaining minority distributed proportionally
    - All targets are FRACTIONS that sum to 1.0 per constraint

    Args:
        k: Number of districts
        target_mm: Number of target MM districts
        total_pop: Total population
        total_minority: Total minority VAP

    Returns:
        tpwgts: Array of shape (k, 2) with [pop_fraction, minority_fraction] per district
    """

    # Population: Equal across all districts (1/k each)
    pop_per_district = 1.0 / k

    # Minority: Try to concentrate into target_mm districts
    # Goal: MM districts get 60% minority, others get remainder

    # If we could achieve 60% minority in each MM district:
    # - Total VAP per district = total_pop / k
    # - Minority VAP per MM district = 0.60 * (total_pop / k)
    # - But minority VAP is actually: minority_fraction * total_minority
    # So: minority_fraction * total_minority = 0.60 * (total_pop / k)
    # Therefore: minority_fraction = 0.60 * total_pop / (k * total_minority)

    overall_minority_fraction = total_minority / total_pop

    # Target fraction of total minority for each MM district
    # We want: (minority_fraction * total_minority) / (total_pop / k) = 0.60
    # So: minority_fraction = 0.60 * (total_pop / k) / total_minority
    #                      = 0.60 / (k * overall_minority_fraction)

    if overall_minority_fraction > 0:
        target_mm_fraction = 0.60 / (k * overall_minority_fraction)
    else:
        target_mm_fraction = 1.0 / k

    # Check if this is feasible (all MM districts can't exceed 100% of minority population)
    if target_mm * target_mm_fraction > 1.0:
        # Not enough minority population - distribute more evenly
        print(f"  WARNING: Insufficient minority population for {target_mm} MM districts at 60%")
        print(f"  Distributing available minority population proportionally")
        target_mm_fraction = 0.95 / target_mm  # Give 95% to MM districts, 5% to others

    # Calculate fractions for non-MM districts
    remaining_minority = 1.0 - (target_mm * target_mm_fraction)
    num_non_mm = k - target_mm

    if num_non_mm > 0 and remaining_minority > 0:
        target_non_mm_fraction = remaining_minority / num_non_mm
    else:
        target_non_mm_fraction = 0.0

    # Build target weight array
    tpwgts = []

    # First target_mm districts get higher minority targets
    for i in range(target_mm):
        tpwgts.append([pop_per_district, target_mm_fraction])

    # Remaining districts get lower minority targets
    for i in range(target_mm, k):
        tpwgts.append([pop_per_district, target_non_mm_fraction])

    tpwgts = np.array(tpwgts)

    # Verify normalization (should already sum to 1.0)
    pop_sum = tpwgts[:, 0].sum()
    min_sum = tpwgts[:, 1].sum()

    if not np.isclose(pop_sum, 1.0):
        print(f"  WARNING: Population fractions sum to {pop_sum:.6f}, normalizing")
        tpwgts[:, 0] /= pop_sum

    if not np.isclose(min_sum, 1.0):
        print(f"  WARNING: Minority fractions sum to {min_sum:.6f}, normalizing")
        tpwgts[:, 1] /= min_sum

    # Print target allocation for verification
    print(f"  Target weights calculation:")
    print(f"    Overall minority %: {overall_minority_fraction*100:.1f}%")
    print(f"    MM districts (n={target_mm}): {target_mm_fraction*100:.2f}% of total minority each")
    print(f"    Non-MM districts (n={num_non_mm}): {target_non_mm_fraction*100:.2f}% of total minority each")

    # Calculate expected minority % in MM districts
    expected_mm_pct = (target_mm_fraction * total_minority) / (total_pop / k) * 100
    print(f"    Expected minority % in MM districts: {expected_mm_pct:.1f}%")

    return tpwgts


def run_multi_constraint(
    state_code: str,
    tracts: gpd.GeoDataFrame,
    k: int,
    target_mm: int,
    ubvec: List[float]
) -> Dict:
    """
    Run multi-constraint partitioning for a state - FIXED VERSION

    NOW CORRECTLY PASSES target_weights TO METIS!

    Args:
        state_code: State abbreviation (e.g., 'AL')
        tracts: GeoDataFrame with tract geometries and demographics
        k: Number of districts
        target_mm: Target number of MM districts
        ubvec: Imbalance constraints [pop_tolerance, minority_tolerance]

    Returns:
        Dictionary with results
    """

    start_time = time.time()

    # Build adjacency graph
    adjacency_list, _, _, _, _ = build_adjacency_graph(tracts)

    # Create 2D vertex weights: [population, minority_vap]
    vertex_weights_2d = np.column_stack([
        tracts['total_pop'].values,
        tracts['minority_vap'].values
    ])

    total_pop = tracts['total_pop'].sum()
    total_minority = tracts['minority_vap'].sum()

    print(f"  Testing multi-constraint with ubvec={ubvec}")
    print(f"  Total minority: {total_minority:,.0f} ({total_minority/total_pop*100:.1f}%)")
    print(f"  Target: {target_mm} MM districts (>= 50% minority)")

    # ✅ FIX: CREATE TARGET WEIGHTS (was missing in original!)
    target_weights = create_target_weights(k, target_mm, total_pop, total_minority)

    # Convert to list of lists for METIS
    target_weights_list = target_weights.tolist()

    # ✅ FIX: PASS target_weights TO METIS (was missing in original!)
    try:
        partition = partition_graph_with_executable(
            adjacency_list,
            vertex_weights_2d,
            nparts=k,
            target_weights=target_weights_list,  # ← CRITICAL FIX: Added this line!
            ubvec=ubvec,
            niter=100,
            debug=True  # Enable debug to see METIS command
        )
    except Exception as e:
        print(f"  ERROR: METIS failed: {e}")
        return {
            'state': state_code,
            'ubvec': ubvec,
            'success': False,
            'error': str(e),
            'runtime': time.time() - start_time
        }

    # Analyze results
    tracts['district'] = partition

    # Calculate district statistics
    district_stats = []
    for dist_id in range(k):
        dist_tracts = tracts[tracts['district'] == dist_id]
        dist_pop = dist_tracts['total_pop'].sum()
        dist_minority = dist_tracts['minority_vap'].sum()
        dist_pct = dist_minority / dist_pop if dist_pop > 0 else 0

        district_stats.append({
            'district': dist_id,
            'population': dist_pop,
            'minority_vap': dist_minority,
            'pct_minority': dist_pct
        })

    df_stats = pd.DataFrame(district_stats)

    # Count MM districts (>= 50% minority)
    mm_count = (df_stats['pct_minority'] >= 0.50).sum()
    max_minority_pct = df_stats['pct_minority'].max()
    success = (mm_count >= target_mm)

    # Calculate edge cut (for compactness comparison)
    edge_cut = 0
    for node_idx, neighbors in enumerate(adjacency_list):
        node_district = partition[node_idx]
        for neighbor_idx in neighbors:
            if partition[neighbor_idx] != node_district:
                edge_cut += 1
    edge_cut //= 2  # Each edge counted twice

    runtime = time.time() - start_time

    print(f"  MM districts: {mm_count}/{target_mm} ({'SUCCESS' if success else 'FAIL'})")
    print(f"  Max minority %: {max_minority_pct*100:.1f}%")
    print(f"  Edge cut: {edge_cut}")
    print(f"  Runtime: {runtime:.1f}s")

    # Print actual district minority percentages for verification
    print(f"  District minority %: {[f'{p*100:.1f}%' for p in sorted(df_stats['pct_minority'], reverse=True)]}")

    return {
        'state': state_code,
        'k': k,
        'target_mm': target_mm,
        'ubvec_pop': ubvec[0],
        'ubvec_minority': ubvec[1],
        'mm_count': mm_count,
        'success': success,
        'max_minority_pct': max_minority_pct,
        'edge_cut': edge_cut,
        'district_pcts': df_stats['pct_minority'].tolist(),
        'runtime': runtime
    }


def main():
    """Run multi-constraint experiments for all states and configurations"""

    print("=" * 80)
    print("Multi-Constraint Partitioning Experiments - FIXED VERSION")
    print("=" * 80)
    print()
    print("CRITICAL FIX: Now correctly passes target_weights to METIS")
    print("  Original bug: Multi-constraint balanced minorities EQUALLY across districts")
    print("  Fixed version: Multi-constraint CONCENTRATES minorities into MM districts")
    print()
    print(f"States: {len(STATES)}")
    print(f"ubvec configurations: {len(UBVEC_CONFIGS)}")
    print(f"Total experiments: {len(STATES) * len(UBVEC_CONFIGS)}")
    print()

    results = []

    for state_code, state_info in STATES.items():
        state_name = state_info['name']
        fips_code = state_info['fips']
        k = state_info['k']
        target_mm = state_info['target_mm']

        print(f"\n{'='*80}")
        print(f"{state_name} ({state_code}): k={k}, target={target_mm} MM districts")
        print(f"{'='*80}")

        try:
            # Load state data
            tracts = load_state_data(state_code, fips_code, state_name)

            # Test each ubvec configuration
            for ubvec in UBVEC_CONFIGS:
                print(f"\nTesting ubvec={ubvec}...")

                result = run_multi_constraint(
                    state_code,
                    tracts,
                    k,
                    target_mm,
                    ubvec
                )

                results.append(result)

        except Exception as e:
            print(f"ERROR processing {state_name}: {e}")
            import traceback
            traceback.print_exc()

    # Save results
    df_results = pd.DataFrame(results)

    output_dir = Path('research/gerry-multi-vs-edge/results')
    output_dir.mkdir(parents=True, exist_ok=True)

    output_file = output_dir / 'multi_constraint_results_FIXED.csv'
    df_results.to_csv(output_file, index=False)

    print()
    print("=" * 80)
    print("EXPERIMENTS COMPLETE")
    print("=" * 80)
    print(f"Results saved to: {output_file}")
    print()

    # Summary statistics
    print("Summary:")
    print(f"  Total runs: {len(df_results)}")
    print(f"  Successful (achieved target MM): {df_results['success'].sum()}")
    print(f"  Success rate: {df_results['success'].mean()*100:.1f}%")
    print()

    # By state
    print("By state:")
    for state_code in STATES.keys():
        state_results = df_results[df_results['state'] == state_code]
        success_count = state_results['success'].sum()
        total_count = len(state_results)
        print(f"  {state_code}: {success_count}/{total_count} success")
    print()

    # By ubvec
    print("By ubvec (minority tolerance):")
    for ubvec in UBVEC_CONFIGS:
        ubvec_results = df_results[df_results['ubvec_minority'] == ubvec[1]]
        success_count = ubvec_results['success'].sum()
        total_count = len(ubvec_results)
        print(f"  ubvec={ubvec}: {success_count}/{total_count} success")

    print()
    print("=" * 80)
    print("COMPARISON WITH ORIGINAL (BROKEN) RESULTS")
    print("=" * 80)

    # Load original results for comparison
    original_file = output_dir / 'multi_constraint_results.csv'
    if original_file.exists():
        df_original = pd.read_csv(original_file)

        print(f"\nOriginal (broken) success rate: {df_original['success'].mean()*100:.1f}%")
        print(f"Fixed success rate: {df_results['success'].mean()*100:.1f}%")
        print(f"Difference: {(df_results['success'].mean() - df_original['success'].mean())*100:+.1f} percentage points")

        # Compare by state
        print("\nBy state comparison:")
        for state_code in STATES.keys():
            orig_success = df_original[df_original['state'] == state_code]['success'].mean() * 100
            fixed_success = df_results[df_results['state'] == state_code]['success'].mean() * 100
            print(f"  {state_code}: {orig_success:.0f}% -> {fixed_success:.0f}% ({fixed_success - orig_success:+.0f} pp)")


if __name__ == '__main__':
    main()
