"""
Spatial adjacency graph construction from TIGER shapefile geometries.

This module computes block adjacency using Queen contiguity and adds
water-based adjacency adaptation for blocks separated by water bodies.
"""

import pickle
from pathlib import Path
from typing import Dict, List, Tuple, Optional

import geopandas as gpd
import numpy as np
from libpysal import weights
from tqdm import tqdm


def _compute_boundary_lengths(
    blocks_gdf: gpd.GeoDataFrame,
    adjacency: List[List[int]],
    index_to_geoid: Dict[int, str],
    water_distance_km: float = 1.0
) -> Dict[Tuple[int, int], float]:
    """
    Compute shared boundary lengths for all adjacent pairs.

    Parameters
    ----------
    blocks_gdf : gpd.GeoDataFrame
        Block geometries
    adjacency : List[List[int]]
        Adjacency list
    index_to_geoid : Dict[int, str]
        Mapping from index to GEOID
    water_distance_km : float, default 1.0
        Water distance threshold (not used in current implementation, kept for compatibility)

    Returns
    -------
    Dict[Tuple[int, int], float]
        Edge weights dictionary. Key: (i, j) where i < j, Value: boundary length in meters

    Notes
    -----
    Boundary length handling (adaptive approach):
    - Land-based adjacency: Actual shared boundary length (LineString intersection)
    - Water-based adjacency: Median of all land boundary lengths (adaptive to state scale)
    - Point adjacency: 0.1 meters (corners touching, very easy to split)

    Rationale for median approach:
    - Treats water crossings as "average difficulty" splits
    - Automatically adapts to each state's geographic scale
    - Balances land vs water splits naturally
    """
    edge_weights = {}
    n_nodes = len(adjacency)

    # Project to appropriate CRS for accurate distance calculation
    target_crs = _get_target_crs(blocks_gdf)
    blocks_projected = blocks_gdf.to_crs(target_crs)

    # Create lookup for fast geometry access by index
    geom_by_index = {i: blocks_projected.iloc[i].geometry for i in range(len(blocks_projected))}

    # Count total edges for progress bar
    total_edges = sum(len(neighbors) for neighbors in adjacency) // 2

    print(f"  Computing boundary lengths for {total_edges:,} edges...")
    print(f"  Pass 1: Computing real boundary lengths...")

    # Pass 1: Compute all boundary lengths and collect land boundaries for median
    land_boundaries = []
    temp_edge_data = {}  # Store (i, j) -> (boundary_length, edge_type)

    edges_processed = 0
    for i in tqdm(range(n_nodes), desc="  Processing nodes", unit="node", ncols=100):
        geom_i = geom_by_index[i]

        for j in adjacency[i]:
            if j <= i:  # Only process each edge once (i < j)
                continue

            geom_j = geom_by_index[j]

            try:
                # Compute intersection of boundaries
                intersection = geom_i.intersection(geom_j)

                # Classify edge type and compute length
                if intersection.geom_type in ['LineString', 'MultiLineString']:
                    # Actual shared boundary (land-based adjacency)
                    boundary_length = intersection.length
                    edge_type = 'land'
                    land_boundaries.append(boundary_length)
                elif intersection.geom_type == 'Point':
                    # Point adjacency (corners touching)
                    boundary_length = 0.1
                    edge_type = 'point'
                elif intersection.is_empty:
                    # No shared boundary - water-based adjacency
                    boundary_length = None  # Will fill with median later
                    edge_type = 'water'
                else:
                    # Handle other geometries (rare)
                    boundary_length = None  # Will fill with median later
                    edge_type = 'other'

                temp_edge_data[(i, j)] = (boundary_length, edge_type)
                edges_processed += 1

            except Exception as e:
                # If intersection fails, use median later
                temp_edge_data[(i, j)] = (None, 'error')

    print(f"  Computed {edges_processed:,} edges")
    print(f"  Land boundaries: {len(land_boundaries):,}")

    # Pass 2: Calculate median and fill in water/special edges
    if land_boundaries:
        median_length = np.median(land_boundaries)
        print(f"  Median land boundary length: {median_length:.1f} m")
    else:
        # Fallback if no land boundaries (shouldn't happen)
        median_length = 100.0
        print(f"  Warning: No land boundaries found, using fallback: {median_length:.1f} m")

    # Fill in final edge weights
    water_count = 0
    point_count = 0
    for (i, j), (length, edge_type) in temp_edge_data.items():
        if length is None:
            # Use median for water, error, or other special cases
            edge_weights[(i, j)] = median_length
            if edge_type == 'water':
                water_count += 1
        else:
            edge_weights[(i, j)] = length
            if edge_type == 'point':
                point_count += 1

    if water_count > 0:
        print(f"  Water-based edges (using median): {water_count:,}")
    if point_count > 0:
        print(f"  Point adjacencies: {point_count:,}")

    # Report statistics
    if edge_weights:
        lengths = list(edge_weights.values())
        mean_length = np.mean(lengths)
        median_length = np.median(lengths)
        min_length = np.min(lengths)
        max_length = np.max(lengths)
        print(f"  Boundary length statistics:")
        print(f"    Mean: {mean_length:.1f} m")
        print(f"    Median: {median_length:.1f} m")
        print(f"    Range: [{min_length:.1f}, {max_length:.1f}] m")

    return edge_weights


def _filter_small_boundaries(
    adjacency: List[List[int]],
    edge_weights: Dict[Tuple[int, int], float],
    minimum_boundary_length: float
) -> Tuple[List[List[int]], Dict[Tuple[int, int], float]]:
    """
    Filter out edges with boundary length below threshold.

    Parameters
    ----------
    adjacency : List[List[int]]
        Current adjacency list
    edge_weights : Dict[Tuple[int, int], float]
        Edge weights (boundary lengths in meters)
    minimum_boundary_length : float
        Minimum boundary length threshold (meters)

    Returns
    -------
    adjacency : List[List[int]]
        Filtered adjacency list
    edge_weights : Dict[Tuple[int, int], float]
        Filtered edge weights

    Notes
    -----
    Removes edges where shared boundary length < threshold.
    Common use: Filter out tiny corner touches (point adjacencies).
    """
    # Identify edges to remove
    edges_to_remove = set()
    for (i, j), length in edge_weights.items():
        if length < minimum_boundary_length:
            edges_to_remove.add((i, j))
            edges_to_remove.add((j, i))  # Both directions

    if not edges_to_remove:
        print(f"  No edges below {minimum_boundary_length}m threshold")
        return adjacency, edge_weights

    print(f"  Found {len(edges_to_remove) // 2:,} edges below threshold")

    # Filter adjacency list
    new_adjacency = []
    edges_removed_count = 0
    for i, neighbors in enumerate(adjacency):
        new_neighbors = []
        for j in neighbors:
            edge = (min(i, j), max(i, j))
            if edge not in edges_to_remove:
                new_neighbors.append(j)
            else:
                edges_removed_count += 1
        new_adjacency.append(new_neighbors)

    # Filter edge_weights dict
    new_edge_weights = {}
    for (i, j), length in edge_weights.items():
        if (i, j) not in edges_to_remove:
            new_edge_weights[(i, j)] = length

    print(f"  Removed {edges_removed_count:,} directed edges ({edges_removed_count // 2:,} undirected)")
    print(f"  Remaining edges: {len(new_edge_weights):,}")

    # Report boundary length stats after filtering
    if new_edge_weights:
        lengths = list(new_edge_weights.values())
        min_length = np.min(lengths)
        mean_length = np.mean(lengths)
        print(f"  New minimum boundary length: {min_length:.1f} m (was < {minimum_boundary_length:.1f} m)")
        print(f"  New mean boundary length: {mean_length:.1f} m")

    return new_adjacency, new_edge_weights


def build_adjacency_graph(
    blocks_gdf: gpd.GeoDataFrame,
    water_distance_km: float = 1.0,
    include_water_adjacency: bool = True,
    compute_boundary_lengths: bool = False,
    minimum_boundary_length: float = 0.0,
    output_path: Optional[str] = None
) -> Tuple[List[List[int]], np.ndarray, Dict[int, str], Dict[str, int], Optional[Dict[Tuple[int, int], float]]]:
    """
    Build spatial adjacency graph from TIGER shapefile geometries.

    Includes water-based adjacency adaptation: blocks separated by water
    but within threshold distance are considered adjacent.

    Supports two modes:
    1. **Normal mode** (compute_boundary_lengths=False): Binary adjacency only
    2. **Edge-weighted mode** (compute_boundary_lengths=True): Includes boundary lengths

    Parameters
    ----------
    blocks_gdf : gpd.GeoDataFrame
        Block geometries with GEOID and population
    water_distance_km : float, default 1.0
        Water distance threshold in kilometers
    include_water_adjacency : bool, default True
        Whether to include water-based adjacencies
    compute_boundary_lengths : bool, default False
        Whether to compute boundary lengths for edge-weighted partitioning
    minimum_boundary_length : float, default 0.0
        Minimum shared boundary length (meters) to consider tracts adjacent.
        Filters out tiny corner touches. Recommended values:
        - 0: Include all adjacencies (default)
        - 10-25: Filter very small corner touches
        - 50-100: More aggressive filtering
    output_path : str, optional
        Path to save the adjacency graph pickle file
    water_distance_km : float, default 1.0
        Maximum distance (km) for water-based adjacency
    include_water_adjacency : bool, default True
        Whether to include water-based adjacency adaptation
    compute_boundary_lengths : bool, default False
        Whether to compute shared boundary lengths for edge-weighted partitioning
        If True, returns edge_weights dict. Adds ~2x computation time.
    output_path : str, optional
        Path to save adjacency graph pickle

    Returns
    -------
    adjacency : List[List[int]]
        Adjacency list in CSR format for METIS
        adjacency[i] = list of neighbor indices for block i
    vertex_weights : np.ndarray
        Vertex weights (population) for each block
    index_to_geoid : Dict[int, str]
        Mapping from array index to GEOID
    geoid_to_index : Dict[str, int]
        Mapping from GEOID to array index
    edge_weights : Dict[Tuple[int, int], float] or None
        Edge weights (boundary lengths in meters) if compute_boundary_lengths=True
        Key: (node_i, node_j) where i < j
        Value: shared boundary length in meters
        None if compute_boundary_lengths=False

    Notes
    -----
    - Uses Queen contiguity for land-based adjacency (shares vertex or edge)
    - Adds distance-based adjacency for coastal blocks across water
    - Boundary length computation: ~2x processing time vs normal mode
    - Processing time: ~5-10 minutes for single county, 30-60 minutes for full state (normal)
    - Processing time: ~10-20 minutes for single county, 60-120 minutes for full state (edge-weighted)
    """
    print(f"Building adjacency graph for {len(blocks_gdf):,} blocks...")
    print(f"  Mode: {'Edge-weighted (boundary lengths)' if compute_boundary_lengths else 'Normal (binary adjacency)'}")
    print(f"  Water-based adjacency: {'enabled' if include_water_adjacency else 'disabled'}")
    if include_water_adjacency:
        print(f"  Water distance threshold: {water_distance_km} km")
    if compute_boundary_lengths and minimum_boundary_length > 0:
        print(f"  Minimum boundary length: {minimum_boundary_length} m (filtering tiny corners)")

    # Initialize edge_weights dictionary (only if computing boundary lengths)
    edge_weights = {} if compute_boundary_lengths else None

    # Ensure we have a valid CRS
    if blocks_gdf.crs is None:
        print("  Warning: No CRS set, assuming EPSG:4326 (WGS84)")
        blocks_gdf = blocks_gdf.set_crs('EPSG:4326')

    # Create index mappings
    geoid_to_index = {geoid: idx for idx, geoid in enumerate(blocks_gdf['GEOID'])}
    index_to_geoid = {idx: geoid for geoid, idx in geoid_to_index.items()}

    # Extract vertex weights (population)
    vertex_weights = blocks_gdf['population'].values.astype(np.int32)

    # Step 1: Compute Queen contiguity (land-based adjacency)
    print("\nStep 1: Computing Queen contiguity (land-based adjacency)...")
    queen_weights = weights.contiguity.Queen.from_dataframe(
        blocks_gdf,
        use_index=False,  # Use integer index
        silence_warnings=True
    )
    print(f"  Found {queen_weights.n} nodes, {queen_weights.s0 / 2:.0f} edges")
    print(f"  Average neighbors per block: {queen_weights.mean_neighbors:.1f}")

    # Check connectivity
    n_components = queen_weights.n_components
    if n_components > 1:
        print(f"  Warning: Graph has {n_components} disconnected components")
        print(f"           Water adjacency may help connect them")

    # Step 2: Add water-based adjacency (if enabled)
    combined_weights = queen_weights
    if include_water_adjacency:
        print("\nStep 2: Computing county-based bridge connections...")

        try:
            # Use county-based closest-neighbor approach to connect disconnected components
            from apportionment.data.adjacency_county_bridge import connect_components_by_county

            target_crs = _get_target_crs(blocks_gdf)
            print(f"  Projecting to {target_crs} for distance calculation...")

            new_edges = connect_components_by_county(blocks_gdf, queen_weights, target_crs)

            if new_edges:
                # Create weights from new edges
                neighbors_dict = {i: list(queen_weights.neighbors[i]) for i in range(queen_weights.n)}

                # Add new bridge edges
                for idx1, idx2 in new_edges:
                    if idx2 not in neighbors_dict[idx1]:
                        neighbors_dict[idx1].append(idx2)
                    if idx1 not in neighbors_dict[idx2]:
                        neighbors_dict[idx2].append(idx1)

                # Create new weights object
                combined_weights = weights.W(neighbors_dict)
                print(f"  Combined graph: {combined_weights.n} nodes, {combined_weights.s0 / 2:.0f} edges")

                # Check connectivity
                n_components_after = combined_weights.n_components
                if n_components_after < n_components:
                    print(f"  Bridge connections reduced components: {n_components} -> {n_components_after}")
                if n_components_after == 1:
                    print(f"  SUCCESS: Graph is now fully connected!")
                elif n_components_after < n_components:
                    print(f"  PARTIAL: Still {n_components_after} components remaining")
                else:
                    print(f"  WARNING: No improvement in connectivity")

        except Exception as e:
            print(f"  Warning: Bridge connection failed: {e}")
            print(f"           Using land-based adjacency only")
            combined_weights = queen_weights

    # Step 3: Convert to METIS adjacency list format (CSR)
    print("\nStep 3: Converting to METIS adjacency list format...")
    adjacency = weights_to_adjacency_list(combined_weights)

    # Step 4: Compute boundary lengths (if requested)
    if compute_boundary_lengths:
        print("\nStep 4: Computing shared boundary lengths for edge-weighted partitioning...")
        edge_weights = _compute_boundary_lengths(blocks_gdf, adjacency, index_to_geoid, water_distance_km)

        # Step 4.5: Filter edges below minimum boundary length threshold
        if minimum_boundary_length > 0:
            print(f"\nStep 4.5: Filtering edges below {minimum_boundary_length}m boundary length...")
            adjacency, edge_weights = _filter_small_boundaries(
                adjacency, edge_weights, minimum_boundary_length
            )

    # Validate adjacency list
    print(f"\nValidating adjacency graph...")
    _validate_adjacency(adjacency, vertex_weights)

    # Save if output path specified
    if output_path:
        output_path = Path(output_path)
        output_path.parent.mkdir(parents=True, exist_ok=True)

        graph_data = {
            'adjacency': adjacency,
            'vertex_weights': vertex_weights,
            'index_to_geoid': index_to_geoid,
            'geoid_to_index': geoid_to_index,
            'edge_weights': edge_weights,  # None if not computed
            'n_components': combined_weights.n_components,
            'mean_neighbors': combined_weights.mean_neighbors,
        }

        print(f"\nSaving adjacency graph to {output_path}...")
        with open(output_path, 'wb') as f:
            pickle.dump(graph_data, f, protocol=pickle.HIGHEST_PROTOCOL)

        # Report file size
        file_size_mb = output_path.stat().st_size / 1e6
        print(f"Saved graph ({file_size_mb:.1f} MB)")

    return adjacency, vertex_weights, index_to_geoid, geoid_to_index, edge_weights


def merge_spatial_weights(w1: weights.W, w2: weights.W) -> weights.W:
    """
    Merge two spatial weights objects (union of adjacencies).

    Parameters
    ----------
    w1 : weights.W
        First spatial weights
    w2 : weights.W
        Second spatial weights

    Returns
    -------
    weights.W
        Merged spatial weights with union of adjacencies
    """
    # Get all IDs
    all_ids = sorted(set(w1.neighbors.keys()) | set(w2.neighbors.keys()))

    # Create combined neighbor dictionary
    combined_neighbors = {}
    for id in all_ids:
        neighbors = set()
        if id in w1.neighbors:
            neighbors.update(w1.neighbors[id])
        if id in w2.neighbors:
            neighbors.update(w2.neighbors[id])
        # Remove self-loops
        neighbors.discard(id)
        combined_neighbors[id] = list(neighbors)

    # Create new weights object
    return weights.W(combined_neighbors, silence_warnings=True)


def _remap_weights(w: weights.W, index_mapping: np.ndarray) -> weights.W:
    """
    Remap weights object indices.

    Used to map coastal block indices back to full block array indices.

    Parameters
    ----------
    w : weights.W
        Weights object with indices in [0, n_coastal)
    index_mapping : np.ndarray
        Mapping from weights indices to full array indices

    Returns
    -------
    weights.W
        Weights with remapped indices
    """
    remapped_neighbors = {}
    for local_id, neighbors in w.neighbors.items():
        global_id = index_mapping[local_id]
        global_neighbors = [index_mapping[n] for n in neighbors]
        remapped_neighbors[global_id] = global_neighbors

    return weights.W(remapped_neighbors, silence_warnings=True)


def weights_to_adjacency_list(w: weights.W) -> List[List[int]]:
    """
    Convert libpysal weights object to METIS adjacency list format.

    METIS expects CSR (Compressed Sparse Row) format:
    adjacency[i] = sorted list of neighbor indices for node i

    Parameters
    ----------
    w : weights.W
        Spatial weights object

    Returns
    -------
    List[List[int]]
        Adjacency list in METIS CSR format
    """
    n = w.n
    adjacency = [[] for _ in range(n)]

    for node_id, neighbors in w.neighbors.items():
        adjacency[node_id] = sorted(neighbors)

    return adjacency


def _get_target_crs(blocks_gdf: gpd.GeoDataFrame) -> str:
    """
    Determine appropriate projected CRS for distance calculation.

    Parameters
    ----------
    blocks_gdf : gpd.GeoDataFrame
        Block or tract geometries

    Returns
    -------
    str
        EPSG code for projected CRS
    """
    # Get state FIPS from first block/tract
    # Try STATEFP column (blocks) first, fall back to GEOID (tracts)
    if 'STATEFP' in blocks_gdf.columns:
        state_fips = blocks_gdf.iloc[0]['STATEFP']
    elif 'GEOID' in blocks_gdf.columns:
        # For tracts, state FIPS is first 2 digits of GEOID
        state_fips = str(blocks_gdf.iloc[0]['GEOID'])[:2]
    else:
        # Fallback: use centroid to determine UTM zone
        state_fips = None

    # State-specific projections (add more as needed)
    state_projections = {
        '06': 'EPSG:3310',  # California Albers
        '36': 'EPSG:32618', # New York UTM Zone 18N
        '48': 'EPSG:3083',  # Texas Conic
        '12': 'EPSG:3086',  # Florida GDL Albers
    }

    if state_fips in state_projections:
        return state_projections[state_fips]

    # Fallback: use UTM zone based on centroid
    centroid = blocks_gdf.dissolve().centroid.iloc[0]
    lon = centroid.x
    utm_zone = int((lon + 180) / 6) + 1
    hemisphere = 'north' if centroid.y >= 0 else 'south'

    # Northern hemisphere: EPSG:326XX, Southern: EPSG:327XX
    if hemisphere == 'north':
        epsg_code = f"EPSG:326{utm_zone:02d}"
    else:
        epsg_code = f"EPSG:327{utm_zone:02d}"

    return epsg_code


def _validate_adjacency(adjacency: List[List[int]], vertex_weights: np.ndarray):
    """
    Validate adjacency list structure.

    Parameters
    ----------
    adjacency : List[List[int]]
        Adjacency list
    vertex_weights : np.ndarray
        Vertex weights
    """
    n_nodes = len(adjacency)
    assert len(vertex_weights) == n_nodes, "Vertex weights length mismatch"

    n_edges = sum(len(neighbors) for neighbors in adjacency)
    print(f"  Nodes: {n_nodes:,}")
    print(f"  Edges: {n_edges // 2:,} (undirected)")
    print(f"  Average degree: {n_edges / n_nodes:.1f}")

    # Check for self-loops
    self_loops = sum(1 for i, neighbors in enumerate(adjacency) if i in neighbors)
    if self_loops > 0:
        print(f"  Warning: Found {self_loops} self-loops")

    # Check for invalid neighbor indices
    invalid = sum(
        1 for neighbors in adjacency
        for n in neighbors
        if n < 0 or n >= n_nodes
    )
    if invalid > 0:
        print(f"  Warning: Found {invalid} invalid neighbor indices")

    # Check symmetry (undirected graph)
    asymmetric = 0
    for i, neighbors in enumerate(adjacency):
        for j in neighbors:
            if i not in adjacency[j]:
                asymmetric += 1

    if asymmetric > 0:
        print(f"  Warning: Found {asymmetric} asymmetric edges (graph should be undirected)")


def load_adjacency_graph(file_path: str) -> Tuple[List[List[int]], np.ndarray, Dict[int, str], Dict[str, int], Optional[Dict[Tuple[int, int], float]]]:
    """
    Load adjacency graph from pickle file.

    Parameters
    ----------
    file_path : str
        Path to pickled graph data

    Returns
    -------
    adjacency : List[List[int]]
        Adjacency list
    vertex_weights : np.ndarray
        Vertex weights (population)
    index_to_geoid : Dict[int, str]
        Index to GEOID mapping
    geoid_to_index : Dict[str, int]
        GEOID to index mapping
    edge_weights : Dict[Tuple[int, int], float] or None
        Edge weights (boundary lengths in meters) if available, None otherwise
    """
    print(f"Loading adjacency graph from {file_path}...")
    with open(file_path, 'rb') as f:
        graph_data = pickle.load(f)

    adjacency = graph_data['adjacency']
    vertex_weights = graph_data['vertex_weights']
    index_to_geoid = graph_data['index_to_geoid']
    geoid_to_index = graph_data['geoid_to_index']

    # Edge weights (backward compatibility: may not exist in old pickle files)
    edge_weights = graph_data.get('edge_weights', None)

    print(f"Loaded graph: {len(adjacency):,} nodes, "
          f"{sum(len(neighbors) for neighbors in adjacency) // 2:,} edges")
    if edge_weights is not None:
        print(f"  Edge weights: {len(edge_weights):,} (edge-weighted mode)")
    else:
        print(f"  Edge weights: Not available (normal mode)")

    return adjacency, vertex_weights, index_to_geoid, geoid_to_index, edge_weights
