"""
Run 2010 census redistricting for temporal stability analysis.

Runs both n-way and recursive bisection for 5 southern states using 2010 census data.
"""

import sys
from pathlib import Path
import pandas as pd
import geopandas as gpd
from datetime import datetime

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent))

from src.apportionment.partition.metis_wrapper import partition_graph
from src.apportionment.partition.recursive_bisection import recursive_bisection
from src.apportionment.data.adjacency import load_adjacency_matrix
from scripts.config_2010 import STATE_DISTRICTS

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

# Edge weighting parameters (from VRA papers)
EDGE_WEIGHT_FACTOR = 5
MINORITY_THRESHOLD = 0.40


def load_2010_census_data(state: str):
    """Load 2010 census tracts with demographics."""
    print(f"Loading 2010 data for {state}...")

    # Get FIPS code for this state
    fips = STATE_TO_FIPS[state]

    # Load tract geometries (using FIPS-based directory)
    tracts_dir = Path(f'data/2010/tiger/tracts/tl_2010_{fips}_tract10')
    tracts = gpd.read_file(tracts_dir)

    # Load demographics (using state name file)
    demographics_file = Path(f'data/2010/demographics/{state}_demographics_2010.csv')
    demographics = pd.read_csv(demographics_file)

    # Merge
    data = tracts.merge(demographics, on='GEOID', how='inner')

    print(f"  Loaded {len(data)} tracts")
    return data


def compute_edge_weights(tracts: gpd.GeoDataFrame, adjacency,
                        weight_factor: float, threshold: float):
    """Compute edge weights for minority concentration."""
    # Calculate minority percentage
    tracts['minority_pct'] = (
        tracts['hispanic_pop'] + tracts['black_pop'] +
        tracts['asian_pop'] + tracts['native_pop']
    ) / tracts['total_pop']

    # Create edge weight array
    edge_weights = adjacency.copy()
    edge_weights = edge_weights.tocoo()

    weights_data = []
    for i, j, v in zip(edge_weights.row, edge_weights.col, edge_weights.data):
        # Both tracts above threshold? Apply weight
        if (tracts.iloc[i]['minority_pct'] >= threshold and
            tracts.iloc[j]['minority_pct'] >= threshold):
            weights_data.append(weight_factor)
        else:
            weights_data.append(1.0)

    edge_weights.data = weights_data
    return edge_weights.tocsr()


def run_nway_partition(state: str, tracts: gpd.GeoDataFrame, adjacency,
                       n_districts: int):
    """Run n-way partitioning with edge weighting."""
    print(f"Running n-way partitioning for {state} (2010)...")

    # Compute edge weights
    edge_weights = compute_edge_weights(tracts, adjacency,
                                       EDGE_WEIGHT_FACTOR, MINORITY_THRESHOLD)

    # Extract populations
    populations = tracts['total_pop'].values

    # Run METIS n-way
    partition = partition_graph(
        adjacency=edge_weights,
        n_parts=n_districts,
        vertex_weights=populations,
        recursive=False  # N-way mode
    )

    return partition


def run_recursive_partition(state: str, tracts: gpd.GeoDataFrame, adjacency,
                            n_districts: int):
    """Run recursive bisection with edge weighting."""
    print(f"Running recursive bisection for {state} (2010)...")

    # Compute edge weights
    edge_weights = compute_edge_weights(tracts, adjacency,
                                       EDGE_WEIGHT_FACTOR, MINORITY_THRESHOLD)

    # Extract populations
    populations = tracts['total_pop'].values

    # Run recursive bisection
    partition = recursive_bisection(
        adjacency=edge_weights,
        n_districts=n_districts,
        populations=populations
    )

    return partition


def save_partition_results(state: str, year: int, method: str,
                           partition, tracts: gpd.GeoDataFrame):
    """Save partition results to CSV."""
    output_dir = Path('research/gerry-temporal-stability/results')
    output_dir.mkdir(parents=True, exist_ok=True)

    # Create results dataframe
    results = pd.DataFrame({
        'GEOID': tracts['GEOID'],
        'district': partition,
        'total_pop': tracts['total_pop'],
        'minority_pop': (
            tracts['hispanic_pop'] + tracts['black_pop'] +
            tracts['asian_pop'] + tracts['native_pop']
        )
    })

    # Add method and year metadata
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
    print(f"  {mm_count} majority-minority districts")

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
            tracts = load_2010_census_data(state)

            # Load adjacency
            adj_file = Path(f'outputs/data/2010/adjacency/{state}_adjacency.npz')
            if not adj_file.exists():
                print(f"  ERROR: Adjacency file not found: {adj_file}")
                print(f"  Run: python scripts/data/build_adjacency.py --state {state} --year 2010")
                continue

            adjacency = load_adjacency_matrix(adj_file)
            print(f"  Loaded adjacency: {adjacency.shape}")

            # Run n-way partitioning
            partition_nway = run_nway_partition(state, tracts, adjacency, n_districts)
            save_partition_results(state, 2010, 'nway', partition_nway, tracts)

            # Run recursive bisection
            partition_recursive = run_recursive_partition(state, tracts, adjacency, n_districts)
            save_partition_results(state, 2010, 'recursive', partition_recursive, tracts)

            results_summary.append({
                'state': state,
                'n_districts': n_districts,
                'n_tracts': len(tracts),
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

    print(f"\nCompleted: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")


if __name__ == '__main__':
    main()
