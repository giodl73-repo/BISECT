"""County-based bridge connections for disconnected graph components.

This module provides functionality to connect disconnected components in a tract
adjacency graph by finding the closest neighbor within the same county.
"""

import numpy as np
import networkx as nx
from scipy.spatial import cKDTree


def extract_county_from_geoid(geoid):
    """Extract county FIPS from tract GEOID.

    GEOID format: SSCCCTTTTTT (11 digits)
    - SS: state FIPS (2 digits)
    - CCC: county FIPS (3 digits)
    - TTTTTT: tract code (6 digits)

    Returns: SSCCC (5-character state+county code)
    """
    return str(geoid)[:5]


def connect_components_by_county(blocks_gdf, queen_weights, target_crs):
    """Connect disconnected components using closest neighbors within same county.

    Args:
        blocks_gdf: GeoDataFrame with tract geometries and GEOIDs
        queen_weights: Existing Queen contiguity weights (from land adjacency)
        target_crs: Target CRS for distance calculations

    Returns:
        List of (idx1, idx2) tuples representing new edges to add
    """
    # Build NetworkX graph from weights
    graph = nx.Graph()
    # First, add all nodes (including isolated nodes with no neighbors)
    for i in range(queen_weights.n):
        graph.add_node(i)
    # Then add edges
    for i in range(queen_weights.n):
        neighbors = queen_weights.neighbors[i]
        for j in neighbors:
            graph.add_edge(i, j)

    # Find connected components
    components = list(nx.connected_components(graph))
    components.sort(key=len, reverse=True)

    n_components = len(components)
    if n_components == 1:
        print("  Graph is already fully connected")
        return []

    print(f"  Found {n_components} disconnected components")
    print(f"  Component sizes: {[len(c) for c in components[:10]]}")

    # Extract county from GEOID for each tract
    if 'GEOID' not in blocks_gdf.columns:
        print("  Warning: No GEOID column found, cannot use county-based bridging")
        return []

    counties = blocks_gdf['GEOID'].apply(extract_county_from_geoid).values

    # Project to target CRS for distance calculations
    blocks_projected = blocks_gdf.to_crs(target_crs)
    centroids = np.array([[geom.centroid.x, geom.centroid.y]
                          for geom in blocks_projected.geometry])

    # Build KD-tree for fast nearest neighbor search
    tree = cKDTree(centroids)

    # Connect each small component to main component within same county
    new_edges = []
    main_component = components[0]  # Largest component

    for comp_idx, component in enumerate(components[1:], start=1):
        component_list = list(component)

        # For each tract in this component, find closest tract in main component
        # within the same county
        for tract_idx in component_list:
            tract_county = counties[tract_idx]
            tract_centroid = centroids[tract_idx]

            # Find all tracts in main component with same county
            same_county_main = [idx for idx in main_component
                               if counties[idx] == tract_county]

            if not same_county_main:
                # No tracts in main component from same county
                # Fall back to closest tract in main component regardless of county
                same_county_main = list(main_component)

            # Find closest tract
            candidates = np.array([centroids[idx] for idx in same_county_main])
            distances = np.linalg.norm(candidates - tract_centroid, axis=1)
            closest_idx = same_county_main[np.argmin(distances)]
            closest_distance_km = distances.min() / 1000  # Convert m to km

            new_edges.append((tract_idx, closest_idx))

            # Only print for first few
            if len(new_edges) <= 5:
                same_county_str = "same county" if counties[closest_idx] == tract_county else "diff county"
                print(f"    Component {comp_idx}: Connecting tract {tract_idx} -> {closest_idx} "
                      f"({same_county_str}, {closest_distance_km:.1f} km)")

    print(f"  Created {len(new_edges)} bridge connections")
    return new_edges
