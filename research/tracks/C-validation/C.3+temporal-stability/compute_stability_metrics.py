"""
Compute temporal stability metrics for redistricting methods.

Compares 2010 → 2020 redistricting results for both recursive bisection
and n-way partitioning to measure how much districts change over time.

Key metrics:
1. Tract reassignment rate: % of tracts that changed district
2. Population disruption: Population in reassigned tracts
3. District correspondence: How well 2020 districts match 2010 districts
"""

import sys
from pathlib import Path
import pandas as pd
import numpy as np
from collections import Counter

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

# States for analysis
STATES = ['alabama', 'georgia', 'louisiana', 'mississippi', 'south_carolina']

STATE_TO_CODE = {
    'alabama': 'AL',
    'georgia': 'GA',
    'louisiana': 'LA',
    'mississippi': 'MS',
    'south_carolina': 'SC'
}


def load_partition_results(state: str, year: int, method: str):
    """Load partition results for a state/year/method."""
    project_root = Path(__file__).parent.parent.parent
    results_dir = project_root / 'research/gerry-temporal-stability/results'

    filename = f'{state}_{year}_{method}_partition.csv'
    filepath = results_dir / filename

    if not filepath.exists():
        raise FileNotFoundError(f"Missing: {filename}")

    df = pd.read_csv(filepath)
    return df


def compute_district_mapping(df_2010, df_2020):
    """
    Find best mapping from 2010 districts to 2020 districts.

    For each 2010 district, find which 2020 district contains most of its tracts.
    Returns dict: {2010_district: 2020_district}
    """
    mapping = {}

    for district_2010 in sorted(df_2010['district'].unique()):
        # Get tracts in this 2010 district
        tracts_2010 = set(df_2010[df_2010['district'] == district_2010]['GEOID'])

        # Count which 2020 districts these tracts ended up in
        df_2020_subset = df_2020[df_2020['GEOID'].isin(tracts_2010)]

        if len(df_2020_subset) == 0:
            continue

        # Find most common 2020 district
        district_counts = df_2020_subset['district'].value_counts()
        best_match = district_counts.index[0]
        mapping[district_2010] = best_match

    return mapping


def compute_stability_metrics(state: str, method: str):
    """Compute temporal stability metrics for one state and method."""

    # Load results
    df_2010 = load_partition_results(state, 2010, method)
    df_2020 = load_partition_results(state, 2020, method)

    # Find common tracts (GEOIDs that exist in both years)
    geoids_2010 = set(df_2010['GEOID'])
    geoids_2020 = set(df_2020['GEOID'])
    common_geoids = geoids_2010 & geoids_2020

    # Filter to common tracts only
    df_2010_common = df_2010[df_2010['GEOID'].isin(common_geoids)].copy()
    df_2020_common = df_2020[df_2020['GEOID'].isin(common_geoids)].copy()

    # Create merged dataframe with both years
    df_merged = df_2010_common.merge(
        df_2020_common,
        on='GEOID',
        suffixes=('_2010', '_2020')
    )

    # Find best district mapping (2010 → 2020)
    district_mapping = compute_district_mapping(df_2010_common, df_2020_common)

    # Map 2010 districts to their best 2020 matches
    df_merged['district_2010_mapped'] = df_merged['district_2010'].map(district_mapping)

    # Compute metrics
    total_tracts = len(df_merged)
    reassigned_tracts = (df_merged['district_2010_mapped'] != df_merged['district_2020']).sum()
    reassignment_rate = reassigned_tracts / total_tracts if total_tracts > 0 else 0

    # Population disruption
    total_pop = df_merged['total_pop_2010'].sum()
    reassigned_pop = df_merged[
        df_merged['district_2010_mapped'] != df_merged['district_2020']
    ]['total_pop_2010'].sum()
    pop_disruption_rate = reassigned_pop / total_pop if total_pop > 0 else 0

    # Tract coverage
    tract_coverage_2010 = len(common_geoids) / len(geoids_2010) if len(geoids_2010) > 0 else 0
    tract_coverage_2020 = len(common_geoids) / len(geoids_2020) if len(geoids_2020) > 0 else 0

    # Average population disruption per district
    pop_disruption_by_district = df_merged.groupby('district_2010').apply(
        lambda g: g[g['district_2010_mapped'] != g['district_2020']]['total_pop_2010'].sum() / g['total_pop_2010'].sum()
        if g['total_pop_2010'].sum() > 0 else 0,
        include_groups=False
    )
    avg_district_disruption = pop_disruption_by_district.mean()

    return {
        'state': state,
        'method': method,
        'total_tracts_2010': len(geoids_2010),
        'total_tracts_2020': len(geoids_2020),
        'common_tracts': len(common_geoids),
        'tract_coverage_2010': tract_coverage_2010,
        'tract_coverage_2020': tract_coverage_2020,
        'reassigned_tracts': reassigned_tracts,
        'reassignment_rate': reassignment_rate,
        'total_pop': total_pop,
        'reassigned_pop': reassigned_pop,
        'pop_disruption_rate': pop_disruption_rate,
        'avg_district_disruption': avg_district_disruption,
        'num_districts': len(df_2010['district'].unique())
    }


def main():
    """Compute temporal stability metrics for all states and methods."""
    print("="*70)
    print("TEMPORAL STABILITY ANALYSIS: 2010 -> 2020")
    print("="*70)
    print()

    results = []

    # Compare true recursive bisection vs n-way partitioning
    for state in STATES:
        for method in ['true_recursive', 'nway']:
            print(f"Computing metrics: {state} ({method})...")
            try:
                metrics = compute_stability_metrics(state, method)
                results.append(metrics)

                print(f"  Common tracts: {metrics['common_tracts']} "
                      f"({metrics['tract_coverage_2010']*100:.1f}% of 2010)")
                print(f"  Reassignment rate: {metrics['reassignment_rate']*100:.1f}%")
                print(f"  Population disruption: {metrics['pop_disruption_rate']*100:.1f}%")
                print()

            except Exception as e:
                print(f"  ERROR: {e}")
                import traceback
                traceback.print_exc()
                print()

    # Create results dataframe
    df_results = pd.DataFrame(results)

    # Save detailed results
    project_root = Path(__file__).parent.parent.parent
    output_dir = project_root / 'research/gerry-temporal-stability/results'
    output_file = output_dir / 'temporal_stability_metrics.csv'

    df_results.to_csv(output_file, index=False)
    print(f"Saved detailed metrics: {output_file.name}")
    print()

    # Summary comparison
    print("="*70)
    print("SUMMARY: TRUE RECURSIVE BISECTION vs N-WAY PARTITIONING")
    print("="*70)
    print()

    # Average metrics by method
    summary = df_results.groupby('method').agg({
        'reassignment_rate': 'mean',
        'pop_disruption_rate': 'mean',
        'avg_district_disruption': 'mean',
        'tract_coverage_2010': 'mean'
    })

    summary['reassignment_rate_pct'] = summary['reassignment_rate'] * 100
    summary['pop_disruption_pct'] = summary['pop_disruption_rate'] * 100
    summary['avg_district_disruption_pct'] = summary['avg_district_disruption'] * 100
    summary['tract_coverage_pct'] = summary['tract_coverage_2010'] * 100

    print("Average across 5 southern states:")
    print()
    print(f"{'Method':<15} {'Reassignment':<15} {'Pop Disruption':<18} {'Dist Disruption':<18} {'Coverage':<12}")
    print(f"{'':15} {'Rate (%)':<15} {'Rate (%)':<18} {'Rate (%)':<18} {'(%)':<12}")
    print("-"*70)

    for method in ['true_recursive', 'nway']:
        if method in summary.index:
            row = summary.loc[method]
            method_display = 'recursive' if method == 'true_recursive' else 'nway'
            print(f"{method_display:<15} {row['reassignment_rate_pct']:>12.1f}   "
                  f"{row['pop_disruption_pct']:>15.1f}   "
                  f"{row['avg_district_disruption_pct']:>15.1f}   "
                  f"{row['tract_coverage_pct']:>9.1f}")

    print()
    print("="*70)

    # Determine winner
    recursive_disruption = summary.loc['true_recursive', 'pop_disruption_rate']
    nway_disruption = summary.loc['nway', 'pop_disruption_rate']

    if recursive_disruption < nway_disruption:
        improvement = ((nway_disruption - recursive_disruption) / nway_disruption) * 100
        print(f"RECURSIVE BISECTION is {improvement:.1f}% more stable")
    else:
        improvement = ((recursive_disruption - nway_disruption) / recursive_disruption) * 100
        print(f"N-WAY PARTITIONING is {improvement:.1f}% more stable")

    print("="*70)
    print()

    # State-by-state comparison
    print("STATE-BY-STATE BREAKDOWN:")
    print()

    for state in STATES:
        state_code = STATE_TO_CODE[state]
        print(f"{state.upper()} ({state_code}):")

        state_results = df_results[df_results['state'] == state]

        for _, row in state_results.iterrows():
            method_display = 'recursive' if row['method'] == 'true_recursive' else 'nway'
            print(f"  {method_display:<12}: "
                  f"{row['reassignment_rate']*100:>5.1f}% reassignment, "
                  f"{row['pop_disruption_rate']*100:>5.1f}% pop disruption")

        # Which method is better for this state?
        recursive_row = state_results[state_results['method'] == 'true_recursive'].iloc[0]
        nway_row = state_results[state_results['method'] == 'nway'].iloc[0]

        if recursive_row['pop_disruption_rate'] < nway_row['pop_disruption_rate']:
            diff = nway_row['pop_disruption_rate'] - recursive_row['pop_disruption_rate']
            print(f"  -> Recursive is {diff*100:.1f}% more stable")
        else:
            diff = recursive_row['pop_disruption_rate'] - nway_row['pop_disruption_rate']
            print(f"  -> N-way is {diff*100:.1f}% more stable")

        print()


if __name__ == '__main__':
    main()
