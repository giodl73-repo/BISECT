"""
Run 2010 census redistricting for temporal stability analysis.

Simplified version using existing pipeline components.
"""

import sys
from pathlib import Path
import pandas as pd
import numpy as np
from datetime import datetime
from scipy import sparse

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent))

from src.apportionment.partition.recursive_bisection import RecursiveBisection
from src.apportionment.partition.metis_wrapper import partition_graph

# Helper to load adjacency matrix
def load_adjacency_matrix(file_path):
    """Load adjacency matrix from .npz file."""
    return sparse.load_npz(file_path)

# States for temporal stability analysis
STATES = ['alabama', 'georgia', 'louisiana', 'mississippi', 'south_carolina']

# State name to FIPS code mapping
STATE_TO_FIPS = {
    'alabama': '01',
    'georgia': '13',
    'louisiana': '22',
    'mississippi': '28',
    'south_carolina': '45'
}

# District counts for 2010 (from config_2010.py)
STATE_DISTRICTS = {
    'alabama': 7,
    'georgia': 14,
    'louisiana': 6,
    'mississippi': 4,
    'south_carolina': 7
}

# Edge weighting parameters (from VRA papers)
EDGE_WEIGHT_FACTOR = 5
MINORITY_THRESHOLD = 0.40


def load_2010_data(state: str):
    """Load 2010 census tracts with demographics and adjacency."""
    print(f"Loading 2010 data for {state}...")

    project_root = Path(__file__).parent.parent.parent.parent
    fips = STATE_TO_FIPS[state]

    # Load demographics
    demographics_file = project_root / f'data/2010/demographics/{state}_demographics_2010.csv'
    demographics = pd.read_csv(demographics_file)
    print(f"  Loaded {len(demographics)} tracts from demographics")

    # Load adjacency
    adjacency_file = project_root / f'outputs/data/2010/adjacency/{state}_adjacency.npz'
    adjacency = load_adjacency_matrix(adjacency_file)
    print(f"  Loaded adjacency: {adjacency.shape}")

    return demographics, adjacency


def compute_edge_weights(demographics: pd.DataFrame, adjacency, weight_factor: float, threshold: float):
    """Compute edge weights for minority concentration."""
    # Calculate minority percentage (total minority coalition)
    # 2010 demographics use: hispanic, black_non_hispanic, asian_non_hispanic, other
    demographics['minority_pct'] = (
        demographics['hispanic'] + demographics['black_non_hispanic'] +
        demographics['asian_non_hispanic'] + demographics.get('other', 0)
    ) / demographics['total_pop']

    # Convert adjacency to edge weight dict
    adjacency_coo = adjacency.tocoo()
    edge_weights = {}

    for i, j in zip(adjacency_coo.row, adjacency_coo.col):
        if i < j:  # Only process each edge once
            # Both tracts above threshold? Apply weight
            if (demographics.iloc[i]['minority_pct'] >= threshold and
                demographics.iloc[j]['minority_pct'] >= threshold):
                edge_weights[(i, j)] = weight_factor
                edge_weights[(j, i)] = weight_factor
            else:
                edge_weights[(i, j)] = 1.0
                edge_weights[(j, i)] = 1.0

    print(f"  Computed edge weights: {len(edge_weights)} edges")
    return edge_weights


def run_nway_partition(state: str, demographics: pd.DataFrame, adjacency, n_districts: int):
    """Run n-way partitioning with edge weighting."""
    print(f"Running n-way partitioning for {state} (2010)...")

    # Compute edge weights
    edge_weights = compute_edge_weights(demographics, adjacency, EDGE_WEIGHT_FACTOR, MINORITY_THRESHOLD)

    # Extract populations
    populations = demographics['total_pop'].values.astype(np.int32)

    # Convert adjacency to adjacency list format for partition_graph
    adjacency_list = [[] for _ in range(len(demographics))]
    adjacency_coo = adjacency.tocoo()
    for i, j in zip(adjacency_coo.row, adjacency_coo.col):
        adjacency_list[i].append(j)

    # Run METIS n-way partition
    partition = partition_graph(
        adjacency=adjacency_list,
        nparts=n_districts,
        vertex_weights=populations,
        edge_weights=edge_weights,
        recursive=False
    )

    print(f"  Partitioned into {len(set(partition))} districts")
    return partition


def run_recursive_partition(state: str, demographics: pd.DataFrame, adjacency, n_districts: int):
    """Run recursive bisection with edge weighting."""
    print(f"Running recursive bisection for {state} (2010)...")

    # Compute edge weights
    edge_weights = compute_edge_weights(demographics, adjacency, EDGE_WEIGHT_FACTOR, MINORITY_THRESHOLD)

    # Extract populations
    populations = demographics['total_pop'].values.astype(np.int32)

    # Convert adjacency to adjacency list format
    adjacency_list = [[] for _ in range(len(demographics))]
    adjacency_coo = adjacency.tocoo()
    for i, j in zip(adjacency_coo.row, adjacency_coo.col):
        adjacency_list[i].append(j)

    # Run recursive bisection
    rb = RecursiveBisection(
        adjacency=adjacency_list,
        vertex_weights=populations,
        num_districts=n_districts,
        edge_weights=edge_weights
    )

    partition_dict = rb.partition()
    partition = [partition_dict[i] for i in range(len(demographics))]

    print(f"  Partitioned into {len(set(partition))} districts")
    return partition


def save_partition_results(state: str, year: int, method: str, partition, demographics: pd.DataFrame):
    """Save partition results to CSV."""
    project_root = Path(__file__).parent.parent.parent.parent
    output_dir = project_root / 'research/gerry-temporal-stability/results'
    output_dir.mkdir(parents=True, exist_ok=True)

    # Create results dataframe
    results = pd.DataFrame({
        'GEOID': demographics['GEOID'],
        'district': partition,
        'total_pop': demographics['total_pop'],
        'minority_pop': (
            demographics['hispanic'] + demographics['black_non_hispanic'] +
            demographics['asian_non_hispanic'] + demographics.get('other', 0)
        )
    })

    # Add metadata
    results['method'] = method
    results['year'] = year
    results['state'] = state

    # Save
    output_file = output_dir / f'{state}_{year}_{method}_partition.csv'
    results.to_csv(output_file, index=False)
    print(f"  Saved: {output_file}")

    # Compute summary stats
    district_summary = results.groupby('district').agg({
        'total_pop': 'sum',
        'minority_pop': 'sum'
    })
    district_summary['minority_pct'] = (
        district_summary['minority_pop'] / district_summary['total_pop']
    )

    # Count majority-minority districts
    mm_count = (district_summary['minority_pct'] >= 0.50).sum()
    print(f"  {mm_count} majority-minority districts (>= 50%)")

    return results


def main():
    """Run 2010 redistricting for all states."""
    print("="*70)
    print("2010 CENSUS REDISTRICTING FOR TEMPORAL STABILITY ANALYSIS")
    print("="*70)
    print(f"Started: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"States: {', '.join(STATES)}")
    print(f"Edge weight: {EDGE_WEIGHT_FACTOR}x at {MINORITY_THRESHOLD*100}% threshold")
    print()

    results_summary = []

    for state in STATES:
        print(f"\n{'='*70}")
        print(f"STATE: {state.upper()}")
        print(f"{'='*70}")

        # Get district count
        n_districts = STATE_DISTRICTS[state]
        print(f"Target districts: {n_districts}")

        try:
            # Load data
            demographics, adjacency = load_2010_data(state)

            # Run n-way partitioning
            partition_nway = run_nway_partition(state, demographics, adjacency, n_districts)
            save_partition_results(state, 2010, 'nway', partition_nway, demographics)

            # Run recursive bisection
            partition_recursive = run_recursive_partition(state, demographics, adjacency, n_districts)
            save_partition_results(state, 2010, 'recursive', partition_recursive, demographics)

            results_summary.append({
                'state': state,
                'n_districts': n_districts,
                'n_tracts': len(demographics),
                'status': 'SUCCESS'
            })

        except Exception as e:
            print(f"  ERROR: {e}")
            import traceback
            traceback.print_exc()
            results_summary.append({
                'state': state,
                'n_districts': n_districts,
                'n_tracts': 0,
                'status': f'FAILED: {e}'
            })

    # Print summary
    print(f"\n{'='*70}")
    print("SUMMARY")
    print(f"{'='*70}")
    summary_df = pd.DataFrame(results_summary)
    print(summary_df.to_string(index=False))

    successful = sum(1 for r in results_summary if r['status'] == 'SUCCESS')
    print(f"\nCompleted: {successful}/{len(STATES)} states")
    print(f"Finished: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")


if __name__ == '__main__':
    main()
