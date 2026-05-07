#!/usr/bin/env python3
"""
Download Census demographic data at the tract level.

This script downloads:
- Population by sex (male/female)
- Population by race/ethnicity (White, Black, Hispanic, Asian, Other)

Supports: 2000, 2010, 2020 Decennial Census
Data source: Census API
Output: CSV files in data/Census {year}/demographics/
"""

import pandas as pd
import requests
import time
from pathlib import Path
from tqdm import tqdm


# Census year configurations
CENSUS_CONFIGS = {
    2000: {
        'api_base': 'https://api.census.gov/data/2000/dec/sf1',
        'variables': {
            'total_pop': 'P001001',          # Total population
            'male': 'P012002',               # Male
            'female': 'P012026',             # Female
            'total_pop_race': 'P001001',     # Total (for validation)
            'white_non_hispanic': 'P005003', # White alone (not Hispanic)
            'black_non_hispanic': 'P005004', # Black alone (not Hispanic)
            'asian_non_hispanic': 'P005006', # Asian alone (not Hispanic)
            'hispanic': 'P008010',           # Hispanic or Latino (any race)
        }
    },
    2010: {
        'api_base': 'https://api.census.gov/data/2010/dec/sf1',
        'variables': {
            'total_pop': 'P001001',          # Total population
            'male': 'P012002',               # Male
            'female': 'P012026',             # Female
            'total_pop_race': 'P001001',     # Total (for validation)
            'white_non_hispanic': 'P005003', # White alone (not Hispanic)
            'black_non_hispanic': 'P005004', # Black alone (not Hispanic)
            'asian_non_hispanic': 'P005006', # Asian alone (not Hispanic)
            'hispanic': 'P008010',           # Hispanic or Latino (any race)
        }
    },
    2020: {
        'api_base': 'https://api.census.gov/data/2020/dec/dhc',
        'variables': {
            'total_pop': 'P12_001N',         # Total population
            'male': 'P12_002N',              # Male
            'female': 'P12_026N',            # Female
            'total_pop_race': 'P5_001N',     # Total (for validation)
            'white_non_hispanic': 'P5_003N', # White alone (not Hispanic)
            'black_non_hispanic': 'P5_004N', # Black alone (not Hispanic)
            'asian_non_hispanic': 'P5_006N', # Asian alone (not Hispanic)
            'hispanic': 'P5_010N',           # Hispanic or Latino (any race)
        }
    }
}


def download_state_demographics(state_fips, state_name, year, output_dir, delay=0.5):
    """
    Download demographic data for all tracts in a state.

    Args:
        state_fips: 2-digit state FIPS code
        state_name: State name for display
        year: Census year (2000, 2010, or 2020)
        output_dir: Directory to save output
        delay: Delay between requests (seconds)

    Returns:
        DataFrame with tract-level demographics or None if failed
    """

    print(f"Downloading demographics for {state_name} (FIPS: {state_fips})...")

    # Get configuration for this census year
    config = CENSUS_CONFIGS[year]
    api_base = config['api_base']
    variables = config['variables']

    # Build API request with year-specific variables (deduplicate)
    variables_list = list(set(variables.values()))  # Remove duplicates
    variables_str = ','.join(variables_list)
    url = f"{api_base}?get={variables_str}&for=tract:*&in=state:{state_fips}"

    try:
        response = requests.get(url)
        response.raise_for_status()

        data = response.json()

        # First row is headers
        headers = data[0]
        rows = data[1:]

        # Create DataFrame
        df = pd.DataFrame(rows, columns=headers)

        # Create GEOID (state + county + tract)
        df['GEOID'] = df['state'] + df['county'] + df['tract']

        # Convert numeric columns
        unique_vars = set(variables.values())
        for api_var in unique_vars:
            if api_var in df.columns:
                df[api_var] = pd.to_numeric(df[api_var], errors='coerce')

        # Create standard name columns (handle duplicates where multiple names map to same API var)
        for standard_name, api_var in variables.items():
            if api_var in df.columns:
                df[standard_name] = df[api_var]

        # Calculate "Other" category (total - white - black - asian - hispanic)
        df['other'] = (
            df['total_pop_race'] -
            df['white_non_hispanic'] -
            df['black_non_hispanic'] -
            df['asian_non_hispanic'] -
            df['hispanic']
        )

        # Select and order final columns
        df = df[[
            'GEOID', 'state', 'county', 'tract',
            'total_pop', 'male', 'female',
            'white_non_hispanic', 'black_non_hispanic',
            'asian_non_hispanic', 'hispanic', 'other'
        ]]

        print(f"  Downloaded {len(df):,} tracts")

        # Save to CSV
        output_file = output_dir / f'{state_name.lower().replace(" ", "_")}_demographics_{year}.csv'
        df.to_csv(output_file, index=False)
        print(f"  Saved: {output_file}")

        # Wait before next request
        time.sleep(delay)

        return df

    except requests.exceptions.RequestException as e:
        print(f"  ERROR: Failed to download data for {state_name}: {e}")
        return None
    except Exception as e:
        print(f"  ERROR: Failed to process data for {state_name}: {e}")
        return None


def main():
    """Download demographic data for all 50 states + DC."""
    import argparse

    parser = argparse.ArgumentParser(description='Download Census demographic data')
    parser.add_argument('-y', '--year', type=int, default=2020, choices=[2000, 2010, 2020],
                       help='Census year (default: 2020)')
    parser.add_argument('-o', '--output-dir', type=str,
                       help='Output directory (default: data/Census {year}/demographics)')
    args = parser.parse_args()

    # Set output directory
    if args.output_dir:
        output_dir = Path(args.output_dir)
    else:
        output_dir = Path(f'data/Census {args.year}/demographics')

    # State FIPS codes
    STATES = {
        '01': 'Alabama', '02': 'Alaska', '04': 'Arizona', '05': 'Arkansas',
        '06': 'California', '08': 'Colorado', '09': 'Connecticut', '10': 'Delaware',
        '11': 'District of Columbia', '12': 'Florida', '13': 'Georgia',
        '15': 'Hawaii', '16': 'Idaho', '17': 'Illinois', '18': 'Indiana',
        '19': 'Iowa', '20': 'Kansas', '21': 'Kentucky', '22': 'Louisiana',
        '23': 'Maine', '24': 'Maryland', '25': 'Massachusetts', '26': 'Michigan',
        '27': 'Minnesota', '28': 'Mississippi', '29': 'Missouri', '30': 'Montana',
        '31': 'Nebraska', '32': 'Nevada', '33': 'New Hampshire', '34': 'New Jersey',
        '35': 'New Mexico', '36': 'New York', '37': 'North Carolina', '38': 'North Dakota',
        '39': 'Ohio', '40': 'Oklahoma', '41': 'Oregon', '42': 'Pennsylvania',
        '44': 'Rhode Island', '45': 'South Carolina', '46': 'South Dakota',
        '47': 'Tennessee', '48': 'Texas', '49': 'Utah', '50': 'Vermont',
        '51': 'Virginia', '53': 'Washington', '54': 'West Virginia',
        '55': 'Wisconsin', '56': 'Wyoming'
    }

    output_dir.mkdir(parents=True, exist_ok=True)

    print("=" * 70)
    print(f"DOWNLOADING {args.year} CENSUS DEMOGRAPHIC DATA")
    print("=" * 70)
    print(f"Output directory: {output_dir}")
    print(f"States to download: {len(STATES)}")
    print(f"API endpoint: {CENSUS_CONFIGS[args.year]['api_base']}")
    print("=" * 70)
    print()

    successful = []
    failed = []

    for state_fips, state_name in tqdm(STATES.items(), desc="Downloading states"):
        result = download_state_demographics(state_fips, state_name, args.year, output_dir, delay=0.5)
        if result is not None:
            successful.append(state_name)
        else:
            failed.append(state_name)

    # Summary
    print()
    print("=" * 70)
    print("DOWNLOAD SUMMARY")
    print("=" * 70)
    print(f"Successful: {len(successful)}/{len(STATES)}")
    print(f"Failed: {len(failed)}/{len(STATES)}")

    if failed:
        print(f"\nFailed states:")
        for state in failed:
            print(f"  - {state}")

    print("=" * 70)

    if len(failed) == 0:
        print("\n[SUCCESS] All states downloaded successfully!")
        print(f"\nNext step: Process the data:")
        print(f"  python scripts/data/demographics/process_demographic_data.py --year {args.year}")
    else:
        print(f"\n[WARNING] Some states failed to download. Check API limits and retry.")

    return 0 if len(failed) == 0 else 1


if __name__ == '__main__':
    exit(main())
