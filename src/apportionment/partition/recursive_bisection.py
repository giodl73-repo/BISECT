"""
Recursive bifurcation algorithm for redistricting.

Implements recursive splitting using METIS gpmetis to create equal-population districts.
"""

from typing import Dict, List, Set, Optional, Tuple

import numpy as np
from tqdm import tqdm

from .metis_wrapper import partition_graph


def _extract_subgraph_edge_weights(
    edge_weights: Optional[Dict[Tuple[int, int], float]],
    block_indices: Set[int],
    global_to_local: Dict[int, int]
) -> Optional[Dict[Tuple[int, int], float]]:
    """
    Extract edge weights for subgraph from global edge weights.

    Parameters
    ----------
    edge_weights : dict or None
        Global edge weights mapping (i, j) to weight
    block_indices : set
        Block indices in this subgraph
    global_to_local : dict
        Mapping from global to local indices

    Returns
    -------
    dict or None
        Subgraph edge weights with local indices, or None if no global weights
    """
    if edge_weights is None:
        return None

    subgraph_edge_weights = {}
    for (i, j), weight in edge_weights.items():
        # Check if both nodes are in the subgraph
        if i in block_indices and j in block_indices:
            local_i = global_to_local[i]
            local_j = global_to_local[j]
            # Store with canonical ordering (smaller index first)
            key = (min(local_i, local_j), max(local_i, local_j))
            subgraph_edge_weights[key] = weight

    return subgraph_edge_weights


def _split_node_worker(task: dict) -> dict:
    """
    Worker function for parallel node splitting.
    Must be at module level for multiprocessing pickling.

    Parameters
    ----------
    task : dict
        Task containing node data and parameters

    Returns
    -------
    dict
        Split result with child node data
    """
    # Extract task data
    block_indices = task['block_indices']
    target_districts = task['target_districts']
    depth = task['depth']
    name = task['name']
    state_code = task['state_code']
    adjacency = task['adjacency']
    vertex_weights = task['vertex_weights']
    debug = task['debug']
    edge_weights = task.get('edge_weights', None)
    ufactor_override = task.get('ufactor_override', None)
    niter = task.get('niter', 100)
    objtype = task.get('objtype', 'cut')
    seed = task.get('seed', None)
    vra_mode = task.get('vra_mode', False)
    vra_target_tree = task.get('vra_target_tree', None)

    k = target_districts

    if k == 1:
        # No split needed
        return None

    # Calculate split sizes
    k_left = k // 2
    k_right = k - k_left

    # Create subgraph
    block_mapping = sorted(block_indices)
    global_to_local = {global_idx: local_idx for local_idx, global_idx in enumerate(block_mapping)}

    n_subgraph = len(block_mapping)
    subgraph_adj = [[] for _ in range(n_subgraph)]

    for global_idx in block_indices:
        local_idx = global_to_local[global_idx]
        for neighbor_global in adjacency[global_idx]:
            if neighbor_global in block_indices:
                neighbor_local = global_to_local[neighbor_global]
                subgraph_adj[local_idx].append(neighbor_local)

    subgraph_weights = np.array([vertex_weights[i] for i in block_mapping], dtype=np.int32)

    # Extract subgraph edge weights
    subgraph_edge_weights = _extract_subgraph_edge_weights(edge_weights, block_indices, global_to_local)

    # Calculate target weights
    # Try to get VRA targets from tree if in VRA mode
    if vra_mode and vra_target_tree is not None:
        # Extract binary path from node name
        node_name = task['name']
        binary_path = ''.join(c for c in node_name if c in '01')

        # Navigate tree
        tree_node = vra_target_tree
        for direction in binary_path:
            if tree_node.get('is_leaf', False):
                break
            if direction == '0':
                tree_node = tree_node.get('left', {})
            else:
                tree_node = tree_node.get('right', {})

        # Check if found matching node
        if tree_node.get('districts') == k and not tree_node.get('is_leaf', False):
            target_weights = tree_node.get('target_weights')
            if debug:
                print(f"[VRA Worker] Using VRA targets for {node_name} ({k} districts): {target_weights}")
        else:
            target_weights = [k_left / k, k_right / k]
    else:
        target_weights = [k_left / k, k_right / k]

    # Calculate ufactor (use override if provided, otherwise depth-based)
    if ufactor_override is not None:
        # Convert integer to decimal (e.g., 5 -> 1.005)
        ufactor = 1.0 + (ufactor_override / 1000.0)
    else:
        # Original depth-based calculation
        if depth == 1:
            ufactor = 1.001
        elif depth == 2:
            ufactor = 1.002
        elif depth == 3:
            ufactor = 1.003
        else:
            ufactor = 1.005

    # Call METIS
    parts = partition_graph(
        adjacency=subgraph_adj,
        vertex_weights=subgraph_weights,
        nparts=2,
        target_weights=target_weights,
        recursive=True,
        ufactor=ufactor,
        niter=niter,
        objtype=objtype,
        seed=seed,
        debug=debug,
        edge_weights=subgraph_edge_weights,
        multi_constraint=(vra_target_tree is not None)  # only when using 2D vertex weights
    )

    # Create result
    left_blocks = {block_mapping[i] for i, p in enumerate(parts) if p == 0}
    right_blocks = {block_mapping[i] for i, p in enumerate(parts) if p == 1}

    left_name = f"{name}0"
    right_name = f"{name}1"

    # Calculate populations — always use first dim (handles both 1D and 2D weights)
    if vertex_weights.ndim == 2:
        left_pop = float(sum(vertex_weights[i, 0] for i in left_blocks))
        right_pop = float(sum(vertex_weights[i, 0] for i in right_blocks))
    else:
        left_pop = sum(vertex_weights[i] for i in left_blocks)
        right_pop = sum(vertex_weights[i] for i in right_blocks)

    return {
        'left_blocks': left_blocks,
        'right_blocks': right_blocks,
        'left_name': left_name,
        'right_name': right_name,
        'left_depth': depth + 1,
        'right_depth': depth + 1,
        'left_target': k_left,
        'right_target': k_right,
        'left_pop': left_pop,
        'right_pop': right_pop,
        'state_code': state_code
    }


class PartitionNode:
    """
    Node in the recursive bisection tree.

    Each node represents a region containing a set of census blocks.
    Leaf nodes represent final districts.
    """

    def __init__(self, block_indices: Set[int], depth: int = 0, name: str = "ROOT", state_code: str = ""):
        """
        Initialize partition node.

        Parameters
        ----------
        block_indices : Set[int]
            Set of block indices in this region
        depth : int, default 0
            Depth in the partition tree (root = 0)
        name : str, default "ROOT"
            Hierarchical binary path name (e.g., "CA0", "CA01", "CA101")
        state_code : str, default ""
            State code for naming (e.g., "CA")
        """
        self.block_indices = block_indices
        self.depth = depth
        self.name = name
        self.state_code = state_code
        self.left_child = None
        self.right_child = None
        self.district_id = None
        self.population = 0
        self.target_districts = 0  # Number of final districts this region will contain

    def is_leaf(self) -> bool:
        """Check if this is a leaf node (final district)."""
        return self.left_child is None and self.right_child is None

    def get_leaves(self) -> List['PartitionNode']:
        """Get all leaf nodes (districts) under this node."""
        if self.is_leaf():
            return [self]
        leaves = []
        if self.left_child:
            leaves.extend(self.left_child.get_leaves())
        if self.right_child:
            leaves.extend(self.right_child.get_leaves())
        return leaves


class RecursiveBisection:
    """
    Recursive bifurcation redistricting algorithm.

    Uses METIS gpmetis to recursively split regions in half until
    each region contains a single district.

    Example for 52 districts:
        52 → 26/26 → 13/13, 13/13 → 7/6, 7/6, 7/6, 7/6 → ...
    """

    def __init__(
        self,
        adjacency: List[List[int]],
        vertex_weights: np.ndarray,
        num_districts: int,
        save_intermediate: bool = False,
        intermediate_dir: Optional[str] = None,
        state_code: str = "",
        tqdm_position: int = 0,
        debug: bool = False,
        edge_weights: Optional[Dict[Tuple[int, int], float]] = None,
        ufactor: int = 5,
        niter: int = 100,
        objtype: str = 'cut',
        seed: Optional[int] = None,
        vra_mode: bool = False,
        vra_target_weights: Optional[List[List[float]]] = None
    ):
        """
        Initialize recursive bisection algorithm.

        Parameters
        ----------
        adjacency : List[List[int]]
            Adjacency list in CSR format
        vertex_weights : np.ndarray
            Vertex weights (block populations)
        num_districts : int
            Target number of districts
        save_intermediate : bool, default False
            Save intermediate partitions at each recursion level
        intermediate_dir : str, optional
            Directory to save intermediate results
        state_code : str, optional
            State code for region naming (e.g., "CA")
        debug : bool, default False
            Print detailed statistics and progress messages
        edge_weights : dict, optional
            Dictionary mapping (i, j) tuples to edge weights (boundary lengths in meters)
        ufactor : int, default 5
            METIS imbalance tolerance factor (5 = 0.5%)
        niter : int, default 100
            METIS refinement iterations
        objtype : str, default 'cut'
            METIS objective function ('cut' or 'vol')
        seed : int, optional
            METIS random seed for reproducibility
        vra_mode : bool, default False
            Enable VRA-aware multi-constraint partitioning
        vra_target_weights : Dict, optional
            VRA target tree (from vra_targets.create_vra_target_tree)
            Contains target weights for each node in the recursion tree
        """
        self.adjacency = adjacency
        self.vertex_weights = vertex_weights
        self.num_districts = num_districts
        self.n_blocks = len(vertex_weights)
        self.root = None
        self.save_intermediate = save_intermediate
        self.intermediate_dir = intermediate_dir
        self.state_code = state_code
        self.tqdm_position = tqdm_position
        self.debug = debug
        self.edge_weights = edge_weights
        self.ufactor = ufactor
        self.niter = niter
        self.objtype = objtype
        self.seed = seed
        self.vra_mode = vra_mode
        self.vra_target_tree = vra_target_weights  # Renamed: now expects tree, not weights

        # Track intermediate results by depth
        self.intermediate_results = {}  # depth -> assignments dict

        # Statistics — vertex_weights is always 1D (population only).
        # VRA edge-weighting does not use 2D multi-constraint vertex weights.
        self.total_population = int(
            vertex_weights[:, 0].sum() if vertex_weights.ndim == 2 else vertex_weights.sum()
        )

        if num_districts == 0:
            raise ValueError(f"num_districts cannot be 0 (state: {state_code})")

        if self.total_population == 0:
            raise ValueError(f"Total population is 0 - check input data (state: {state_code})")

        self.ideal_district_pop = self.total_population / num_districts

    def _get_vra_target_weights_for_node(self, node) -> Optional[List[List[float]]]:
        """
        Get VRA target weights for a specific node from the target tree.

        Parameters
        ----------
        node : PartitionNode
            Node being split

        Returns
        -------
        Optional[List[List[float]]]
            Target weights [[left_pop, left_minority], [right_pop, right_minority]]
            or None if not in VRA mode or node not found in tree
        """
        if not self.vra_mode or self.vra_target_tree is None:
            return None

        # Navigate tree based on node's target_districts count and path
        # Node path is encoded in its name (e.g., "AL01" = left then right)
        target_districts = node.target_districts

        # Extract binary path from node name (remove state code prefix)
        # E.g., "AL01" -> "01", "CA101" -> "101"
        name = node.name
        binary_path = ""
        for char in name:
            if char in '01':
                binary_path += char

        # Navigate to matching node in VRA tree
        tree_node = self.vra_target_tree
        for direction in binary_path:
            if tree_node.get('is_leaf', False):
                break
            if direction == '0':
                tree_node = tree_node.get('left', {})
            else:
                tree_node = tree_node.get('right', {})

        # Check if we found the right node (matching district count)
        if tree_node.get('districts') == target_districts and not tree_node.get('is_leaf', False):
            return tree_node.get('target_weights')

        return None

    def partition(self) -> Dict[int, int]:
        """
        Perform recursive bifurcation to create districts.

        Returns
        -------
        Dict[int, int]
            Mapping from block index to district_id
        """
        # Initialize root with all blocks
        all_blocks = set(range(self.n_blocks))
        root_name = self.state_code if self.state_code else "ROOT"
        self.root = PartitionNode(all_blocks, depth=0, name=root_name, state_code=self.state_code)
        self.root.population = self.total_population
        self.root.target_districts = self.num_districts

        # Calculate max depth (number of rounds)
        max_depth = self._calculate_max_depth(self.num_districts)

        # Track total splits completed (for progress reporting to parent)
        total_splits_needed = self.num_districts - 1
        splits_completed = 0

        # Process splits level-by-level (round-by-round)
        for depth in range(1, max_depth + 1):
            # Get all nodes at previous depth that need splitting
            nodes_to_split = self._get_nodes_to_split_at_depth(depth - 1)

            if not nodes_to_split:
                break

            # Count how many splits in this round
            num_splits = len(nodes_to_split)

            # Get state name for better descriptions
            state_name = self.state_code
            try:
                from scripts.config_2020 import STATE_CONFIG_2020
                state_name = STATE_CONFIG_2020.get(self.state_code, {}).get('name', self.state_code)
            except ImportError:
                try:
                    from scripts.config_2010 import STATE_CONFIG_2010
                    state_name = STATE_CONFIG_2010.get(self.state_code, {}).get('name', self.state_code)
                except ImportError:
                    pass

            # Create single progress bar showing round and splits (but not if position==999)
            if self.tqdm_position == 999:
                # Parallel mode - use dummy progress bar (no display)
                round_pbar = tqdm(total=num_splits, disable=True)
            else:
                round_pbar = tqdm(total=num_splits,
                                desc=f"{state_name} [{self.num_districts}D] Round {depth}/{max_depth}: {num_splits} splits",
                                unit="split",
                                ncols=120,
                                position=self.tqdm_position,
                                leave=False)

            # No second progress bar - all info in one line
            split_pbar = round_pbar  # Use same bar for updates

            # Split nodes in parallel if multiple nodes
            if num_splits > 1:
                splits_completed = self._split_nodes_parallel(nodes_to_split, split_pbar, round_pbar,
                                                              splits_completed, total_splits_needed)
            else:
                # Single node - no need for parallelization overhead
                self._split_single_node(nodes_to_split[0])
                split_pbar.update(1)
                round_pbar.update(1)
                splits_completed += 1
                # Print progress for parent process
                if self.tqdm_position == 999:
                    print(f"PROGRESS:{splits_completed}/{total_splits_needed}", flush=True)

            # Close progress bar for this round (only once since they're the same)
            round_pbar.close()

        self._assign_district_ids()

        # Extract final assignments
        assignments = self._extract_assignments()

        # Save intermediate results if requested
        if self.save_intermediate:
            self._save_intermediate_results()

        # Print statistics if debug mode is enabled
        # _print_statistics() will check self.debug internally
        self._print_statistics()

        return assignments

    def _split_node(self, node: PartitionNode, k: int, pbar: Optional[tqdm] = None):
        """
        Recursively split node into k districts.

        Parameters
        ----------
        node : PartitionNode
            Node to split
        k : int
            Number of districts to create from this node
        pbar : tqdm, optional
            Progress bar
        """
        if k == 1:
            # Base case: single district, no split needed
            node.district_id = -1  # Will be assigned later
            return

        # Calculate split sizes (handle odd numbers)
        k_left = k // 2
        k_right = k - k_left  # e.g., 13 → 7/6

        # Create subgraph for this node's blocks
        subgraph_adj, subgraph_weights, block_mapping = self._create_subgraph(node.block_indices)

        # Extract subgraph edge weights
        global_to_local = {global_idx: local_idx for local_idx, global_idx in enumerate(block_mapping)}
        subgraph_edge_weights = _extract_subgraph_edge_weights(self.edge_weights, node.block_indices, global_to_local)

        # Calculate target weights for METIS
        # Try to get VRA target weights first if in VRA mode
        vra_targets = self._get_vra_target_weights_for_node(node)
        if vra_targets is not None:
            target_weights = vra_targets
            if self.debug:
                print(f"[VRA] Using VRA targets for {node.name} ({k} districts): {target_weights}")
        else:
            # Standard mode: proportional split
            # Left partition should get k_left/k of total population
            # Right partition should get k_right/k of total population
            target_weights = [k_left / k, k_right / k]

        # Calculate dynamic ufactor: tighter at early depths, looser at later depths
        # Depth 1 (2 regions): 1.001 (0.1% tolerance) - very tight
        # Depth 2 (4 regions): 1.002 (0.2% tolerance)
        # Depth 3 (8 regions): 1.003 (0.3% tolerance)
        # Depth 4+: 1.005 (0.5% tolerance)
        if node.depth == 1:
            ufactor = 1.001
        elif node.depth == 2:
            ufactor = 1.002
        elif node.depth == 3:
            ufactor = 1.003
        else:
            ufactor = 1.005

        # Call METIS to split into 2 parts
        try:
            parts = partition_graph(
                adjacency=subgraph_adj,
                vertex_weights=subgraph_weights,
                nparts=2,
                target_weights=target_weights,
                recursive=True,
                ufactor=ufactor,
                niter=self.niter,
                objtype=self.objtype,
                seed=self.seed,
                debug=self.debug,
                edge_weights=subgraph_edge_weights,
                multi_constraint=(self.vra_target_tree is not None)  # only with 2D vertex weights
            )
        except Exception as e:
            raise RuntimeError(
                f"Failed to partition node at depth {node.depth} with {len(node.block_indices)} blocks: {e}"
            )

        # Create child nodes with hierarchical binary path names
        left_blocks = {block_mapping[i] for i, p in enumerate(parts) if p == 0}
        right_blocks = {block_mapping[i] for i, p in enumerate(parts) if p == 1}

        # Generate child names using binary encoding
        # Root (e.g., "CA") → "CA0", "CA1"
        # "CA0" → "CA00", "CA01"
        # "CA1" → "CA10", "CA11"
        left_name = f"{node.name}0"
        right_name = f"{node.name}1"

        node.left_child = PartitionNode(left_blocks, node.depth + 1, left_name, node.state_code)
        node.right_child = PartitionNode(right_blocks, node.depth + 1, right_name, node.state_code)

        # Set target districts for children
        node.left_child.target_districts = k_left
        node.right_child.target_districts = k_right

        # Calculate populations — always use first dim (handles both 1D and 2D weights)
        if self.vertex_weights.ndim == 2:
            node.left_child.population = float(sum(self.vertex_weights[i, 0] for i in left_blocks))
            node.right_child.population = float(sum(self.vertex_weights[i, 0] for i in right_blocks))
        else:
            node.left_child.population = sum(self.vertex_weights[i] for i in left_blocks)
            node.right_child.population = sum(self.vertex_weights[i] for i in right_blocks)

        # Update progress
        if pbar:
            pbar.update(1)

        # Recursively split children
        self._split_node(node.left_child, k_left, pbar)
        self._split_node(node.right_child, k_right, pbar)

    def _calculate_max_depth(self, k: int) -> int:
        """Calculate maximum depth needed to create k districts."""
        if k == 1:
            return 0
        depth = 0
        current = 1
        while current < k:
            current *= 2
            depth += 1
        return depth

    def _get_nodes_to_split_at_depth(self, depth: int) -> List[PartitionNode]:
        """
        Get all nodes at a specific depth that need splitting (target_districts > 1).

        Parameters
        ----------
        depth : int
            Depth level to check

        Returns
        -------
        List[PartitionNode]
            Nodes at this depth that need to be split
        """
        if depth == 0:
            # Root node
            if self.root.target_districts > 1:
                return [self.root]
            return []

        nodes = []
        queue = [(self.root, 0)]

        while queue:
            node, curr_depth = queue.pop(0)

            if curr_depth == depth:
                # At target depth - include if needs splitting
                if node.target_districts > 1:
                    nodes.append(node)
            elif curr_depth < depth:
                # Haven't reached target depth yet - continue traversing
                if node.left_child:
                    queue.append((node.left_child, curr_depth + 1))
                if node.right_child:
                    queue.append((node.right_child, curr_depth + 1))

        return nodes

    def _split_nodes_parallel(self, nodes: List[PartitionNode], split_pbar, round_pbar,
                             splits_completed: int, total_splits_needed: int) -> int:
        """
        Split multiple nodes in parallel using multiprocessing.

        Parameters
        ----------
        nodes : List[PartitionNode]
            Nodes to split in parallel
        split_pbar : tqdm
            Progress bar for splitting operations
        round_pbar : tqdm
            Progress bar for round progress
        splits_completed : int
            Number of splits completed so far
        total_splits_needed : int
            Total number of splits needed for all districts

        Returns
        -------
        int
            Updated number of splits completed
        """
        from concurrent.futures import ProcessPoolExecutor, as_completed
        import multiprocessing

        # Prepare split tasks (node data that can be pickled)
        split_tasks = []
        for node in nodes:
            task = {
                'block_indices': node.block_indices,
                'target_districts': node.target_districts,
                'depth': node.depth,
                'name': node.name,
                'state_code': node.state_code,
                'adjacency': self.adjacency,
                'vertex_weights': self.vertex_weights,
                'debug': self.debug,
                'edge_weights': self.edge_weights,
                'ufactor_override': self.ufactor,
                'niter': self.niter,
                'objtype': self.objtype,
                'seed': self.seed,
                'vra_mode': self.vra_mode,
                'vra_target_tree': self.vra_target_tree
            }
            split_tasks.append((node, task))

        # Use up to 4 workers (or number of CPUs, whichever is smaller)
        max_workers = min(4, multiprocessing.cpu_count())

        # Execute splits in parallel
        with ProcessPoolExecutor(max_workers=max_workers) as executor:
            # Submit all tasks
            future_to_node = {
                executor.submit(_split_node_worker, task): node
                for node, task in split_tasks
            }

            # Process completed tasks and update progress
            for future in as_completed(future_to_node):
                node = future_to_node[future]
                try:
                    result = future.result()
                    # Apply the split result to the node
                    self._apply_split_result(node, result)

                    # Update progress bars
                    split_pbar.update(1)
                    round_pbar.update(1)
                    splits_completed += 1
                    # Print progress for parent process
                    if self.tqdm_position == 999:
                        print(f"PROGRESS:{splits_completed}/{total_splits_needed}", flush=True)
                except Exception as e:
                    raise RuntimeError(f"Parallel split failed for node {node.name}: {e}")

        return splits_completed

    def _apply_split_result(self, node: PartitionNode, result: dict):
        """
        Apply the result of a parallel split to a node.

        Parameters
        ----------
        node : PartitionNode
            Node to update
        result : dict
            Split result containing child node data
        """
        # Create child nodes from result
        node.left_child = PartitionNode(
            result['left_blocks'],
            result['left_depth'],
            result['left_name'],
            result['state_code']
        )
        node.right_child = PartitionNode(
            result['right_blocks'],
            result['right_depth'],
            result['right_name'],
            result['state_code']
        )

        node.left_child.target_districts = result['left_target']
        node.right_child.target_districts = result['right_target']
        node.left_child.population = result['left_pop']
        node.right_child.population = result['right_pop']

    def _split_single_node(self, node: PartitionNode):
        """
        Split a single node into two children (non-recursive).

        Parameters
        ----------
        node : PartitionNode
            Node to split
        """
        k = node.target_districts

        if k == 1:
            # No split needed
            return

        # Calculate split sizes
        k_left = k // 2
        k_right = k - k_left

        # Create subgraph for this node's blocks
        subgraph_adj, subgraph_weights, block_mapping = self._create_subgraph(node.block_indices)

        # Extract subgraph edge weights
        global_to_local = {global_idx: local_idx for local_idx, global_idx in enumerate(block_mapping)}
        subgraph_edge_weights = _extract_subgraph_edge_weights(self.edge_weights, node.block_indices, global_to_local)

        # Calculate target weights for METIS
        # Try to get VRA-specific targets first, otherwise use proportional split
        vra_targets = self._get_vra_target_weights_for_node(node)
        if vra_targets is not None:
            target_weights = vra_targets
            if self.debug:
                print(f"[VRA] Using VRA targets for node {node.name} ({k} districts): {vra_targets}")
        else:
            # Standard mode: proportional population split
            target_weights = [k_left / k, k_right / k]

        # Calculate dynamic ufactor
        if node.depth == 1:
            ufactor = 1.001
        elif node.depth == 2:
            ufactor = 1.002
        elif node.depth == 3:
            ufactor = 1.003
        else:
            ufactor = 1.005

        # Call METIS to split into 2 parts
        try:
            parts = partition_graph(
                adjacency=subgraph_adj,
                vertex_weights=subgraph_weights,
                nparts=2,
                target_weights=target_weights,
                recursive=True,
                ufactor=ufactor,
                niter=self.niter,
                objtype=self.objtype,
                seed=self.seed,
                debug=self.debug,
                edge_weights=subgraph_edge_weights,
                multi_constraint=(self.vra_target_tree is not None)  # only with 2D vertex weights
            )
        except Exception as e:
            raise RuntimeError(
                f"Failed to partition node at depth {node.depth} with {len(node.block_indices)} blocks: {e}"
            )

        # Create child nodes
        left_blocks = {block_mapping[i] for i, p in enumerate(parts) if p == 0}
        right_blocks = {block_mapping[i] for i, p in enumerate(parts) if p == 1}

        left_name = f"{node.name}0"
        right_name = f"{node.name}1"

        node.left_child = PartitionNode(left_blocks, node.depth + 1, left_name, node.state_code)
        node.right_child = PartitionNode(right_blocks, node.depth + 1, right_name, node.state_code)

        node.left_child.target_districts = k_left
        node.right_child.target_districts = k_right

        # Calculate populations — always use first dim (handles both 1D and 2D weights)
        if self.vertex_weights.ndim == 2:
            node.left_child.population = float(sum(self.vertex_weights[i, 0] for i in left_blocks))
            node.right_child.population = float(sum(self.vertex_weights[i, 0] for i in right_blocks))
        else:
            node.left_child.population = sum(self.vertex_weights[i] for i in left_blocks)
            node.right_child.population = sum(self.vertex_weights[i] for i in right_blocks)

    def _create_subgraph(self, block_indices: Set[int]):
        """
        Create subgraph for a subset of blocks.

        Parameters
        ----------
        block_indices : Set[int]
            Set of block indices to include

        Returns
        -------
        subgraph_adj : List[List[int]]
            Adjacency list for subgraph (local indices)
        subgraph_weights : np.ndarray
            Vertex weights for subgraph
        block_mapping : List[int]
            Mapping from local index to global block index
        """
        # Create mapping from global index to local index
        block_mapping = sorted(block_indices)
        global_to_local = {global_idx: local_idx for local_idx, global_idx in enumerate(block_mapping)}

        n_subgraph = len(block_mapping)

        # Build subgraph adjacency list
        subgraph_adj = [[] for _ in range(n_subgraph)]

        for global_idx in block_indices:
            local_idx = global_to_local[global_idx]
            # Add neighbors that are also in the subgraph
            for neighbor_global in self.adjacency[global_idx]:
                if neighbor_global in block_indices:
                    neighbor_local = global_to_local[neighbor_global]
                    subgraph_adj[local_idx].append(neighbor_local)

        # Extract subgraph weights
        subgraph_weights = np.array([self.vertex_weights[i] for i in block_mapping], dtype=np.int32)

        return subgraph_adj, subgraph_weights, block_mapping

    def _count_splits(self, k: int) -> int:
        """Count total number of splits needed for k districts."""
        if k == 1:
            return 0
        return 1 + self._count_splits(k // 2) + self._count_splits(k - k // 2)

    def _assign_district_ids(self):
        """Assign sequential district IDs (1 to num_districts) to leaf nodes."""
        leaves = self.root.get_leaves()
        assert len(leaves) == self.num_districts, \
            f"Expected {self.num_districts} districts, got {len(leaves)}"

        # Sort leaves by some criteria (e.g., leftmost in tree)
        # For now, just assign in tree traversal order
        for district_id, leaf in enumerate(leaves, start=1):
            leaf.district_id = district_id

    def _extract_assignments(self) -> Dict[int, int]:
        """
        Extract block-to-district assignments from leaf nodes.

        Returns
        -------
        Dict[int, int]
            Mapping from block index to district_id
        """
        assignments = {}
        leaves = self.root.get_leaves()

        for leaf in leaves:
            for block_idx in leaf.block_indices:
                assignments[block_idx] = leaf.district_id

        assert len(assignments) == self.n_blocks, \
            f"Missing blocks in assignments: {self.n_blocks} blocks, {len(assignments)} assigned"

        return assignments

    def _print_statistics(self):
        """Print redistricting statistics."""
        if not self.debug:
            return

        leaves = self.root.get_leaves()

        populations = [leaf.population for leaf in leaves]

        # Avoid division by zero
        if self.ideal_district_pop == 0:
            deviations = [0.0 for _ in populations]
        else:
            deviations = [
                abs(pop - self.ideal_district_pop) / self.ideal_district_pop * 100
                for pop in populations
            ]

        # Only print statistics when debug is enabled
        if self.debug:
            print("\nRedistricting Statistics:")
            print(f"  Districts created: {len(leaves)}")
            print(f"  Population range: {min(populations):,} - {max(populations):,}")
            print(f"  Mean deviation from ideal: ±{np.mean(deviations):.2f}%")
            print(f"  Max deviation from ideal: ±{max(deviations):.2f}%")

            # Print worst 5 districts
            worst_districts = sorted(
                [(leaf.district_id, leaf.population, dev) for leaf, dev in zip(leaves, deviations)],
                key=lambda x: x[2],
                reverse=True
            )[:5]

            print("\n  Most unbalanced districts:")
            for district_id, pop, dev in worst_districts:
                print(f"    District {district_id}: {pop:,} ({dev:+.2f}%)")

    def _get_nodes_at_depth(self, depth: int) -> List[PartitionNode]:
        """
        Get all nodes at a specific depth in the tree.

        Parameters
        ----------
        depth : int
            Depth level (0 = root, 1 = after first split, etc.)

        Returns
        -------
        List[PartitionNode]
            All nodes at the specified depth
        """
        if depth == 0:
            return [self.root]

        nodes = []
        queue = [(self.root, 0)]

        while queue:
            node, curr_depth = queue.pop(0)

            if curr_depth == depth:
                nodes.append(node)
            elif curr_depth < depth:
                if node.left_child:
                    queue.append((node.left_child, curr_depth + 1))
                if node.right_child:
                    queue.append((node.right_child, curr_depth + 1))

        return nodes

    def _get_leaves_up_to_depth(self, depth: int) -> List[PartitionNode]:
        """
        Get all leaf nodes at or before a specific depth.

        This handles cases where some branches terminate early (e.g., when k=1)
        while others continue splitting.

        Parameters
        ----------
        depth : int
            Maximum depth to consider

        Returns
        -------
        List[PartitionNode]
            All leaf nodes at depth <= depth
        """
        leaves = []
        queue = [(self.root, 0)]

        while queue:
            node, curr_depth = queue.pop(0)

            if node.is_leaf():
                # This is a leaf - include if at or before target depth
                if curr_depth <= depth:
                    leaves.append(node)
            elif curr_depth < depth:
                # Not a leaf and haven't reached target depth - continue
                if node.left_child:
                    queue.append((node.left_child, curr_depth + 1))
                if node.right_child:
                    queue.append((node.right_child, curr_depth + 1))
            else:
                # Reached target depth - treat as leaf even if not terminal
                leaves.append(node)

        return leaves

    def get_assignments_at_depth(self, depth: int) -> Dict[int, int]:
        """
        Extract block-to-region assignments at a specific depth.

        Includes all leaves up to and at this depth to handle cases where
        some branches terminate early (when target districts reached).

        Parameters
        ----------
        depth : int
            Depth level (0 = 1 region, 1 = 2 regions, 2 = 4 regions, etc.)

        Returns
        -------
        Dict[int, int]
            Mapping from block index to region_id (0-indexed)
        """
        nodes = self._get_leaves_up_to_depth(depth)

        assignments = {}
        for region_id, node in enumerate(nodes):
            for block_idx in node.block_indices:
                assignments[block_idx] = region_id

        return assignments

    def _save_intermediate_results(self):
        """Save intermediate partition results at each depth level."""
        if not self.intermediate_dir:
            return

        from pathlib import Path
        import json

        intermediate_path = Path(self.intermediate_dir)
        intermediate_path.mkdir(parents=True, exist_ok=True)

        # Calculate max depth
        max_depth = 0
        queue = [(self.root, 0)]
        while queue:
            node, depth = queue.pop(0)
            max_depth = max(max_depth, depth)
            if node.left_child:
                queue.append((node.left_child, depth + 1))
            if node.right_child:
                queue.append((node.right_child, depth + 1))

        # Save assignments at each depth
        for depth in range(1, max_depth + 1):
            # Get all leaves up to this depth (handles uneven tree depths)
            nodes = self._get_leaves_up_to_depth(depth)
            num_regions = len(nodes)

            assignments = self.get_assignments_at_depth(depth)

            # Calculate statistics for this level
            region_stats = []
            for region_id, node in enumerate(nodes):
                pop = node.population
                num_blocks = len(node.block_indices)
                # Handle numpy types - convert to Python scalar
                if isinstance(pop, np.ndarray):
                    pop_int = int(pop.item())
                elif isinstance(pop, (np.integer, np.floating)):
                    pop_int = int(pop)
                else:
                    pop_int = int(pop)
                region_stats.append({
                    'region_id': region_id,
                    'name': node.name,
                    'population': pop_int,
                    'num_blocks': num_blocks,
                    'target_districts': node.target_districts
                })

            # Save metadata
            metadata = {
                'depth': depth,
                'num_regions': num_regions,
                'total_districts': self.num_districts,
                'total_population': self.total_population,
                'regions': region_stats
            }

            # Save to files
            prefix = f"round_{depth}_{num_regions}_regions"

            # Save assignments (block_idx -> region_id)
            assignments_file = intermediate_path / f"{prefix}_assignments.json"
            with open(assignments_file, 'w') as f:
                json.dump(assignments, f)

            # Save metadata
            metadata_file = intermediate_path / f"{prefix}_metadata.json"
            with open(metadata_file, 'w') as f:
                json.dump(metadata, f, indent=2)

            if self.debug:
                print(f"  Round {depth}: {num_regions} regions saved")


def calculate_split_sizes(k: int) -> tuple:
    """
    Calculate split sizes for k-way partition.

    Parameters
    ----------
    k : int
        Number of partitions

    Returns
    -------
    tuple
        (k_left, k_right) split sizes

    Examples
    --------
    >>> calculate_split_sizes(52)
    (26, 26)
    >>> calculate_split_sizes(26)
    (13, 13)
    >>> calculate_split_sizes(13)
    (7, 6)
    """
    k_left = k // 2
    k_right = k - k_left
    return k_left, k_right
