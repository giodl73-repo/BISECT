#!/usr/bin/env python3
"""
Merge 2000 census tract population data with NHGIS geometries.

Combines:
- Population CSV from parse_pl94171_tracts_2000.py
- NHGIS shapefile geometries (manually downloaded)

Produces:
- Final parquet files: [state]_tracts_2000.parquet

IMPORTANT: This script requires NHGIS 2000 tract shapefiles.
Run download_tiger_tracts_2000.py for download instructions.
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


def find_shapefile(tiger_dir, state_code, state_fips):
    """Find NHGIS shapefile for a state, trying multiple naming patterns."""

    state_lower = state_code.lower()

    # Pattern 1: Processed by download_tiger_tracts_2000.py --process-nhgis
    shapefile = tiger_dir / f"{state_lower}_tracts_2000.shp"
    if shapefile.exists():
        return shapefile

    # Pattern 2: NHGIS standard naming (state FIPS in filename)
    candidates = list(tiger_dir.rglob(f"*{state_fips}*tract*.shp"))
    if candidates:
        return candidates[0]

    # Pattern 3: NHGIS extract directories
    nhgis_dirs = list(tiger_dir.glob("nhgis*"))
    for nhgis_dir in nhgis_dirs:
        # Look for shapefiles with state FIPS or state code
        candidates = list(nhgis_dir.rglob(f"*{state_fips}*.shp"))
        candidates.extend(list(nhgis_dir.rglob(f"*{state_code}*.shp")))

        # Filter for tract-related shapefiles
        tract_files = [f for f in candidates if 'tract' in f.stem.lower()]
        if tract_files:
            return tract_files[0]

        # If no explicit "tract" in name, try any shapefile
        if candidates:
            return candidates[0]

    return None


def merge_state_data(state_code, state_fips, census_dir, tiger_dir, output_dir):
    """Merge census population data with NHGIS geometries for one state."""

    state_lower = state_code.lower()

    # Input files
    pop_csv = census_dir / f"{state_lower}_tracts_2000_population.csv"

    if not pop_csv.exists():
        print(f"  ERROR: Population CSV not found: {pop_csv}")
        return False

    # Find shapefile (NHGIS has variable naming)
    shapefile = find_shapefile(tiger_dir, state_code, state_fips)

    if shapefile is None:
        print(f"  ERROR: Shapefile not found in {tiger_dir}")
        print(f"  Have you downloaded NHGIS 2000 tract shapefiles?")
        print(f"  Run: python scripts/data/geography/download_tiger_tracts_2000.py")
        return False

    print(f"  Using shapefile: {shapefile.name}")

    # Read population data
    print(f"  Reading population data...")
    pop_df = pd.read_csv(pop_csv, dtype={'GEOID': str})

    # Read shapefile
    print(f"  Reading shapefile...")
    gdf = gpd.read_file(shapefile)

    # NHGIS 2000 shapefiles may use different column names
    # Common patterns: GEOID, GISJOIN, NHGISST + NHGISCTY + NHGISTRT
    geoid_col = None

    if 'GEOID' in gdf.columns:
        geoid_col = 'GEOID'
    elif 'GEOID00' in gdf.columns:
        geoid_col = 'GEOID00'
    elif 'CTIDFP00' in gdf.columns:
        geoid_col = 'CTIDFP00'
    elif 'GISJOIN' in gdf.columns and 'NHGISST' in gdf.columns and 'NHGISCTY' in gdf.columns:
        # NHGIS GISJOIN format: G + SSS (3) + CCCC (4) + TTTTTT (6) = 14 chars
        # NHGISST and NHGISCTY have trailing zeros (values multiplied by 10)
        # Example: G0800010007800 -> state=080/10=08, county=0010/10=001, tract=007800
        print(f"  Converting NHGIS GISJOIN to GEOID...")
        gdf['GEOID'] = (
            (gdf['NHGISST'].astype(int) // 10).astype(str).str.zfill(2) +
            (gdf['NHGISCTY'].astype(int) // 10).astype(str).str.zfill(3) +
            gdf['GISJOIN'].str[8:14]  # Last 6 digits are the tract
        )
        geoid_col = 'GEOID'
    elif all(col in gdf.columns for col in ['STATEFP00', 'COUNTYFP00', 'TRACTCE00']):
        # Construct GEOID from components
        print(f"  Constructing GEOID from components...")
        gdf['GEOID'] = gdf['STATEFP00'].astype(str) + gdf['COUNTYFP00'].astype(str) + gdf['TRACTCE00'].astype(str)
        geoid_col = 'GEOID'
    else:
        print(f"  ERROR: Cannot identify GEOID column in shapefile")
        print(f"  Available columns: {list(gdf.columns)}")
        return False

    # Rename to standard GEOID if needed
    if geoid_col != 'GEOID':
        gdf = gdf.rename(columns={geoid_col: 'GEOID'})

    # Convert GEOID to string for consistent merging
    gdf['GEOID'] = gdf['GEOID'].astype(str)

    # Merge on GEOID
    print(f"  Merging data...")
    merged = gdf[['GEOID', 'geometry']].merge(pop_df, on='GEOID', how='inner')

    if len(merged) == 0:
        print(f"  ERROR: No matching records after merge")
        print(f"  Shapefile records: {len(gdf)}")
        print(f"  Population records: {len(pop_df)}")
        print(f"  Sample shapefile GEOIDs: {list(gdf['GEOID'].head())}")
        print(f"  Sample population GEOIDs: {list(pop_df['GEOID'].head())}")
        return False

    if len(merged) < len(pop_df):
        print(f"  WARNING: Some tracts have no geometry ({len(pop_df)} tracts, {len(merged)} with geometry)")

    # Ensure CRS is set (NHGIS typically uses EPSG:4326 or EPSG:4269)
    if merged.crs is None:
        merged = merged.set_crs('EPSG:4269')

    # Save to parquet
    output_file = output_dir / f"{state_lower}_tracts_2000.parquet"
    merged.to_parquet(output_file, index=False)

    print(f"  Saved {len(merged)} tracts to {output_file}")
    print(f"  CRS: {merged.crs}")

    return True


def main():
    parser = argparse.ArgumentParser(
        description='Merge 2000 census tract data with NHGIS geometries'
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
        default=Path('data/geography/tiger_2000_tracts'),
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
        print("Run for instructions:")
        print("  python scripts/data/geography/download_tiger_tracts_2000.py")
        return 1

    # Create output directory
    args.output_dir.mkdir(parents=True, exist_ok=True)

    # Determine which states to process
    if args.states:
        states_to_process = [(s.upper(), STATE_FIPS[s.upper()]) for s in args.states]
    else:
        states_to_process = [(code, fips) for code, fips in sorted(STATE_FIPS.items())]

    print(f"Merging {len(states_to_process)} states...")
    print(f"Census directory: {args.census_dir}")
    print(f"NHGIS directory: {args.tiger_dir}")
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

    return 0 if success_count > 0 else 1


if __name__ == '__main__':
    sys.exit(main())
