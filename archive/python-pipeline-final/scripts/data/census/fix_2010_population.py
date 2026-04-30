#!/usr/bin/env python3
"""
Fix 2010 census tract files that have zero population.

This script:
1. Identifies tract files with zero population
2. Fetches only the population data from Census API
3. Updates the parquet files in place
"""

import geopandas as gpd
import pandas as pd
from pathlib import Path
import requests
import time
from tqdm import tqdm

# State FIPS codes (2-digit)
STATE_FIPS = {
    'AL': '01', 'AK': '02', 'AZ': '04', 'AR': '05', 'CA': '06', 'CO': '08',
    'CT': '09', 'DE': '10', 'FL': '12', 'GA': '13', 'HI': '15', 'ID': '16',
    'IL': '17', 'IN': '18', 'IA': '19', 'KS': '20', 'KY': '21', 'LA': '22',
    'ME': '23', 'MD': '24', 'MA': '25', 'MI': '26', 'MN': '27', 'MS': '28',
    'MO': '29', 'MT': '30', 'NE': '31', 'NV': '32', 'NH': '33', 'NJ': '34',
    'NM': '35', 'NY': '36', 'NC': '37', 'ND': '38', 'OH': '39', 'OK': '40',
    'OR': '41', 'PA': '42', 'RI': '44', 'SC': '45', 'SD': '46', 'TN': '47',
    'TX': '48', 'UT': '49', 'VT': '50', 'VA': '51', 'WA': '53', 'WV': '54',
    'WI': '55', 'WY': '56'
}


def fetch_population_for_state(state_code, delay=2.0):
    """Fetch population data for a state from Census API."""

    state_fips = STATE_FIPS[state_code]

    # Try to fetch all tracts in one request
    url = f"https://api.census.gov/data/2010/dec/sf1?get=GEO_ID,P001001&for=tract:*&in=state:{state_fips}"

    try:
        response = requests.get(url)
        response.raise_for_status()
        data = response.json()

        # Parse response
        headers = data[0]
        rows = data[1:]

        df = pd.DataFrame(rows, columns=headers)

        # Extract GEOID from GEO_ID (format: 1400000US + 11-digit GEOID)
        df['GEOID'] = df['GEO_ID'].str[-11:]
        df['population'] = pd.to_numeric(df['P001001'], errors='coerce')

        result = df[['GEOID', 'population']].set_index('GEOID')

        time.sleep(delay)
        return result

    except Exception as e:
        print(f"  ERROR: {e}")
        time.sleep(delay * 2)  # Longer wait on error
        return None


def fix_state_population(state_file, delay=2.0):
    """Fix population data for a single state."""

    state_code = state_file.stem.split('_')[0].upper()

    print(f"\n{state_code}:", end=" ")

    # Load existing file
    gdf = gpd.read_parquet(state_file)

    # Check if needs fixing
    if gdf['population'].sum() > 0:
        print("✓ Already has population")
        return True

    print(f"Fetching population for {len(gdf):,} tracts...", end=" ")

    # Fetch population
    pop_data = fetch_population_for_state(state_code, delay=delay)

    if pop_data is None:
        print("✗ Failed")
        return False

    # Update population in geodataframe
    gdf['GEOID_str'] = gdf['GEOID'].astype(str).str.zfill(11)
    gdf['population'] = gdf['GEOID_str'].map(pop_data['population']).fillna(0).astype(int)
    gdf = gdf.drop('GEOID_str', axis=1)

    total_pop = gdf['population'].sum()

    if total_pop == 0:
        print(f"✗ Still zero (API rate limit?)")
        return False

    # Save updated file
    gdf.to_parquet(state_file)
    print(f"✓ Fixed! Total pop: {total_pop:,}")

    return True


def main():
    print("=" * 70)
    print("FIX 2010 CENSUS POPULATION DATA")
    print("=" * 70)

    data_dir = Path('data/raw')

    # Find all 2010 tract files
    tract_files = sorted(data_dir.glob('*_tracts_2010.parquet'))

    print(f"Found {len(tract_files)} 2010 tract files")

    # Identify files needing fixes
    needs_fix = []

    for f in tract_files:
        gdf = gpd.read_parquet(f)
        if gdf['population'].sum() == 0:
            needs_fix.append(f)

    print(f"States needing population fix: {len(needs_fix)}")

    if len(needs_fix) == 0:
        print("\n✓ All states already have population data!")
        return 0

    print(f"\nStates to fix: {', '.join([f.stem.split('_')[0].upper() for f in needs_fix])}")
    print(f"\nUsing 2-second delay between requests to avoid rate limits...")
    print("=" * 70)

    # Fix each state
    successful = []
    failed = []

    for state_file in needs_fix:
        success = fix_state_population(state_file, delay=2.0)

        if success:
            successful.append(state_file.stem.split('_')[0].upper())
        else:
            failed.append(state_file.stem.split('_')[0].upper())

    # Summary
    print("\n" + "=" * 70)
    print("SUMMARY")
    print("=" * 70)
    print(f"Fixed: {len(successful)}/{len(needs_fix)}")
    print(f"Failed: {len(failed)}/{len(needs_fix)}")

    if successful:
        print(f"\n✓ Successfully fixed: {', '.join(successful)}")

    if failed:
        print(f"\n✗ Still need fixing: {', '.join(failed)}")
        print(f"\nIf this hit rate limits, wait 10-15 minutes and run again.")

    print("=" * 70)

    return 0 if len(failed) == 0 else 1


if __name__ == '__main__':
    exit(main())
