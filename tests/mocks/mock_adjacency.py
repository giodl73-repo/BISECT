"""
Mock adjacency graph generator for testing.

Generates NetworkX graphs representing tract-level adjacency relationships
matching the structure used by METIS partitioning.
"""

import networkx as nx
import numpy as np
from shapely.geometry import Point
import random
import warnings


def generate_mock_adjacency(tracts_df, connectivity=0.2, edge_weighted=False, seed=42):
    """
    Generate mock NetworkX adjacency graph from tracts.

    Parameters
    ----------
    tracts_df : geopandas.GeoDataFrame
        Mock tract data
    connectivity : float
        Target connectivity ratio (edges per node)
    edge_weighted : bool
        If True, add edge weights (boundary lengths in meters)
    seed : int
        Random seed for reproducibility

    Returns
    -------
    networkx.Graph
        Adjacency graph with tract indices as nodes

    Examples
    --------
    >>> from tests.mocks.mock_tracts import generate_mock_tracts
    >>> tracts = generate_mock_tracts(num_tracts=50)
    >>> graph = generate_mock_adjacency(tracts, connectivity=0.2)
    >>> nx.is_connected(graph)
    True
    >>> len(graph.nodes)
    50
    """
    np.random.seed(seed)
    random.seed(seed)

    num_tracts = len(tracts_df)

    # Create empty graph
    graph = nx.Graph()

    # Add all nodes explicitly (CRITICAL for isolated tracts)
    for i in range(num_tracts):
        graph.add_node(i)

    # Get tract centroids for distance-based adjacency
    # Suppress geopandas CRS warning (mock data uses geographic CRS for simplicity)
    with warnings.catch_warnings():
        warnings.filterwarnings('ignore', message='.*geographic CRS.*', category=UserWarning)
        centroids = tracts_df.geometry.centroid

    # Build adjacency based on proximity
    # Method 1: Connect to K nearest neighbors to ensure connectivity
    k_neighbors = max(3, int(connectivity * num_tracts))

    for i in range(num_tracts):
        centroid_i = centroids.iloc[i]

        # Calculate distances to all other tracts
        distances = []
        for j in range(num_tracts):
            if i != j:
                centroid_j = centroids.iloc[j]
                dist = centroid_i.distance(centroid_j)
                distances.append((j, dist))

        # Sort by distance and connect to K nearest
        distances.sort(key=lambda x: x[1])
        for j, dist in distances[:k_neighbors]:
            if not graph.has_edge(i, j):
                if edge_weighted:
                    # Edge weight = boundary length in meters (mock)
                    # Use distance as proxy, scaled to meters
                    weight_meters = int(dist * 10000)  # ~10km per 0.1 degree
                    weight_meters = max(100, weight_meters)  # At least 100m
                    graph.add_edge(i, j, weight=weight_meters)
                else:
                    graph.add_edge(i, j)

    # Ensure graph is connected (single component)
    if not nx.is_connected(graph):
        # Connect components by adding bridges
        components = list(nx.connected_components(graph))
        for i in range(len(components) - 1):
            # Get random node from each component
            node1 = random.choice(list(components[i]))
            node2 = random.choice(list(components[i + 1]))

            # Add bridge edge
            if edge_weighted:
                # Bridge edges get median weight
                existing_weights = [data.get('weight', 1000) for _, _, data in graph.edges(data=True)]
                bridge_weight = int(np.median(existing_weights)) if existing_weights else 1000
                graph.add_edge(node1, node2, weight=bridge_weight)
            else:
                graph.add_edge(node1, node2)

    # Validate
    assert nx.is_connected(graph), "Graph must be connected"
    assert len(graph.nodes) == num_tracts, "All tracts must be in graph"

    return graph


def generate_mock_adjacency_from_geometry(tracts_df, edge_weighted=False, seed=42):
    """
    Generate adjacency graph based on actual tract geometries (Queen contiguity).

    This is more realistic but slower. Uses Shapely's touches() method.

    Parameters
    ----------
    tracts_df : geopandas.GeoDataFrame
        Mock tract data
    edge_weighted : bool
        If True, calculate actual boundary lengths
    seed : int
        Random seed

    Returns
    -------
    networkx.Graph
        Adjacency graph based on geometric touching
    """
    np.random.seed(seed)

    num_tracts = len(tracts_df)
    graph = nx.Graph()

    # Add all nodes
    for i in range(num_tracts):
        graph.add_node(i)

    # Check geometric adjacency
    for i in range(num_tracts):
        geom_i = tracts_df.geometry.iloc[i]

        for j in range(i + 1, num_tracts):
            geom_j = tracts_df.geometry.iloc[j]

            # Check if geometries touch or intersect
            if geom_i.touches(geom_j) or geom_i.intersects(geom_j):
                if edge_weighted:
                    # Calculate actual boundary length
                    boundary = geom_i.intersection(geom_j)
                    if boundary.length > 0:
                        # Convert to meters (rough approximation)
                        weight_meters = int(boundary.length * 100000)
                        weight_meters = max(100, weight_meters)
                        graph.add_edge(i, j, weight=weight_meters)
                else:
                    graph.add_edge(i, j)

    # Ensure connectivity (may not be connected with pure geometric adjacency)
    if not nx.is_connected(graph):
        # Fall back to distance-based connections
        return generate_mock_adjacency(tracts_df, connectivity=0.2, edge_weighted=edge_weighted, seed=seed)

    return graph


def adjacency_graph_to_metis_format(graph, output_file):
    """
    Write adjacency graph to METIS format file.

    METIS format:
    - Line 1: num_nodes num_edges [format_code]
    - Lines 2+: For each node, list of adjacent nodes (space-separated)

    Parameters
    ----------
    graph : networkx.Graph
        Adjacency graph
    output_file : Path or str
        Output file path

    Notes
    -----
    Format codes:
    - 000: Unweighted graph
    - 001: Edge-weighted graph
    - 010: Node-weighted graph
    - 011: Edge and node-weighted graph
    """
    num_nodes = len(graph.nodes)
    num_edges = len(graph.edges)

    # Check if edge-weighted
    has_edge_weights = any('weight' in data for _, _, data in graph.edges(data=True))
    format_code = '011' if has_edge_weights else '000'

    with open(output_file, 'w') as f:
        # Header line
        f.write(f"{num_nodes} {num_edges} {format_code}\n")

        # Adjacency lists (1-indexed for METIS)
        for node in sorted(graph.nodes):
            neighbors = sorted(graph.neighbors(node))

            if has_edge_weights:
                # Format: neighbor1 weight1 neighbor2 weight2 ...
                neighbor_strings = []
                for neighbor in neighbors:
                    weight = graph[node][neighbor].get('weight', 1)
                    neighbor_strings.append(f"{neighbor + 1} {weight}")
                f.write(" ".join(neighbor_strings) + "\n")
            else:
                # Format: neighbor1 neighbor2 neighbor3 ...
                neighbor_strings = [str(n + 1) for n in neighbors]
                f.write(" ".join(neighbor_strings) + "\n")


def validate_mock_adjacency(graph, num_tracts):
    """
    Validate mock adjacency graph.

    Parameters
    ----------
    graph : networkx.Graph
        Mock adjacency graph
    num_tracts : int
        Expected number of tracts

    Raises
    ------
    AssertionError
        If validation fails
    """
    # Check node count
    assert len(graph.nodes) == num_tracts, \
        f"Expected {num_tracts} nodes, found {len(graph.nodes)}"

    # Check all nodes present (no gaps)
    expected_nodes = set(range(num_tracts))
    actual_nodes = set(graph.nodes)
    assert expected_nodes == actual_nodes, \
        f"Missing nodes: {expected_nodes - actual_nodes}"

    # Check graph is connected
    assert nx.is_connected(graph), \
        f"Graph is not connected: {nx.number_connected_components(graph)} components"

    # Check no self-loops
    assert not any(u == v for u, v in graph.edges), "Graph has self-loops"

    # Check connectivity
    avg_degree = sum(dict(graph.degree()).values()) / len(graph.nodes)

    print(f"[OK] Mock adjacency validated:")
    print(f"  Nodes: {len(graph.nodes)}")
    print(f"  Edges: {len(graph.edges)}")
    print(f"  Average degree: {avg_degree:.2f}")
    print(f"  Connected: {nx.is_connected(graph)}")


if __name__ == '__main__':
    # Test generation
    from tests.mocks.mock_tracts import generate_mock_tracts

    print("Generating mock adjacency graphs...")

    # Small dataset
    tracts_small = generate_mock_tracts(num_tracts=50, state='vermont')
    graph_small = generate_mock_adjacency(tracts_small, connectivity=0.2)
    validate_mock_adjacency(graph_small, len(tracts_small))
    print("  Small graph: OK")

    # Edge-weighted version
    graph_weighted = generate_mock_adjacency(tracts_small, connectivity=0.2, edge_weighted=True)
    validate_mock_adjacency(graph_weighted, len(tracts_small))

    # Check edge weights exist
    edge_weights = [data.get('weight', 0) for _, _, data in graph_weighted.edges(data=True)]
    print(f"  Edge weights: min={min(edge_weights)}, max={max(edge_weights)}, mean={np.mean(edge_weights):.0f}")

    print("[OK] Mock adjacency generation working correctly")
