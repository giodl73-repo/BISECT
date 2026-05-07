"""
Multi-year validation: Run same experiments on Census 2000, 2010, 2020
to demonstrate robustness across demographic changes
"""

import pandas as pd
import numpy as np
from pathlib import Path
import sys

project_root = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(project_root / 'src'))
sys.path.insert(0, str(project_root))

from apportionment.partition.metis_executable import partition_graph_with_executable
from apportionment.data.adjacency import build_adjacency_graph
import geopandas as gpd

def load_state_data(state_name, year):
    """Load tract data and demographics for a state and year"""
    # Load tracts
    tracts_path = Path(f"outputs/data/{year}/units/{state_name}_{year}_tracts.geojson")
    if not tracts_path.exists():
        return None

    tracts = gpd.read_file(tracts_path)

    # Load demographics
    demo_path = Path(f"outputs/data/{year}/demographics/{state_name}_demographics.csv")
    if not demo_path.exists():
        return None

    demo = pd.read_csv(demo_path)
    tracts = tracts.merge(demo, left_on='GEOID', right_on='geoid', how='left')

    return tracts

def get_district_count(state_name, year):
    """Get district count for state and year"""
    # Import appropriate config
    if year == 2000:
        from scripts.config_2000 import STATE_SEATS
    elif year == 2010:
        from scripts.config_2010 import STATE_SEATS
    else:  # 2020
        from scripts.config_2020 import STATE_SEATS

    return STATE_SEATS[state_name.upper()]

def calculate_target_mm(total_minority_pct, k):
    """Calculate target MM districts based on proportional representation"""
    return int(round(total_minority_pct * k))

def run_multi_constraint_ubvec(tracts, k, target_mm, ubvec_minority):
    """Run multi-constraint with specific ubvec"""
    # Build adjacency
    adj_list = build_adjacency_graph(tracts)

    # 2D weights
    vertex_weights = [[int(row['total_pop']), int(row['minority_vap'])]
                      for _, row in tracts.iterrows()]

    # Target weights
    total_pop = tracts['total_pop'].sum()
    total_minority = tracts['minority_vap'].sum()

    tpwgts = []
    for i in range(k):
        if i < target_mm:
            mm_frac = 0.60 * (total_minority / k) / total_minority
        else:
            rem = k - target_mm
            mm_frac = (total_minority - target_mm * 0.60 * (total_minority / k)) / (rem * total_minority) if rem > 0 else 0
        tpwgts.extend([1/k, mm_frac])

    # Run METIS
    partition = partition_graph_with_executable(
        adj_list,
        vertex_weights,
        nparts=k,
        tpwgts=tpwgts,
        ubvec=[1.005, ubvec_minority],
        niter=100,
        seed=42
    )

    return partition

def run_edge_weighted(tracts, k, weight_factor, threshold):
    """Run edge-weighted with specific parameters"""
    # Build adjacency with edge weights
    adj_list = build_adjacency_graph(tracts, mode='edge_weighted',
                                     minority_threshold=threshold,
                                     weight_factor=weight_factor)

    # 1D weights
    vertex_weights = [int(row['total_pop']) for _, row in tracts.iterrows()]

    # Run METIS
    partition = partition_graph_with_executable(
        adj_list,
        vertex_weights,
        nparts=k,
        ufactor=1.005,
        niter=100,
        seed=42
    )

    return partition

def analyze_partition(tracts, partition, target_mm):
    """Analyze partition results"""
    tracts_copy = tracts.copy()
    tracts_copy['district'] = partition

    # Calculate district stats
    stats = []
    for district in sorted(tracts_copy['district'].unique()):
        dist_tracts = tracts_copy[tracts_copy['district'] == district]
        total_vap = dist_tracts['total_vap'].sum()
        minority_vap = dist_tracts['minority_vap'].sum()
        minority_pct = minority_vap / total_vap if total_vap > 0 else 0

        stats.append({
            'district': district,
            'minority_pct': minority_pct,
            'is_mm': minority_pct >= 0.50
        })

    stats_df = pd.DataFrame(stats)

    mm_count = stats_df['is_mm'].sum()
    max_minority_pct = stats_df['minority_pct'].max()
    success = mm_count >= target_mm

    return {
        'mm_count': mm_count,
        'max_minority_pct': max_minority_pct,
        'success': success
    }

def run_experiments():
    """Run experiments across all years and states"""
    results = []

    states = ['AL', 'GA', 'LA', 'MS', 'SC']
    state_names = {
        'AL': 'alabama',
        'GA': 'georgia',
        'LA': 'louisiana',
        'MS': 'mississippi',
        'SC': 'south_carolina'
    }

    years = [2000, 2010, 2020]

    # Best configurations from 2020 analysis
    best_configs_multi = {
        'AL': 5.0,
        'GA': 1.3,
        'LA': 5.0,
        'MS': 1.3,
        'SC': 2.0
    }

    best_configs_edge = {
        'AL': (5, 0.40),
        'GA': (10, 0.45),
        'LA': (50, 0.55),
        'MS': (50, 0.40),
        'SC': (10, 0.50)
    }

    for year in years:
        print(f"\n{'='*70}")
        print(f"Processing Census {year}")
        print('='*70)

        for state_code in states:
            state_name = state_names[state_code]

            print(f"\n{state_name.title()}...")

            # Load data
            tracts = load_state_data(state_name, year)
            if tracts is None:
                print(f"  Data not found for {state_name} {year}")
                continue

            # Get district count
            k = get_district_count(state_name, year)

            # Calculate demographics
            total_pop = tracts['total_pop'].sum()
            total_vap = tracts['total_vap'].sum()
            total_minority = tracts['minority_vap'].sum()
            minority_pct = total_minority / total_vap

            # Calculate target MM
            target_mm = calculate_target_mm(minority_pct, k)

            print(f"  Districts: {k}, Minority: {minority_pct*100:.1f}%, Target MM: {target_mm}")

            # Multi-constraint
            ubvec = best_configs_multi[state_code]
            print(f"  [1/2] Multi-constraint (ubvec={ubvec})...")
            partition_multi = run_multi_constraint_ubvec(tracts, k, target_mm, ubvec)
            result_multi = analyze_partition(tracts, partition_multi, target_mm)

            results.append({
                'year': year,
                'state': state_code,
                'method': 'multi',
                'config': f'ubvec={ubvec}',
                'k': k,
                'total_minority_pct': minority_pct,
                'target_mm': target_mm,
                'mm_count': result_multi['mm_count'],
                'max_minority_pct': result_multi['max_minority_pct'],
                'success': result_multi['success']
            })

            print(f"    MM: {result_multi['mm_count']}/{target_mm}, Max: {result_multi['max_minority_pct']*100:.1f}%")

            # Edge-weighted
            weight_factor, threshold = best_configs_edge[state_code]
            print(f"  [2/2] Edge-weighted ({weight_factor}x @ {threshold*100:.0f}%)...")
            partition_edge = run_edge_weighted(tracts, k, weight_factor, threshold)
            result_edge = analyze_partition(tracts, partition_edge, target_mm)

            results.append({
                'year': year,
                'state': state_code,
                'method': 'edge',
                'config': f'{weight_factor}x@{threshold*100:.0f}%',
                'k': k,
                'total_minority_pct': minority_pct,
                'target_mm': target_mm,
                'mm_count': result_edge['mm_count'],
                'max_minority_pct': result_edge['max_minority_pct'],
                'success': result_edge['success']
            })

            print(f"    MM: {result_edge['mm_count']}/{target_mm}, Max: {result_edge['max_minority_pct']*100:.1f}%")

    return pd.DataFrame(results)

def main():
    print("\n" + "="*70)
    print("Multi-Year Validation: Census 2000, 2010, 2020")
    print("="*70)

    # Run experiments
    results_df = run_experiments()

    # Save results
    output_path = Path("research/gerry-multi-vs-edge/results/multi_year_validation.csv")
    results_df.to_csv(output_path, index=False)
    print(f"\n[OK] Saved: {output_path}")

    # Summary statistics
    print("\n" + "="*70)
    print("SUMMARY: Success Rate by Year and Method")
    print("="*70)

    summary = results_df.groupby(['year', 'method'])['success'].agg(['sum', 'count', 'mean'])
    summary.columns = ['successes', 'total', 'success_rate']
    summary['success_rate'] = summary['success_rate'] * 100
    print(summary.to_string())

    # Overall by method
    print("\n" + "="*70)
    print("OVERALL: Across All Years")
    print("="*70)

    overall = results_df.groupby('method')['success'].agg(['sum', 'count', 'mean'])
    overall.columns = ['successes', 'total', 'success_rate']
    overall['success_rate'] = overall['success_rate'] * 100
    print(overall.to_string())

    # State consistency
    print("\n" + "="*70)
    print("STATE CONSISTENCY: Success Across All Years")
    print("="*70)

    state_consistency = results_df.pivot_table(
        index='state',
        columns=['year', 'method'],
        values='success',
        aggfunc='sum'
    )
    print(state_consistency.to_string())

    print("\n" + "="*70)
    print("DONE!")
    print("="*70)

if __name__ == '__main__':
    main()
