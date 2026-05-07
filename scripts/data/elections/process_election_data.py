#!/usr/bin/env python3
"""
Process raw election CSV data and convert to optimized parquet format.

This script:
1. Reads the tract-level election CSV from Harvard Dataverse
2. Selects relevant columns
3. Calculates derived metrics (vote percentages, margins, etc.)
4. Saves as parquet for efficient loading
"""

import argparse
import os
from pathlib import Path
import warnings

# Suppress warnings that can disrupt hierarchical display
warnings.filterwarnings('ignore')

import pandas as pd
import numpy as np


def process_2020_presidential(input_file, output_file):
    """Process 2020 presidential election data."""

    # Check if running as child process
    is_standalone = int(os.environ.get('TQDM_POSITION', '-1')) < 0

    if is_standalone:
        print(f"Reading: {input_file}")
    df = pd.read_csv(input_file, encoding='utf-8-sig')  # utf-8-sig to handle BOM

    if is_standalone:
        print(f"Original shape: {df.shape}")
        print(f"Columns: {len(df.columns)}")

    # Select relevant columns for 2020 presidential election
    columns_to_keep = [
        'tract_GEOID',
        'tract_state_fp',
        'tract_county_fp',
        'tract_population',
        'tract_hh_population',
        'tract_land_area',
        'tract_water_area',
        'num_precincts_contributing',
        'G20PRERTRU',  # Trump (Republican)
        'G20PREDBID',  # Biden (Democratic)
        'G20PRELJOR',  # Jorgensen (Libertarian)
        'G20PREOWRI',  # Write-ins
    ]

    # Check which columns exist
    missing = [col for col in columns_to_keep if col not in df.columns]
    if missing:
        if is_standalone:
            print(f"WARNING: Missing columns: {missing}")
        columns_to_keep = [col for col in columns_to_keep if col in df.columns]

    df = df[columns_to_keep].copy()

    # Rename columns for clarity
    df.columns = [
        'GEOID',
        'state_fips',
        'county_fips',
        'population',
        'hh_population',
        'land_area',
        'water_area',
        'num_precincts',
        'trump',
        'biden',
        'jorgensen',
        'write_in',
    ]

    # Calculate total votes
    df['total_votes'] = df[['trump', 'biden', 'jorgensen', 'write_in']].sum(axis=1)

    # Calculate two-party vote (Trump + Biden)
    df['two_party_votes'] = df['trump'] + df['biden']

    # Calculate percentages
    df['trump_pct'] = np.where(
        df['total_votes'] > 0,
        (df['trump'] / df['total_votes'] * 100).round(2),
        np.nan
    )
    df['biden_pct'] = np.where(
        df['total_votes'] > 0,
        (df['biden'] / df['total_votes'] * 100).round(2),
        np.nan
    )
    df['other_pct'] = np.where(
        df['total_votes'] > 0,
        ((df['jorgensen'] + df['write_in']) / df['total_votes'] * 100).round(2),
        np.nan
    )

    # Calculate two-party percentages
    df['trump_two_party_pct'] = np.where(
        df['two_party_votes'] > 0,
        (df['trump'] / df['two_party_votes'] * 100).round(2),
        np.nan
    )
    df['biden_two_party_pct'] = np.where(
        df['two_party_votes'] > 0,
        (df['biden'] / df['two_party_votes'] * 100).round(2),
        np.nan
    )

    # Calculate margin (Biden - Trump, in two-party vote)
    df['dem_margin'] = (df['biden_two_party_pct'] - df['trump_two_party_pct']).round(2)

    # Classify political lean
    def classify_lean(margin):
        if pd.isna(margin):
            return 'No Data'
        elif margin >= 20:
            return 'Strong D'
        elif margin >= 10:
            return 'Lean D'
        elif margin >= 5:
            return 'Tilt D'
        elif margin >= -5:
            return 'Tossup'
        elif margin >= -10:
            return 'Tilt R'
        elif margin >= -20:
            return 'Lean R'
        else:
            return 'Strong R'

    df['lean'] = df['dem_margin'].apply(classify_lean)

    # Calculate turnout rate (votes per population)
    df['turnout_rate'] = np.where(
        df['population'] > 0,
        (df['total_votes'] / df['population'] * 100).round(2),
        np.nan
    )

    # Convert GEOID to string to preserve leading zeros
    df['GEOID'] = df['GEOID'].astype(str)

    # Ensure GEOID is 11 digits (2-digit state + 3-digit county + 6-digit tract)
    df['GEOID'] = df['GEOID'].str.zfill(11)

    if is_standalone:
        print(f"\nProcessed shape: {df.shape}")
        print(f"\nSample data:")
        print(df.head(3).to_string())

        print(f"\nPolitical lean distribution:")
        print(df['lean'].value_counts().sort_index())

        print(f"\nSummary statistics:")
        print(df[['biden_pct', 'trump_pct', 'dem_margin', 'turnout_rate']].describe())

    # Save as parquet
    if is_standalone:
        print(f"\nSaving: {output_file}")
    df.to_parquet(output_file, engine='pyarrow', compression='snappy', index=False)

    if is_standalone:
        print(f"Saved {len(df):,} tracts")

    return df


def main():
    # Check if running as child process
    is_standalone = int(os.environ.get('TQDM_POSITION', '-1')) < 0

    parser = argparse.ArgumentParser(description='Process election data to parquet')
    parser.add_argument('--year', type=str, default='2020', choices=['2016', '2020'],
                       help='Election year (default: 2020)')
    parser.add_argument('--version', type=str,
                       help='Version identifier (for compatibility with parent scripts, not used directly)')
    parser.add_argument('--input-file', type=str,
                       help='Input CSV file path (if not provided, uses input-dir)')
    parser.add_argument('--input-dir', type=str,
                       default='data/raw/elections',
                       help='Input directory with raw CSV files (used if input-file not provided)')
    parser.add_argument('--output-dir', type=str,
                       default='data/processed/elections',
                       help='Output directory for parquet files')
    args = parser.parse_args()

    # Construct input path
    if args.input_file:
        input_file = Path(args.input_file)
    else:
        input_base = Path(args.input_dir) / f'{args.year}_president' / 'Contiguous USA - Main Method' / 'Census Tracts'
        input_file = input_base / f'tracts-{args.year}-RLCR.csv'

    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    output_file = output_dir / f'{args.year}_president_tract.parquet'

    if not input_file.exists():
        if is_standalone:
            print(f"ERROR: Input file not found: {input_file}")
            print(f"\nRun download_election_data.py first to download the raw data.")
        return 1

    if is_standalone:
        print("="*70)
        print("PROCESSING ELECTION DATA")
        print("="*70)
        print(f"Year: {args.year}")
        print(f"Input: {input_file}")
        print(f"Output: {output_file}")
        print("="*70)
        print()

    try:
        if args.year == '2020':
            process_2020_presidential(input_file, output_file)
        else:
            if is_standalone:
                print(f"ERROR: Year {args.year} processing not yet implemented")
            return 1

        if is_standalone:
            print("\n" + "="*70)
            print("PROCESSING COMPLETE!")
            print("="*70)

        return 0

    except Exception as e:
        if is_standalone:
            print(f"ERROR: {e}")
            import traceback
            traceback.print_exc()
        return 1


if __name__ == '__main__':
    exit(main())
