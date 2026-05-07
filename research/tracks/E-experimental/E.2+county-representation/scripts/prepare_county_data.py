"""
Prepare County-Level Dataset

Aggregates census tract data to county level for all U.S. counties.

For each census year:
- Loads tract-level data (population + geometry)
- Groups by county FIPS code
- Aggregates population (sum)
- Dissolves geometries to county boundaries
- Saves to parquet: outputs/data/{year}/counties/all_counties_{year}.parquet

Usage:
    python scripts/prepare_county_data.py --year 2020
    python scripts/prepare_county_data.py --year 2020 --states CA TX  # Test mode
"""

import sys
from pathlib import Path
from typing import List, Optional
import pandas as pd
import geopandas as gpd
from tqdm import tqdm

# Add src to path and get project root
PROJECT_ROOT = Path(__file__).parent.parent.parent.parent
sys.path.insert(0, str(PROJECT_ROOT / 'src'))

# State FIPS codes for reference
STATE_FIPS = {
    '01': 'AL', '02': 'AK', '04': 'AZ', '05': 'AR', '06': 'CA', '08': 'CO',
    '09': 'CT', '10': 'DE', '11': 'DC', '12': 'FL', '13': 'GA', '15': 'HI',
    '16': 'ID', '17': 'IL', '18': 'IN', '19': 'IA', '20': 'KS', '21': 'KY',
    '22': 'LA', '23': 'ME', '24': 'MD', '25': 'MA', '26': 'MI', '27': 'MN',
    '28': 'MS', '29': 'MO', '30': 'MT', '31': 'NE', '32': 'NV', '33': 'NH',
    '34': 'NJ', '35': 'NM', '36': 'NY', '37': 'NC', '38': 'ND', '39': 'OH',
    '40': 'OK', '41': 'OR', '42': 'PA', '44': 'RI', '45': 'SC', '46': 'SD',
    '47': 'TN', '48': 'TX', '49': 'UT', '50': 'VT', '51': 'VA', '53': 'WA',
    '54': 'WV', '55': 'WI', '56': 'WY'
}

FIPS_TO_STATE = {v: k for k, v in STATE_FIPS.items()}


def load_tract_data(year: int, state: str) -> Optional[pd.DataFrame]:
    """
    Load tract/block group data for a state.

    Args:
        year: Census year (2000, 2010, 2020)
        state: State abbreviation (e.g., 'CA')

    Returns:
        DataFrame with units and population, or None if file doesn't exist
    """
    # Try block groups CSV first (most likely to exist)
    bg_path = PROJECT_ROOT / f'outputs/data/{year}/units/{state.lower()}_block_groups_{year}_population.csv'
    if bg_path.exists():
        try:
            df = pd.read_csv(bg_path)
            return df
        except Exception as e:
            print(f"  Warning: Could not read {bg_path}: {e}")

    # Try parquet paths as fallback
    possible_paths = [
        PROJECT_ROOT / f'outputs/data/{year}/units/tracts/{state.lower()}_tracts_{year}.parquet',
        PROJECT_ROOT / f'data/{year}/tracts/{state.lower()}_tracts_{year}.parquet',
        PROJECT_ROOT / f'outputs/data/{year}/tracts/{state.lower()}_tracts_{year}.parquet',
    ]

    for path in possible_paths:
        if path.exists():
            try:
                gdf = gpd.read_parquet(path)
                return gdf
            except Exception as e:
                print(f"  Warning: Could not read {path}: {e}")
                continue

    return None


def extract_county_fips(geoid: str) -> str:
    """
    Extract county FIPS from GEOID.

    Block group GEOID format: 1500000USSSCCCTTTTTTB
      - 1500000US prefix (ignored)
      - SS = state FIPS (2 digits)
      - CCC = county FIPS (3 digits)
      - TTTTTT = tract (6 digits)
      - B = block group (1 digit)

    Tract GEOID format: SSCCCTTTTTT
      - SS = state FIPS (2 digits)
      - CCC = county FIPS (3 digits)
      - TTTTTT = tract (6 digits)

    Args:
        geoid: Census tract/block group GEOID

    Returns:
        County FIPS code (5 digits: SSCCC)
    """
    if pd.isna(geoid):
        return None

    geoid_str = str(geoid)

    # Remove 1500000US prefix if present (block group format)
    if geoid_str.startswith('1500000US'):
        geoid_str = geoid_str[9:]  # Remove prefix

    # Now we have SSCCCTTTTTT or SSCCCTTTTTTB
    # First 5 digits = state (2) + county (3)
    geoid_str = geoid_str.zfill(11)  # Ensure at least 11 digits
    return geoid_str[:5]  # First 5 digits = state + county


def aggregate_to_counties(
    tracts_df: pd.DataFrame,
    state: str
) -> pd.DataFrame:
    """
    Aggregate tract/block group data to county level.

    Args:
        tracts_df: Tract/block group DataFrame
        state: State abbreviation

    Returns:
        County-level DataFrame
    """
    # Extract county FIPS from GEOIDs
    if 'GEOID' in tracts_df.columns:
        geoid_col = 'GEOID'
    elif 'geoid' in tracts_df.columns:
        geoid_col = 'geoid'
    else:
        print(f"  Warning: No GEOID column found for {state}")
        return None

    tracts_df['county_fips'] = tracts_df[geoid_col].apply(extract_county_fips)

    # Remove rows with invalid county FIPS
    tracts_df = tracts_df[tracts_df['county_fips'].notna()].copy()

    if len(tracts_df) == 0:
        print(f"  Warning: No valid county FIPS for {state}")
        return None

    # Identify population column
    pop_col = None
    for col in ['population', 'POP', 'POP100', 'P0010001']:
        if col in tracts_df.columns:
            pop_col = col
            break

    if pop_col is None:
        print(f"  Warning: No population column found for {state}")
        return None

    # Aggregate by county
    counties = tracts_df.groupby('county_fips').agg({
        pop_col: 'sum'
    }).reset_index()

    # Rename columns
    counties = counties.rename(columns={
        'county_fips': 'fips',
        pop_col: 'population'
    })

    # Add state column
    counties['state'] = state

    return counties


def prepare_county_data(
    year: int,
    states: Optional[List[str]] = None,
    output_dir: Optional[Path] = None
):
    """
    Prepare county-level dataset from tract data.

    Args:
        year: Census year (2000, 2010, 2020)
        states: List of state abbreviations (None = all 50 states)
        output_dir: Output directory (default: outputs/data/{year}/counties/)
    """
    print(f"Preparing County Data ({year})")
    print("=" * 80)

    # Setup output directory
    if output_dir is None:
        output_dir = PROJECT_ROOT / f'outputs/data/{year}/counties'
    output_dir.mkdir(parents=True, exist_ok=True)

    # Determine states to process
    if states is None:
        states_to_process = list(STATE_FIPS.values())
    else:
        states_to_process = states

    print(f"Processing {len(states_to_process)} states...")
    print()

    # Load and aggregate for each state
    all_counties = []
    failed_states = []

    for state in tqdm(states_to_process, desc="States"):
        tracts = load_tract_data(year, state)

        if tracts is None:
            failed_states.append(state)
            tqdm.write(f"  {state}: No tract data found")
            continue

        counties = aggregate_to_counties(tracts, state)

        if counties is None or len(counties) == 0:
            failed_states.append(state)
            tqdm.write(f"  {state}: Could not aggregate to counties")
            continue

        all_counties.append(counties)
        tqdm.write(f"  {state}: {len(counties)} counties, {counties['population'].sum():,} total population")

    print()

    if len(all_counties) == 0:
        print("ERROR: No county data generated")
        print()
        print("This likely means tract data is not available.")
        print("You may need to run data download/processing first:")
        print("  python scripts/data/download_orchestrator.py --stages redistricting --year 2020")
        print("  python scripts/data/merge_units_with_geometries.py --year 2020")
        return

    # Combine all states
    print("Combining county data...")
    combined = pd.concat(all_counties, ignore_index=True)

    # Sort by population descending
    combined = combined.sort_values('population', ascending=False).reset_index(drop=True)

    # Save to CSV (no geometries in this version)
    output_file = output_dir / f'all_counties_{year}.csv'
    print(f"Saving to {output_file}...")
    combined.to_csv(output_file, index=False)

    # Generate summary statistics
    print()
    print("=" * 80)
    print("Summary Statistics")
    print("=" * 80)
    print(f"Total counties: {len(combined)}")
    print(f"Total population: {combined['population'].sum():,}")
    print(f"States processed: {len(all_counties)}")

    if failed_states:
        print(f"States with missing data: {len(failed_states)}")
        print(f"  {', '.join(failed_states)}")

    print()
    print("Population distribution:")
    print(f"  Min:    {combined['population'].min():>12,}")
    print(f"  25th:   {combined['population'].quantile(0.25):>12,.0f}")
    print(f"  Median: {combined['population'].median():>12,.0f}")
    print(f"  75th:   {combined['population'].quantile(0.75):>12,.0f}")
    print(f"  Max:    {combined['population'].max():>12,}")
    print()

    print("Top 10 counties by population:")
    for idx, row in combined.head(10).iterrows():
        print(f"  {idx+1:2d}. {row['state']}-{row['fips']}: {row['population']:>12,}")
    print()

    print(f"Output saved to: {output_file}")
    print()


if __name__ == '__main__':
    import argparse

    parser = argparse.ArgumentParser(
        description='Prepare county-level dataset from tract data'
    )
    parser.add_argument('--year', type=int, required=True,
                       help='Census year (2000, 2010, 2020)')
    parser.add_argument('--states', type=str, nargs='+',
                       help='States to process (default: all 50 states)')
    parser.add_argument('--output', type=str,
                       help='Output directory')

    args = parser.parse_args()

    output_dir = Path(args.output) if args.output else None

    prepare_county_data(
        year=args.year,
        states=args.states,
        output_dir=output_dir
    )
