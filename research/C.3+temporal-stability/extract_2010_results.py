"""
Extract 2010 redistricting results to CSV format for temporal stability analysis.
"""

import sys
from pathlib import Path
import pickle
import pandas as pd

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

# States and their output directories
STATES = {
    'alabama': 'alabama_full_20260208_081326',
    'georgia': 'georgia_full_20260208_081326',
    'louisiana': 'louisiana_full_20260208_081326',
    'mississippi': 'mississippi_full_20260208_081326',
    'south_carolina': 'south_carolina_full_20260208_081326'
}

# State name to code mapping
STATE_TO_CODE = {
    'alabama': 'AL',
    'georgia': 'GA',
    'louisiana': 'LA',
    'mississippi': 'MS',
    'south_carolina': 'SC'
}


def extract_state_results(state: str, output_dir: str):
    """Extract results for one state."""
    print(f"\nExtracting {state}...")

    project_root = Path(__file__).parent.parent.parent

    # Load partition results (dict: tract_idx -> district)
    pkl_file = project_root / f'outputs/{output_dir}/data/final_assignments.pkl'

    with open(pkl_file, 'rb') as f:
        partition = pickle.load(f)

    print(f"  Loaded {len(partition)} tract assignments")
    print(f"  Districts: {len(set(partition.values()))}")

    # Load GEOID mapping from adjacency file
    state_code = STATE_TO_CODE[state]
    adj_file = project_root / 'outputs/v1/data/2010/adjacency' / f'{state_code.lower()}_adjacency_2010.pkl'

    with open(adj_file, 'rb') as f:
        adj_data = pickle.load(f)

    index_to_geoid = adj_data['index_to_geoid']
    print(f"  Loaded GEOID mapping: {len(index_to_geoid)} tracts")

    # Load demographics to get populations
    demographics_file = project_root / f'data/2010/demographics/{state}_demographics_2010.csv'
    demographics = pd.read_csv(demographics_file)

    # Zero-pad GEOIDs to 11 digits
    demographics['GEOID'] = demographics['GEOID'].astype(str).str.zfill(11)

    print(f"  Loaded {len(demographics)} tracts from demographics")

    # Create GEOID to demographics mapping
    demographics_dict = demographics.set_index('GEOID').to_dict('index')

    # Create results dataframe
    results_data = []
    for idx, district in partition.items():
        geoid = str(index_to_geoid[idx])
        demo = demographics_dict.get(geoid, {})

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

    results = pd.DataFrame(results_data)

    # Add metadata
    results['method'] = 'recursive'  # edge-weighted recursive bisection
    results['year'] = 2010
    results['state'] = state

    # Save
    output_dir_path = project_root / 'research/gerry-temporal-stability/results'
    output_dir_path.mkdir(parents=True, exist_ok=True)
    output_file = output_dir_path / f'{state}_2010_recursive_partition.csv'

    results.to_csv(output_file, index=False)
    print(f"  Saved: {output_file}")
    print(f"  Size: {output_file.stat().st_size / 1024:.1f} KB")

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


def main():
    """Extract all state results."""
    print("="*70)
    print("EXTRACT 2010 REDISTRICTING RESULTS")
    print("="*70)
    print(f"States: {', '.join(STATES.keys())}")
    print()

    for state, output_dir in STATES.items():
        try:
            extract_state_results(state, output_dir)
        except Exception as e:
            print(f"  ERROR: {e}")
            import traceback
            traceback.print_exc()

    print()
    print("="*70)
    print("EXTRACTION COMPLETE")
    print("="*70)


if __name__ == '__main__':
    main()
