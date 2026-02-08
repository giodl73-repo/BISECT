"""
Comprehensive 50-state RECURSIVE BISECTION ablation with edge-weighting.

Mirrors test_50_states_ablation.py but uses recursive bisection instead of n-way.
Tests same parameters for fair comparison.

Total: 50 states × 5 weights × 4 thresholds = 1,000 runs
Expected runtime: ~2 hours
"""

import sys
from pathlib import Path
import pandas as pd
import numpy as np
import time
from datetime import datetime

project_root = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(project_root / 'src'))
sys.path.insert(0, str(project_root))

import geopandas as gpd
from apportionment.data.adjacency import build_adjacency_graph
from apportionment.partition.metis_executable import partition_graph_with_executable

# All 50 states - same as n-way version
STATES = {
    'alabama': {'fips': '01', 'k': 7},
    'alaska': {'fips': '02', 'k': 1},
    'arizona': {'fips': '04', 'k': 9},
    'arkansas': {'fips': '05', 'k': 4},
    'california': {'fips': '06', 'k': 52},
    'colorado': {'fips': '08', 'k': 8},
    'connecticut': {'fips': '09', 'k': 5},
    'delaware': {'fips': '10', 'k': 1},
    'florida': {'fips': '12', 'k': 28},
    'georgia': {'fips': '13', 'k': 14},
    'hawaii': {'fips': '15', 'k': 2},
    'idaho': {'fips': '16', 'k': 2},
    'illinois': {'fips': '17', 'k': 17},
    'indiana': {'fips': '18', 'k': 9},
    'iowa': {'fips': '19', 'k': 4},
    'kansas': {'fips': '20', 'k': 4},
    'kentucky': {'fips': '21', 'k': 6},
    'louisiana': {'fips': '22', 'k': 6},
    'maine': {'fips': '23', 'k': 2},
    'maryland': {'fips': '24', 'k': 8},
    'massachusetts': {'fips': '25', 'k': 9},
    'michigan': {'fips': '26', 'k': 13},
    'minnesota': {'fips': '27', 'k': 8},
    'mississippi': {'fips': '28', 'k': 4},
    'missouri': {'fips': '29', 'k': 8},
    'montana': {'fips': '30', 'k': 2},
    'nebraska': {'fips': '31', 'k': 3},
    'nevada': {'fips': '32', 'k': 4},
    'new_hampshire': {'fips': '33', 'k': 2},
    'new_jersey': {'fips': '34', 'k': 12},
    'new_mexico': {'fips': '35', 'k': 3},
    'new_york': {'fips': '36', 'k': 26},
    'north_carolina': {'fips': '37', 'k': 14},
    'north_dakota': {'fips': '38', 'k': 1},
    'ohio': {'fips': '39', 'k': 15},
    'oklahoma': {'fips': '40', 'k': 5},
    'oregon': {'fips': '41', 'k': 6},
    'pennsylvania': {'fips': '42', 'k': 17},
    'rhode_island': {'fips': '44', 'k': 2},
    'south_carolina': {'fips': '45', 'k': 7},
    'south_dakota': {'fips': '46', 'k': 1},
    'tennessee': {'fips': '47', 'k': 9},
    'texas': {'fips': '48', 'k': 38},
    'utah': {'fips': '49', 'k': 4},
    'vermont': {'fips': '50', 'k': 1},
    'virginia': {'fips': '51', 'k': 11},
    'washington': {'fips': '53', 'k': 10},
    'west_virginia': {'fips': '54', 'k': 2},
    'wisconsin': {'fips': '55', 'k': 8},
    'wyoming': {'fips': '56', 'k': 1},
}


def load_state_data(state_name, fips_code, year='2020'):
    """Load tracts and demographics for a state"""
    try:
        data_dir = project_root / 'data' / year
        tiger_dir = data_dir / 'tiger' / 'tracts' / f'tl_{year}_{fips_code}_tract'
        demographics_file = data_dir / 'demographics' / f'{state_name}_demographics_{year}.csv'

        # Load tracts
        shapefiles = list(tiger_dir.glob('*.shp'))
        if not shapefiles:
            return None, f"No shapefile found"

        tracts = gpd.read_file(shapefiles[0])

        # Load demographics
        if not demographics_file.exists():
            return None, f"Demographics file not found"

        demographics = pd.read_csv(demographics_file)

        # Merge
        tracts['GEOID'] = tracts['GEOID'].astype(str)
        demographics['GEOID'] = demographics['GEOID'].astype(str).str.zfill(11)
        tracts = tracts.merge(demographics, on='GEOID', how='inner')

        if len(tracts) == 0:
            return None, f"No tracts after merge"

        # Compute total minority
        if 'white_non_hispanic' not in tracts.columns or 'total_pop' not in tracts.columns:
            return None, f"Missing required columns"

        tracts['minority_vap'] = tracts['total_pop'] - tracts['white_non_hispanic']
        tracts['pct_minority'] = (tracts['minority_vap'] / tracts['total_pop']).fillna(0)

        # Add population column
        if 'population' not in tracts.columns:
            tracts['population'] = tracts['total_pop']

        return tracts, None

    except Exception as e:
        return None, f"Error: {str(e)}"


def generate_balanced_tree(k):
    """Generate balanced binary tree split for k districts"""
    if k <= 1:
        return None
    # Simple balanced split: divide k as evenly as possible
    left = k // 2
    right = k - left
    return (left, right)


def recursive_bisection_with_edge_weighting(tracts, k, minority_threshold, weight_factor, target_mm):
    """
    Recursive bisection with edge-weighting.

    At each split, creates edge weights and partitions into 2 groups,
    then recursively partitions each group.
    """
    if k == 1:
        # Base case: all tracts in one district
        return np.zeros(len(tracts), dtype=int)

    # Get tree structure (balanced split)
    tree = generate_balanced_tree(k)
    if tree is None:
        return np.zeros(len(tracts), dtype=int)

    left_k, right_k = tree

    # Build adjacency for current set of tracts
    adjacency_list, _, _, _, _ = build_adjacency_graph(tracts)

    # Classify minority tracts
    minority_mask = tracts['pct_minority'] >= minority_threshold

    # Create edge weights for binary split
    edge_weights = {}
    for node_idx, neighbors in enumerate(adjacency_list):
        is_minority = minority_mask.iloc[node_idx]
        for neighbor_idx in neighbors:
            if neighbor_idx > node_idx:
                edge_key = (node_idx, neighbor_idx)
                if is_minority and minority_mask.iloc[neighbor_idx]:
                    edge_weights[edge_key] = weight_factor
                else:
                    edge_weights[edge_key] = 1.0

    # Binary split
    vertex_weights_1d = tracts['total_pop'].values
    binary_partition = partition_graph_with_executable(
        adjacency_list,
        vertex_weights_1d,
        nparts=2,
        ufactor=1.005,
        niter=100,
        debug=False,
        edge_weights=edge_weights
    )

    # Split tracts into left and right
    left_mask = (binary_partition == 0)
    right_mask = (binary_partition == 1)

    left_tracts = tracts[left_mask].reset_index(drop=True)
    right_tracts = tracts[right_mask].reset_index(drop=True)

    # Distribute MM targets (simple: proportional to k)
    left_target_mm = int(target_mm * left_k / k)
    right_target_mm = target_mm - left_target_mm

    # Recursively partition each side
    if left_k == 1:
        left_assignments = np.zeros(len(left_tracts), dtype=int)
    else:
        left_assignments = recursive_bisection_with_edge_weighting(
            left_tracts, left_k, minority_threshold, weight_factor, left_target_mm
        )

    if right_k == 1:
        right_assignments = np.zeros(len(right_tracts), dtype=int)
    else:
        right_assignments = recursive_bisection_with_edge_weighting(
            right_tracts, right_k, minority_threshold, weight_factor, right_target_mm
        )

    # Combine assignments (right side gets offset by left_k)
    final_assignments = np.zeros(len(tracts), dtype=int)
    final_assignments[left_mask] = left_assignments
    final_assignments[right_mask] = right_assignments + left_k

    return final_assignments


def analyze_partition(partition, tracts, target_mm):
    """Analyze partition results"""
    tracts = tracts.copy()
    tracts['district'] = partition

    # District stats
    district_stats = tracts.groupby('district').agg({
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
        'success': success
    }


def test_single_run(state_name, config, weight_factor, minority_threshold):
    """Test a single configuration with recursive bisection"""
    # Load data
    tracts, error = load_state_data(state_name, config['fips'])
    if tracts is None:
        return {
            'state': state_name,
            'weight_factor': weight_factor,
            'minority_threshold': minority_threshold,
            'error': error,
            'skipped': True
        }

    # Skip single-district states
    if config['k'] == 1:
        return {
            'state': state_name,
            'weight_factor': weight_factor,
            'minority_threshold': minority_threshold,
            'skipped': True,
            'error': 'Single-district state'
        }

    # Compute state-level minority percentage
    state_minority_pct = tracts['minority_vap'].sum() / tracts['total_pop'].sum()

    # Estimate target MM districts (proportional)
    target_mm = int(state_minority_pct * config['k'])

    try:
        # Run recursive bisection
        start_time = time.time()
        partition = recursive_bisection_with_edge_weighting(
            tracts, config['k'], minority_threshold, weight_factor, target_mm
        )
        runtime = time.time() - start_time

        # Analyze
        analysis = analyze_partition(partition, tracts, target_mm)

        return {
            'state': state_name,
            'method': 'recursive_bisection',
            'k': config['k'],
            'target_mm': target_mm,
            'state_minority_pct': state_minority_pct,
            'weight_factor': weight_factor,
            'minority_threshold': minority_threshold,
            'runtime': runtime,
            'skipped': False,
            **analysis
        }

    except Exception as e:
        return {
            'state': state_name,
            'weight_factor': weight_factor,
            'minority_threshold': minority_threshold,
            'error': str(e),
            'skipped': True
        }


def main():
    """Run comprehensive 50-state RECURSIVE BISECTION ablation"""
    # Parameters - SAME as n-way for fair comparison
    weight_factors = [1, 5, 10, 50, 100]
    minority_thresholds = [0.40, 0.45, 0.50, 0.55]

    # ALL 50 states
    test_states = list(STATES.keys())

    print(f"{'='*70}")
    print(f"50-STATE RECURSIVE BISECTION ABLATION STUDY")
    print(f"Total minority (all non-white coalition) + edge-weighting")
    print(f"{'='*70}")
    print(f"Parameters:")
    print(f"  Weight factors: {weight_factors}")
    print(f"  Thresholds: {minority_thresholds}")
    print(f"  Total runs: {len(test_states) * len(weight_factors) * len(minority_thresholds)}")
    print(f"{'='*70}\n")

    results = []
    run_count = 0
    total_runs = len(test_states) * len(weight_factors) * len(minority_thresholds)

    start_time = time.time()

    for state_name in test_states:
        config = STATES[state_name]

        print(f"\n{state_name.upper()} (k={config['k']} districts)")
        print("-" * 70)

        for weight_factor in weight_factors:
            for threshold in minority_thresholds:
                run_count += 1
                print(f"  [{run_count}/{total_runs}] weight={weight_factor:4d}x, threshold={threshold:.2f}...", end=" ")

                result = test_single_run(state_name, config, weight_factor, threshold)
                results.append(result)

                if result.get('skipped'):
                    print(f"SKIPPED ({result.get('error', 'unknown')})")
                else:
                    status = "SUCCESS" if result['success'] else "FAIL"
                    print(f"{result['mm_count']}/{result['target_mm']} MM ({status}) "
                          f"max={result['max_minority_pct']:.1%} {result['runtime']:.1f}s")

    elapsed = time.time() - start_time

    # Save results
    results_df = pd.DataFrame(results)
    output_path = project_root / 'research' / 'gerry-nway-vs-recursive' / 'results'
    output_path.mkdir(parents=True, exist_ok=True)
    output_file = output_path / '50_states_recursive_ablation.csv'
    results_df.to_csv(output_file, index=False)

    print(f"\n{'='*70}")
    print(f"RECURSIVE BISECTION ABLATION COMPLETE")
    print(f"{'='*70}")
    print(f"Results saved to: {output_file}")
    print(f"Total runs: {run_count}")
    print(f"Elapsed time: {elapsed:.1f}s ({elapsed/60:.1f} minutes)")

    # Summary statistics
    if not results_df.empty and not results_df['skipped'].all():
        valid_results = results_df[~results_df['skipped']]

        print(f"\nOVERALL SUMMARY:")
        print(f"  States processed: {len(valid_results['state'].unique())}")
        print(f"  Total valid runs: {len(valid_results)}")
        print(f"  Successful runs: {valid_results['success'].sum()} ({valid_results['success'].mean()*100:.1f}%)")

        # Best configuration by success rate
        config_success = valid_results.groupby(['weight_factor', 'minority_threshold'])['success'].agg(['sum', 'count', 'mean'])
        config_success = config_success.sort_values('mean', ascending=False)

        print(f"\n  Top 3 configurations by success rate:")
        for idx, (config, row) in enumerate(config_success.head(3).iterrows(), 1):
            weight, threshold = config
            print(f"    {idx}. weight={weight}x, threshold={threshold:.2f}: "
                  f"{row['sum']:.0f}/{row['count']:.0f} states ({row['mean']*100:.1f}%)")

        print(f"\n  Average runtime per run: {valid_results['runtime'].mean():.1f}s")


if __name__ == '__main__':
    main()
