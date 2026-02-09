#!/usr/bin/env python3
"""
Improved Census tract downloader with rate limiting and retry logic.

Key improvements:
- Delays between API requests to avoid rate limiting
- Exponential backoff retry logic for failed requests
- Progress saving for resumable downloads
- Better error handling and reporting
"""

import argparse
import sys
import time
from pathlib import Path
from typing import Optional
import json

sys.path.insert(0, str(Path(__file__).parent.parent / 'src'))

import geopandas as gpd
import pandas as pd
from tqdm import tqdm


def download_tracts_improved(
    state: str,
    year: int = 2020,
    cache_dir: str = 'cache',
    output_dir: str = 'data/raw',
    delay_between_requests: float = 3.0,
    max_retries: int = 5,
    resume: bool = True
):
    """
    Download Census tract shapefiles with population (with improved rate limiting).

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
    delay_between_requests : float
        Seconds to wait between API requests (default: 1.5)
    max_retries : int
        Maximum retries for failed requests (default: 5)
    resume : bool
        Resume from partial download if available (default: True)
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
    print(f"Rate limiting: {delay_between_requests}s delay between API requests")

    # Download tracts
    tracts_gdf = tracts(state=state, year=year, cache=True)

    print(f"Downloaded {len(tracts_gdf):,} tracts")

    # Extract relevant columns based on year
    if year == 2020:
        geoid_col = 'GEOID'
        name_col = 'NAME' if 'NAME' in tracts_gdf.columns else 'NAMELSAD'
        aland_col = 'ALAND'
        awater_col = 'AWATER'
        intptlat_col = 'INTPTLAT' if 'INTPTLAT' in tracts_gdf.columns else None
        intptlon_col = 'INTPTLON' if 'INTPTLON' in tracts_gdf.columns else None
    elif year == 2010:
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

    # Add population data with improved rate limiting
    print("Fetching tract-level population data...")
    tracts_gdf = _add_tract_population_improved(
        tracts_gdf, state, year,
        delay_between_requests=delay_between_requests,
        max_retries=max_retries,
        resume=resume
    )

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

    # Clean up progress file if successful
    progress_file = Path(cache_dir) / f'{state.lower()}_{year}_progress.json'
    if progress_file.exists():
        progress_file.unlink()

    return tracts_gdf


def _add_tract_population_improved(
    tracts_gdf,
    state,
    year,
    delay_between_requests=1.5,
    max_retries=5,
    resume=True
):
    """Add population data from Census API with improved rate limiting."""
    try:
        import cenpy

        # Get state FIPS
        state_fips = _state_to_fips(state)

        print(f"Fetching population for {len(tracts_gdf):,} tracts...")

        # Connect to Census API based on year
        if year == 2020:
            api_name = 'DECENNIALPL2020'
            pop_var = 'P1_001N'
        elif year == 2010:
            api_name = 'DECENNIALSF12010'
            pop_var = 'P001001'
        else:
            print(f"Warning: Population API not configured for year {year}, using zeros")
            tracts_gdf['population'] = 0
            return tracts_gdf

        conn = cenpy.remote.APIConnection(api_name)

        # Get counties
        print("Fetching county list...")
        counties_data = conn.query(
            cols=['NAME'],
            geo_unit='county',
            geo_filter={'state': state_fips}
        )

        county_fips_list = [row['county'] for _, row in counties_data.iterrows()]
        print(f"Found {len(county_fips_list)} counties")

        # Check for resume file
        progress_file = Path('cache') / f'{state.lower()}_{year}_progress.json'
        completed_counties = set()

        if resume and progress_file.exists():
            with open(progress_file, 'r') as f:
                progress_data = json.load(f)
                completed_counties = set(progress_data.get('completed_counties', []))
                print(f"Resuming: {len(completed_counties)} counties already completed")

        all_tract_data = []
        failed_counties = []

        for county_fips in tqdm(county_fips_list, desc="Fetching counties"):
            # Skip if already completed
            if county_fips in completed_counties:
                continue

            # Try with exponential backoff
            retry_count = 0
            success = False

            while retry_count < max_retries and not success:
                try:
                    # Add delay before API request (not on first retry)
                    if retry_count > 0:
                        backoff_delay = delay_between_requests * (2 ** retry_count)
                        time.sleep(backoff_delay)
                    else:
                        time.sleep(delay_between_requests)

                    # Make API request
                    tract_data = conn.query(
                        cols=['GEO_ID', pop_var],
                        geo_unit='tract',
                        geo_filter={'state': state_fips, 'county': county_fips}
                    )

                    # VALIDATION: Check that we got valid data
                    if tract_data.empty:
                        raise ValueError(f"Empty response for county {county_fips}")

                    if pop_var not in tract_data.columns:
                        raise ValueError(f"Population column '{pop_var}' not in response")

                    # Validate we got actual tract data (not error messages)
                    if 'GEO_ID' not in tract_data.columns:
                        raise ValueError(f"GEO_ID column not in response for county {county_fips}")

                    num_tracts = len(tract_data)
                    if num_tracts == 0:
                        raise ValueError(f"No tracts returned for county {county_fips}")

                    # Check that we have at least some non-zero population
                    # Convert to numeric first (API returns strings)
                    try:
                        pop_sum = pd.to_numeric(tract_data[pop_var], errors='coerce').sum()
                        if pop_sum <= 0:
                            tqdm.write(f"Warning: County {county_fips} has zero population (likely uninhabited)")
                    except Exception:
                        # If conversion fails, that's ok - we'll catch it in the merge step
                        pass

                    all_tract_data.append(tract_data)
                    completed_counties.add(county_fips)
                    success = True

                    # Save progress every 5 counties
                    if len(completed_counties) % 5 == 0:
                        progress_file.parent.mkdir(parents=True, exist_ok=True)
                        with open(progress_file, 'w') as f:
                            json.dump({'completed_counties': list(completed_counties)}, f)

                except Exception as e:
                    retry_count += 1
                    if retry_count >= max_retries:
                        tqdm.write(f"Failed county {county_fips} after {max_retries} attempts: {e}")
                        failed_counties.append(county_fips)
                    else:
                        tqdm.write(f"Retry {retry_count}/{max_retries} for county {county_fips}: {e}")

        # Final progress save
        if completed_counties:
            with open(progress_file, 'w') as f:
                json.dump({'completed_counties': list(completed_counties)}, f)

        if failed_counties:
            print(f"\nWarning: {len(failed_counties)} counties failed after retries")
            print(f"Failed counties: {failed_counties}")

        if not all_tract_data:
            print("Warning: Could not fetch any population data, using zeros")
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

        success_count = (tracts_gdf['population'] > 0).sum()
        print(f"Successfully fetched population for {success_count:,} / {len(tracts_gdf):,} tracts "
              f"({success_count/len(tracts_gdf)*100:.1f}%)")

        return tracts_gdf

    except Exception as e:
        print(f"Error fetching population: {e}")
        import traceback
        traceback.print_exc()
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
    parser = argparse.ArgumentParser(
        description='Download Census tract data with improved rate limiting'
    )
    parser.add_argument('--state', type=str, required=True, help='State code (e.g., CA)')
    parser.add_argument('--year', type=int, default=2020, help='Census year (default: 2020)')
    parser.add_argument('--cache-dir', type=str, default='cache', help='Cache directory')
    parser.add_argument('--output-dir', type=str, default='data/raw', help='Output directory')
    parser.add_argument('--delay', type=float, default=3.0,
                        help='Delay between API requests in seconds (default: 3.0)')
    parser.add_argument('--max-retries', type=int, default=5,
                        help='Maximum retries for failed requests (default: 5)')
    parser.add_argument('--no-resume', action='store_true',
                        help='Do not resume from partial download')

    args = parser.parse_args()

    try:
        download_tracts_improved(
            args.state,
            args.year,
            args.cache_dir,
            args.output_dir,
            delay_between_requests=args.delay,
            max_retries=args.max_retries,
            resume=not args.no_resume
        )
        return 0
    except Exception as e:
        print(f"\nError: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == '__main__':
    sys.exit(main())
