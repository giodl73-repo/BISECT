#!/usr/bin/env python3
"""
Add Census 2020 population data using cenpy (no API key required).
"""

import sys
from pathlib import Path
import pandas as pd
import geopandas as gpd
from tqdm import tqdm

sys.path.insert(0, str(Path(__file__).parent.parent / 'src'))

from apportionment.data.census import load_blocks


def get_population_cenpy(state_fips: str) -> pd.DataFrame:
    """
    Get block-level population using cenpy.

    Parameters
    ----------
    state_fips : str
        2-digit state FIPS code

    Returns
    -------
    pd.DataFrame
        Block-level population with GEOID and population
    """
    import cenpy

    print(f"Fetching block-level population for state FIPS {state_fips}...")
    print("This uses the Census PL 94-171 dataset (no API key required)")

    # Connect to 2020 PL dataset
    conn = cenpy.remote.APIConnection('DECENNIALPL2020')

    # Get all counties in the state first
    print("Getting list of counties...")
    counties_data = conn.query(
        cols=['NAME', 'P1_001N'],
        geo_unit='county',
        geo_filter={'state': state_fips}
    )

    county_fips_list = [row['county'] for _, row in counties_data.iterrows()]
    print(f"Found {len(county_fips_list)} counties")

    # Fetch blocks for each county (Census API requires county-level queries for blocks)
    all_blocks = []

    print("Fetching block data by county...")
    for county_fips in tqdm(county_fips_list, desc="Counties"):
        try:
            blocks_data = conn.query(
                cols=['GEO_ID', 'P1_001N'],  # GEO_ID and total population
                geo_unit='block',
                geo_filter={'state': state_fips, 'county': county_fips}
            )

            all_blocks.append(blocks_data)
        except Exception as e:
            print(f"Warning: Failed to fetch county {county_fips}: {e}")
            continue

    # Combine all blocks
    if not all_blocks:
        raise ValueError("Failed to fetch any block data")

    blocks_df = pd.concat(all_blocks, ignore_index=True)

    # Extract GEOID from GEO_ID (format: "1000000US<15-digit-geoid>")
    blocks_df['GEOID'] = blocks_df['GEO_ID'].str.replace('1000000US', '')

    # Rename population column
    blocks_df['population'] = blocks_df['P1_001N'].astype(int)

    result = blocks_df[['GEOID', 'population']].copy()

    print(f"\nTotal blocks: {len(result):,}")
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

    # Get population data
    pop_df = get_population_cenpy(state_fips)

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

    parser = argparse.ArgumentParser(description='Add Census population data using cenpy')
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
