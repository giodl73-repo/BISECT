#!/usr/bin/env python3
"""
Fill in missing city data for districts that don't have an identified city.
Uses ANY place/CDP in the district, or falls back to tract centroids.
"""

import pandas as pd
import geopandas as gpd
import pickle
from pathlib import Path

# State configuration
STATE_CONFIG = {
    'CA': 'California', 'TX': 'Texas', 'FL': 'Florida', 'NY': 'New York',
    'PA': 'Pennsylvania', 'IL': 'Illinois', 'OH': 'Ohio', 'GA': 'Georgia',
    'NC': 'North Carolina', 'MI': 'Michigan', 'NJ': 'New Jersey', 'VA': 'Virginia',
    'WA': 'Washington', 'AZ': 'Arizona', 'MA': 'Massachusetts', 'TN': 'Tennessee',
    'IN': 'Indiana', 'MD': 'Maryland', 'MO': 'Missouri', 'WI': 'Wisconsin',
    'CO': 'Colorado', 'MN': 'Minnesota', 'SC': 'South Carolina', 'AL': 'Alabama',
    'LA': 'Louisiana', 'KY': 'Kentucky', 'OR': 'Oregon', 'OK': 'Oklahoma',
    'CT': 'Connecticut', 'UT': 'Utah', 'IA': 'Iowa', 'NV': 'Nevada',
    'AR': 'Arkansas', 'MS': 'Mississippi', 'KS': 'Kansas', 'NM': 'New Mexico',
    'NE': 'Nebraska', 'ID': 'Idaho', 'WV': 'West Virginia', 'HI': 'Hawaii',
    'NH': 'New Hampshire', 'ME': 'Maine', 'RI': 'Rhode Island', 'MT': 'Montana',
    'DE': 'Delaware', 'SD': 'South Dakota', 'ND': 'North Dakota', 'AK': 'Alaska',
    'VT': 'Vermont', 'WY': 'Wyoming'
}


def find_any_place_in_district(state_code, district_num):
    """Find ANY place in a district, even small ones."""

    state_name = STATE_CONFIG[state_code]
    state_dir = Path(f'outputs/us_2020_redistricting/{state_name.lower().replace(" ", "_")}')

    # Load tract and place data
    tracts_file = f'data/raw/{state_code.lower()}_tracts_2020.parquet'
    places_file = f'data/raw/{state_code.lower()}_places_2020.parquet'
    assignments_file = state_dir / 'data' / 'final_assignments.pkl'

    if not Path(tracts_file).exists() or not Path(places_file).exists():
        return None

    tracts = gpd.read_parquet(tracts_file)
    places = gpd.read_parquet(places_file)

    # Load assignments
    with open(assignments_file, 'rb') as f:
        assignments = pickle.load(f)

    tracts['district'] = [assignments[i] for i in range(len(tracts))]

    # Get district tracts
    district_tracts = tracts[tracts['district'] == district_num]

    if len(district_tracts) == 0:
        return None

    # Ensure same CRS
    if tracts.crs != places.crs:
        places = places.to_crs(tracts.crs)

    # Spatial join to find ANY place in this district (no population filter)
    places_points = places.copy()
    places_points['geometry'] = places_points.geometry.representative_point()

    places_in_district = gpd.sjoin(
        places_points,
        district_tracts[['geometry', 'district']],
        how='inner',
        predicate='within'
    )

    if len(places_in_district) > 0:
        # Take the most populous place
        largest = places_in_district.nlargest(1, 'population').iloc[0]
        centroid = largest.geometry
        return {
            'name': largest['NAME'],
            'population': int(largest['population']),
            'lon': centroid.x,
            'lat': centroid.y
        }

    # Fallback: Use the most populous tract centroid and label as "Rural District"
    most_populous_tract = district_tracts.nlargest(1, 'population').iloc[0]
    centroid = most_populous_tract.geometry.representative_point()

    return {
        'name': f'Rural District (Tract {most_populous_tract["GEOID"][-6:]})',
        'population': 0,
        'lon': centroid.x,
        'lat': centroid.y
    }


def main():
    """Fill in missing city data."""

    print("\n" + "="*70)
    print("Filling Missing City Data")
    print("="*70)

    # Load current data
    us_districts = pd.read_csv('outputs/us_2020_redistricting/us_all_districts.csv')

    # Find missing
    missing_mask = us_districts['city_lon'].isna()
    missing = us_districts[missing_mask]

    print(f"\nDistricts missing city data: {len(missing)}/435")
    print(f"\nProcessing {len(missing)} districts...")

    filled_count = 0

    for idx, row in missing.iterrows():
        state_code = row['state_code']
        state_name = row['state']
        district = row['district']

        print(f"  {state_name} District {district}...", end=" ")

        place_info = find_any_place_in_district(state_code, district)

        if place_info:
            us_districts.at[idx, 'largest_city'] = place_info['name']
            us_districts.at[idx, 'city_population'] = place_info['population']
            us_districts.at[idx, 'city_lon'] = place_info['lon']
            us_districts.at[idx, 'city_lat'] = place_info['lat']
            print(f"[OK] {place_info['name']}")
            filled_count += 1

            # Also update the state-level CSV
            state_dir = Path(f'outputs/us_2020_redistricting/{state_name.lower().replace(" ", "_")}')
            state_cities_file = state_dir / 'data' / 'district_cities.csv'

            if state_cities_file.exists():
                state_df = pd.read_csv(state_cities_file)
                district_mask = state_df['district'] == district
                if district_mask.any():
                    state_df.loc[district_mask, 'largest_city'] = place_info['name']
                    state_df.loc[district_mask, 'city_population'] = place_info['population']
                    state_df.loc[district_mask, 'city_lon'] = place_info['lon']
                    state_df.loc[district_mask, 'city_lat'] = place_info['lat']
                    state_df.to_csv(state_cities_file, index=False)
        else:
            print("[FAIL] Could not find place")

    # Save updated file
    us_districts.to_csv('outputs/us_2020_redistricting/us_all_districts.csv', index=False)

    # Check remaining missing
    remaining_missing = us_districts['city_lon'].isna().sum()

    print("\n" + "="*70)
    print("SUMMARY")
    print("="*70)
    print(f"  Filled: {filled_count}/{len(missing)}")
    print(f"  Remaining missing: {remaining_missing}/435")
    print("="*70)


if __name__ == '__main__':
    main()
