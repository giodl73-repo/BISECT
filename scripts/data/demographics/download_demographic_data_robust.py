#!/usr/bin/env python3
"""
Download Census demographic data with exponential backoff retry logic.

This script handles Census API rate limits gracefully with:
- Exponential backoff on 429 errors
- Multiple retry attempts
- State-by-state resumability (skips already downloaded states)

Supports: 2000, 2010, 2020 Decennial Census
Data source: Census API
Output: CSV files in data/Census {year}/demographics/
"""

import pandas as pd
import requests
import time
from pathlib import Path
from tqdm import tqdm
import random


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


def download_with_retry(state_fips, state_name, year, max_retries=5, base_delay=1.0):
    """
    Download demographic data with exponential backoff.

    Args:
        state_fips: 2-digit state FIPS code
        state_name: State name
        year: Census year (2000, 2010, or 2020)
        max_retries: Maximum number of retry attempts
        base_delay: Initial delay in seconds (doubles each retry)

    Returns:
        DataFrame or None if all retries failed
    """

    # Get configuration for this census year
    config = CENSUS_CONFIGS[year]
    api_base = config['api_base']
    variables = config['variables']

    # Build API request with year-specific variables (deduplicate)
    variables_list = list(set(variables.values()))  # Remove duplicates
    variables_str = ','.join(variables_list)
    url = f"{api_base}?get={variables_str}&for=tract:*&in=state:{state_fips}"

    for attempt in range(max_retries):
        try:
            response = requests.get(url)

            # Success!
            if response.status_code == 200:
                data = response.json()
                headers = data[0]
                rows = data[1:]

                df = pd.DataFrame(rows, columns=headers)
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

                # Calculate "Other" category
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

                return df

            # Rate limited - use exponential backoff
            elif response.status_code == 429:
                if attempt < max_retries - 1:
                    # Calculate delay: base_delay * (2^attempt) + random jitter
                    delay = base_delay * (2 ** attempt) + random.uniform(0, 1)
                    print(f"  Rate limited. Retry {attempt+1}/{max_retries} after {delay:.1f}s...")
                    time.sleep(delay)
                    continue
                else:
                    print(f"  Failed after {max_retries} attempts (rate limited)")
                    return None

            # Other HTTP error
            else:
                response.raise_for_status()

        except requests.exceptions.RequestException as e:
            if attempt < max_retries - 1:
                delay = base_delay * (2 ** attempt)
                print(f"  Error: {e}. Retry {attempt+1}/{max_retries} after {delay:.1f}s...")
                time.sleep(delay)
                continue
            else:
                print(f"  Failed after {max_retries} attempts: {e}")
                return None
        except Exception as e:
            print(f"  ERROR processing data: {e}")
            return None

    return None


def download_state_demographics(state_fips, state_name, year, output_dir, base_delay=1.0):
    """Download demographic data for a state."""

    # Check if already exists
    output_file = output_dir / f'{state_name.lower().replace(" ", "_")}_demographics_{year}.csv'
    if output_file.exists():
        return 'skipped'

    print(f"\n{state_name} (FIPS: {state_fips})")

    df = download_with_retry(state_fips, state_name, year, max_retries=5, base_delay=base_delay)

    if df is not None:
        print(f"  Downloaded {len(df):,} tracts")
        df.to_csv(output_file, index=False)
        print(f"  Saved: {output_file.name}")

        # Small delay before next request
        time.sleep(0.5)
        return 'success'
    else:
        return 'failed'


def main():
    """Download demographic data for all states with robust retry logic."""
    import argparse

    parser = argparse.ArgumentParser(description='Download Census demographic data (robust)')
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
    print(f"DOWNLOADING {args.year} CENSUS DEMOGRAPHIC DATA (ROBUST)")
    print("=" * 70)
    print(f"Output directory: {output_dir}")
    print(f"API endpoint: {CENSUS_CONFIGS[args.year]['api_base']}")
    print(f"Features: Exponential backoff, auto-retry, resumable")
    print("=" * 70)
    print()

    successful = []
    failed = []
    skipped = []

    for state_fips, state_name in STATES.items():
        result = download_state_demographics(state_fips, state_name, args.year, output_dir, base_delay=1.0)

        if result == 'success':
            successful.append(state_name)
        elif result == 'failed':
            failed.append(state_name)
        elif result == 'skipped':
            skipped.append(state_name)

    print()
    print("=" * 70)
    print("DOWNLOAD SUMMARY")
    print("=" * 70)
    print(f"Successful: {len(successful)}/{len(STATES)}")
    print(f"Skipped (already exists): {len(skipped)}/{len(STATES)}")
    print(f"Failed: {len(failed)}/{len(STATES)}")

    if failed:
        print(f"\nFailed states:")
        for state in failed:
            print(f"  - {state}")
        print(f"\nYou can run this script again - it will only retry failed states.")
    else:
        if len(skipped) == len(STATES):
            print("\nAll states already downloaded!")
        else:
            print("\nAll states downloaded successfully!")
        print("\nNext step: Process the data:")
        print(f"  python scripts/data/demographics/process_demographic_data.py --year {args.year}")

    print("=" * 70)

    return 0 if len(failed) == 0 else 1


if __name__ == '__main__':
    exit(main())
