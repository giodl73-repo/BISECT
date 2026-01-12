#!/usr/bin/env python3
"""Debug: See all places in a specific district."""

import sys
from pathlib import Path
import pickle
import geopandas as gpd

run_dir = Path('outputs/california_full_20260109_165645')
tracts = gpd.read_parquet('data/raw/ca_tracts_2020.parquet')
places = gpd.read_parquet('data/raw/ca_places_2020.parquet')

with open(run_dir / 'final_assignments.pkl', 'rb') as f:
    assignments = pickle.load(f)

tracts['district'] = [assignments[i] for i in range(len(tracts))]

# Ensure same CRS
if tracts.crs != places.crs:
    places = places.to_crs(tracts.crs)

# Spatial join
places_points = places.copy()
places_points['geometry'] = places_points.geometry.representative_point()
places_with_tracts = gpd.sjoin(places_points, tracts[['geometry', 'district']], how='left', predicate='within')

# Check District 6
district_id = 6
district_places = places_with_tracts[places_with_tracts['district'] == district_id].sort_values('population', ascending=False)

print(f"\nAll places in District {district_id}:")
print("=" * 80)
for idx, row in district_places.head(15).iterrows():
    print(f"  {row['NAME']:30s} - {row['population']:>10,} people")
