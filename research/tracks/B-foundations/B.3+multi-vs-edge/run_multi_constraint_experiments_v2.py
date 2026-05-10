"""
Multi-Constraint Partitioning Experiments for Paper 4 - VERSION 2 (P1-3)
Balanced experimental design: 28 configs per state (matches edge-weighted)

CHANGES FROM FIXED VERSION:
- Expanded ubvec from 4 to 28 values (full constraint tightness spectrum)
- Total configs: 28 × 5 states = 140 (matches edge-weighted's 140)
- Addresses P1-3 reviewer concern about asymmetric configuration counts

This addresses Dr. Cynthia Phillips's concern:
"The experimental comparison is fundamentally unfair: 140 edge-weighted
configurations give 140 chances to find a good solution, while 4
multi-constraint configurations give only 4 chances."
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

# P1-3: Expanded ubvec configurations (28 values)
# Tests full constraint tightness spectrum from very tight to very loose
UBVEC_CONFIGS = [
    # Very tight (may fail due to over-constraint)
    [1.005, 1.10],
    [1.005, 1.15],
    [1.005, 1.20],
    [1.005, 1.25],

    # Tight-to-moderate (original range expanded)
    [1.005, 1.30],  # Original
    [1.005, 1.35],
    [1.005, 1.40],
    [1.005, 1.45],
    [1.005, 1.50],  # Original
    [1.005, 1.55],
    [1.005, 1.60],
    [1.005, 1.70],
    [1.005, 1.80],
    [1.005, 1.90],
    [1.005, 2.00],  # Original

    # Moderate-to-loose
    [1.005, 2.25],
    [1.005, 2.50],
    [1.005, 2.75],
    [1.005, 3.00],
    [1.005, 3.25],
    [1.005, 3.50],
    [1.005, 3.75],
    [1.005, 4.00],

    # Very loose (may fail due to insufficient guidance)
    [1.005, 4.50],
    [1.005, 5.00],  # Original
    [1.005, 6.00],
    [1.005, 7.00],
    [1.005, 10.0],
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

    CORRECTED LOGIC (from P1-1):
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

    overall_minority_fraction = total_minority / total_pop

    # Target fraction of total minority for each MM district
    # We want: (minority_fraction * total_minority) / (total_pop / k) = 0.60
    # So: minority_fraction = 0.60 / (k * overall_minority_fraction)

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
    Run multi-constraint partitioning for a state - FIXED VERSION (from P1-1)

    CORRECTLY PASSES target_weights TO METIS!

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

    # Create target weights (from P1-1 fix)
    target_weights = create_target_weights(k, target_mm, total_pop, total_minority)

    # Convert to list of lists for METIS
    target_weights_list = target_weights.tolist()

    # Pass target_weights TO METIS (critical fix from P1-1)
    try:
        partition = partition_graph_with_executable(
            adjacency=adjacency_list,
            vertex_weights=vertex_weights_2d,
            nparts=k,
            ubvec=ubvec,
            target_weights=target_weights_list,  # <-- CRITICAL: Must pass this!
            niter=100
        )
    except Exception as e:
        print(f"  METIS failed: {e}")
        return {
            'state': state_code,
            'ubvec_pop': ubvec[0],
            'ubvec_minority': ubvec[1],
            'success': False,
            'error': str(e),
            'runtime': time.time() - start_time
        }

    # Analyze districts
    tracts['district'] = partition

    district_stats = []
    for district_id in range(k):
        district_tracts = tracts[tracts['district'] == district_id]

        if len(district_tracts) == 0:
            continue

        d_pop = district_tracts['total_pop'].sum()
        d_minority = district_tracts['minority_vap'].sum()
        d_pct_minority = (d_minority / d_pop * 100) if d_pop > 0 else 0

        district_stats.append({
            'district': district_id,
            'population': d_pop,
            'minority_vap': d_minority,
            'pct_minority': d_pct_minority,
            'is_mm': d_pct_minority >= 50.0
        })

    df_districts = pd.DataFrame(district_stats)

    # Count MM districts
    mm_count = (df_districts['pct_minority'] >= 50.0).sum()
    max_minority = df_districts['pct_minority'].max()

    # Success if achieved target MM count
    success = (mm_count >= target_mm)

    # Calculate edge cut (unweighted)
    edge_cut = 0
    for tract_idx in range(len(tracts)):
        tract_district = partition[tract_idx]
        for neighbor_idx in adjacency_list[tract_idx]:
            neighbor_district = partition[neighbor_idx]
            if tract_district != neighbor_district:
                edge_cut += 1
    edge_cut = edge_cut // 2  # Each edge counted twice

    runtime = time.time() - start_time

    print(f"  Results: {mm_count} MM districts (target={target_mm}), max minority={max_minority:.1f}%")
    print(f"  Success: {success}, Edge cut: {edge_cut}, Runtime: {runtime:.1f}s")

    return {
        'state': state_code,
        'ubvec_pop': ubvec[0],
        'ubvec_minority': ubvec[1],
        'num_mm': mm_count,
        'target_mm': target_mm,
        'max_minority_pct': max_minority,
        'edge_cut': edge_cut,
        'success': success,
        'runtime': runtime
    }


def main():
    """Run all experiments (28 configs × 5 states = 140 total)"""

    print("="*80)
    print("Multi-Constraint Partitioning Experiments - VERSION 2 (P1-3)")
    print("Balanced Design: 28 configs per state = 140 total")
    print("="*80)
    print()

    print(f"States: {len(STATES)}")
    print(f"Configurations per state: {len(UBVEC_CONFIGS)}")
    print(f"Total experiments: {len(STATES) * len(UBVEC_CONFIGS)}")
    print()

    results = []
    total_experiments = len(STATES) * len(UBVEC_CONFIGS)
    experiment_num = 0

    for state_code, state_info in STATES.items():
        print(f"\n{'='*80}")
        print(f"STATE: {state_info['name']} ({state_code})")
        print(f"Districts: {state_info['k']}, Target MM: {state_info['target_mm']}")
        print(f"{'='*80}\n")

        # Load data
        tracts = load_state_data(state_code, state_info['fips'], state_info['name'])

        # Run all configurations for this state
        for config_idx, ubvec in enumerate(UBVEC_CONFIGS):
            experiment_num += 1
            print(f"\n[{experiment_num}/{total_experiments}] Config {config_idx+1}/{len(UBVEC_CONFIGS)}: ubvec={ubvec}")

            result = run_multi_constraint(
                state_code=state_code,
                tracts=tracts,
                k=state_info['k'],
                target_mm=state_info['target_mm'],
                ubvec=ubvec
            )

            results.append(result)

    # Save results
    df_results = pd.DataFrame(results)
    output_file = Path('research/gerry-multi-vs-edge/results/multi_constraint_results_v2.csv')
    output_file.parent.mkdir(parents=True, exist_ok=True)
    df_results.to_csv(output_file, index=False)

    print(f"\n{'='*80}")
    print("SUMMARY")
    print(f"{'='*80}\n")

    # Overall statistics
    total_success = df_results['success'].sum()
    total_configs = len(df_results)
    success_rate = (total_success / total_configs * 100) if total_configs > 0 else 0

    print(f"Total configurations: {total_configs}")
    print(f"Successful configurations: {total_success} ({success_rate:.1f}%)")
    print(f"\nResults saved to: {output_file}")

    # Per-state summary
    print(f"\nPer-state success rates:")
    for state_code in STATES.keys():
        state_results = df_results[df_results['state'] == state_code]
        state_success = state_results['success'].sum()
        state_total = len(state_results)
        state_rate = (state_success / state_total * 100) if state_total > 0 else 0
        print(f"  {state_code}: {state_success}/{state_total} ({state_rate:.1f}%)")

    # Ubvec sensitivity
    print(f"\nSuccess by ubvec (minority tolerance):")
    for ubvec in UBVEC_CONFIGS[:10]:  # Show first 10
        ubvec_results = df_results[df_results['ubvec_minority'] == ubvec[1]]
        ubvec_success = ubvec_results['success'].sum()
        ubvec_total = len(ubvec_results)
        ubvec_rate = (ubvec_success / ubvec_total * 100) if ubvec_total > 0 else 0
        print(f"  ubvec={ubvec[1]:.2f}: {ubvec_success}/{ubvec_total} ({ubvec_rate:.1f}%)")
    print(f"  ... (showing first 10 of {len(UBVEC_CONFIGS)} configs)")

    print(f"\n{'='*80}")
    print("COMPLETE")
    print(f"{'='*80}")


if __name__ == '__main__':
    main()
