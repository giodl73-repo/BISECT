#!/usr/bin/env python3
"""
Merge 2010 census tract population data with TIGER/Line geometries.

Combines:
- Population CSV from parse_pl94171_tracts_2010.py
- Shapefile geometries from download_tiger_tracts_2010.py

Produces:
- Final parquet files: [state]_tracts_2010.parquet
"""

import argparse
import sys
from pathlib import Path
import pandas as pd
import geopandas as gpd
from tqdm import tqdm

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


def merge_state_data(state_code, state_fips, census_dir, tiger_dir, output_dir):
    """Merge census population data with shapefile geometries for one state."""

    state_lower = state_code.lower()

    # Input files
    pop_csv = census_dir / f"{state_lower}_tracts_2010_population.csv"
    shapefile = tiger_dir / f"tl_2010_{state_fips}_tract10" / f"tl_2010_{state_fips}_tract10.shp"

    if not pop_csv.exists():
        print(f"  ERROR: Population CSV not found: {pop_csv}")
        return False

    if not shapefile.exists():
        print(f"  ERROR: Shapefile not found: {shapefile}")
        return False

    # Read population data
    print(f"  Reading population data...")
    pop_df = pd.read_csv(pop_csv, dtype={'GEOID': str})

    # Read shapefile
    print(f"  Reading shapefile...")
    gdf = gpd.read_file(shapefile)

    # The 2010 TIGER/Line shapefiles use GEOID10 field
    if 'GEOID10' not in gdf.columns:
        print(f"  ERROR: GEOID10 column not found in shapefile")
        print(f"  Available columns: {list(gdf.columns)}")
        return False

    # Rename GEOID10 to GEOID for consistency
    gdf = gdf.rename(columns={'GEOID10': 'GEOID'})

    # Convert GEOID to string for consistent merging
    gdf['GEOID'] = gdf['GEOID'].astype(str)
    pop_df['GEOID'] = pop_df['GEOID'].astype(str)

    # Merge on GEOID
    print(f"  Merging data...")
    merged = gdf[['GEOID', 'geometry']].merge(pop_df, on='GEOID', how='inner')

    if len(merged) == 0:
        print(f"  ERROR: No matching records after merge")
        print(f"  Shapefile records: {len(gdf)}")
        print(f"  Population records: {len(pop_df)}")
        return False

    if len(merged) < len(pop_df):
        print(f"  WARNING: Some tracts have no geometry ({len(pop_df)} tracts, {len(merged)} with geometry)")

    # Ensure CRS is set (should be EPSG:4269 for TIGER/Line 2010)
    if merged.crs is None:
        merged = merged.set_crs('EPSG:4269')

    # Save to parquet
    output_file = output_dir / f"{state_lower}_tracts_2010.parquet"
    merged.to_parquet(output_file, index=False)

    print(f"  Saved {len(merged)} tracts to {output_file}")
    print(f"  CRS: {merged.crs}")

    return True


def main():
    parser = argparse.ArgumentParser(
        description='Merge 2010 census tract data with geometries'
    )
    parser.add_argument(
        '--census-dir',
        type=Path,
        default=Path('data/processed/census_2010'),
        help='Directory containing population CSV files'
    )
    parser.add_argument(
        '--tiger-dir',
        type=Path,
        default=Path('data/geography/tiger_2010_tracts'),
        help='Directory containing TIGER/Line shapefiles'
    )
    parser.add_argument(
        '--output-dir',
        type=Path,
        default=Path('data/tracts/2010'),
        help='Output directory for merged parquet files'
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
        states_to_process = [(s.upper(), STATE_FIPS[s.upper()]) for s in args.states]
    else:
        states_to_process = [(code, fips) for code, fips in sorted(STATE_FIPS.items())]

    print(f"Merging {len(states_to_process)} states...")
    print(f"Census directory: {args.census_dir}")
    print(f"TIGER directory: {args.tiger_dir}")
    print(f"Output directory: {args.output_dir}")
    print()

    # Process each state
    success_count = 0
    failed_states = []

    for state_code, state_fips in tqdm(states_to_process, desc="Merging states"):
        print(f"\n{state_code}:")
        if merge_state_data(state_code, state_fips, args.census_dir, args.tiger_dir, args.output_dir):
            success_count += 1
        else:
            failed_states.append(state_code)

    # Summary
    print("\n" + "=" * 60)
    print(f"Merged {success_count}/{len(states_to_process)} states successfully")
    if failed_states:
        print(f"Failed states: {', '.join(failed_states)}")
    print("=" * 60)


if __name__ == '__main__':
    main()
