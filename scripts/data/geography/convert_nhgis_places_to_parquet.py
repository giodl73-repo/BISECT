#!/usr/bin/env python3
"""
Convert NHGIS 2000 Census places data to standardized parquet format.

This script takes NHGIS shapefiles with 2000 Census place boundaries and
population data, and converts them to the standardized format used by the
redistricting pipeline.

Expected NHGIS format:
- Shapefile with place boundaries
- Population column (typically H7V001, P001001, or similar)
- State and place FIPS codes
- Place names

Output format (consistent with 2010/2020):
- Columns: GEOID, NAME, NAMELSAD, population, geometry
- File: data/raw/{state}_places_2000.parquet

Usage:
    python convert_nhgis_places_to_parquet.py --input nhgis_places_2000.shp
    python convert_nhgis_places_to_parquet.py --input nhgis_dir/ --output-dir data/raw
"""

import argparse
import sys
from pathlib import Path
import geopandas as gpd
from tqdm import tqdm


# State FIPS to abbreviation mapping
FIPS_TO_STATE = {
    '01': 'AL', '02': 'AK', '04': 'AZ', '05': 'AR', '06': 'CA',
    '08': 'CO', '09': 'CT', '10': 'DE', '11': 'DC', '12': 'FL',
    '13': 'GA', '15': 'HI', '16': 'ID', '17': 'IL', '18': 'IN',
    '19': 'IA', '20': 'KS', '21': 'KY', '22': 'LA', '23': 'ME',
    '24': 'MD', '25': 'MA', '26': 'MI', '27': 'MN', '28': 'MS',
    '29': 'MO', '30': 'MT', '31': 'NE', '32': 'NV', '33': 'NH',
    '34': 'NJ', '35': 'NM', '36': 'NY', '37': 'NC', '38': 'ND',
    '39': 'OH', '40': 'OK', '41': 'OR', '42': 'PA', '44': 'RI',
    '45': 'SC', '46': 'SD', '47': 'TN', '48': 'TX', '49': 'UT',
    '50': 'VT', '51': 'VA', '53': 'WA', '54': 'WV', '55': 'WI',
    '56': 'WY'
}


def identify_columns(gdf):
    """
    Identify which columns in the NHGIS data correspond to our standard fields.

    Returns dict mapping: {'geoid': col_name, 'name': col_name, 'pop': col_name, ...}
    """
    cols = {c.upper(): c for c in gdf.columns}

    result = {}

    # GEOID - try various patterns
    for pattern in ['GISJOIN', 'GEOID', 'PLACE_FIPS', 'PLACEFP', 'PLCIDFP']:
        if pattern in cols:
            result['geoid'] = cols[pattern]
            break

    # State FIPS
    for pattern in ['STATEFP', 'STATE_FIPS', 'STFIPS', 'STATEA']:
        if pattern in cols:
            result['state_fips'] = cols[pattern]
            break

    # Place FIPS
    for pattern in ['PLACEFP', 'PLACE_FIPS', 'PLACEA']:
        if pattern in cols:
            result['place_fips'] = cols[pattern]
            break

    # Name
    for pattern in ['NAME', 'PLACENAME', 'PLACE_NAME']:
        if pattern in cols:
            result['name'] = cols[pattern]
            break

    # NAMELSAD
    for pattern in ['NAMELSAD', 'NAME_LSAD', 'PLACE_LSAD']:
        if pattern in cols:
            result['namelsad'] = cols[pattern]
            break

    # Population - try various NHGIS codes
    for pattern in ['FL5001', 'H7V001', 'P001001', 'H76001', 'POPULATION', 'POP2000', 'TOTAL_POP', 'NP001A001', 'NP001A']:
        if pattern in cols:
            result['population'] = cols[pattern]
            break

    return result


def convert_nhgis_to_standard(gdf, column_map):
    """
    Convert NHGIS data to standardized format.

    Returns GeoDataFrame with columns: GEOID, NAME, NAMELSAD, population, geometry
    """
    result = gpd.GeoDataFrame()

    # GEOID - construct if needed
    if 'geoid' in column_map:
        result['GEOID'] = gdf[column_map['geoid']].astype(str)
    elif 'state_fips' in column_map and 'place_fips' in column_map:
        result['GEOID'] = (gdf[column_map['state_fips']].astype(str) +
                           gdf[column_map['place_fips']].astype(str))
    else:
        raise ValueError("Cannot construct GEOID - need either GEOID column or STATE_FIPS + PLACE_FIPS")

    # Clean GEOID - remove non-numeric characters if present (GISJOIN format)
    if result['GEOID'].str.contains('G', na=False).any():
        # GISJOIN format like "G01001234" -> "01001234"
        # Just remove the 'G' prefix; the rest is already correct
        result['GEOID'] = result['GEOID'].str.replace('G', '', regex=False)

    # NAME
    if 'name' in column_map:
        result['NAME'] = gdf[column_map['name']].astype(str)
    else:
        raise ValueError("Cannot find NAME column")

    # NAMELSAD
    if 'namelsad' in column_map:
        result['NAMELSAD'] = gdf[column_map['namelsad']].astype(str)
    else:
        # Construct from NAME if not available
        result['NAMELSAD'] = result['NAME'] + ' city'

    # Population
    if 'population' in column_map:
        result['population'] = gdf[column_map['population']].astype(int)
    else:
        print("WARNING: No population column found - setting to 0")
        result['population'] = 0

    # Geometry
    result['geometry'] = gdf.geometry

    # Set CRS if not present
    if result.crs is None:
        result.set_crs('EPSG:4326', inplace=True)

    return result


def process_nhgis_file(input_path, output_dir, csv_path=None):
    """
    Process a single NHGIS shapefile containing all states.

    Splits into per-state parquet files.
    """
    print(f"\nReading NHGIS data from {input_path}...")
    gdf = gpd.read_file(input_path)

    print(f"Loaded {len(gdf)} places")
    print(f"Columns: {list(gdf.columns)}")

    # Join with CSV data if provided
    if csv_path:
        import pandas as pd
        print(f"\nReading population data from {csv_path}...")
        csv_df = pd.read_csv(csv_path)
        print(f"Loaded {len(csv_df)} records")

        # Join on GISJOIN
        print("Joining shapefile with population data...")
        gdf = gdf.merge(csv_df[['GISJOIN', 'FL5001']], on='GISJOIN', how='left')
        print(f"After join: {len(gdf)} places")

    # Identify columns
    print("\nIdentifying columns...")
    column_map = identify_columns(gdf)
    print("Column mapping:")
    for key, val in column_map.items():
        print(f"  {key}: {val}")

    if not column_map:
        print("\nERROR: Could not identify required columns!")
        print("Please specify column names manually using --geoid-col, --name-col, --pop-col")
        return False

    # Convert to standard format
    print("\nConverting to standard format...")
    std_gdf = convert_nhgis_to_standard(gdf, column_map)

    # Extract state FIPS from GEOID (first 2 digits)
    std_gdf['state_fips'] = std_gdf['GEOID'].str[:2]

    # Split by state and save
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    states_processed = []
    states_skipped = []

    print(f"\nSaving per-state files to {output_dir}...")
    for state_fips, state_code in tqdm(FIPS_TO_STATE.items(), desc="Processing states"):
        state_data = std_gdf[std_gdf['state_fips'] == state_fips].copy()

        if len(state_data) == 0:
            states_skipped.append(state_code)
            continue

        # Drop the temporary state_fips column
        state_data = state_data[['GEOID', 'NAME', 'NAMELSAD', 'population', 'geometry']]

        # Save
        output_file = output_dir / f"{state_code.lower()}_places_2000.parquet"
        state_data.to_parquet(output_file)

        states_processed.append(state_code)

    print(f"\n{'='*60}")
    print(f"Conversion Complete")
    print(f"{'='*60}")
    print(f"States processed: {len(states_processed)}")
    print(f"Total places: {len(std_gdf)}")
    print(f"Output directory: {output_dir}")

    if states_skipped:
        print(f"\nStates with no data: {', '.join(states_skipped)}")

    return True


def main():
    parser = argparse.ArgumentParser(
        description='Convert NHGIS 2000 Census places to standardized parquet format'
    )
    parser.add_argument('--input', type=str, required=True,
                       help='Path to NHGIS shapefile (.shp)')
    parser.add_argument('--csv', type=str,
                       help='Path to NHGIS CSV with population data (optional)')
    parser.add_argument('--output-dir', type=str, default='data/raw',
                       help='Output directory for parquet files (default: data/raw)')

    # Manual column specification (if auto-detection fails)
    parser.add_argument('--geoid-col', type=str,
                       help='Name of GEOID column (if auto-detection fails)')
    parser.add_argument('--name-col', type=str,
                       help='Name of NAME column (if auto-detection fails)')
    parser.add_argument('--pop-col', type=str,
                       help='Name of population column (if auto-detection fails)')

    args = parser.parse_args()

    input_path = Path(args.input)
    if not input_path.exists():
        print(f"ERROR: Input file not found: {input_path}")
        return 1

    csv_path = None
    if args.csv:
        csv_path = Path(args.csv)
        if not csv_path.exists():
            print(f"ERROR: CSV file not found: {csv_path}")
            return 1

    try:
        success = process_nhgis_file(input_path, args.output_dir, csv_path)
        return 0 if success else 1
    except Exception as e:
        print(f"\nERROR: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == '__main__':
    sys.exit(main())
