#!/usr/bin/env python3
"""
Download Census Metropolitan/Micropolitan Statistical Areas (CBSAs) boundaries.

MSAs (Metropolitan Statistical Areas) and MCSAs (Micropolitan Statistical Areas)
are collectively called CBSAs (Core-Based Statistical Areas).

Example usage:
    python scripts/data/geography/download_metro_boundaries.py --year 2020
"""

import argparse
import sys
from pathlib import Path
import geopandas as gpd
import pandas as pd

def download_metro_boundaries(year: int = 2020, output_dir: str = 'data/raw'):
    """
    Download Census CBSA (MSA/MCSA) boundaries for the entire US.

    CBSAs include both:
    - MSAs (Metropolitan Statistical Areas) - urban areas with 50,000+ population
    - MCSAs (Micropolitan Statistical Areas) - urban areas with 10,000-49,999 population

    Args:
        year: Census year (2020, 2010, etc.)
        output_dir: Directory to save output file
    """
    print(f"\nDownloading {year} Census CBSA boundaries...")

    output_path = Path(output_dir)
    output_path.mkdir(parents=True, exist_ok=True)

    # Download CBSA boundaries from Census TIGER/Line
    # CBSAs are national-level files (not state-by-state)
    if year == 2010:
        # 2010 has different URL structure
        url = f"https://www2.census.gov/geo/tiger/TIGER2010/CBSA/2010/tl_2010_us_cbsa10.zip"
    elif year == 2020:
        # 2020 format
        url = f"https://www2.census.gov/geo/tiger/TIGER{year}/CBSA/tl_{year}_us_cbsa.zip"
    else:
        # Generic format for other years
        url = f"https://www2.census.gov/geo/tiger/TIGER{year}/CBSA/tl_{year}_us_cbsa.zip"

    print(f"Fetching from: {url}")

    try:
        cbsa_gdf = gpd.read_file(url)
        print(f"Downloaded {len(cbsa_gdf)} CBSAs")
    except Exception as e:
        print(f"ERROR: Failed to download CBSA boundaries")
        print(f"URL: {url}")
        print(f"Error: {e}")
        return False

    # Column names vary by year
    if year == 2010:
        geoid_col = 'GEOID10'
        name_col = 'NAME10'
        namelsad_col = 'NAMELSAD10'
        lsad_col = 'LSAD10'
    else:
        geoid_col = 'GEOID'
        name_col = 'NAME'
        namelsad_col = 'NAMELSAD'
        lsad_col = 'LSAD'

    # Standardize column names
    rename_map = {}
    if geoid_col != 'GEOID':
        rename_map[geoid_col] = 'GEOID'
    if name_col != 'NAME':
        rename_map[name_col] = 'NAME'
    if namelsad_col != 'NAMELSAD':
        rename_map[namelsad_col] = 'NAMELSAD'
    if lsad_col != 'LSAD':
        rename_map[lsad_col] = 'LSAD'

    if rename_map:
        cbsa_gdf = cbsa_gdf.rename(columns=rename_map)

    # LSAD codes:
    # M1 = Metropolitan Statistical Area
    # M2 = Micropolitan Statistical Area

    # Separate MSAs and MCSAs for reporting
    msas = cbsa_gdf[cbsa_gdf['LSAD'] == 'M1']
    mcsas = cbsa_gdf[cbsa_gdf['LSAD'] == 'M2']

    print(f"  MSAs (Metropolitan): {len(msas)}")
    print(f"  MCSAs (Micropolitan): {len(mcsas)}")

    # Ensure geometries are in EPSG:4326 (WGS84)
    if cbsa_gdf.crs != 'EPSG:4326':
        print(f"Converting from {cbsa_gdf.crs} to EPSG:4326...")
        cbsa_gdf = cbsa_gdf.to_crs('EPSG:4326')

    # Save to parquet
    output_file = output_path / f'us_cbsa_{year}.parquet'
    cbsa_gdf.to_parquet(output_file)

    print(f"\n{'='*70}")
    print(f"SUCCESS: CBSA boundaries saved to {output_file}")
    print(f"Total CBSAs: {len(cbsa_gdf)}")
    print(f"  - Metropolitan (MSAs): {len(msas)}")
    print(f"  - Micropolitan (MCSAs): {len(mcsas)}")
    print(f"{'='*70}\n")

    # Print top 20 MSAs by area (as a proxy for size/importance)
    msas_sorted = msas.copy()
    msas_sorted['area_sqkm'] = msas_sorted.geometry.to_crs('EPSG:5070').area / 1e6
    msas_sorted = msas_sorted.sort_values('area_sqkm', ascending=False)

    print("\nTop 20 Largest Metropolitan Areas (by geographic area):")
    print(f"{'Rank':<6} {'GEOID':<8} {'Name':<60} {'Area (sq km)':<12}")
    print('-' * 90)
    for idx, (_, row) in enumerate(msas_sorted.head(20).iterrows(), 1):
        print(f"{idx:<6} {row['GEOID']:<8} {row['NAME']:<60} {row['area_sqkm']:>12,.1f}")

    return True

def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description='Download Census CBSA (Metropolitan/Micropolitan Statistical Area) boundaries'
    )
    parser.add_argument(
        '--year',
        type=int,
        default=2020,
        choices=[2020, 2010, 2000],
        help='Census year (default: 2020)'
    )
    parser.add_argument(
        '--output-dir',
        type=str,
        default='data/raw',
        help='Output directory (default: data/raw)'
    )

    args = parser.parse_args()

    success = download_metro_boundaries(
        year=args.year,
        output_dir=args.output_dir
    )

    return 0 if success else 1

if __name__ == '__main__':
    sys.exit(main())
