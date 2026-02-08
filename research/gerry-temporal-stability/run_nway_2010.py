"""
Run n-way partitioning for 2010 census data.

Uses direct METIS k-way partitioning (recursive=False) with edge weighting.
"""

import sys
from pathlib import Path
import pickle
import pandas as pd
import numpy as np
from datetime import datetime

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from src.apportionment.partition.metis_wrapper import partition_graph

# States for temporal stability analysis
STATES = ['AL', 'GA', 'LA', 'MS', 'SC']

STATE_TO_NAME = {
    'AL': 'alabama',
    'GA': 'georgia',
    'LA': 'louisiana',
    'MS': 'mississippi',
    'SC': 'south_carolina'
}

# District counts for 2010
STATE_DISTRICTS = {
    'AL': 7,
    'GA': 14,
    'LA': 6,
    'MS': 4,
    'SC': 7
}


def run_nway_partition(state_code: str, year: str = '2010'):
    """Run n-way partitioning for one state."""
    state_name = STATE_TO_NAME[state_code]
    n_districts = STATE_DISTRICTS[state_code]

    print(f"\n{'='*70}")
    print(f"N-WAY PARTITIONING: {state_name.upper()} ({year})")
    print(f"{'='*70}")
    print(f"Districts: {n_districts}")

    project_root = Path(__file__).parent.parent.parent

    # Load adjacency (with edge weights)
    adj_file = project_root / f'outputs/v1/data/{year}/adjacency/{state_code.lower()}_adjacency_{year}.pkl'

    print(f"Loading adjacency from {adj_file.name}...")
    with open(adj_file, 'rb') as f:
        graph_data = pickle.load(f)

    adjacency = graph_data['adjacency']
    vertex_weights = graph_data['vertex_weights']
    index_to_geoid = graph_data['index_to_geoid']
    edge_weights = graph_data.get('edge_weights', None)

    print(f"  Loaded {len(adjacency)} tracts")
    print(f"  Total population: {vertex_weights.sum():,}")
    if edge_weights:
        print(f"  Edge weights: {len(edge_weights):,} edges")

    # Run n-way partitioning (recursive=False)
    print(f"\nRunning METIS {n_districts}-way partitioning...")
    print(f"  Mode: direct k-way (recursive=False)")
    print(f"  Edge weighted: {'Yes' if edge_weights else 'No'}")

    start_time = datetime.now()

    partition = partition_graph(
        adjacency=adjacency,
        vertex_weights=vertex_weights,
        nparts=n_districts,
        target_weights=None,
        recursive=False,  # N-WAY MODE!
        ufactor=5,  # 0.5% imbalance
        niter=100,
        objtype='cut',
        debug=False,
        edge_weights=edge_weights
    )

    elapsed = (datetime.now() - start_time).total_seconds()
    print(f"  Completed in {elapsed:.2f}s")
    print(f"  Districts created: {len(set(partition))}")

    # Load demographics for analysis
    demographics_file = project_root / f'data/{year}/demographics/{state_name}_demographics_{year}.csv'
    demographics = pd.read_csv(demographics_file)
    demographics['GEOID'] = demographics['GEOID'].astype(str).str.zfill(11)

    # Create GEOID mapping dict for faster lookup
    demographics_dict = demographics.set_index('GEOID').to_dict('index')

    # Create results dataframe
    results_data = []
    for idx, district in enumerate(partition):
        geoid = str(index_to_geoid[idx]).zfill(11)  # Zero-pad to 11 digits
        demo = demographics_dict.get(geoid)

        if demo is not None:
            minority_pop = (
                demo.get('hispanic', 0) +
                demo.get('black_non_hispanic', 0) +
                demo.get('asian_non_hispanic', 0) +
                demo.get('other', 0)
            )

            results_data.append({
                'GEOID': geoid,
                'district': district,
                'total_pop': demo.get('total_pop', 0),
                'minority_pop': minority_pop
            })
        else:
            print(f"  WARNING: GEOID {geoid} not found in demographics")

    if len(results_data) == 0:
        print(f"  ERROR: No matching GEOIDs found!")
        print(f"  Sample partition GEOID: {str(index_to_geoid[0])}")
        print(f"  Sample demographics GEOID: {demographics['GEOID'].iloc[0]}")
        raise ValueError("No matching GEOIDs between partition and demographics")

    results = pd.DataFrame(results_data)
    results['method'] = 'nway'
    results['year'] = int(year)
    results['state'] = state_name

    # Save
    output_dir = project_root / 'research/gerry-temporal-stability/results'
    output_dir.mkdir(parents=True, exist_ok=True)
    output_file = output_dir / f'{state_name}_{year}_nway_partition.csv'

    results.to_csv(output_file, index=False)
    print(f"\n  Saved: {output_file.name} ({len(results)} tracts)")

    # Summary stats
    district_summary = results.groupby('district').agg({
        'total_pop': 'sum',
        'minority_pop': 'sum'
    })
    district_summary['minority_pct'] = (
        district_summary['minority_pop'] / district_summary['total_pop']
    )

    mm_count = (district_summary['minority_pct'] >= 0.50).sum()
    print(f"  Majority-minority districts: {mm_count} (>= 50%)")

    return elapsed


def main():
    """Run n-way partitioning for all states."""
    print("="*70)
    print("2010 N-WAY PARTITIONING FOR TEMPORAL STABILITY ANALYSIS")
    print("="*70)
    print(f"Started: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"States: {', '.join(STATES)}")
    print()

    results_summary = []

    for state_code in STATES:
        try:
            elapsed = run_nway_partition(state_code, '2010')
            results_summary.append({
                'state': STATE_TO_NAME[state_code],
                'districts': STATE_DISTRICTS[state_code],
                'runtime': f'{elapsed:.2f}s',
                'status': 'SUCCESS'
            })
        except Exception as e:
            print(f"\n  ERROR: {e}")
            import traceback
            traceback.print_exc()
            results_summary.append({
                'state': STATE_TO_NAME[state_code],
                'districts': STATE_DISTRICTS[state_code],
                'runtime': 'N/A',
                'status': f'FAILED: {str(e)[:50]}'
            })

    # Summary
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
