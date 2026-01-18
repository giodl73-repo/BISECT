#!/usr/bin/env python3
"""
Analyze political characteristics of redistricting results.

This script:
1. Loads tract-level election data (from process_election_data.py)
2. Loads district assignments from a redistricting run
3. Joins them by GEOID
4. Calculates district-level political metrics
5. Analyzes intermediate rounds to see how regions split
"""

import pandas as pd
import numpy as np
import argparse
from pathlib import Path
import json
import pickle
import os
import sys

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))
from scripts.utils import get_state_config


def load_election_data(year='2020'):
    """Load processed election data."""
    election_file = Path(f'data/processed/elections/{year}_president_tract.parquet')
    if not election_file.exists():
        raise FileNotFoundError(f"Election data not found: {election_file}\n"
                               f"Run process_election_data.py first.")

    df = pd.read_parquet(election_file)
    print(f"Loaded {len(df):,} tracts with election data")
    return df


def load_district_assignments(run_dir, tracts_gdf):
    """Load final district assignments from a redistricting run.

    Args:
        run_dir: Path to redistricting run directory
        tracts_gdf: GeoDataFrame with tract GEOIDs (for index to GEOID mapping)

    Returns:
        Dictionary mapping GEOID (str) -> district (int)
    """
    assignments_file = Path(run_dir) / 'data' / 'final_assignments.pkl'
    if not assignments_file.exists():
        raise FileNotFoundError(f"Assignments not found: {assignments_file}")

    with open(assignments_file, 'rb') as f:
        assignments_by_index = pickle.load(f)

    print(f"Loaded assignments for {len(assignments_by_index):,} tracts")

    # Convert index-based assignments to GEOID-based
    # The assignments dictionary uses tract indices as keys
    assignments = {}
    for idx, district in assignments_by_index.items():
        if idx < len(tracts_gdf):
            geoid = str(tracts_gdf.iloc[idx]['GEOID']).zfill(11)
            assignments[geoid] = district

    print(f"Mapped {len(assignments):,} tract indices to GEOIDs")
    return assignments


def analyze_districts(election_df, assignments_dict, num_districts=435):
    """Analyze political characteristics of each district."""

    # Convert assignments to DataFrame
    # Convert GEOID to string with proper zero-padding (11 digits)
    assignments_df = pd.DataFrame([
        {'GEOID': str(geoid).zfill(11), 'district': district}
        for geoid, district in assignments_dict.items()
    ])

    # Join with election data
    df = election_df.merge(assignments_df, on='GEOID', how='inner')

    print(f"\nJoined {len(df):,} tracts ({len(df)/len(election_df)*100:.1f}% of election data)")

    # Calculate district-level statistics
    district_stats = []

    for district in range(1, num_districts + 1):
        district_data = df[df['district'] == district]

        if len(district_data) == 0:
            continue

        # Sum votes
        total_trump = district_data['trump'].sum()
        total_biden = district_data['biden'].sum()
        total_other = district_data['jorgensen'].sum() + district_data['write_in'].sum()
        total_votes = total_trump + total_biden + total_other
        two_party_votes = total_trump + total_biden

        # Calculate percentages
        trump_pct = (total_trump / total_votes * 100) if total_votes > 0 else 0
        biden_pct = (total_biden / total_votes * 100) if total_votes > 0 else 0
        trump_two_party_pct = (total_trump / two_party_votes * 100) if two_party_votes > 0 else 0
        biden_two_party_pct = (total_biden / two_party_votes * 100) if two_party_votes > 0 else 0

        # Democratic margin (two-party)
        dem_margin = biden_two_party_pct - trump_two_party_pct

        # Classify lean
        if dem_margin >= 20:
            lean = 'Strong D'
        elif dem_margin >= 10:
            lean = 'Lean D'
        elif dem_margin >= 5:
            lean = 'Tilt D'
        elif dem_margin >= -5:
            lean = 'Tossup'
        elif dem_margin >= -10:
            lean = 'Tilt R'
        elif dem_margin >= -20:
            lean = 'Lean R'
        else:
            lean = 'Strong R'

        # Other statistics
        total_population = district_data['population'].sum()
        num_tracts = len(district_data)

        # Weighted average turnout
        weighted_turnout = (
            (district_data['total_votes'] * district_data['population']).sum() /
            district_data['population'].sum()
        ) if total_population > 0 else 0

        district_stats.append({
            'district': district,
            'num_tracts': num_tracts,
            'population': total_population,
            'total_votes': total_votes,
            'trump': total_trump,
            'biden': total_biden,
            'other': total_other,
            'trump_pct': round(trump_pct, 2),
            'biden_pct': round(biden_pct, 2),
            'trump_two_party_pct': round(trump_two_party_pct, 2),
            'biden_two_party_pct': round(biden_two_party_pct, 2),
            'dem_margin': round(dem_margin, 2),
            'lean': lean,
            'turnout_rate': round(weighted_turnout, 2)
        })

    return pd.DataFrame(district_stats)


def analyze_intermediate_rounds(election_df, run_dir, tracts_gdf):
    """Analyze political characteristics at each intermediate round.

    Args:
        election_df: DataFrame with election data
        run_dir: Path to redistricting run directory
        tracts_gdf: GeoDataFrame with tract GEOIDs (for index to GEOID mapping)

    Returns:
        DataFrame with round statistics or None
    """
    intermediate_dir = Path(run_dir) / 'intermediate'
    if not intermediate_dir.exists():
        return None

    # Find all round assignment files
    round_files = sorted(intermediate_dir.glob('round_*_assignments.json'))

    if len(round_files) == 0:
        return None

    round_stats = []

    for assignments_file in round_files:
        # Load metadata
        metadata_file = assignments_file.parent / assignments_file.name.replace('_assignments.json', '_metadata.json')
        if not metadata_file.exists():
            continue

        with open(metadata_file, 'r') as f:
            metadata = json.load(f)

        round_num = metadata['depth']
        num_regions = metadata['num_regions']

        # Load assignments (keys are tract indices as strings in JSON)
        with open(assignments_file, 'r') as f:
            assignments_by_index_str = json.load(f)

        # Convert index strings to GEOIDs
        assignments_df_data = []
        for idx_str, region in assignments_by_index_str.items():
            idx = int(idx_str)
            if idx < len(tracts_gdf):
                geoid = str(tracts_gdf.iloc[idx]['GEOID']).zfill(11)
                assignments_df_data.append({'GEOID': geoid, 'region': region})

        # Convert to DataFrame
        assignments_df = pd.DataFrame(assignments_df_data)

        # Join with election data
        df = election_df.merge(assignments_df, on='GEOID', how='inner')

        # Calculate region-level statistics
        for region in range(num_regions):
            region_data = df[df['region'] == region]

            if len(region_data) == 0:
                continue

            # Sum votes
            total_trump = region_data['trump'].sum()
            total_biden = region_data['biden'].sum()
            two_party_votes = total_trump + total_biden

            # Calculate two-party percentages
            trump_two_party_pct = (total_trump / two_party_votes * 100) if two_party_votes > 0 else 0
            biden_two_party_pct = (total_biden / two_party_votes * 100) if two_party_votes > 0 else 0
            dem_margin = biden_two_party_pct - trump_two_party_pct

            # Classify lean
            if dem_margin >= 20:
                lean = 'Strong D'
            elif dem_margin >= 10:
                lean = 'Lean D'
            elif dem_margin >= 5:
                lean = 'Tilt D'
            elif dem_margin >= -5:
                lean = 'Tossup'
            elif dem_margin >= -10:
                lean = 'Tilt R'
            elif dem_margin >= -20:
                lean = 'Lean R'
            else:
                lean = 'Strong R'

            round_stats.append({
                'round': round_num,
                'num_regions': num_regions,
                'region': region + 1,  # 1-based
                'population': region_data['population'].sum(),
                'trump': total_trump,
                'biden': total_biden,
                'trump_two_party_pct': round(trump_two_party_pct, 2),
                'biden_two_party_pct': round(biden_two_party_pct, 2),
                'dem_margin': round(dem_margin, 2),
                'lean': lean
            })

    return pd.DataFrame(round_stats) if round_stats else None


def main():
    parser = argparse.ArgumentParser(description='Analyze political characteristics of districts')
    parser.add_argument('run_dir', type=str,
                       help='Redistricting run directory (e.g., outputs/us_2020_v2)')
    parser.add_argument('--state', type=str, required=True, help='State code (e.g., CA, NY)')
    parser.add_argument('--year', type=str, default='2020', choices=['2020', '2016'],
                       help='Election year (default: 2020)')
    parser.add_argument('--census-year', type=str, default=None, choices=['2020', '2010', '2000'],
                       help='Census year for tract boundaries (default: same as election year)')
    parser.add_argument('--output-dir', type=str, default=None,
                       help='Output directory (default: run_dir/political_analysis)')
    parser.add_argument('--force', action='store_true',
                       help='Force regeneration even if outputs exist')
    args = parser.parse_args()

    # Get state code from arguments
    state_code = args.state.upper()

    # Default census year to election year if not specified
    if args.census_year is None:
        args.census_year = args.year

    run_dir = Path(args.run_dir)

    if not run_dir.exists():
        print(f"ERROR: Run directory not found: {run_dir}")
        return 1

    # Set output directory
    if args.output_dir:
        output_dir = Path(args.output_dir)
    else:
        output_dir = run_dir  # Political subdirectory created later

    # Don't create output_dir here - it's the state directory which already exists

    # Check if being called from parent pipeline
    position = int(os.environ.get('TQDM_POSITION', '-1'))
    send_status = position >= 0

    def report_progress(msg):
        """Report progress to parent pipeline."""
        if send_status:
            print(f"STATUS:{position}:{msg}", flush=True)

    is_standalone = not send_status

    if is_standalone:
        print("="*70)
        print("POLITICAL ANALYSIS")
        print("="*70)
        print(f"Run: {run_dir}")
        print(f"Election Year: {args.year}")
        print(f"Output: {output_dir}")
        print("="*70)
        print()

    # Check if analysis already exists
    district_file = output_dir / f'district_political_{args.year}.csv'
    rounds_file = output_dir / f'rounds_political_{args.year}.csv'

    if not args.force and district_file.exists() and rounds_file.exists():
        print("Political analysis already exists - skipping")
        print(f"  {district_file}")
        print(f"  {rounds_file}")
        print("\nUse --force to regenerate")
        return 0

    try:

        # Skip Alaska and Hawaii - no tract-level election data available
        if state_code in ['AK', 'HI']:
            state_display = run_dir.name.replace('_', ' ').title()
            report_progress(f"Analyzing {state_display} - Skipped (no tract-level election data)")
            if is_standalone:
                print(f"\nSkipping {state_code} - No tract-level election data available")
                print("Political analysis requires precinct-to-tract geocoded data")

            # Create empty output files to indicate completion
            empty_district_df = pd.DataFrame(columns=[
                'district', 'biden_votes', 'trump_votes', 'total_votes',
                'biden_pct', 'trump_pct', 'dem_margin', 'lean_category'
            ])
            empty_rounds_df = pd.DataFrame(columns=[
                'round', 'region_id', 'biden_votes', 'trump_votes', 'total_votes',
                'biden_pct', 'trump_pct', 'dem_margin', 'lean_category'
            ])

            # Create political subdirectory
            political_dir = output_dir / 'political'
            political_dir.mkdir(parents=True, exist_ok=True)

            district_file = political_dir / 'district_political.csv'
            rounds_file = political_dir / 'rounds_political.csv'
            empty_district_df.to_csv(district_file, index=False)
            empty_rounds_df.to_csv(rounds_file, index=False)

            if is_standalone:
                print(f"Created empty output files:")
                print(f"  {district_file}")
                print(f"  {rounds_file}")

            return 0  # Success - expected skip

        # Load tract file to get GEOID mapping (unified directory structure)
        import geopandas as gpd
        state_code_lower = state_code.lower()
        tracts_file = Path(f'data/tracts/{args.census_year}/{state_code_lower}_tracts_{args.census_year}.parquet')

        if not tracts_file.exists():
            raise FileNotFoundError(f"Tract file not found: {tracts_file}")

        state_display = run_dir.name.replace('_', ' ').title()
        report_progress(f"Analyzing {state_display} - Loading tract data...")

        if is_standalone:
            print(f"Loading tract data for {state_code}...")
        tracts_gdf = gpd.read_parquet(tracts_file)
        if is_standalone:
            print(f"Loaded {len(tracts_gdf):,} tracts with GEOIDs")

        # Load election data
        report_progress(f"Analyzing {state_display} - Loading election data...")
        if is_standalone:
            print("\nLoading election data...")
        election_df = load_election_data(args.year)
        if is_standalone:
            print()

        # Load district assignments
        report_progress(f"Analyzing {state_display} - Loading district assignments...")
        if is_standalone:
            print("Loading district assignments...")
        assignments = load_district_assignments(run_dir, tracts_gdf)
        if is_standalone:
            print()

        # Get number of districts from config
        state_config = get_state_config(args.census_year)
        config = state_config.get(state_code, {})
        num_districts = config.get('districts', 1)
        print(f"State has {num_districts} congressional districts")

        # Analyze final districts
        report_progress(f"Analyzing {state_display} - Calculating district statistics...")
        if is_standalone:
            print("\nAnalyzing final districts...")
        district_stats = analyze_districts(election_df, assignments, num_districts)

        # Create political subdirectory
        political_dir = output_dir / 'political'
        political_dir.mkdir(parents=True, exist_ok=True)

        # Save district statistics (no year suffix)
        output_file = political_dir / 'district_political.csv'
        district_stats.to_csv(output_file, index=False)
        print(f"\nSaved district statistics: {output_file}")

        # Print summary
        print("\n" + "="*70)
        print("DISTRICT POLITICAL SUMMARY")
        print("="*70)
        print(f"\nTotal Districts Analyzed: {len(district_stats)}")
        print("\nLean Distribution:")
        print(district_stats['lean'].value_counts().sort_index())

        print("\nSummary Statistics:")
        print(district_stats[['biden_two_party_pct', 'trump_two_party_pct', 'dem_margin']].describe())

        # Show most competitive districts
        print("\nMost Competitive Districts (smallest margin):")
        district_stats['abs_margin'] = district_stats['dem_margin'].abs()
        competitive = district_stats.nsmallest(10, 'abs_margin')
        print(competitive[['district', 'biden_two_party_pct', 'trump_two_party_pct', 'dem_margin', 'lean']])

        # Analyze intermediate rounds
        print("\n" + "="*70)
        print("ANALYZING INTERMEDIATE ROUNDS")
        print("="*70)

        round_stats = analyze_intermediate_rounds(election_df, run_dir, tracts_gdf)

        if round_stats is not None:
            # Save to political subdirectory (no year suffix)
            output_file = political_dir / 'rounds_political.csv'
            round_stats.to_csv(output_file, index=False)
            print(f"\nSaved round statistics: {output_file}")

            print("\nRound Summary:")
            for round_num in sorted(round_stats['round'].unique()):
                round_data = round_stats[round_stats['round'] == round_num]
                num_regions = round_data['num_regions'].iloc[0]
                print(f"\nRound {round_num} ({num_regions} regions):")
                print(round_data['lean'].value_counts().sort_index())
        else:
            print("\nNo intermediate round data found")

        print("\n" + "="*70)
        print("ANALYSIS COMPLETE!")
        print("="*70)

        return 0

    except Exception as e:
        print(f"ERROR: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == '__main__':
    exit(main())
