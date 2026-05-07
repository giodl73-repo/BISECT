"""
Analyze hierarchical structure from saved recursive bisection trees.

Generates:
1. Dendrograms for each state (2010 and 2020)
2. Level-wise stability analysis
3. Parent-child preservation metrics
4. Tree comparison visualizations
"""
import sys
from pathlib import Path

# Add paths for unpickling
BASE_DIR = Path(__file__).parent
PROJECT_ROOT = BASE_DIR.parent.parent
sys.path.insert(0, str(PROJECT_ROOT))  # For "src.apportionment..." imports
sys.path.insert(0, str(PROJECT_ROOT / "src"))  # For "apportionment..." imports

import pickle
from typing import Dict, List, Tuple, Set
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib
matplotlib.use('Agg')  # Use non-interactive backend

# Configuration
STATES = ['alabama', 'georgia', 'louisiana', 'mississippi', 'south_carolina']
TREE_DIR = BASE_DIR / "trees"
FIGURES_DIR = BASE_DIR / "figures" / "hierarchical"
FIGURES_DIR.mkdir(parents=True, exist_ok=True)


class TreeNode:
    """Simplified tree node for analysis."""
    def __init__(self, name: str, depth: int, population: int,
                 tract_count: int, left=None, right=None):
        self.name = name
        self.depth = depth
        self.population = population
        self.tract_count = tract_count
        self.left = left
        self.right = right

    def is_leaf(self):
        return self.left is None and self.right is None


def extract_tree_structure(root):
    """
    Extract tree structure from PartitionNode.

    Parameters
    ----------
    root : PartitionNode
        Root of the recursive bisection tree

    Returns
    -------
    TreeNode
        Simplified tree structure for analysis
    """
    if root is None:
        return None

    # Create simplified node
    node = TreeNode(
        name=root.name,
        depth=root.depth,
        population=root.population,
        tract_count=len(root.block_indices),
        left=None,
        right=None
    )

    # Recursively extract children
    if root.left_child is not None:
        node.left = extract_tree_structure(root.left_child)
    if root.right_child is not None:
        node.right = extract_tree_structure(root.right_child)

    return node


def get_tree_levels(root: TreeNode) -> Dict[int, List[TreeNode]]:
    """
    Get all nodes organized by tree level.

    Parameters
    ----------
    root : TreeNode
        Root of the tree

    Returns
    -------
    Dict[int, List[TreeNode]]
        Dictionary mapping depth to list of nodes at that depth
    """
    levels = {}

    def traverse(node):
        if node is None:
            return

        if node.depth not in levels:
            levels[node.depth] = []
        levels[node.depth].append(node)

        traverse(node.left)
        traverse(node.right)

    traverse(root)
    return levels


def compute_tree_similarity(tree1: TreeNode, tree2: TreeNode) -> float:
    """
    Compute structural similarity between two trees.

    Uses normalized tree edit distance based on node names.

    Parameters
    ----------
    tree1, tree2 : TreeNode
        Trees to compare

    Returns
    -------
    float
        Similarity score (0-1, 1 = identical structure)
    """
    def get_node_set(node, prefix=""):
        """Extract set of (path, depth) tuples."""
        if node is None:
            return set()

        nodes = {(prefix, node.depth)}
        nodes.update(get_node_set(node.left, prefix + "0"))
        nodes.update(get_node_set(node.right, prefix + "1"))
        return nodes

    nodes1 = get_node_set(tree1)
    nodes2 = get_node_set(tree2)

    if len(nodes1) == 0 and len(nodes2) == 0:
        return 1.0

    intersection = len(nodes1 & nodes2)
    union = len(nodes1 | nodes2)

    return intersection / union if union > 0 else 0.0


def compute_level_wise_stability(tree1: TreeNode, tree2: TreeNode) -> pd.DataFrame:
    """
    Compute stability at each level of the tree.

    Parameters
    ----------
    tree1, tree2 : TreeNode
        Trees from 2010 and 2020

    Returns
    -------
    pd.DataFrame
        Level-wise stability metrics
    """
    levels1 = get_tree_levels(tree1)
    levels2 = get_tree_levels(tree2)

    max_depth = max(max(levels1.keys()), max(levels2.keys()))

    results = []
    for depth in range(max_depth + 1):
        nodes1 = levels1.get(depth, [])
        nodes2 = levels2.get(depth, [])

        # Extract node paths (binary strings)
        def get_path(name):
            return ''.join(c for c in name if c in '01')

        paths1 = {get_path(n.name) for n in nodes1}
        paths2 = {get_path(n.name) for n in nodes2}

        # Compute overlap
        if len(paths1) == 0 and len(paths2) == 0:
            stability = 1.0
        else:
            intersection = len(paths1 & paths2)
            union = len(paths1 | paths2)
            stability = intersection / union if union > 0 else 0.0

        results.append({
            'level': depth,
            'nodes_2010': len(nodes1),
            'nodes_2020': len(nodes2),
            'common_paths': len(paths1 & paths2) if paths1 and paths2 else 0,
            'stability': stability
        })

    return pd.DataFrame(results)


def visualize_dendrogram(tree: TreeNode, year: int, state: str):
    """
    Create dendrogram visualization of tree structure.

    Parameters
    ----------
    tree : TreeNode
        Tree to visualize
    year : int
        Census year (2010 or 2020)
    state : str
        State name
    """
    fig, ax = plt.subplots(figsize=(12, 8))

    def plot_tree(node, x, y, width):
        """Recursively plot tree structure."""
        if node is None:
            return

        # Plot node
        color = 'lightblue' if node.is_leaf() else 'lightcoral'
        ax.scatter(x, y, s=100, c=color, zorder=3, edgecolors='black')

        # Add label
        label = f"{node.name}\nn={node.tract_count}\npop={node.population:,}"
        ax.text(x, y - 0.3, label, ha='center', va='top', fontsize=6)

        # Plot children
        if not node.is_leaf():
            # Left child
            left_x = x - width / 2
            left_y = y - 1
            ax.plot([x, left_x], [y, left_y], 'k-', linewidth=1)
            plot_tree(node.left, left_x, left_y, width / 2)

            # Right child
            right_x = x + width / 2
            right_y = y - 1
            ax.plot([x, right_x], [y, right_y], 'k-', linewidth=1)
            plot_tree(node.right, right_x, right_y, width / 2)

    # Calculate tree depth for spacing
    def get_depth(node):
        if node is None or node.is_leaf():
            return 0
        return 1 + max(get_depth(node.left), get_depth(node.right))

    depth = get_depth(tree)
    initial_width = 2 ** (depth - 1)

    # Plot tree
    plot_tree(tree, 0, depth, initial_width)

    # Format plot
    ax.set_title(f"{state.title()} {year} - Recursive Bisection Tree Structure",
                 fontsize=14, fontweight='bold')
    ax.set_xlabel("Horizontal Position")
    ax.set_ylabel("Tree Depth (0 = Root)")
    ax.grid(True, alpha=0.3)
    ax.invert_yaxis()  # Root at top

    # Legend
    from matplotlib.patches import Patch
    legend_elements = [
        Patch(facecolor='lightcoral', edgecolor='black', label='Internal Node'),
        Patch(facecolor='lightblue', edgecolor='black', label='Leaf (District)')
    ]
    ax.legend(handles=legend_elements, loc='upper right')

    plt.tight_layout()
    output_file = FIGURES_DIR / f"{state}_{year}_dendrogram.png"
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    plt.close()

    print(f"Saved dendrogram: {output_file}")


def visualize_tree_comparison(tree1: TreeNode, tree2: TreeNode, state: str):
    """
    Create side-by-side comparison of 2010 and 2020 trees.

    Parameters
    ----------
    tree1, tree2 : TreeNode
        Trees from 2010 and 2020
    state : str
        State name
    """
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(16, 8))

    def plot_tree_simple(node, ax, x, y, width):
        """Simple tree visualization for comparison."""
        if node is None:
            return

        color = 'lightblue' if node.is_leaf() else 'lightcoral'
        ax.scatter(x, y, s=80, c=color, zorder=3, edgecolors='black')

        if not node.is_leaf():
            left_x = x - width / 2
            left_y = y - 1
            ax.plot([x, left_x], [y, left_y], 'k-', linewidth=1, alpha=0.5)
            plot_tree_simple(node.left, ax, left_x, left_y, width / 2)

            right_x = x + width / 2
            right_y = y - 1
            ax.plot([x, right_x], [y, right_y], 'k-', linewidth=1, alpha=0.5)
            plot_tree_simple(node.right, ax, right_x, right_y, width / 2)

    # Get depths
    def get_depth(node):
        if node is None or node.is_leaf():
            return 0
        return 1 + max(get_depth(node.left), get_depth(node.right))

    depth1 = get_depth(tree1)
    depth2 = get_depth(tree2)

    # Plot 2010 tree
    plot_tree_simple(tree1, ax1, 0, depth1, 2 ** (depth1 - 1))
    ax1.set_title(f"{state.title()} 2010", fontsize=12, fontweight='bold')
    ax1.invert_yaxis()
    ax1.grid(True, alpha=0.3)
    ax1.set_ylabel("Tree Depth")

    # Plot 2020 tree
    plot_tree_simple(tree2, ax2, 0, depth2, 2 ** (depth2 - 1))
    ax2.set_title(f"{state.title()} 2020", fontsize=12, fontweight='bold')
    ax2.invert_yaxis()
    ax2.grid(True, alpha=0.3)

    plt.suptitle(f"{state.title()} - Tree Structure Comparison (2010 vs 2020)",
                 fontsize=14, fontweight='bold')

    plt.tight_layout()
    output_file = FIGURES_DIR / f"{state}_tree_comparison.png"
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    plt.close()

    print(f"Saved tree comparison: {output_file}")


def main():
    """Main analysis pipeline."""
    print("="*60)
    print("HIERARCHICAL STRUCTURE ANALYSIS")
    print("="*60)

    all_stability = []

    for state in STATES:
        print(f"\n{'='*60}")
        print(f"Analyzing {state.upper()}")
        print(f"{'='*60}")

        # Load trees
        tree_file_2010 = TREE_DIR / f"{state}_2010_tree.pkl"
        tree_file_2020 = TREE_DIR / f"{state}_2020_tree.pkl"

        if not tree_file_2010.exists() or not tree_file_2020.exists():
            print(f"ERROR: Missing tree files for {state}")
            continue

        with open(tree_file_2010, 'rb') as f:
            root_2010 = pickle.load(f)

        with open(tree_file_2020, 'rb') as f:
            root_2020 = pickle.load(f)

        # Extract simplified trees
        tree_2010 = extract_tree_structure(root_2010)
        tree_2020 = extract_tree_structure(root_2020)

        # Compute level-wise stability
        stability_df = compute_level_wise_stability(tree_2010, tree_2020)
        stability_df['state'] = state
        all_stability.append(stability_df)

        print(f"\nLevel-wise Stability:")
        print(stability_df[['level', 'nodes_2010', 'nodes_2020', 'stability']])

        # Compute overall tree similarity
        similarity = compute_tree_similarity(tree_2010, tree_2020)
        print(f"\nOverall tree similarity: {similarity:.2%}")

        # Generate visualizations
        print(f"\nGenerating visualizations...")
        visualize_dendrogram(tree_2010, 2010, state)
        visualize_dendrogram(tree_2020, 2020, state)
        visualize_tree_comparison(tree_2010, tree_2020, state)

    # Combine all stability results
    if all_stability:
        combined_df = pd.concat(all_stability, ignore_index=True)
        output_file = BASE_DIR / "results" / "hierarchical_stability.csv"
        combined_df.to_csv(output_file, index=False)
        print(f"\n{'='*60}")
        print(f"Saved combined stability results to: {output_file}")
        print(f"{'='*60}")

        # Print summary statistics
        print("\nSummary Statistics by Level:")
        summary = combined_df.groupby('level').agg({
            'stability': ['mean', 'std', 'min', 'max'],
            'state': 'count'
        }).round(3)
        print(summary)

if __name__ == "__main__":
    main()
