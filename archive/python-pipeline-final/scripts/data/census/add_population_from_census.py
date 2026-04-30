#!/usr/bin/env python3
"""
Download and add Census 2020 PL 94-171 population data to block shapefiles.

This downloads the official redistricting data files directly from Census FTP,
no API key required.
"""

import sys
from pathlib import Path
import pandas as pd
import geopandas as gpd
from io import StringIO
import requests
from tqdm import tqdm

sys.path.insert(0, str(Path(__file__).parent.parent / 'src'))

from apportionment.data.census import load_blocks


def download_pl94171_data(state_fips: str, year: int = 2020) -> pd.DataFrame:
    """
    Download PL 94-171 redistricting data from Census website.

    Parameters
    ----------
    state_fips : str
        2-digit state FIPS code (e.g., '06' for California)
    year : int
        Census year (2020)

    Returns
    -------
    pd.DataFrame
        Block-level population data with GEOID and population
    """
    import zipfile
    import tempfile

    base_url = f"https://www2.census.gov/programs-surveys/decennial/2020/data/01-Redistricting_File--PL_94-171/"

    # State abbreviations for filenames
    state_abbrevs = {
        '01': 'al', '02': 'ak', '04': 'az', '05': 'ar',
        '06': 'ca', '08': 'co', '09': 'ct', '10': 'de',
        '12': 'fl', '13': 'ga', '15': 'hi', '16': 'id',
        '17': 'il', '18': 'in', '19': 'ia', '20': 'ks',
        '21': 'ky', '22': 'la', '23': 'me', '24': 'md',
        '25': 'ma', '26': 'mi', '27': 'mn', '28': 'ms',
        '29': 'mo', '30': 'mt', '31': 'ne', '32': 'nv',
        '33': 'nh', '34': 'nj', '35': 'nm', '36': 'ny',
        '37': 'nc', '38': 'nd', '39': 'oh', '40': 'ok',
        '41': 'or', '42': 'pa', '44': 'ri', '45': 'sc',
        '46': 'sd', '47': 'tn', '48': 'tx', '49': 'ut',
        '50': 'vt', '51': 'va', '53': 'wa', '54': 'wv',
        '55': 'wi', '56': 'wy'
    }

    # State names for directory
    state_names = {
        '01': 'Alabama', '02': 'Alaska', '04': 'Arizona', '05': 'Arkansas',
        '06': 'California', '08': 'Colorado', '09': 'Connecticut', '10': 'Delaware',
        '12': 'Florida', '13': 'Georgia', '15': 'Hawaii', '16': 'Idaho',
        '17': 'Illinois', '18': 'Indiana', '19': 'Iowa', '20': 'Kansas',
        '21': 'Kentucky', '22': 'Louisiana', '23': 'Maine', '24': 'Maryland',
        '25': 'Massachusetts', '26': 'Michigan', '27': 'Minnesota', '28': 'Mississippi',
        '29': 'Missouri', '30': 'Montana', '31': 'Nebraska', '32': 'Nevada',
        '33': 'New Hampshire', '34': 'New Jersey', '35': 'New Mexico', '36': 'New York',
        '37': 'North Carolina', '38': 'North Dakota', '39': 'Ohio', '40': 'Oklahoma',
        '41': 'Oregon', '42': 'Pennsylvania', '44': 'Rhode Island', '45': 'South Carolina',
        '46': 'South Dakota', '47': 'Tennessee', '48': 'Texas', '49': 'Utah',
        '50': 'Vermont', '51': 'Virginia', '53': 'Washington', '54': 'West Virginia',
        '55': 'Wisconsin', '56': 'Wyoming'
    }

    state_abbrev = state_abbrevs.get(state_fips)
    state_name = state_names.get(state_fips)
    if not state_abbrev or not state_name:
        raise ValueError(f"Unknown state FIPS: {state_fips}")

    # Download the ZIP file
    # Format: https://www2.census.gov/programs-surveys/decennial/2020/data/01-Redistricting_File--PL_94-171/California/ca2020.pl.zip
    zip_url = f"{base_url}{state_name}/{state_abbrev}2020.pl.zip"

    print(f"Downloading PL 94-171 data for {state_name}...")
    print(f"URL: {zip_url}")

    response = requests.get(zip_url, stream=True, timeout=120)
    response.raise_for_status()

    total_size = int(response.headers.get('content-length', 0))

    # Download with progress bar
    with tempfile.NamedTemporaryFile(delete=False, suffix='.zip') as tmp_file:
        with tqdm(total=total_size, unit='B', unit_scale=True, desc='Downloading') as pbar:
            for chunk in response.iter_content(chunk_size=8192):
                tmp_file.write(chunk)
                pbar.update(len(chunk))
        tmp_path = tmp_file.name

    # Extract and parse the ZIP file
    print("Extracting and parsing data...")

    with zipfile.ZipFile(tmp_path, 'r') as zip_ref:
        # List files in ZIP
        file_list = zip_ref.namelist()
        print(f"Files in ZIP: {file_list[:5]}...")  # Show first 5

        # Find the geographic header file and data file
        # Typical pattern: {state}geo2020.pl - geographic data
        #                  {state}000012020.pl - demographic data file 1
        geo_file = None
        data_file = None

        for fname in file_list:
            if 'geo2020.pl' in fname.lower():
                geo_file = fname
            elif '000012020.pl' in fname:
                data_file = fname

        if not geo_file or not data_file:
            raise ValueError(f"Could not find required files in ZIP. Files: {file_list}")

        print(f"Using geo file: {geo_file}")
        print(f"Using data file: {data_file}")

        # Read geographic file (contains GEOIDs)
        geo_data = zip_ref.read(geo_file).decode('latin-1')
        geo_df = pd.read_csv(StringIO(geo_data), delimiter='|', dtype=str, encoding='latin-1')

        # Read data file (contains population counts)
        data_data = zip_ref.read(data_file).decode('latin-1')
        data_df = pd.read_csv(StringIO(data_data), delimiter='|', dtype=str, encoding='latin-1')

    # Clean up temp file
    Path(tmp_path).unlink()

    print(f"Geo records: {len(geo_df):,}")
    print(f"Data records: {len(data_df):,}")
    print(f"Geo columns: {list(geo_df.columns)[:10]}...")
    print(f"Data columns: {list(data_df.columns)[:10]}...")

    # Filter to block level
    # Try different possible column names for summary level
    sumlev_col = None
    for col in ['SUMLEV', 'SUMLV', 'SUMMARY_LEVEL', 'SUMLEVEL']:
        if col in geo_df.columns:
            sumlev_col = col
            break

    if sumlev_col:
        geo_df = geo_df[geo_df[sumlev_col] == '750'].copy()
        print(f"Block-level records (SUMLEV=750): {len(geo_df):,}")
    else:
        # If no summary level column, use all records and hope they're blocks
        print(f"Warning: Could not find SUMLEV column, using all records")
        print(f"Available columns: {list(geo_df.columns)}")

    # Merge geo and data on LOGRECNO
    merged = geo_df.merge(data_df, on=['FILEID', 'STUSAB', 'CHARITER', 'CIFSN', 'LOGRECNO'], how='inner')

    print(f"Merged records: {len(merged):,}")

    # Extract GEOID (15-digit block code)
    # GEOID is in the GEOCODE column or construct from components
    if 'GEOCODE' in merged.columns:
        merged['GEOID'] = merged['GEOCODE']
    else:
        # Construct: STATE(2) + COUNTY(3) + TRACT(6) + BLOCK(4)
        merged['GEOID'] = (
            merged['STATE'].str.zfill(2) +
            merged['COUNTY'].str.zfill(3) +
            merged['TRACT'].str.zfill(6) +
            merged['BLOCK'].str.zfill(4)
        )

    # P0010001 is total population
    merged['population'] = pd.to_numeric(merged['P0010001'], errors='coerce').fillna(0).astype(int)

    result = merged[['GEOID', 'population']].copy()

    print(f"Extracted population for {len(result):,} blocks")
    print(f"Total population: {result['population'].sum():,}")

    return result


def add_population_to_blocks(blocks_file: str, state_fips: str, output_file: str = None):
    """
    Add Census population data to existing block shapefile.

    Parameters
    ----------
    blocks_file : str
        Path to blocks parquet file
    state_fips : str
        2-digit state FIPS code
    output_file : str, optional
        Output file path (default: overwrite input)
    """
    # Load existing blocks
    print(f"\nLoading blocks from {blocks_file}...")
    blocks_gdf = load_blocks(blocks_file)

    print(f"Current population: {blocks_gdf['population'].sum():,}")

    # Download population data
    pop_df = download_pl94171_data(state_fips)

    # Merge with blocks
    print("\nMerging population data with blocks...")
    blocks_gdf = blocks_gdf.drop(columns=['population'], errors='ignore')
    blocks_gdf = blocks_gdf.merge(pop_df, on='GEOID', how='left')
    blocks_gdf['population'] = blocks_gdf['population'].fillna(0).astype(int)

    # Summary
    print("\nPopulation Summary:")
    print(f"  Total blocks: {len(blocks_gdf):,}")
    print(f"  Blocks with population > 0: {(blocks_gdf['population'] > 0).sum():,}")
    print(f"  Total population: {blocks_gdf['population'].sum():,}")
    print(f"  Population range: {blocks_gdf['population'].min()} - {blocks_gdf['population'].max():,}")

    # Save
    if output_file is None:
        output_file = blocks_file

    print(f"\nSaving to {output_file}...")
    blocks_gdf.to_parquet(output_file, compression='snappy')
    print(f"Done! File size: {Path(output_file).stat().st_size / 1e6:.1f} MB")


def main():
    import argparse

    parser = argparse.ArgumentParser(description='Add Census PL 94-171 population data to blocks')
    parser.add_argument('blocks_file', help='Input blocks parquet file')
    parser.add_argument('state_fips', help='2-digit state FIPS code (e.g., 06 for CA)')
    parser.add_argument('--output', help='Output file (default: overwrite input)')

    args = parser.parse_args()

    try:
        add_population_to_blocks(args.blocks_file, args.state_fips, args.output)
        return 0
    except Exception as e:
        print(f"\nError: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == '__main__':
    sys.exit(main())
