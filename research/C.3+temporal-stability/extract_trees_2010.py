"""
Extract hierarchical tree structures from 2010 recursive bisection runs.

Loads adjacency data from outputs/v1/data/2010/ and re-runs recursive bisection
with tree saving enabled.
"""
import sys
from pathlib import Path
import pickle

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from src.apportionment.partition.recursive_bisection import RecursiveBisection

# Configuration
STATES = {
    'AL': ('alabama', 7),
    'GA': ('georgia', 14),
    'LA': ('louisiana', 6),
    'MS': ('mississippi', 4),
    'SC': ('south_carolina', 7)
}

BASE_DIR = Path(__file__).parent
PROJECT_ROOT = BASE_DIR.parent.parent
TREE_DIR = BASE_DIR / "trees"
TREE_DIR.mkdir(exist_ok=True)

def extract_tree_for_state(state_code: str, state_name: str, n_districts: int, year: str = '2010'):
    """Extract tree structure for one state."""
    print(f"\n{'='*60}")
    print(f"Extracting tree: {state_name.upper()} ({year}) - {n_districts} districts")
    print(f"{'='*60}")

    # Load adjacency data
    adj_file = PROJECT_ROOT / f'outputs/v1/data/{year}/adjacency/{state_code.lower()}_adjacency_{year}.pkl'

    if not adj_file.exists():
        print(f"ERROR: Adjacency file not found: {adj_file}")
        return

    print(f"Loading adjacency from {adj_file.name}...")
    with open(adj_file, 'rb') as f:
        graph_data = pickle.load(f)

    adjacency = graph_data['adjacency']
    vertex_weights = graph_data['vertex_weights']
    index_to_geoid = graph_data['index_to_geoid']
    edge_weights = graph_data.get('edge_weights', None)

    print(f"  Loaded {len(adjacency)} tracts")
    print(f"  Total population: {vertex_weights.sum():,}")
    if edge_weights:
        print(f"  Edge weights: {len(edge_weights):,} edges")

    # Run recursive bisection
    print(f"Running recursive bisection...")
    partitioner = RecursiveBisection(
        adjacency=adjacency,
        vertex_weights=vertex_weights,
        num_districts=n_districts,
        save_intermediate=False,
        state_code=state_code,
        tqdm_position=-1,
        debug=False,
        edge_weights=edge_weights,
        ufactor=5,  # 0.5% imbalance
        niter=100,
        objtype='cut',
        seed=None
    )

    final_assignments = partitioner.partition()
    print(f"  Completed: {len(set(final_assignments.values()))} districts created")

    # Save tree structure
    tree_file = TREE_DIR / f"{state_name}_{year}_tree.pkl"
    with open(tree_file, 'wb') as f:
        pickle.dump(partitioner.root, f)

    print(f"  Saved tree structure to: {tree_file.name}")

def main():
    """Extract trees for all states."""
    print("="*60)
    print("TREE EXTRACTION - 2010")
    print("="*60)

    for state_code, (state_name, n_districts) in STATES.items():
        try:
            extract_tree_for_state(state_code, state_name, n_districts, '2010')
        except Exception as e:
            print(f"ERROR processing {state_name}: {e}")
            import traceback
            traceback.print_exc()
            continue

    print("\n" + "="*60)
    print("TREE EXTRACTION COMPLETE")
    print("="*60)

if __name__ == "__main__":
    main()
