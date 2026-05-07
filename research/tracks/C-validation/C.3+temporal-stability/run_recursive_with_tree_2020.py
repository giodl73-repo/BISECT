"""
Run true recursive bisection for 2020 data and save tree structure for hierarchical analysis.
"""
import sys
from pathlib import Path

# Add src directory to Python path
BASE_DIR = Path(__file__).parent
PROJECT_ROOT = BASE_DIR.parent.parent
sys.path.insert(0, str(PROJECT_ROOT / "src"))

import pickle
import pandas as pd
import numpy as np

from apportionment.partition.recursive_bisection import RecursiveBisection

# Configuration
STATES = {
    'alabama': 7,
    'georgia': 14,
    'louisiana': 6,
    'mississippi': 4,
    'south_carolina': 7
}

DATA_DIR = BASE_DIR / "data_2020"
RESULTS_DIR = BASE_DIR / "results"
TREE_DIR = BASE_DIR / "trees"
TREE_DIR.mkdir(exist_ok=True)

def load_edge_weights(state: str, year: int = 2020) -> dict:
    """Load edge weights from CSV file."""
    edge_file = DATA_DIR / f"{state}_edge_weights.csv"
    if not edge_file.exists():
        print(f"No edge weights found for {state}, using uniform weights")
        return None

    df = pd.read_csv(edge_file)
    edge_weights = {}
    for _, row in df.iterrows():
        idx1 = int(row['idx1'])
        idx2 = int(row['idx2'])
        weight = float(row['weight'])
        key = (min(idx1, idx2), max(idx1, idx2))
        edge_weights[key] = weight

    print(f"Loaded {len(edge_weights)} edge weights for {state}")
    return edge_weights

def run_recursive_bisection_with_tree(state: str, n_districts: int):
    """Run recursive bisection and save both partition and tree structure."""
    print(f"\n{'='*60}")
    print(f"Processing {state.upper()} (2020) - {n_districts} districts")
    print(f"{'='*60}")

    # Load data
    adjacency_file = DATA_DIR / f"{state}_adjacency.csv"
    tract_file = DATA_DIR / f"{state}_tracts.parquet"

    if not adjacency_file.exists() or not tract_file.exists():
        print(f"ERROR: Missing data files for {state}")
        return

    # Load adjacency
    adj_df = pd.read_csv(adjacency_file)
    max_idx = max(adj_df['idx'].max(), adj_df['neighbor'].max())
    adjacency = [[] for _ in range(max_idx + 1)]

    for _, row in adj_df.iterrows():
        idx = int(row['idx'])
        neighbor = int(row['neighbor'])
        if neighbor not in adjacency[idx]:
            adjacency[idx].append(neighbor)

    # Load tracts
    tracts_df = pd.read_parquet(tract_file)
    index_to_geoid = {int(row['index']): row['GEOID'] for _, row in tracts_df.iterrows()}

    # Create vertex weights and index mapping
    vertex_weights = np.zeros(len(adjacency), dtype=np.int32)
    for idx, geoid in index_to_geoid.items():
        tract_row = tracts_df[tracts_df['index'] == idx].iloc[0]
        vertex_weights[idx] = int(tract_row['total_pop'])

    # Load edge weights
    edge_weights = load_edge_weights(state, 2020)

    print(f"Loaded {len(adjacency)} vertices, {len(index_to_geoid)} tracts")
    print(f"Total population: {vertex_weights.sum():,}")

    # Run recursive bisection
    state_code = state[:2].upper()
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

    print(f"Running recursive bisection...")
    final_assignments = partitioner.partition()

    # Save tree structure
    tree_file = TREE_DIR / f"{state}_2020_tree.pkl"
    with open(tree_file, 'wb') as f:
        pickle.dump(partitioner.root, f)
    print(f"Saved tree structure to {tree_file}")

    # Convert to district assignments DataFrame
    results = []
    for idx, district_id in final_assignments.items():
        if idx in index_to_geoid:
            geoid = str(index_to_geoid[idx]).zfill(11)
            results.append({
                'GEOID': geoid,
                'district': int(district_id)
            })

    # Save partition
    results_df = pd.DataFrame(results)
    output_file = RESULTS_DIR / f"{state}_2020_true_recursive_partition.csv"
    results_df.to_csv(output_file, index=False)

    print(f"Saved {len(results_df)} tract assignments to {output_file}")
    print(f"Districts: {sorted(results_df['district'].unique())}")

def main():
    """Run all states."""
    for state, n_districts in STATES.items():
        try:
            run_recursive_bisection_with_tree(state, n_districts)
        except Exception as e:
            print(f"ERROR processing {state}: {e}")
            import traceback
            traceback.print_exc()
            continue

if __name__ == "__main__":
    main()
