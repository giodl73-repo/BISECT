#!/usr/bin/env python3
"""
Process raw demographic CSV files into a single parquet file.

This script:
1. Loads all state demographic CSV files
2. Calculates demographic percentages
3. Combines into a single parquet file for efficient analysis

Input: data/Census {year}/demographics/*.csv (from download_demographic_data.py)
Output: outputs/data/demographics/{year}_demographics_tract.parquet
"""

import argparse
import os
from pathlib import Path
import warnings

# Suppress warnings that can disrupt hierarchical display
warnings.filterwarnings('ignore')

import pandas as pd
from tqdm import tqdm


def process_demographic_data(year, input_dir, output_dir):
    """Process all demographic CSV files into a single parquet file."""

    # Check if running as child process
    is_standalone = int(os.environ.get('TQDM_POSITION', '-1')) < 0

    input_path = Path(input_dir)
    output_path = Path(output_dir)
    output_path.mkdir(parents=True, exist_ok=True)

    if not input_path.exists():
        if is_standalone:
            print(f"ERROR: Input directory not found: {input_path}")
            print("Run download_demographic_data.py first.")
        return 1

    # Find all CSV files
    csv_files = sorted(input_path.glob(f'*_demographics_{year}.csv'))

    if len(csv_files) == 0:
        if is_standalone:
            print(f"ERROR: No demographic CSV files found in {input_dir}")
            print("Run download_demographic_data.py first.")
        return 1

    if is_standalone:
        print("=" * 70)
        print("PROCESSING DEMOGRAPHIC DATA")
        print("=" * 70)
        print(f"Year: {year}")
        print(f"Input directory: {input_path}")
        print(f"Output directory: {output_path}")
        print(f"CSV files found: {len(csv_files)}")
        print("=" * 70)
        print()

    # Load and combine all CSV files
    all_data = []

    for csv_file in tqdm(csv_files, desc="Loading CSV files"):
        df = pd.read_csv(csv_file)
        all_data.append(df)

    # Combine into single DataFrame
    combined = pd.concat(all_data, ignore_index=True)

    if is_standalone:
        print(f"\nTotal tracts: {len(combined):,}")

    # Calculate percentages
    if is_standalone:
        print("\nCalculating demographic percentages...")

    # Sex percentages
    combined['male_pct'] = (combined['male'] / combined['total_pop'] * 100).round(2)
    combined['female_pct'] = (combined['female'] / combined['total_pop'] * 100).round(2)

    # Race/ethnicity percentages
    combined['white_pct'] = (combined['white_non_hispanic'] / combined['total_pop'] * 100).round(2)
    combined['black_pct'] = (combined['black_non_hispanic'] / combined['total_pop'] * 100).round(2)
    combined['asian_pct'] = (combined['asian_non_hispanic'] / combined['total_pop'] * 100).round(2)
    combined['hispanic_pct'] = (combined['hispanic'] / combined['total_pop'] * 100).round(2)
    combined['other_pct'] = (combined['other'] / combined['total_pop'] * 100).round(2)

    # Handle divisions by zero (tracts with no population)
    combined = combined.fillna(0)

    # Select final columns
    final_columns = [
        'GEOID', 'state', 'county', 'tract',
        'total_pop',
        'male', 'female', 'male_pct', 'female_pct',
        'white_non_hispanic', 'black_non_hispanic', 'asian_non_hispanic', 'hispanic', 'other',
        'white_pct', 'black_pct', 'asian_pct', 'hispanic_pct', 'other_pct'
    ]

    combined = combined[final_columns]

    # Save to parquet
    output_file = output_path / f'{year}_demographics_tract.parquet'
    combined.to_parquet(output_file, index=False, engine='pyarrow', compression='snappy')

    if is_standalone:
        print(f"\nSaved: {output_file}")
        print(f"File size: {output_file.stat().st_size / 1024 / 1024:.1f} MB")

        # Print summary statistics
        print("\n" + "=" * 70)
        print("SUMMARY STATISTICS")
        print("=" * 70)

        print("\nGender Distribution (US Average):")
        print(f"  Male: {combined['male_pct'].mean():.2f}%")
        print(f"  Female: {combined['female_pct'].mean():.2f}%")

        print("\nRace/Ethnicity Distribution (US Average):")
        print(f"  White (non-Hispanic): {combined['white_pct'].mean():.2f}%")
        print(f"  Black (non-Hispanic): {combined['black_pct'].mean():.2f}%")
        print(f"  Asian (non-Hispanic): {combined['asian_pct'].mean():.2f}%")
        print(f"  Hispanic: {combined['hispanic_pct'].mean():.2f}%")
        print(f"  Other: {combined['other_pct'].mean():.2f}%")

        print("\n" + "=" * 70)
        print("SUCCESS! Demographic data processed")
        print("=" * 70)

        print(f"\nNext step: Analyze demographics by district:")
        print(f"  python scripts/political/analyze_district_demographics.py <state_dir>")

    return 0


def main():
    parser = argparse.ArgumentParser(
        description='Process raw demographic CSV files into parquet format'
    )
    parser.add_argument('--year', type=int, default=2020, choices=[2000, 2010, 2020],
                       help='Census year (default: 2020)')
    parser.add_argument('--version', type=str,
                       help='Version identifier (for compatibility with parent scripts, not used directly)')
    parser.add_argument('--input-dir', type=str,
                       help='Input directory with CSV files (default: data/Census {year}/demographics)')
    parser.add_argument('--output-dir', type=str,
                       help='Output directory for parquet file (default: outputs/data/demographics)')
    args = parser.parse_args()

    # Set default paths if not provided
    input_dir = args.input_dir if args.input_dir else f'data/Census {args.year}/demographics'
    output_dir = args.output_dir if args.output_dir else 'outputs/data/demographics'

    return process_demographic_data(args.year, input_dir, output_dir)


if __name__ == '__main__':
    exit(main())
