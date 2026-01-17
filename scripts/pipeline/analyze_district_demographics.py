#!/usr/bin/env python3
"""
Analyze demographic characteristics of redistricting results.

This script:
1. Loads tract-level demographic data (from process_demographic_data.py)
2. Loads district assignments from a redistricting run
3. Joins them by GEOID
4. Calculates district-level demographic statistics
"""

import pandas as pd
import numpy as np
import argparse
from pathlib import Path
import pickle


def load_demographic_data(census_year='2020'):
    """Load processed demographic data."""
    demo_file = Path(f'data/processed/demographics/{census_year}_demographics_tract.parquet')
    if not demo_file.exists():
        raise FileNotFoundError(f"Demographic data not found: {demo_file}\n"
                               f"Run process_demographic_data.py first.")

    df = pd.read_parquet(demo_file)
    print(f"Loaded {len(df):,} tracts with demographic data")
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
    assignments = {}
    for idx, district in assignments_by_index.items():
        if idx < len(tracts_gdf):
            geoid = str(tracts_gdf.iloc[idx]['GEOID']).zfill(11)
            assignments[geoid] = district

    print(f"Mapped {len(assignments):,} tract indices to GEOIDs")
    return assignments


def analyze_district_demographics(demo_df, assignments_dict, num_districts):
    """Analyze demographic characteristics of each district."""

    # Convert assignments to DataFrame
    assignments_df = pd.DataFrame([
        {'GEOID': str(geoid).zfill(11), 'district': district}
        for geoid, district in assignments_dict.items()
    ])

    # Ensure both have GEOID as string for consistent merge
    demo_df = demo_df.copy()
    demo_df['GEOID'] = demo_df['GEOID'].astype(str).str.zfill(11)
    assignments_df['GEOID'] = assignments_df['GEOID'].astype(str).str.zfill(11)

    # Join with demographic data
    df = demo_df.merge(assignments_df, on='GEOID', how='inner')

    print(f"\nJoined {len(df):,} tracts ({len(df)/len(demo_df)*100:.1f}% of demographic data)")

    # Calculate district-level statistics
    district_stats = []

    for district in range(1, num_districts + 1):
        district_data = df[df['district'] == district]

        if len(district_data) == 0:
            continue

        # Total population
        total_pop = district_data['total_pop'].sum()

        # Sex totals
        total_male = district_data['male'].sum()
        total_female = district_data['female'].sum()

        # Race/ethnicity totals
        total_white = district_data['white_non_hispanic'].sum()
        total_black = district_data['black_non_hispanic'].sum()
        total_asian = district_data['asian_non_hispanic'].sum()
        total_hispanic = district_data['hispanic'].sum()
        total_other = district_data['other'].sum()

        # Calculate percentages
        male_pct = (total_male / total_pop * 100) if total_pop > 0 else 0
        female_pct = (total_female / total_pop * 100) if total_pop > 0 else 0

        white_pct = (total_white / total_pop * 100) if total_pop > 0 else 0
        black_pct = (total_black / total_pop * 100) if total_pop > 0 else 0
        asian_pct = (total_asian / total_pop * 100) if total_pop > 0 else 0
        hispanic_pct = (total_hispanic / total_pop * 100) if total_pop > 0 else 0
        other_pct = (total_other / total_pop * 100) if total_pop > 0 else 0

        # Determine majority demographic
        race_categories = {
            'White': white_pct,
            'Black': black_pct,
            'Asian': asian_pct,
            'Hispanic': hispanic_pct,
            'Other': other_pct
        }
        majority_race = max(race_categories, key=race_categories.get)
        majority_race_pct = race_categories[majority_race]

        # Check for minority-majority district (non-white > 50%)
        non_white_pct = 100 - white_pct
        minority_majority = 'Yes' if non_white_pct > 50 else 'No'

        district_stats.append({
            'district': district,
            'num_tracts': len(district_data),
            'population': total_pop,
            'male': total_male,
            'female': total_female,
            'male_pct': round(male_pct, 2),
            'female_pct': round(female_pct, 2),
            'white': total_white,
            'black': total_black,
            'asian': total_asian,
            'hispanic': total_hispanic,
            'other': total_other,
            'white_pct': round(white_pct, 2),
            'black_pct': round(black_pct, 2),
            'asian_pct': round(asian_pct, 2),
            'hispanic_pct': round(hispanic_pct, 2),
            'other_pct': round(other_pct, 2),
            'majority_race': majority_race,
            'majority_race_pct': round(majority_race_pct, 2),
            'minority_majority': minority_majority
        })

    return pd.DataFrame(district_stats)


def main():
    parser = argparse.ArgumentParser(description='Analyze demographic characteristics of districts')
    parser.add_argument('run_dir', type=str,
                       help='Redistricting run directory (e.g., outputs/us_2020_v1/states/california)')
    parser.add_argument('--state', type=str, required=True, help='State code (e.g., CA, NY)')
    parser.add_argument('--census-year', type=str, default='2020', choices=['2020', '2010', '2000'],
                       help='Census year (default: 2020)')
    parser.add_argument('--output-dir', type=str, default=None,
                       help='Output directory (default: run_dir/demographic_analysis)')
    parser.add_argument('--force', action='store_true',
                       help='Force regeneration even if outputs exist')
    args = parser.parse_args()

    # Get state code from arguments
    state_code = args.state.upper()

    run_dir = Path(args.run_dir)

    if not run_dir.exists():
        print(f"ERROR: Run directory not found: {run_dir}")
        return 1

    # Set output directory
    if args.output_dir:
        output_dir = Path(args.output_dir)
    else:
        output_dir = run_dir / 'demographic'

    output_dir.mkdir(parents=True, exist_ok=True)

    print("=" * 70)
    print("DEMOGRAPHIC ANALYSIS")
    print("=" * 70)
    print(f"Run: {run_dir}")
    print(f"Output: {output_dir}")
    print("=" * 70)
    print()

    # Check if analysis already exists
    district_file = output_dir / 'district_demographics.csv'

    if not args.force and district_file.exists():
        print("Demographic analysis already exists - skipping")
        print(f"  {district_file}")
        print("\nUse --force to regenerate")
        return 0

    try:

        # Load tract file to get GEOID mapping (unified directory structure)
        import geopandas as gpd
        state_code_lower = state_code.lower()
        tracts_file = Path(f'data/tracts/{args.census_year}/{state_code_lower}_tracts_{args.census_year}.parquet')

        if not tracts_file.exists():
            raise FileNotFoundError(f"Tract file not found: {tracts_file}")

        print(f"Loading tract data for {state_code}...")
        tracts_gdf = gpd.read_parquet(tracts_file)
        print(f"Loaded {len(tracts_gdf):,} tracts with GEOIDs")

        # Load demographic data
        print("\nLoading demographic data...")
        demo_df = load_demographic_data(args.census_year)
        print()

        # Load district assignments
        print("Loading district assignments...")
        assignments = load_district_assignments(run_dir, tracts_gdf)
        print()

        # Get number of districts from config
        import sys
        script_dir = Path(__file__).parent
        project_root = script_dir.parent.parent
        sys.path.insert(0, str(project_root))

        if args.census_year == '2020':
            from scripts.config_2020 import STATE_CONFIG_2020
            config = STATE_CONFIG_2020.get(state_code, {})
        elif args.census_year == '2010':
            from scripts.config_2010 import STATE_CONFIG_2010
            config = STATE_CONFIG_2010.get(state_code, {})
        elif args.census_year == '2000':
            from scripts.config_2000 import STATE_CONFIG_2000
            config = STATE_CONFIG_2000.get(state_code, {})
        else:
            config = {}

        num_districts = config.get('districts', 1)
        print(f"State has {num_districts} congressional districts")

        # Analyze districts
        print("\nAnalyzing district demographics...")
        district_stats = analyze_district_demographics(demo_df, assignments, num_districts)

        # Save statistics
        district_stats.to_csv(district_file, index=False)
        print(f"\nSaved district demographics: {district_file}")

        # Print summary
        print("\n" + "=" * 70)
        print("DISTRICT DEMOGRAPHIC SUMMARY")
        print("=" * 70)
        print(f"\nTotal Districts Analyzed: {len(district_stats)}")

        print("\nGender Distribution:")
        print(f"  Average Male: {district_stats['male_pct'].mean():.2f}%")
        print(f"  Average Female: {district_stats['female_pct'].mean():.2f}%")

        print("\nRace/Ethnicity Distribution:")
        print(f"  Average White: {district_stats['white_pct'].mean():.2f}%")
        print(f"  Average Black: {district_stats['black_pct'].mean():.2f}%")
        print(f"  Average Asian: {district_stats['asian_pct'].mean():.2f}%")
        print(f"  Average Hispanic: {district_stats['hispanic_pct'].mean():.2f}%")
        print(f"  Average Other: {district_stats['other_pct'].mean():.2f}%")

        print("\nMajority Race Distribution:")
        print(district_stats['majority_race'].value_counts())

        print(f"\nMinority-Majority Districts: {(district_stats['minority_majority'] == 'Yes').sum()}/{len(district_stats)}")

        # Show most diverse districts
        print("\nMost Diverse Districts (no single group > 50%):")
        diverse = district_stats[district_stats['majority_race_pct'] < 50].sort_values('majority_race_pct')
        if len(diverse) > 0:
            print(diverse[['district', 'white_pct', 'black_pct', 'asian_pct', 'hispanic_pct', 'majority_race', 'majority_race_pct']].head(10))
        else:
            print("  (None - all districts have a clear majority)")

        print("\n" + "=" * 70)
        print("ANALYSIS COMPLETE!")
        print("=" * 70)

        return 0

    except Exception as e:
        print(f"ERROR: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == '__main__':
    exit(main())
