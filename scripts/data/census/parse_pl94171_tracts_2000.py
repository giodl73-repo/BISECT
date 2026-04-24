#!/usr/bin/env python3
"""
Parse 2000 Census PL 94-171 tract-level data into unified format.

This script extracts census tract records (SUMLEV=140) from the .upl format
geographic files and creates CSV files matching the 2020 tract format.
"""

import argparse
import os
import sys
from pathlib import Path
import warnings

# Ensure project root is on sys.path so 'scripts' package is importable
_project_root = Path(__file__).resolve().parents[3]
if str(_project_root) not in sys.path:
    sys.path.insert(0, str(_project_root))

# Suppress ALL warnings at every level
warnings.filterwarnings('ignore')
os.environ['PYTHONWARNINGS'] = 'ignore'

# Suppress pandas/matplotlib/geopandas at import time
if not sys.warnoptions:
    warnings.simplefilter('ignore')

import pandas as pd
from tqdm import tqdm
from scripts.utils.status_protocol import StatusReporter

# Additional pandas-specific warning suppression
pd.options.mode.chained_assignment = None  # Suppress SettingWithCopyWarning
pd.set_option('mode.copy_on_write', True)

# Redirect stderr to devnull when in child mode to prevent any leakage
if int(os.environ.get('TQDM_POSITION', '-1')) >= 0:
    sys.stderr = open(os.devnull, 'w')

# State FIPS codes
STATE_FIPS = {
    'AL': '01', 'AK': '02', 'AZ': '04', 'AR': '05', 'CA': '06',
    'CO': '08', 'CT': '09', 'DE': '10', 'FL': '12', 'GA': '13',
    'HI': '15', 'ID': '16', 'IL': '17', 'IN': '18', 'IA': '19',
    'KS': '20', 'KY': '21', 'LA': '22', 'ME': '23', 'MD': '24',
    'MA': '25', 'MI': '26', 'MN': '27', 'MS': '28', 'MO': '29',
    'MT': '30', 'NE': '31', 'NV': '32', 'NH': '33', 'NJ': '34',
    'NM': '35', 'NY': '36', 'NC': '37', 'ND': '38', 'OH': '39',
    'OK': '40', 'OR': '41', 'PA': '42', 'RI': '44', 'SC': '45',
    'SD': '46', 'TN': '47', 'TX': '48', 'UT': '49', 'VT': '50',
    'VA': '51', 'WA': '53', 'WV': '54', 'WI': '55', 'WY': '56'
}


# Global reporter instance
_reporter = StatusReporter()


def parse_2000_geo_file(geo_file):
    """Parse 2000 census .upl file for tract-level records.

    Format: Fixed-width .upl format
    Key positions (0-indexed, from actual data inspection):
    - FILEID: 0-6
    - STUSAB: 6-8
    - SUMLEV: 8-16 (14000000 = tract)
    - LOGRECNO: 18-25
    - STATE: 29-31
    - COUNTY: 31-34
    - TRACT: 55-61
    - AREALAND: 170-180
    - AREAWATR: 184-194
    - NAME: 200-290
    - POP100: 292-297
    - INTPTLAT: 310-319
    - INTPTLON: 319-331
    """
    records = []

    with open(geo_file, 'r', encoding='latin-1') as f:
        for line in f:
            # Extract SUMLEV (summary level)
            sumlev = line[8:16].strip()

            # Only process tract-level records (SUMLEV=14000000)
            if sumlev != '14000000':
                continue

            # Extract LOGRECNO (not used in final output, but helpful for debugging)
            logrecno = line[18:25].strip()

            # GEOID construction for tracts (11 digits: SSCCCTTTTTT)
            state_fips = line[29:31]
            county_fips = line[31:34]
            tract = line[55:61].strip()
            geoid = f"{state_fips}{county_fips}{tract}"

            # Name, area, coordinates, population
            name = line[200:290].strip()
            arealand = line[170:180].strip()
            areawatr = line[184:194].strip()
            pop100 = line[292:297].strip()
            intptlat = line[310:319].strip()
            intptlon = line[319:331].strip()

            records.append({
                'GEOID': geoid,
                'NAME': name,
                'AREALAND': int(arealand) if arealand else 0,
                'AREAWATR': int(areawatr) if areawatr else 0,
                'INTPTLAT': float(intptlat) if intptlat else 0.0,
                'INTPTLON': float(intptlon) if intptlon else 0.0,
                'population': int(pop100) if pop100 else 0,
            })

    return pd.DataFrame(records)


def process_state(state_code, census_dir, output_dir, use_status=False):
    """Process one state's 2000 census tract data."""

    state_lower = state_code.lower()
    geo_file = census_dir / 'redistricting' / f"{state_lower}geo.upl"

    def log(msg):
        if use_status:
            _reporter.report(f"{state_code}: {msg}")
        else:
            print(f"  {msg}")

    if not geo_file.exists():
        log(f"ERROR: Geographic file not found: {geo_file}")
        return False

    # Parse file
    log("Parsing geographic file...")
    df = parse_2000_geo_file(geo_file)

    # Calculate total area and water fraction
    df['total_area'] = df['AREALAND'] + df['AREAWATR']
    df['water_fraction'] = df['AREAWATR'] / df['total_area'].replace(0, 1)

    # Reorder columns to match 2020 format
    df = df[['GEOID', 'NAME', 'AREALAND', 'AREAWATR', 'INTPTLAT', 'INTPTLON',
             'population', 'total_area', 'water_fraction']]

    # Save CSV
    output_file = output_dir / f"{state_lower}_tracts_2000_population.csv"
    df.to_csv(output_file, index=False)

    log(f"Saved {len(df)} tracts to {output_file.name}")
    return True


def main():
    parser = argparse.ArgumentParser(
        description='Parse 2000 Census PL 94-171 tract-level data'
    )
    parser.add_argument(
        '--census-dir',
        type=Path,
        default=Path('data/2000'),
        help='Census 2000 data directory'
    )
    parser.add_argument(
        '--output-dir',
        type=Path,
        default=Path('outputs/data/2000/units'),
        help='Output directory for CSV files'
    )
    parser.add_argument(
        '--states',
        type=str,
        nargs='+',
        help='Process specific states (e.g., AL CA TX). If not specified, processes all states.'
    )

    args = parser.parse_args()

    # Create output directory
    args.output_dir.mkdir(parents=True, exist_ok=True)

    # Determine which states to process
    if args.states:
        states_to_process = [s.upper() for s in args.states]
    else:
        states_to_process = sorted(STATE_FIPS.keys())

    # Check if running as child process (uses STATUS protocol)
    tqdm_pos = int(os.environ.get('TQDM_POSITION', '-1'))
    use_status = tqdm_pos >= 0

    if use_status:
        _reporter.report(f"Processing {len(states_to_process)} states (2000)")
    else:
        print(f"Processing {len(states_to_process)} states...")
        print(f"Census directory: {args.census_dir}")
        print(f"Output directory: {args.output_dir}")
        print()

    # Process each state
    success_count = 0
    failed_states = []

    # Use tqdm only when running standalone
    iterator = states_to_process if use_status else tqdm(states_to_process, desc="Processing states")

    for state_code in iterator:
        if not use_status:
            print(f"\n{state_code}:")
        if process_state(state_code, args.census_dir, args.output_dir, use_status):
            success_count += 1
        else:
            failed_states.append(state_code)

    # Summary
    if use_status:
        _reporter.report(f"Completed {success_count}/{len(states_to_process)} states")
        if failed_states:
            _reporter.report(f"Failed: {', '.join(failed_states)}")
    else:
        print("\n" + "=" * 60)
        print(f"Processed {success_count}/{len(states_to_process)} states successfully")
        if failed_states:
            print(f"Failed states: {', '.join(failed_states)}")
        print("=" * 60)


if __name__ == '__main__':
    main()
