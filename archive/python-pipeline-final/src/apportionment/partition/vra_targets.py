"""
VRA Target Weight Calculation for Recursive Bisection

Calculates target weights at each level of the binary tree to achieve
desired minority concentration in final districts.

Bottom-up approach:
1. Define final district targets (which should be MM, what %)
2. Map binary tree structure
3. Calculate intermediate targets by summing descendant needs
4. Use at each recursive split to concentrate minority population
"""

from typing import List, Tuple, Dict
import numpy as np


def calculate_final_district_targets(
    num_districts: int,
    target_mm_districts: int,
    state_minority_pct: float,
    mm_target_pct: float = 0.60
) -> List[Dict]:
    """
    Calculate target minority percentage for each final district.

    Parameters
    ----------
    num_districts : int
        Total number of districts
    target_mm_districts : int
        Number of districts that should be majority-minority
    state_minority_pct : float
        State-wide minority percentage (e.g., 0.369 for Alabama)
    mm_target_pct : float
        Target minority percentage for MM districts (e.g., 0.60 for 60%)

    Returns
    -------
    List[Dict]
        List of district targets with 'district', 'pop_fraction', 'minority_fraction'
    """
    pop_per_district = 1.0 / num_districts

    # Calculate minority used by MM districts
    mm_minority_total = target_mm_districts * pop_per_district * mm_target_pct

    # Remaining minority for non-MM districts
    remaining_minority = state_minority_pct - mm_minority_total
    non_mm_districts = num_districts - target_mm_districts

    if non_mm_districts > 0:
        non_mm_minority_per_district = remaining_minority / non_mm_districts / pop_per_district
    else:
        non_mm_minority_per_district = 0.0

    # Create district targets
    targets = []
    for i in range(num_districts):
        is_mm = i < target_mm_districts  # First N districts are MM

        targets.append({
            'district': i + 1,
            'pop_fraction': pop_per_district,
            'minority_pct_of_district': mm_target_pct if is_mm else non_mm_minority_per_district,
            'minority_fraction_of_state': pop_per_district * (mm_target_pct if is_mm else non_mm_minority_per_district)
        })

    return targets


def build_binary_tree_structure(num_districts: int, first_split=None) -> Dict:
    """
    Build the binary tree structure for recursive bisection.

    Returns a tree showing how num_districts are split recursively.
    Each node has: left_count, right_count, depth

    Parameters
    ----------
    num_districts : int
        Total number of districts
    first_split : tuple, optional
        (left_count, right_count) for first split
        If None, uses default balanced split
    """
    def split_recursive(n: int, depth: int = 0, path: str = "", custom_split=None) -> Dict:
        if n == 1:
            return {
                'districts': 1,
                'depth': depth,
                'path': path,
                'is_leaf': True
            }

        # Use custom split at root level if provided
        if depth == 0 and custom_split is not None:
            left_n, right_n = custom_split
        else:
            # Split as evenly as possible
            left_n = n // 2
            right_n = n - left_n

        return {
            'districts': n,
            'depth': depth,
            'path': path,
            'is_leaf': False,
            'left_count': left_n,
            'right_count': right_n,
            'left': split_recursive(left_n, depth + 1, path + "0"),
            'right': split_recursive(right_n, depth + 1, path + "1")
        }

    return split_recursive(num_districts, custom_split=first_split)


def assign_districts_to_tree(
    tree: Dict,
    district_targets: List[Dict],
    mm_clustering: str = 'left'
) -> Dict:
    """
    Assign final districts to leaf nodes of the binary tree.

    Parameters
    ----------
    tree : Dict
        Binary tree structure from build_binary_tree_structure()
    district_targets : List[Dict]
        Final district targets
    mm_clustering : str
        How to cluster MM districts: 'left', 'right', or 'balanced'

    Returns
    -------
    Dict
        Tree with district assignments at leaves
    """
    # Sort districts by minority percentage (descending) to cluster MM districts
    sorted_districts = sorted(district_targets,
                             key=lambda x: x['minority_pct_of_district'],
                             reverse=True)

    # Collect all leaf nodes
    def collect_leaves(node: Dict, leaves: List):
        if node['is_leaf']:
            leaves.append(node)
        else:
            collect_leaves(node['left'], leaves)
            collect_leaves(node['right'], leaves)

    leaves = []
    collect_leaves(tree, leaves)

    # Sort leaves by path (to get left-to-right ordering if mm_clustering='left')
    leaves.sort(key=lambda x: x['path'])

    # Assign districts to leaves
    for i, (leaf, district) in enumerate(zip(leaves, sorted_districts)):
        leaf['district_target'] = district

    return tree


def calculate_node_targets(tree: Dict, total_minority: float) -> Dict:
    """
    Calculate target weights for each node by summing descendant targets.

    Works bottom-up: sum minority needs of all descendant final districts.

    Parameters
    ----------
    tree : Dict
        Binary tree with district assignments
    total_minority : float
        Total minority population in state

    Returns
    -------
    Dict
        Tree with 'target_weights' at each node: [pop_fraction, minority_fraction]
    """
    def calculate_recursive(node: Dict) -> Tuple[float, float]:
        """Returns (pop_fraction, minority_amount_fraction_of_state)"""
        if node['is_leaf']:
            if 'district_target' in node:
                target = node['district_target']
                pop_frac = target['pop_fraction']
                minority_frac = target['minority_fraction_of_state']
                node['target_weights'] = [pop_frac, minority_frac]
                return pop_frac, minority_frac
            else:
                # Leaf without district (shouldn't happen)
                return 0.0, 0.0

        # Internal node: sum children
        left_pop, left_minority = calculate_recursive(node['left'])
        right_pop, right_minority = calculate_recursive(node['right'])

        total_pop = left_pop + right_pop
        total_minority_node = left_minority + right_minority

        # Calculate target weights for this split (as fractions of node's totals)
        # These represent what fraction of THIS NODE's resources go to each child
        left_pop_target = left_pop / total_pop if total_pop > 0 else 0.5
        right_pop_target = right_pop / total_pop if total_pop > 0 else 0.5

        left_minority_target = left_minority / total_minority_node if total_minority_node > 0 else 0.5
        right_minority_target = right_minority / total_minority_node if total_minority_node > 0 else 0.5

        node['target_weights'] = [
            [left_pop_target, left_minority_target],
            [right_pop_target, right_minority_target]
        ]

        return total_pop, total_minority_node

    calculate_recursive(tree)
    return tree


def get_target_weights_for_node(
    tree: Dict,
    target_districts_in_node: int,
    path: str = ""
) -> List[List[float]]:
    """
    Get target weights for a specific node in the tree.

    Parameters
    ----------
    tree : Dict
        Tree with calculated targets
    target_districts_in_node : int
        Number of districts this node will produce
    path : str
        Binary path to node (e.g., "01" = left then right)

    Returns
    -------
    List[List[float]]
        Target weights: [[left_pop, left_minority], [right_pop, right_minority]]
    """
    # Navigate to node
    node = tree
    for direction in path:
        if direction == '0':
            node = node['left']
        else:
            node = node['right']

    # Check if this node matches the target district count
    if node['districts'] == target_districts_in_node:
        return node.get('target_weights', None)

    return None


def create_vra_target_tree(
    num_districts: int,
    target_mm_districts: int,
    state_minority_pct: float,
    mm_target_pct: float = 0.60,
    first_split: tuple = None
) -> Dict:
    """
    Create complete VRA target tree for recursive bisection.

    This is the main function to call. It:
    1. Calculates final district targets
    2. Builds binary tree structure
    3. Assigns districts to tree
    4. Calculates targets at each node bottom-up

    Parameters
    ----------
    num_districts : int
        Total number of districts (e.g., 7 for Alabama)
    target_mm_districts : int
        Number of MM districts to create (e.g., 2 for Alabama)
    state_minority_pct : float
        State-wide minority percentage (e.g., 0.369)
    mm_target_pct : float
        Target minority % in MM districts (e.g., 0.60 for 60%)

    Returns
    -------
    Dict
        Complete tree with target_weights at each node

    Example
    -------
    >>> tree = create_vra_target_tree(7, 2, 0.369, 0.60)
    >>> # Get targets for root split (7 districts → 3 | 4)
    >>> root_targets = tree['target_weights']
    >>> print(root_targets)
    [[0.429, 0.572], [0.571, 0.428]]  # Left gets more minority
    """
    # Step 1: Calculate final district targets
    district_targets = calculate_final_district_targets(
        num_districts, target_mm_districts, state_minority_pct, mm_target_pct
    )

    # Step 2: Build binary tree (with optional custom first split)
    tree = build_binary_tree_structure(num_districts, first_split=first_split)

    # Step 3: Assign districts to tree (cluster MM districts on left)
    tree = assign_districts_to_tree(tree, district_targets, mm_clustering='left')

    # Step 4: Calculate targets at each node (bottom-up)
    tree = calculate_node_targets(tree, state_minority_pct)

    return tree


def print_vra_tree(tree: Dict, indent: int = 0):
    """Debug: Print VRA target tree structure."""
    prefix = "  " * indent

    if tree['is_leaf']:
        if 'district_target' in tree:
            target = tree['district_target']
            print(f"{prefix}District {target['district']}: {target['minority_pct_of_district']*100:.1f}% minority")
    else:
        weights = tree['target_weights']
        print(f"{prefix}{tree['districts']} districts -> [{tree['left_count']} | {tree['right_count']}]")
        print(f"{prefix}  Targets: L=[{weights[0][0]:.3f} pop, {weights[0][1]:.3f} min] R=[{weights[1][0]:.3f} pop, {weights[1][1]:.3f} min]")
        print(f"{prefix}  Left branch:")
        print_vra_tree(tree['left'], indent + 2)
        print(f"{prefix}  Right branch:")
        print_vra_tree(tree['right'], indent + 2)
