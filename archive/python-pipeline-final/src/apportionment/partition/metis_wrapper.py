"""
METIS graph partitioning wrapper with fallback options.

Provides a unified interface to gpmetis algorithm with support for:
- pymetis (primary)
- gpmetis.exe direct executable (secondary)
- NetworkX native algorithms (emergency fallback)
"""

from typing import List, Optional, Tuple, Dict

import numpy as np


def partition_graph(
    adjacency: List[List[int]],
    vertex_weights: np.ndarray,
    nparts: int = 2,
    target_weights: Optional[List[float]] = None,
    recursive: bool = True,
    ufactor: Optional[float] = None,
    ubvec: Optional[List[float]] = None,
    niter: int = 100,
    objtype: str = 'cut',
    seed: Optional[int] = None,
    debug: bool = False,
    edge_weights: Optional[Dict[Tuple[int, int], float]] = None,
    multi_constraint: bool = False
) -> np.ndarray:
    """
    Partition graph using METIS gpmetis algorithm.

    Parameters
    ----------
    adjacency : List[List[int]]
        Adjacency list in CSR format
        adjacency[i] = sorted list of neighbor indices for node i
    vertex_weights : np.ndarray
        Vertex weights (e.g., block populations)
        - 1D array (n_vertices,) for single constraint
        - 2D array (n_vertices, n_constraints) for multi-constraint (VRA mode)
    nparts : int, default 2
        Number of partitions
    target_weights : List[float], optional
        Target partition weights (must sum to 1.0)
        - Single constraint: [0.5, 0.5] for equal split
        - Multi-constraint: [[0.5, 0.6], [0.5, 0.4]] for 2 partitions, 2 constraints
    recursive : bool, default True
        Use recursive bisection algorithm
    ufactor : float, optional
        Load imbalance tolerance factor (1.0 = strict balance, higher = more tolerance)
    niter : int, default 100
        Number of refinement iterations (default 10 in METIS, 100 for better compactness)
    objtype : str, default 'cut'
        METIS objective function: 'cut' (edge-cut) or 'vol' (volume)
    seed : int, optional
        METIS random seed for reproducibility (default: None = random)
    debug : bool, default False
        Print detailed error and fallback messages
    multi_constraint : bool, default False
        Enable multi-constraint partitioning (VRA mode)

    Returns
    -------
    np.ndarray
        Partition assignments, shape (n_nodes,)
        parts[i] is the partition ID for node i (in range [0, nparts))

    Raises
    ------
    RuntimeError
        If partitioning fails with all available methods
    """
    # Try pymetis first (best quality, fastest)
    try:
        return _partition_with_pymetis(adjacency, vertex_weights, nparts, target_weights, recursive, ufactor, niter, objtype, seed, multi_constraint)
    except ImportError as e:
        if debug:
            print(f"pymetis not available: {e}")
            print("Trying fallback methods...")
    except Exception as e:
        if debug:
            print(f"pymetis failed: {e}")
            print("Trying fallback methods...")

    # Use gpmetis.exe executable (required - no fallbacks to ensure quality)
    try:
        if debug:
            print("Using gpmetis.exe executable...")
        from .metis_executable import partition_graph_with_executable
        return partition_graph_with_executable(adjacency, vertex_weights, nparts, target_weights, ufactor, ubvec=None, niter=niter, debug=debug, edge_weights=edge_weights)
    except ImportError as e:
        raise RuntimeError(
            f"METIS not available: {e}\n"
            "Please build METIS using scripts/build_metis_windows.bat"
        )
    except Exception as e:
        raise RuntimeError(
            f"METIS partitioning failed: {e}\n"
            "Check that gpmetis.exe is available and working correctly"
        )


def _partition_with_pymetis(
    adjacency: List[List[int]],
    vertex_weights: np.ndarray,
    nparts: int,
    target_weights: Optional[List[float]],
    recursive: bool,
    ufactor: Optional[float] = None,
    niter: int = 100,
    objtype: str = 'cut',
    seed: Optional[int] = None,
    multi_constraint: bool = False
) -> np.ndarray:
    """Partition using pymetis library."""
    import pymetis

    # Convert vertex weights to appropriate format
    if multi_constraint:
        # Multi-constraint: vertex_weights is 2D array (n_vertices, n_constraints)
        # Convert to list of lists: vweights[i] = [constraint1, constraint2, ...]
        vweights = vertex_weights.astype(np.int32).tolist()
    else:
        # Single constraint: vertex_weights is 1D array (n_vertices,)
        # Convert to list of integers
        vweights = vertex_weights.astype(np.int32).tolist()

    # Convert target weights if provided
    tpwgts = None
    if target_weights is not None:
        if multi_constraint and isinstance(target_weights[0], (list, tuple)):
            # Multi-constraint: target_weights is 2D: [[p1_c1, p1_c2], [p2_c1, p2_c2], ...]
            # Normalize each constraint separately
            tpwgts = target_weights
        else:
            # Single constraint: normalize to sum = 1.0
            tpwgts = [w / sum(target_weights) for w in target_weights]

    # Set METIS options for improved compactness
    # METIS options array: see METIS manual for details
    options = pymetis.Options()
    options.niter = niter  # Number of refinement iterations (default 10, we use 100 for better compactness)

    if ufactor is not None:
        # Convert ufactor (e.g., 1.001) to ubvec format expected by METIS
        # ubvec is the allowed load imbalance: ufactor of 1.001 = 0.1% tolerance
        # METIS ubvec = (ufactor - 1.0) * 1000, capped at 1 for very tight tolerance
        ubvec_value = max(1, int((ufactor - 1.0) * 1000))
        options.ufactor = ubvec_value

    # Set objective type: 1 = edge-cut (default), 2 = volume
    if objtype == 'vol':
        options.objtype = 2
    else:
        options.objtype = 1

    # Set random seed if provided
    if seed is not None:
        options.seed = seed

    # Call pymetis
    if nparts == 2:
        # Recursive bisection
        n_cuts, parts = pymetis.part_graph(
            nparts=2,
            adjacency=adjacency,
            vweights=vweights,
            tpwgts=tpwgts,
            recursive=recursive,
            options=options
        )
    else:
        # K-way partitioning
        n_cuts, parts = pymetis.part_graph(
            nparts=nparts,
            adjacency=adjacency,
            vweights=vweights,
            tpwgts=tpwgts,
            recursive=False,
            options=options
        )

    return np.array(parts, dtype=np.int32)


def _partition_with_networkx_metis(
    adjacency: List[List[int]],
    vertex_weights: np.ndarray,
    nparts: int
) -> np.ndarray:
    """Partition using networkx-metis library."""
    import networkx as nx
    import nxmetis

    # Convert to NetworkX graph
    G = _adjacency_to_networkx(adjacency, vertex_weights)

    # Partition using nxmetis
    if nparts == 2:
        # Bisection
        parts_list = nxmetis.partition(G, nparts=2, node_weight='weight')
        # parts_list is a list of two sets
        parts = np.zeros(len(adjacency), dtype=np.int32)
        for part_id, nodes in enumerate(parts_list):
            for node in nodes:
                parts[node] = part_id
    else:
        # K-way partitioning
        parts_list = nxmetis.partition(G, nparts=nparts, node_weight='weight')
        parts = np.zeros(len(adjacency), dtype=np.int32)
        for part_id, nodes in enumerate(parts_list):
            for node in nodes:
                parts[node] = part_id

    return parts


def _partition_with_networkx_native(
    adjacency: List[List[int]],
    vertex_weights: np.ndarray,
    nparts: int
) -> np.ndarray:
    """
    Partition using NetworkX native Kernighan-Lin algorithm.

    Warning: This is slower and lower quality than METIS.
    Only used as emergency fallback.
    """
    import networkx as nx
    from networkx.algorithms.community import kernighan_lin_bisection

    # Convert to NetworkX graph
    G = _adjacency_to_networkx(adjacency, vertex_weights)

    # Recursive bisection to get nparts partitions
    def recursive_bisection(graph, k):
        """Recursively bisect graph into k parts."""
        if k == 1:
            return [set(graph.nodes())]

        # Bisect into 2 parts
        try:
            part1, part2 = kernighan_lin_bisection(graph, weight='weight')
        except:
            # If bisection fails, use naive split
            nodes = list(graph.nodes())
            mid = len(nodes) // 2
            part1 = set(nodes[:mid])
            part2 = set(nodes[mid:])

        # Determine split for remaining partitions
        k_left = k // 2
        k_right = k - k_left

        # Recursively partition each subgraph
        if k_left > 1:
            subgraph1 = graph.subgraph(part1).copy()
            parts1 = recursive_bisection(subgraph1, k_left)
        else:
            parts1 = [part1]

        if k_right > 1:
            subgraph2 = graph.subgraph(part2).copy()
            parts2 = recursive_bisection(subgraph2, k_right)
        else:
            parts2 = [part2]

        return parts1 + parts2

    # Perform recursive bisection
    parts_list = recursive_bisection(G, nparts)

    # Convert to partition array
    parts = np.zeros(len(adjacency), dtype=np.int32)
    for part_id, nodes in enumerate(parts_list):
        for node in nodes:
            parts[node] = part_id

    return parts


def _adjacency_to_networkx(adjacency: List[List[int]], vertex_weights: np.ndarray):
    """Convert adjacency list to NetworkX graph."""
    import networkx as nx

    G = nx.Graph()

    # Add nodes with weights
    for i, weight in enumerate(vertex_weights):
        G.add_node(i, weight=int(weight))

    # Add edges
    for i, neighbors in enumerate(adjacency):
        for j in neighbors:
            if i < j:  # Only add each edge once (undirected)
                G.add_edge(i, j)

    return G


def check_metis_installation() -> Tuple[bool, str]:
    """
    Check if METIS is properly installed.

    Returns
    -------
    installed : bool
        True if METIS is available
    method : str
        Which METIS method is available ('pymetis', 'gpmetis.exe', 'none')
    """
    # Try pymetis
    try:
        import pymetis
        return True, 'pymetis'
    except ImportError:
        pass

    # Try gpmetis.exe executable
    try:
        from .metis_executable import find_gpmetis_executable
        if find_gpmetis_executable():
            return True, 'gpmetis.exe'
    except ImportError:
        pass

    return False, 'none'


def print_installation_instructions():
    """Print instructions for installing METIS."""
    print("\n" + "=" * 60)
    print("METIS Installation Instructions")
    print("=" * 60)
    print("\nOption 1: Install pymetis (recommended)")
    print("  pip install pymetis")
    print("\nOption 2: Install from conda-forge (Windows)")
    print("  conda install -c conda-forge metis")
    print("  pip install pymetis")
    print("\nOption 3: Install networkx-metis (fallback)")
    print("  pip install networkx-metis")
    print("\nIf installation fails, the system will use NetworkX native")
    print("algorithms (slower, lower quality).")
    print("=" * 60 + "\n")
