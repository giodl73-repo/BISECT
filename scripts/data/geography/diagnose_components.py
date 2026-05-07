#!/usr/bin/env python3
"""Diagnose disconnected components in adjacency graphs."""

import pickle
import argparse
from pathlib import Path
import networkx as nx
import geopandas as gpd
import numpy as np


def extract_county_from_geoid(geoid):
    """Extract county FIPS from tract GEOID."""
    return str(geoid)[:5]


def diagnose_state(state_code, year='2020'):
    """Diagnose why a state has disconnected components."""
    state_lower = state_code.lower()

    # Load adjacency graph and tracts (unified directory structure)
    graph_file = Path(f'data/adjacency/{year}/{state_lower}_adjacency_{year}.pkl')
    tracts_file = Path(f'data/tracts/{year}/{state_lower}_tracts_{year}.parquet')

    if not graph_file.exists():
        print(f"Error: Graph file not found: {graph_file}")
        return

    if not tracts_file.exists():
        print(f"Error: Tracts file not found: {tracts_file}")
        return

    print(f"\n{'='*70}")
    print(f"Diagnosing {state_code} ({year} Census)")
    print(f"{'='*70}\n")

    # Load graph
    with open(graph_file, 'rb') as f:
        graph_data = pickle.load(f)

    adjacency_list = graph_data['adjacency']
    index_to_geoid = graph_data['index_to_geoid']

    # Build NetworkX graph
    graph = nx.Graph()
    for node_idx, neighbors in enumerate(adjacency_list):
        for neighbor in neighbors:
            graph.add_edge(node_idx, neighbor)

    # Find connected components
    components = list(nx.connected_components(graph))
    components.sort(key=len, reverse=True)

    print(f"Total tracts: {len(adjacency_list)}")
    print(f"Connected components: {len(components)}")
    print(f"Component sizes: {[len(c) for c in components]}\n")

    if len(components) == 1:
        print("State is fully connected!")
        return

    # Load tract geometries
    tracts_gdf = gpd.read_parquet(tracts_file)

    # Create a mapping from GEOID to index
    geoid_to_index = {geoid: idx for idx, geoid in enumerate(index_to_geoid)}

    # Analyze each component
    print(f"{'='*70}")
    print("COMPONENT ANALYSIS")
    print(f"{'='*70}\n")

    for comp_idx, component in enumerate(components):
        component_list = sorted(list(component))
        print(f"Component {comp_idx + 1}: {len(component_list)} tracts")

        # Get GEOIDs and counties for this component
        component_geoids = [index_to_geoid[idx] for idx in component_list]
        component_counties = [extract_county_from_geoid(geoid) for geoid in component_geoids]
        unique_counties = sorted(set(component_counties))

        print(f"  Counties: {unique_counties}")
        print(f"  Sample GEOIDs: {component_geoids[:5]}")

        # Get county names if available
        component_tracts = tracts_gdf[tracts_gdf['GEOID'].isin(component_geoids)]
        if 'NAMELSAD' in component_tracts.columns and len(component_tracts) > 0:
            sample_names = component_tracts['NAMELSAD'].head(3).tolist()
            print(f"  Sample tract names: {sample_names}")

        # Calculate centroids
        if len(component_tracts) > 0:
            # Use original CRS for centroid calculation
            centroids = component_tracts.geometry.centroid
            center_lat = centroids.y.mean()
            center_lon = centroids.x.mean()
            print(f"  Geographic center: ({center_lat:.4f}, {center_lon:.4f})")

        print()

    # Analyze why components aren't connected
    print(f"{'='*70}")
    print("CONNECTIVITY ANALYSIS")
    print(f"{'='*70}\n")

    main_component = components[0]
    main_component_list = list(main_component)
    main_geoids = [index_to_geoid[idx] for idx in main_component_list]
    main_counties = set(extract_county_from_geoid(geoid) for geoid in main_geoids)

    print(f"Main component counties ({len(main_counties)}): {sorted(main_counties)}\n")

    for comp_idx, component in enumerate(components[1:], start=2):
        component_list = list(component)
        component_geoids = [index_to_geoid[idx] for idx in component_list]
        component_counties = set(extract_county_from_geoid(geoid) for geoid in component_geoids)

        # Check for county overlap
        shared_counties = main_counties.intersection(component_counties)

        print(f"Component {comp_idx}:")
        print(f"  Counties: {sorted(component_counties)}")
        if shared_counties:
            print(f"  Shared with main: {sorted(shared_counties)} ✓")
        else:
            print(f"  Shared with main: NONE ✗")

        # Calculate distance to main component
        comp_tracts = tracts_gdf[tracts_gdf['GEOID'].isin(component_geoids)]
        main_tracts = tracts_gdf[tracts_gdf['GEOID'].isin(main_geoids)]

        if len(comp_tracts) > 0 and len(main_tracts) > 0:
            # Project to appropriate CRS for distance calculation
            if state_code in ['AK', 'HI']:
                target_crs = 'EPSG:3857'  # Web Mercator for distant states
            else:
                target_crs = 'EPSG:5070'  # Albers Equal Area for CONUS

            comp_projected = comp_tracts.to_crs(target_crs)
            main_projected = main_tracts.to_crs(target_crs)

            comp_centroids = np.array([[geom.centroid.x, geom.centroid.y]
                                       for geom in comp_projected.geometry])
            main_centroids = np.array([[geom.centroid.x, geom.centroid.y]
                                       for geom in main_projected.geometry])

            # Find closest pair
            min_dist = float('inf')
            for comp_centroid in comp_centroids:
                distances = np.linalg.norm(main_centroids - comp_centroid, axis=1)
                min_dist = min(min_dist, distances.min())

            print(f"  Min distance to main: {min_dist / 1000:.1f} km")

        print()


def main():
    """Diagnose disconnected components."""
    parser = argparse.ArgumentParser(description='Diagnose disconnected adjacency graph components')
    parser.add_argument('--state', type=str, required=True,
                        help='State code (e.g., WA)')
    parser.add_argument('--year', type=str, default='2020',
                        choices=['2020', '2010', '2000'],
                        help='Census year (default: 2020)')
    args = parser.parse_args()

    diagnose_state(args.state, args.year)


if __name__ == '__main__':
    main()
