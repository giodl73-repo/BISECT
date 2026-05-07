#!/usr/bin/env python3
"""
Download Census tract shapefiles and population data.

Example usage:
    python scripts/download_tracts.py --state CA --year 2020
"""

import argparse
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent / 'src'))

import geopandas as gpd
import pandas as pd
from tqdm import tqdm


def download_tracts(
    state: str,
    year: int = 2020,
    cache_dir: str = 'cache',
    output_dir: str = 'data/raw'
):
    """
    Download Census tract shapefiles with population.

    Parameters
    ----------
    state : str
        State code (e.g., 'CA')
    year : int
        Census year
    cache_dir : str
        Cache directory
    output_dir : str
        Output directory
    """
    try:
        import pygris
        from pygris import tracts
    except ImportError:
        raise ImportError("pygris required: pip install pygris")

    import os
    os.environ['PYGRIS_CACHE_DIR'] = cache_dir
    pygris.cache = True

    print(f"\nDownloading {year} Census tracts for {state}...")

    # Download tracts
    tracts_gdf = tracts(state=state, year=year, cache=True)

    print(f"Downloaded {len(tracts_gdf):,} tracts")
    print(f"Available columns: {list(tracts_gdf.columns)}")

    # Extract relevant columns based on year
    if year == 2020:
        geoid_col = 'GEOID'
        name_col = 'NAME' if 'NAME' in tracts_gdf.columns else 'NAMELSAD'
        aland_col = 'ALAND'
        awater_col = 'AWATER'
        intptlat_col = 'INTPTLAT' if 'INTPTLAT' in tracts_gdf.columns else None
        intptlon_col = 'INTPTLON' if 'INTPTLON' in tracts_gdf.columns else None
    elif year == 2010:
        # 2010 uses GEOID10, NAME10, ALAND10, AWATER10
        geoid_col = 'GEOID10' if 'GEOID10' in tracts_gdf.columns else 'GEOID'
        name_col = 'NAME10' if 'NAME10' in tracts_gdf.columns else 'NAME' if 'NAME' in tracts_gdf.columns else 'NAMELSAD10'
        aland_col = 'ALAND10' if 'ALAND10' in tracts_gdf.columns else 'ALAND'
        awater_col = 'AWATER10' if 'AWATER10' in tracts_gdf.columns else 'AWATER'
        intptlat_col = 'INTPTLAT10' if 'INTPTLAT10' in tracts_gdf.columns else 'INTPTLAT' if 'INTPTLAT' in tracts_gdf.columns else None
        intptlon_col = 'INTPTLON10' if 'INTPTLON10' in tracts_gdf.columns else 'INTPTLON' if 'INTPTLON' in tracts_gdf.columns else None
    else:
        geoid_col = 'GEOID'
        name_col = 'NAME'
        aland_col = 'ALAND'
        awater_col = 'AWATER'
        intptlat_col = 'INTPTLAT' if 'INTPTLAT' in tracts_gdf.columns else None
        intptlon_col = 'INTPTLON' if 'INTPTLON' in tracts_gdf.columns else None

    # Select columns
    cols_to_keep = [geoid_col, name_col, aland_col, awater_col, 'geometry']
    if intptlat_col and intptlat_col in tracts_gdf.columns:
        cols_to_keep.extend([intptlat_col, intptlon_col])

    tracts_gdf = tracts_gdf[cols_to_keep].copy()

    # Rename to standard names
    rename_dict = {
        geoid_col: 'GEOID',
        name_col: 'NAME',
        aland_col: 'ALAND',
        awater_col: 'AWATER',
    }
    if intptlat_col:
        rename_dict[intptlat_col] = 'INTPTLAT'
        rename_dict[intptlon_col] = 'INTPTLON'

    tracts_gdf = tracts_gdf.rename(columns=rename_dict)

    # Add population data
    print("Fetching tract-level population data...")
    tracts_gdf = _add_tract_population(tracts_gdf, state, year)

    # Add computed fields
    tracts_gdf['total_area'] = tracts_gdf['ALAND'] + tracts_gdf['AWATER']
    tracts_gdf['water_fraction'] = tracts_gdf['AWATER'] / tracts_gdf['total_area'].replace(0, 1)

    # Summary
    print("\nDataset Summary:")
    print(f"  Total tracts: {len(tracts_gdf):,}")
    print(f"  Tracts with population > 0: {(tracts_gdf['population'] > 0).sum():,}")
    print(f"  Total population: {tracts_gdf['population'].sum():,}")
    print(f"  Total land area: {tracts_gdf['ALAND'].sum() / 1e6:,.0f} km²")

    # Save
    output_path = Path(output_dir)
    output_path.mkdir(parents=True, exist_ok=True)

    filename = f"{state.lower()}_tracts_{year}.parquet"
    output_file = output_path / filename

    print(f"\nSaving to {output_file}...")
    tracts_gdf.to_parquet(output_file, compression='snappy')
    print(f"Saved {len(tracts_gdf):,} tracts ({output_file.stat().st_size / 1e6:.1f} MB)")

    return tracts_gdf


def _add_tract_population(tracts_gdf, state, year):
    """Add population data from Census API."""
    try:
        import cenpy

        # Get state FIPS
        state_fips = _state_to_fips(state)

        print(f"Fetching population for {len(tracts_gdf):,} tracts...")

        # Connect to Census API based on year
        if year == 2020:
            api_name = 'DECENNIALPL2020'
            pop_var = 'P1_001N'  # Total population
        elif year == 2010:
            api_name = 'DECENNIALSF12010'
            pop_var = 'P001001'  # Total population
        else:
            print(f"Warning: Population API not configured for year {year}, using zeros")
            tracts_gdf['population'] = 0
            return tracts_gdf

        conn = cenpy.remote.APIConnection(api_name)

        # Get tract-level population
        # First get counties
        counties_data = conn.query(
            cols=['NAME'],
            geo_unit='county',
            geo_filter={'state': state_fips}
        )

        county_fips_list = [row['county'] for _, row in counties_data.iterrows()]

        all_tract_data = []

        for county_fips in tqdm(county_fips_list, desc="Counties"):
            try:
                tract_data = conn.query(
                    cols=['GEO_ID', pop_var],
                    geo_unit='tract',
                    geo_filter={'state': state_fips, 'county': county_fips}
                )
                all_tract_data.append(tract_data)
            except Exception as e:
                print(f"Warning: Failed to fetch county {county_fips}: {e}")
                continue

        if not all_tract_data:
            print("Warning: Could not fetch population data, using zeros")
            tracts_gdf['population'] = 0
            return tracts_gdf

        # Combine all tract data
        pop_df = pd.concat(all_tract_data, ignore_index=True)

        # Extract GEOID from GEO_ID
        if year == 2020:
            pop_df['GEOID'] = pop_df['GEO_ID'].str.replace('1400000US', '')
        else:  # 2010
            pop_df['GEOID'] = pop_df['GEO_ID'].str.replace('1400000US', '')

        pop_df['population'] = pop_df[pop_var].astype(int)

        # Merge with tracts
        tracts_gdf = tracts_gdf.merge(
            pop_df[['GEOID', 'population']],
            on='GEOID',
            how='left'
        )
        tracts_gdf['population'] = tracts_gdf['population'].fillna(0).astype(int)

        print(f"Successfully fetched population for {(tracts_gdf['population'] > 0).sum():,} tracts")

        return tracts_gdf

    except Exception as e:
        print(f"Error fetching population: {e}")
        print("Using zero population")
        tracts_gdf['population'] = 0
        return tracts_gdf


def _state_to_fips(state: str) -> str:
    """Convert state abbreviation to FIPS code."""
    state_fips = {
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
    return state_fips.get(state.upper(), state)


def main():
    parser = argparse.ArgumentParser(description='Download Census tract data')
    parser.add_argument('--state', type=str, required=True, help='State code (e.g., CA)')
    parser.add_argument('--year', type=int, default=2020, help='Census year (default: 2020)')
    parser.add_argument('--cache-dir', type=str, default='cache', help='Cache directory')
    parser.add_argument('--output-dir', type=str, default='data/raw', help='Output directory')

    args = parser.parse_args()

    try:
        download_tracts(args.state, args.year, args.cache_dir, args.output_dir)
        return 0
    except Exception as e:
        print(f"\nError: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == '__main__':
    sys.exit(main())
