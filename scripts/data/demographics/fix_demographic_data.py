#!/usr/bin/env python3
"""
Fix demographic data for states that failed due to API rate limits.

Only downloads states that are missing from data/raw/demographics/
"""

import pandas as pd
import requests
import time
from pathlib import Path
from tqdm import tqdm


CENSUS_API_BASE = "https://api.census.gov/data/2020/dec/dhc"

VARIABLES = [
    'P12_001N',  # Total population
    'P12_002N',  # Male
    'P12_026N',  # Female
    'P5_001N',   # Total (for validation)
    'P5_003N',   # White alone (not Hispanic)
    'P5_004N',   # Black alone (not Hispanic)
    'P5_006N',   # Asian alone (not Hispanic)
    'P5_010N',   # Hispanic or Latino (any race)
]


def download_state_demographics(state_fips, state_name, output_dir, delay=2.0):
    """Download demographic data for all tracts in a state."""

    print(f"Downloading demographics for {state_name} (FIPS: {state_fips})...")

    variables_str = ','.join(VARIABLES)
    url = f"{CENSUS_API_BASE}?get={variables_str}&for=tract:*&in=state:{state_fips}"

    try:
        response = requests.get(url)
        response.raise_for_status()

        data = response.json()
        headers = data[0]
        rows = data[1:]

        df = pd.DataFrame(rows, columns=headers)
        df['GEOID'] = df['state'] + df['county'] + df['tract']

        for var in VARIABLES:
            df[var] = pd.to_numeric(df[var], errors='coerce')

        df = df.rename(columns={
            'P12_001N': 'total_pop',
            'P12_002N': 'male',
            'P12_026N': 'female',
            'P5_001N': 'total_pop_race',
            'P5_003N': 'white_non_hispanic',
            'P5_004N': 'black_non_hispanic',
            'P5_006N': 'asian_non_hispanic',
            'P5_010N': 'hispanic',
        })

        df['other'] = (
            df['total_pop_race'] -
            df['white_non_hispanic'] -
            df['black_non_hispanic'] -
            df['asian_non_hispanic'] -
            df['hispanic']
        )

        df = df[[
            'GEOID', 'state', 'county', 'tract',
            'total_pop', 'male', 'female',
            'white_non_hispanic', 'black_non_hispanic',
            'asian_non_hispanic', 'hispanic', 'other'
        ]]

        print(f"  Downloaded {len(df):,} tracts")

        output_file = output_dir / f'{state_name.lower().replace(" ", "_")}_demographics_2020.csv'
        df.to_csv(output_file, index=False)
        print(f"  Saved: {output_file}")

        time.sleep(delay)
        return df

    except requests.exceptions.RequestException as e:
        print(f"  ERROR: Failed to download data for {state_name}: {e}")
        time.sleep(delay * 2)
        return None
    except Exception as e:
        print(f"  ERROR: Failed to process data for {state_name}: {e}")
        time.sleep(delay * 2)
        return None


def main():
    """Download demographic data for missing states only."""

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

    output_dir = Path('data/raw/demographics')
    output_dir.mkdir(parents=True, exist_ok=True)

    # Find missing states
    missing_states = {}
    for fips, name in STATES.items():
        filename = f'{name.lower().replace(" ", "_")}_demographics_2020.csv'
        if not (output_dir / filename).exists():
            missing_states[fips] = name

    if len(missing_states) == 0:
        print("All states already downloaded!")
        return 0

    print("=" * 70)
    print("FIXING DEMOGRAPHIC DATA (MISSING STATES ONLY)")
    print("=" * 70)
    print(f"Output directory: {output_dir}")
    print(f"Missing states: {len(missing_states)}")
    print(f"Using 2-second delay to avoid rate limits")
    print("=" * 70)
    print()

    successful = []
    failed = []

    for state_fips, state_name in tqdm(missing_states.items(), desc="Downloading missing states"):
        result = download_state_demographics(state_fips, state_name, output_dir, delay=2.0)
        if result is not None:
            successful.append(state_name)
        else:
            failed.append(state_name)

    print()
    print("=" * 70)
    print("DOWNLOAD SUMMARY")
    print("=" * 70)
    print(f"Successful: {len(successful)}/{len(missing_states)}")
    print(f"Failed: {len(failed)}/{len(missing_states)}")

    if failed:
        print(f"\nFailed states:")
        for state in failed:
            print(f"  - {state}")
        print(f"\nIf still hitting rate limits, wait 10-15 minutes and run again.")
    else:
        print("\nAll missing states downloaded successfully!")
        print("\nNext step: Process the data:")
        print("  python scripts/political/process_demographic_data.py")

    print("=" * 70)

    return 0 if len(failed) == 0 else 1


if __name__ == '__main__':
    exit(main())
