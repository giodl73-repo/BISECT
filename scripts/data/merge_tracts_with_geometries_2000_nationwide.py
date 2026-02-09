#!/usr/bin/env python3
"""
Merge 2000 census tract population data with NHGIS geometries.

Optimized version that loads the nationwide NHGIS shapefile once,
then filters by state for merging.

Combines:
- Population CSV from parse_pl94171_tracts_2000.py
- NHGIS shapefile geometries (manually downloaded)

Produces:
- Final parquet files: [state]_tracts_2000.parquet
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


def find_nationwide_shapefile(tiger_dir):
    """Find the nationwide NHGIS shapefile."""

    # Look for US_tract_2000.shp in nhgis directories
    candidates = list(tiger_dir.rglob("US_tract_2000.shp"))
    if candidates:
        return candidates[0]

    # Try other patterns
    candidates = list(tiger_dir.rglob("*tract*2000*.shp"))
    us_files = [f for f in candidates if f.stem.startswith('US')]
    if us_files:
        return us_files[0]

    return None


def merge_state_data(state_code, state_fips, pop_csv, state_gdf, output_dir):
    """Merge census population data with pre-filtered NHGIS geometries for one state."""

    state_lower = state_code.lower()

    if not pop_csv.exists():
        print(f"  ERROR: Population file not found: {pop_csv}")
        return False

    # Load population data
    print(f"  Reading population data ({pop_csv.name})...")
    pop_df = pd.read_csv(pop_csv)

    if len(pop_df) == 0:
        print(f"  ERROR: No population data found")
        return False

    # Convert population GEOIDs to strings with proper padding
    pop_df['GEOID'] = pop_df['GEOID'].astype(str).str.zfill(11)

    # State GDF already filtered and has GEOID column
    print(f"  Shapefile records: {len(state_gdf)}")
    print(f"  Population records: {len(pop_df)}")

    # Merge on GEOID
    print(f"  Merging data...")
    merged = state_gdf[['GEOID', 'geometry']].merge(pop_df, on='GEOID', how='inner')

    if len(merged) == 0:
        print(f"  ERROR: No matching records after merge")
        print(f"  Sample shapefile GEOIDs: {list(state_gdf['GEOID'].head())}")
        print(f"  Sample population GEOIDs: {list(pop_df['GEOID'].head())}")
        return False

    if len(merged) < len(pop_df):
        missing = len(pop_df) - len(merged)
        print(f"  WARNING: {missing} tracts have no geometry ({len(pop_df)} tracts, {len(merged)} with geometry)")

    # Convert CRS to standard EPSG:4269 (NAD83)
    # NHGIS shapefiles use ESRI projection codes which cause issues with pyproj
    if merged.crs is None:
        merged = merged.set_crs('EPSG:4269')
    else:
        # Convert any CRS to EPSG:4269 for consistency
        merged = merged.to_crs('EPSG:4269')

    # Save to parquet
    output_file = output_dir / f"{state_lower}_tracts_2000.parquet"
    merged.to_parquet(output_file, index=False)

    print(f"  Saved {len(merged)} tracts to {output_file.name}")
    print(f"  CRS: {merged.crs}")

    return True


def main():
    parser = argparse.ArgumentParser(
        description='Merge 2000 census tract data with NHGIS geometries (nationwide shapefile version)'
    )
    parser.add_argument(
        '--census-dir',
        type=Path,
        default=Path('data/processed/census_2000'),
        help='Directory containing population CSV files'
    )
    parser.add_argument(
        '--tiger-dir',
        type=Path,
        default=Path('data/geography/nhgis_2000_tracts'),
        help='Directory containing NHGIS shapefiles'
    )
    parser.add_argument(
        '--output-dir',
        type=Path,
        default=Path('data/tracts/2000'),
        help='Output directory for merged parquet files'
    )
    parser.add_argument(
        '--states',
        type=str,
        nargs='+',
        help='Process specific states (e.g., AL CA TX). If not specified, processes all states.'
    )

    args = parser.parse_args()

    # Check if NHGIS shapefiles exist
    if not args.tiger_dir.exists():
        print(f"ERROR: NHGIS directory not found: {args.tiger_dir}")
        print()
        print("You need to download 2000 census tract shapefiles from NHGIS first.")
        print("Go to: https://nhgis.org/")
        return 1

    # Find nationwide shapefile
    print("Looking for nationwide NHGIS shapefile...")
    shapefile = find_nationwide_shapefile(args.tiger_dir)

    if not shapefile:
        print(f"ERROR: Nationwide tract shapefile not found in {args.tiger_dir}")
        print("Expected: US_tract_2000.shp")
        return 1

    print(f"Found: {shapefile}")
    print()

    # Load nationwide shapefile (do this once!)
    print("Loading nationwide shapefile (this may take a minute)...")
    gdf_nationwide = gpd.read_file(shapefile)
    print(f"Loaded {len(gdf_nationwide)} tracts nationwide")

    # Convert NHGIS GISJOIN to standard GEOID
    if 'GISJOIN' in gdf_nationwide.columns and 'NHGISST' in gdf_nationwide.columns and 'NHGISCTY' in gdf_nationwide.columns:
        print("Converting NHGIS GISJOIN to GEOID...")
        # NHGIS format: G + SSS (3) + CCCC (4) + TTTTTT (6)
        # NHGISST and NHGISCTY have trailing zeros (multiplied by 10)
        gdf_nationwide['GEOID'] = (
            (gdf_nationwide['NHGISST'].astype(int) // 10).astype(str).str.zfill(2) +
            (gdf_nationwide['NHGISCTY'].astype(int) // 10).astype(str).str.zfill(3) +
            gdf_nationwide['GISJOIN'].str[8:14]  # Last 6 digits are the tract
        )
        print("Conversion complete")
    elif 'GEOID' not in gdf_nationwide.columns:
        print("ERROR: Cannot identify GEOID column in shapefile")
        print(f"Available columns: {list(gdf_nationwide.columns)}")
        return 1

    # Create state FIPS lookup for filtering
    # NHGISST contains state * 10, so we need to map back
    gdf_nationwide['state_fips'] = (gdf_nationwide['NHGISST'].astype(int) // 10).astype(str).str.zfill(2)
    print()

    # Create output directory
    args.output_dir.mkdir(parents=True, exist_ok=True)

    # Determine which states to process
    if args.states:
        states_to_process = [(s.upper(), STATE_FIPS[s.upper()]) for s in args.states]
    else:
        states_to_process = [(code, fips) for code, fips in sorted(STATE_FIPS.items())]

    print(f"Merging {len(states_to_process)} states...")
    print(f"Census directory: {args.census_dir}")
    print(f"Output directory: {args.output_dir}")
    print()

    # Process each state
    success_count = 0
    failed_states = []

    for state_code, state_fips in tqdm(states_to_process, desc="Merging states"):
        print(f"\n{state_code}:")

        # Filter nationwide GDF for this state
        state_gdf = gdf_nationwide[gdf_nationwide['state_fips'] == state_fips].copy()

        if len(state_gdf) == 0:
            print(f"  ERROR: No tracts found for state {state_code} (FIPS {state_fips})")
            failed_states.append(state_code)
            continue

        # Population CSV
        state_lower = state_code.lower()
        pop_csv = args.census_dir / f"{state_lower}_tracts_2000_population.csv"

        # Merge
        if merge_state_data(state_code, state_fips, pop_csv, state_gdf, args.output_dir):
            success_count += 1
        else:
            failed_states.append(state_code)

    # Summary
    print("\n" + "=" * 60)
    print(f"Merged {success_count}/{len(states_to_process)} states successfully")
    if failed_states:
        print(f"Failed states: {', '.join(failed_states)}")
    print("=" * 60)

    return 0 if success_count > 0 else 1


if __name__ == '__main__':
    sys.exit(main())
