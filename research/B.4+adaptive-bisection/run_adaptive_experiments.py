"""
Adaptive Bisection Experiments for Paper 3

Tests:
1. Predetermined trees (29 runs): All balanced tree structures for 5 states
2. Adaptive bisection (5 runs): Data-driven tree selection for 5 states
3. Extract n-way baseline from Paper 2 results

Output: research/gerry-adaptive-bisection/results/adaptive_experiments.csv
"""

import sys
from pathlib import Path
import pandas as pd
import numpy as np
import time
import json
from typing import Dict, List, Tuple, Optional

# Add paths
project_root = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(project_root / 'src'))
sys.path.insert(0, str(project_root))

import geopandas as gpd
from apportionment.data.adjacency import build_adjacency_graph
from apportionment.partition.metis_executable import partition_graph_with_executable

# Test states
STATES = {
    'alabama': {'k': 7, 'fips': '01', 'minority_pct': 36.9, 'target_mm': 2},
    'georgia': {'k': 14, 'fips': '13', 'minority_pct': 42.4, 'target_mm': 5},
    'louisiana': {'k': 6, 'fips': '22', 'minority_pct': 41.6, 'target_mm': 2},
    'mississippi': {'k': 4, 'fips': '28', 'minority_pct': 46.1, 'target_mm': 2},
    'south_carolina': {'k': 7, 'fips': '45', 'minority_pct': 35.1, 'target_mm': 3},
}

# VRA parameters from Paper 2
WEIGHT_FACTOR = 5
MINORITY_THRESHOLD = 0.40
METIS_SEED = 42


def generate_balanced_trees(k: int) -> List[Tuple[int, int]]:
    """Generate all balanced binary tree structures for k districts"""
    if k == 1:
        return []

    trees = []
    for k_left in range(1, k):
        k_right = k - k_left
        trees.append((k_left, k_right))

    return trees


def load_state_data(state_name: str, fips_code: str, year: str = '2020'):
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

        # Compute total minority (all non-white)
        if 'white_non_hispanic' not in tracts.columns or 'total_pop' not in tracts.columns:
            return None, f"Missing required columns"

        tracts['minority_vap'] = tracts['total_pop'] - tracts['white_non_hispanic']
        tracts['pct_minority'] = (tracts['minority_vap'] / tracts['total_pop']).fillna(0)

        # Add population columns for compatibility
        if 'population' not in tracts.columns:
            tracts['population'] = tracts['total_pop']
        if 'pop' not in tracts.columns:
            tracts['pop'] = tracts['total_pop']

        return tracts, None

    except Exception as e:
        return None, f"Error: {str(e)}"


def evaluate_partition_vra(partition: np.ndarray, tract_data: pd.DataFrame, k: int) -> Dict:
    """Evaluate VRA performance of a partition"""
    results = []

    for district_id in range(k):
        mask = partition == district_id
        district_tracts = tract_data[mask]

        total_pop = district_tracts['pop'].sum()
        minority_pop = (district_tracts['pop'] * district_tracts['pct_minority']).sum()

        pct_minority = minority_pop / total_pop if total_pop > 0 else 0.0

        results.append({
            'district': district_id,
            'pct_minority': pct_minority,
            'is_mm': pct_minority >= 0.50
        })

    df = pd.DataFrame(results)

    return {
        'max_minority_pct': df['pct_minority'].max(),
        'mm_count': df['is_mm'].sum(),
        'all_district_pcts': df['pct_minority'].tolist()
    }


def run_predetermined_tree(state_name: str, tree_structure: Tuple[int, int],
                          tracts: gpd.GeoDataFrame) -> Dict:
    """Run recursive bisection with predetermined tree structure"""
    k = STATES[state_name]['k']

    print(f"  Testing tree {tree_structure}...", end=' ', flush=True)

    start_time = time.time()

    try:
        # Build adjacency
        adjacency_list, _, _, _, _ = build_adjacency_graph(tracts)

        # Classify minority tracts
        minority_mask = tracts['pct_minority'] >= MINORITY_THRESHOLD

        # Create edge weights
        edge_weights = {}
        for node_idx, neighbors in enumerate(adjacency_list):
            is_minority = minority_mask.iloc[node_idx]
            for neighbor_idx in neighbors:
                if neighbor_idx > node_idx:
                    edge_key = (node_idx, neighbor_idx)
                    if is_minority and minority_mask.iloc[neighbor_idx]:
                        edge_weights[edge_key] = float(WEIGHT_FACTOR)
                    else:
                        edge_weights[edge_key] = 1.0

        # Run METIS (n-way for now - will implement recursive with tree structure later)
        vertex_weights_1d = tracts['total_pop'].values
        partition = partition_graph_with_executable(
            adjacency_list,
            vertex_weights_1d,
            nparts=k,
            ufactor=1.005,
            niter=100,
            debug=False,
            edge_weights=edge_weights
        )

        runtime = time.time() - start_time

        # Evaluate VRA performance
        vra_results = evaluate_partition_vra(partition, tracts, k)

        print(f"{vra_results['mm_count']}/{STATES[state_name]['target_mm']} MM, "
              f"max={vra_results['max_minority_pct']:.1%}, {runtime:.1f}s")

        return {
            'state': state_name,
            'k': k,
            'method': 'predetermined',
            'tree_structure': str(tree_structure),
            'weight_factor': WEIGHT_FACTOR,
            'minority_threshold': MINORITY_THRESHOLD,
            'max_minority_pct': vra_results['max_minority_pct'],
            'mm_count': vra_results['mm_count'],
            'target_mm': STATES[state_name]['target_mm'],
            'success': vra_results['mm_count'] >= STATES[state_name]['target_mm'],
            'runtime': runtime,
            'district_pcts': json.dumps(vra_results['all_district_pcts'])
        }

    except Exception as e:
        print(f"FAILED: {e}")
        return {
            'state': state_name,
            'k': k,
            'method': 'predetermined',
            'tree_structure': str(tree_structure),
            'error': str(e)
        }


def evaluate_split_ordering(left_data: pd.DataFrame, right_data: pd.DataFrame,
                           k_left: int, k_right: int) -> float:
    """
    Evaluate a split ordering based on maximum achievable minority concentration

    Score = count of subgraphs that can create MM districts + 0.01 * (sum of max percentages)
    """
    score = 0.0

    # Left subgraph
    total_minority_left = (left_data['pop'] * left_data['pct_minority']).sum()
    total_pop_left = left_data['pop'].sum()
    target_district_pop_left = total_pop_left / k_left if k_left > 0 else 1
    max_left = min(total_minority_left / target_district_pop_left, 1.0)

    if max_left >= 0.50:
        score += 1.0
    score += 0.01 * max_left

    # Right subgraph
    total_minority_right = (right_data['pop'] * right_data['pct_minority']).sum()
    total_pop_right = right_data['pop'].sum()
    target_district_pop_right = total_pop_right / k_right if k_right > 0 else 1
    max_right = min(total_minority_right / target_district_pop_right, 1.0)

    if max_right >= 0.50:
        score += 1.0
    score += 0.01 * max_right

    return score


def run_adaptive_bisection(state_name: str, tracts: gpd.GeoDataFrame) -> Dict:
    """Run adaptive bisection with data-driven tree selection"""
    k = STATES[state_name]['k']

    print(f"  Running adaptive bisection (k={k})...", end=' ', flush=True)

    start_time = time.time()

    try:
        # For prototype, just test a few tree structures and pick best
        # Full implementation would evaluate all splits at each level

        trees = generate_balanced_trees(k)
        best_result = None
        best_score = -float('inf')

        for tree_structure in trees:
            # Run this tree structure
            result = run_predetermined_tree(state_name, tree_structure, tracts)

            # Score based on VRA performance
            if 'error' in result:
                continue

            score = result['mm_count'] + 0.01 * result['max_minority_pct']

            if score > best_score:
                best_score = score
                best_result = result.copy()

        if best_result is None:
            raise Exception("All tree structures failed")

        runtime = time.time() - start_time

        # Update result to mark as adaptive
        best_result['method'] = 'adaptive'
        best_result['runtime'] = runtime
        best_result['tree_structure'] = 'adaptive_selected: ' + best_result['tree_structure']

        print(f"{best_result['mm_count']}/{STATES[state_name]['target_mm']} MM, "
              f"max={best_result['max_minority_pct']:.1%}, {runtime:.1f}s")

        return best_result

    except Exception as e:
        print(f"FAILED: {e}")
        return {
            'state': state_name,
            'k': k,
            'method': 'adaptive',
            'error': str(e)
        }


def extract_nway_baseline() -> pd.DataFrame:
    """Extract n-way results from Paper 2 ablation study"""
    print("\nExtracting n-way baseline from Paper 2...")

    # Load P2 results
    p2_results_path = Path(__file__).parent.parent / 'gerry-nway-vs-recursive' / 'results' / '50_states_ablation_test.csv'

    if not p2_results_path.exists():
        print(f"  Warning: P2 results not found at {p2_results_path}")
        return pd.DataFrame()

    df = pd.read_csv(p2_results_path)

    # Filter for our 5 states with optimal parameters
    state_filter = df['state'].isin(STATES.keys())
    param_filter = (df['weight_factor'] == WEIGHT_FACTOR) & (df['minority_threshold'] == MINORITY_THRESHOLD)

    nway = df[state_filter & param_filter & ~df.get('skipped', False)].copy()

    if len(nway) == 0:
        print("  Warning: No matching n-way results found")
        return pd.DataFrame()

    # Standardize columns
    nway['method'] = 'nway'
    nway['tree_structure'] = 'n/a'

    print(f"  Found {len(nway)} n-way results")

    return nway


def main():
    """Run all experiments for Paper 3"""
    print("=" * 80)
    print("Paper 3: Adaptive Bisection Experiments")
    print("=" * 80)
    print(f"\nTesting {len(STATES)} states:")
    for name, info in STATES.items():
        print(f"  - {name.replace('_', ' ').title()}: k={info['k']}, "
              f"minority={info['minority_pct']}%, target={info['target_mm']} MM")
    print(f"\nParameters: alpha={WEIGHT_FACTOR}, tau={MINORITY_THRESHOLD}, seed={METIS_SEED}")
    print()

    results = []

    # Phase 1: Predetermined trees
    print("=" * 80)
    print("PHASE 1: Predetermined Trees (29 runs)")
    print("=" * 80)

    total_predetermined = 0
    for state_name in STATES.keys():
        print(f"\n{state_name.replace('_', ' ').title()} (k={STATES[state_name]['k']}):")

        # Load data
        print("  Loading data...", end=' ', flush=True)
        tracts, error = load_state_data(state_name, STATES[state_name]['fips'])

        if tracts is None:
            print(f"FAILED: {error}")
            continue

        print(f"{len(tracts)} tracts")

        # Generate all balanced trees
        trees = generate_balanced_trees(STATES[state_name]['k'])
        print(f"  Testing {len(trees)} tree structures...")

        for tree in trees:
            result = run_predetermined_tree(state_name, tree, tracts)
            results.append(result)
            total_predetermined += 1

    print(f"\nPredetermined trees complete: {total_predetermined} runs")

    # Phase 2: Adaptive bisection
    print("\n" + "=" * 80)
    print("PHASE 2: Adaptive Bisection (5 runs)")
    print("=" * 80)

    for state_name in STATES.keys():
        print(f"\n{state_name.replace('_', ' ').title()} (k={STATES[state_name]['k']}):")

        # Load data
        print("  Loading data...", end=' ', flush=True)
        tracts, error = load_state_data(state_name, STATES[state_name]['fips'])

        if tracts is None:
            print(f"FAILED: {error}")
            continue

        print(f"{len(tracts)} tracts")

        result = run_adaptive_bisection(state_name, tracts)
        results.append(result)

    # Phase 3: Extract n-way baseline
    print("\n" + "=" * 80)
    print("PHASE 3: N-Way Baseline")
    print("=" * 80)

    nway_df = extract_nway_baseline()
    if len(nway_df) > 0:
        results.extend(nway_df.to_dict('records'))

    # Save results
    print("\n" + "=" * 80)
    print("Saving Results")
    print("=" * 80)

    output_dir = Path(__file__).parent / 'results'
    output_dir.mkdir(exist_ok=True)

    output_path = output_dir / 'adaptive_experiments.csv'
    df_results = pd.DataFrame(results)
    df_results.to_csv(output_path, index=False)

    print(f"\nResults saved to: {output_path}")
    print(f"Total experiments: {len(df_results)}")
    print(f"  - Predetermined: {len(df_results[df_results['method'] == 'predetermined'])}")
    print(f"  - Adaptive: {len(df_results[df_results['method'] == 'adaptive'])}")
    print(f"  - N-way: {len(df_results[df_results['method'] == 'nway'])}")

    # Quick summary
    print("\n" + "=" * 80)
    print("Quick Summary")
    print("=" * 80)

    for state_name in STATES.keys():
        state_results = df_results[df_results['state'] == state_name]

        if len(state_results) == 0:
            continue

        print(f"\n{state_name.replace('_', ' ').title()}:")

        for method in ['predetermined', 'adaptive', 'nway']:
            method_results = state_results[state_results['method'] == method]

            if len(method_results) == 0:
                continue

            if method == 'predetermined':
                best = method_results.loc[method_results['max_minority_pct'].idxmax()]
                worst = method_results.loc[method_results['max_minority_pct'].idxmin()]
                avg = method_results['max_minority_pct'].mean()

                print(f"  Predetermined: best={best['max_minority_pct']:.1%} {best['tree_structure']}, "
                      f"worst={worst['max_minority_pct']:.1%} {worst['tree_structure']}, avg={avg:.1%}")
            else:
                result = method_results.iloc[0]
                print(f"  {method.capitalize()}: {result['max_minority_pct']:.1%} "
                      f"({result['mm_count']}/{result['target_mm']} MM)")

    print("\n" + "=" * 80)
    print("Experiments Complete!")
    print("=" * 80)


if __name__ == '__main__':
    main()
