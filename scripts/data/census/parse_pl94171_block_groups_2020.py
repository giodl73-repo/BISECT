#!/usr/bin/env python3
"""
Parse 2020 Census PL 94-171 block group-level data into unified format.

This script extracts census block group records (SUMLEV=150) from the geographic
and population files and creates CSV files matching the standard unit format.
"""

import argparse
import os
import sys
from pathlib import Path
import warnings

# Suppress ALL warnings at every level
warnings.filterwarnings('ignore')
os.environ['PYTHONWARNINGS'] = 'ignore'

# Suppress pandas/matplotlib/geopandas at import time
import sys
if not sys.warnoptions:
    warnings.simplefilter('ignore')

import pandas as pd
from tqdm import tqdm

# Add project root to path for imports
sys.path.insert(0, str(Path(__file__).resolve().parents[3]))
from scripts.utils.status_protocol import StatusReporter

# Additional pandas-specific warning suppression
pd.options.mode.chained_assignment = None  # Suppress SettingWithCopyWarning

# Disable pandas progress bars
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


def parse_2020_geo_file(geo_file):
    """Parse 2020 census geographic header file for block group-level records.

    Format: Pipe-delimited with header
    Key fields:
    - SUMLEV: Summary level (150 = block group)
    - LOGRECNO: Logical record number (for joining with population file)
    - GEOID: Geographic identifier (12 digits for block groups: SSCCCTTTTTTB)
    - NAME: Geographic name
    - AREALAND: Land area (square meters)
    - AREAWATR: Water area (square meters)
    - INTPTLAT: Internal point latitude
    - INTPTLON: Internal point longitude
    """
    # Column names from aa_geo_header.csv
    columns = [
        'FILEID', 'STUSAB', 'SUMLEV', 'GEOVAR', 'GEOCOMP', 'CHARITER', 'CIFSN', 'LOGRECNO',
        'GEOID', 'GEOCODE', 'REGION', 'DIVISION', 'STATE', 'STATENS', 'COUNTY', 'COUNTYCC',
        'COUNTYNS', 'COUSUB', 'COUSUBCC', 'COUSUBNS', 'SUBMCD', 'SUBMCDCC', 'SUBMCDNS',
        'ESTATE', 'ESTATECC', 'ESTATENS', 'CONCIT', 'CONCITCC', 'CONCITNS', 'PLACE',
        'PLACECC', 'PLACENS', 'TRACT', 'BLKGRP', 'BLOCK', 'AIANHH', 'AIHHTLI', 'AIANHHFP',
        'AIANHHCC', 'AIANHHNS', 'AITS', 'AITSFP', 'AITSCC', 'AITSNS', 'TTRACT', 'TBLKGRP',
        'ANRC', 'ANRCCC', 'ANRCNS', 'CBSA', 'MEMI', 'CSA', 'METDIV', 'NECTA', 'NMEMI',
        'CNECTA', 'NECTADIV', 'CBSAPCI', 'NECTAPCI', 'UA', 'UATYPE', 'UR', 'CD116', 'CD118',
        'CD119', 'CD120', 'CD121', 'SLDU18', 'SLDU22', 'SLDU24', 'SLDU26', 'SLDU28',
        'SLDL18', 'SLDL22', 'SLDL24', 'SLDL26', 'SLDL28', 'VTD', 'VTDI', 'ZCTA', 'SDELM',
        'SDSEC', 'SDUNI', 'PUMA', 'AREALAND', 'AREAWATR', 'BASENAME', 'NAME', 'FUNCSTAT',
        'GCUNI', 'POP100', 'HU100', 'INTPTLAT', 'INTPTLON', 'LSADC', 'PARTFLAG', 'UGA'
    ]

    # Read pipe-delimited file
    df = pd.read_csv(
        geo_file,
        sep='|',
        header=None,
        names=columns,
        encoding='latin-1',
        dtype={'LOGRECNO': str, 'GEOID': str, 'SUMLEV': str}
    )

    # Filter to block group level (SUMLEV='150')
    df = df[df['SUMLEV'] == '150'].copy()

    # Select and rename columns
    df = df[['LOGRECNO', 'GEOID', 'NAME', 'AREALAND', 'AREAWATR', 'INTPTLAT', 'INTPTLON']]

    # Convert numeric columns
    df['AREALAND'] = pd.to_numeric(df['AREALAND'], errors='coerce').fillna(0).astype(int)
    df['AREAWATR'] = pd.to_numeric(df['AREAWATR'], errors='coerce').fillna(0).astype(int)
    df['INTPTLAT'] = pd.to_numeric(df['INTPTLAT'], errors='coerce').fillna(0.0)
    df['INTPTLON'] = pd.to_numeric(df['INTPTLON'], errors='coerce').fillna(0.0)

    return df


def parse_2020_pop_file(pop_file):
    """Parse 2020 census population file (File 1: P1 data).

    Format: Pipe-delimited
    Fields: FILEID|STUSAB|CHARITER|CIFSN|LOGRECNO|P0010001|...
    P0010001 = Total population (from P1 table)
    """
    # Read pipe-delimited CSV
    # Force LOGRECNO to string type for merging with geo data
    df = pd.read_csv(pop_file, sep='|', header=None, encoding='latin-1',
                     names=['FILEID', 'STUSAB', 'CHARITER', 'CIFSN', 'LOGRECNO', 'POP100'],
                     usecols=[0, 1, 2, 3, 4, 5],
                     dtype={'LOGRECNO': str})

    return df[['LOGRECNO', 'POP100']]


def process_state(state_code, census_dir, output_dir, use_status=False):
    """Process one state's 2020 census block group data."""

    state_lower = state_code.lower()
    state_dir = census_dir / 'redistricting' / f"{state_lower}2020.pl"

    def log(msg):
        if use_status:
            _reporter.report(f"{state_code}: {msg}")
        else:
            print(f"  {msg}")

    if not state_dir.exists():
        log(f"ERROR: Directory not found: {state_dir}")
        return False

    # File paths
    geo_file = state_dir / f"{state_lower}geo2020.pl"
    pop_file = state_dir / f"{state_lower}000012020.pl"

    if not geo_file.exists():
        log(f"ERROR: Geographic file not found: {geo_file}")
        return False

    if not pop_file.exists():
        log(f"ERROR: Population file not found: {pop_file}")
        return False

    # Parse files
    log("Parsing geographic file...")
    geo_df = parse_2020_geo_file(geo_file)

    log("Parsing population file...")
    pop_df = parse_2020_pop_file(pop_file)

    # Merge on LOGRECNO
    log("Merging data...")
    df = geo_df.merge(pop_df, on='LOGRECNO', how='left')
    df = df.drop(columns=['LOGRECNO'])

    # Calculate total area and water fraction
    df['total_area'] = df['AREALAND'] + df['AREAWATR']
    df['water_fraction'] = df['AREAWATR'] / df['total_area'].replace(0, 1)

    # Rename population column
    df = df.rename(columns={'POP100': 'population'})

    # Reorder columns to match standard format
    df = df[['GEOID', 'NAME', 'AREALAND', 'AREAWATR', 'INTPTLAT', 'INTPTLON',
             'population', 'total_area', 'water_fraction']]

    # Save CSV
    output_file = output_dir / f"{state_lower}_block_groups_2020_population.csv"
    df.to_csv(output_file, index=False)

    log(f"Saved {len(df)} block groups to {output_file.name}")
    return True


def main():
    parser = argparse.ArgumentParser(
        description='Parse 2020 Census PL 94-171 block group-level data'
    )
    parser.add_argument(
        '--census-dir',
        type=Path,
        default=Path('data/2020'),
        help='Census 2020 data directory'
    )
    parser.add_argument(
        '--output-dir',
        type=Path,
        default=Path('outputs/data/2020/units'),
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
        _reporter.report(f"Processing {len(states_to_process)} states (2020 block groups)")
    else:
        print(f"Processing {len(states_to_process)} states (block groups)...")
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
