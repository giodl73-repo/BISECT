#!/usr/bin/env python3
"""
Download Census places (cities/towns) data.

Example usage:
    python scripts/download_places.py --state CA --year 2020
"""

import argparse
import sys
from pathlib import Path
import geopandas as gpd
import requests
from tqdm import tqdm

def download_places(state_code: str, year: int = 2020, output_dir: str = 'data/raw'):
    """
    Download Census places for a state.

    Places include incorporated cities, towns, and Census Designated Places (CDPs).
    """
    print(f"\nDownloading {year} Census places for {state_code}...")

    # Complete state FIPS code mapping
    fips_map = {
        'AL': '01', 'AK': '02', 'AZ': '04', 'AR': '05', 'CA': '06',
        'CO': '08', 'CT': '09', 'DE': '10', 'FL': '12', 'GA': '13',
        'HI': '15', 'ID': '16', 'IL': '17', 'IN': '18', 'IA': '19',
        'KS': '20', 'KY': '21', 'LA': '22', 'ME': '23', 'MD': '24',
        'MA': '25', 'MI': '26', 'MN': '27', 'MS': '28', 'MO': '29',
        'MT': '30', 'NE': '31', 'NV': '32', 'NH': '33', 'NJ': '34',
        'NM': '35', 'NY': '36', 'NC': '37', 'ND': '38', 'OH': '39',
        'OK': '40', 'OR': '41', 'PA': '42', 'RI': '44', 'SC': '45',
        'SD': '46', 'TN': '47', 'TX': '48', 'UT': '49', 'VT': '50',
        'VA': '51', 'WA': '53', 'WV': '54', 'WI': '55', 'WY': '56',
        'DC': '11'
    }

    state_fips = fips_map.get(state_code.upper())
    if not state_fips:
        raise ValueError(f"Invalid state code: {state_code}. Must be a valid 2-letter state code.")

    # Download place boundaries from Census TIGER
    # URL structure differs between census years
    if year == 2010:
        # 2010 has different directory structure and filename pattern
        url = f"https://www2.census.gov/geo/tiger/TIGER2010/PLACE/2010/tl_2010_{state_fips}_place10.zip"
    elif year == 2000:
        # 2000 has different directory structure
        url = f"https://www2.census.gov/geo/tiger/TIGER2010/PLACE/2000/tl_2010_{state_fips}_place00.zip"
    else:
        # 2020 and other years
        url = f"https://www2.census.gov/geo/tiger/TIGER{year}/PLACE/tl_{year}_{state_fips}_place.zip"

    print(f"Fetching from: {url}")
    places_gdf = gpd.read_file(url)

    print(f"Downloaded {len(places_gdf)} places")

    # Column names differ by year
    if year == 2010:
        geoid_col = 'GEOID10'
        name_col = 'NAME10'
        namelsad_col = 'NAMELSAD10'
    elif year == 2000:
        geoid_col = 'PLCIDFP00'  # Place ID FP in 2000
        name_col = 'NAME00'
        namelsad_col = 'NAMELSAD00'
    else:
        geoid_col = 'GEOID'
        name_col = 'NAME'
        namelsad_col = 'NAMELSAD'

    # Fetch place-level population data from Census API
    print("Fetching place-level population data...")

    # API endpoints and variables differ by year
    if year == 2020:
        pop_url = f"https://api.census.gov/data/{year}/dec/pl?get=NAME,P1_001N&for=place:*&in=state:{state_fips}"
        pop_var = 'P1_001N'
    elif year == 2010:
        pop_url = f"https://api.census.gov/data/{year}/dec/sf1?get=NAME,P001001&for=place:*&in=state:{state_fips}"
        pop_var = 'P001001'
    elif year == 2000:
        pop_url = f"https://api.census.gov/data/{year}/sf1?get=NAME,P001001&for=place:*&in=state:{state_fips}"
        pop_var = 'P001001'
    else:
        print(f"Warning: Population API not configured for year {year}, using zeros")
        places_gdf['population'] = 0
        pop_var = None

    if pop_var:
        response = requests.get(pop_url)
        if not response.ok:
            print(f"Warning: Could not fetch population data: {response.status_code}")
            places_gdf['population'] = 0
        else:
            pop_data = response.json()
            header = pop_data[0]
            pop_dict = {}

            for row in pop_data[1:]:
                place_fips = row[header.index('place')]
                pop = int(row[header.index(pop_var)])
                # Create full GEOID (state + place)
                geoid = state_fips + place_fips
                pop_dict[geoid] = pop

            # Map population to GeoDataFrame
            places_gdf['population'] = places_gdf[geoid_col].map(pop_dict).fillna(0).astype(int)

            print(f"Successfully fetched population for {len(pop_dict)} places")

    # Rename columns to standard names
    places_gdf = places_gdf.rename(columns={
        geoid_col: 'GEOID',
        name_col: 'NAME',
        namelsad_col: 'NAMELSAD'
    })

    # Keep only relevant columns
    places_gdf = places_gdf[['GEOID', 'NAME', 'NAMELSAD', 'population', 'geometry']]

    # Filter out zero-population places (only if we have population data)
    # For years where API is unavailable (like 2000), keep all places
    if pop_var and places_gdf['population'].sum() > 0:
        places_gdf = places_gdf[places_gdf['population'] > 0]
    else:
        print(f"  Note: Keeping all places (population data unavailable for {year})")

    print(f"\nDataset Summary:")
    print(f"  Total places: {len(places_gdf)}")
    print(f"  Total population: {places_gdf['population'].sum():,}")

    # Show top 10 by population
    print(f"\nTop 10 places by population:")
    top_10 = places_gdf.nlargest(10, 'population')[['NAME', 'population']]
    for idx, row in top_10.iterrows():
        print(f"    {row['NAME']}: {row['population']:,}")

    # Save
    output_path = Path(output_dir)
    output_path.mkdir(parents=True, exist_ok=True)

    output_file = output_path / f"{state_code.lower()}_places_{year}.parquet"
    print(f"\nSaving to {output_file}...")
    places_gdf.to_parquet(output_file)

    file_size_mb = output_file.stat().st_size / 1e6
    print(f"Saved {len(places_gdf)} places ({file_size_mb:.1f} MB)")
    print(f"SUCCESS: {state_code} completed")


def main():
    parser = argparse.ArgumentParser(description='Download Census places (cities/towns)')
    parser.add_argument('--state', type=str, required=True, help='State code (e.g., CA)')
    parser.add_argument('--year', type=int, default=2020, help='Census year')
    parser.add_argument('--output-dir', type=str, default='data/raw', help='Output directory')

    args = parser.parse_args()

    try:
        download_places(args.state, args.year, args.output_dir)
        return 0
    except Exception as e:
        print(f"\nError: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == '__main__':
    sys.exit(main())
