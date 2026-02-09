#!/usr/bin/env python3
"""Check isolated tracts in adjacency graph."""

import pickle
import argparse
from pathlib import Path
import geopandas as gpd
import numpy as np


def extract_county_from_geoid(geoid):
    """Extract county FIPS from tract GEOID."""
    return str(geoid)[:5]


def check_isolated_tracts(state_code, year='2020'):
    """Check why specific tracts are isolated."""
    state_lower = state_code.lower()

    # Load adjacency graph and tracts (unified directory structure)
    graph_file = Path(f'data/adjacency/{year}/{state_lower}_adjacency_{year}.pkl')
    tracts_file = Path(f'data/tracts/{year}/{state_lower}_tracts_{year}.parquet')

    print(f"\n{'='*70}")
    print(f"Checking Isolated Tracts - {state_code} ({year} Census)")
    print(f"{'='*70}\n")

    # Load graph
    with open(graph_file, 'rb') as f:
        graph_data = pickle.load(f)

    adjacency_list = graph_data['adjacency']
    index_to_geoid = graph_data['index_to_geoid']

    # Find isolated nodes
    isolated = [idx for idx, neighbors in enumerate(adjacency_list) if not neighbors]

    if not isolated:
        print("No isolated tracts found!")
        return

    print(f"Found {len(isolated)} isolated tracts:\n")

    # Load tract geometries
    tracts_gdf = gpd.read_parquet(tracts_file)

    for idx in isolated:
        geoid = index_to_geoid[idx]
        county = extract_county_from_geoid(geoid)

        # Find this tract in the GeoDataFrame
        tract_row = tracts_gdf[tracts_gdf['GEOID'] == geoid]

        if len(tract_row) == 0:
            print(f"Index {idx}: GEOID {geoid} - NOT FOUND IN TRACTS FILE")
            continue

        tract_row = tract_row.iloc[0]

        print(f"Index {idx}: GEOID {geoid}")
        print(f"  County: {county}")

        if 'NAMELSAD' in tracts_gdf.columns:
            print(f"  Name: {tract_row['NAMELSAD']}")

        # Check water area
        water_column = 'AWATER' if 'AWATER' in tracts_gdf.columns else 'AREAWATR'
        if water_column in tracts_gdf.columns:
            water_area = tract_row[water_column]
            print(f"  Water area: {water_area:,.0f} sq m")

        # Check land area
        land_column = 'ALAND' if 'ALAND' in tracts_gdf.columns else 'AREALAND'
        if land_column in tracts_gdf.columns:
            land_area = tract_row[land_column]
            print(f"  Land area: {land_area:,.0f} sq m")

        # Get centroid location
        centroid = tract_row.geometry.centroid
        print(f"  Centroid: ({centroid.y:.6f}, {centroid.x:.6f})")

        # Count other tracts in same county
        same_county_tracts = tracts_gdf[tracts_gdf['GEOID'].str[:5] == county]
        print(f"  Other tracts in same county: {len(same_county_tracts) - 1}")

        # Check geometry type
        print(f"  Geometry type: {tract_row.geometry.geom_type}")

        # Check if geometry is valid
        if not tract_row.geometry.is_valid:
            print(f"  WARNING: Invalid geometry!")

        print()


def main():
    """Check isolated tracts."""
    parser = argparse.ArgumentParser(description='Check isolated tracts in adjacency graph')
    parser.add_argument('--state', type=str, required=True,
                        help='State code (e.g., WA)')
    parser.add_argument('--year', type=str, default='2020',
                        choices=['2020', '2010', '2000'],
                        help='Census year (default: 2020)')
    args = parser.parse_args()

    check_isolated_tracts(args.state, args.year)


if __name__ == '__main__':
    main()
