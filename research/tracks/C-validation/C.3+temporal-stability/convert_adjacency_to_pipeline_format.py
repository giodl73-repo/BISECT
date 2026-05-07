"""
Convert our 2010 adjacency files to pipeline-compatible format.

Converts scipy sparse adjacency matrices (.npz) to pickle format with
the structure expected by run_state_redistricting.py:
- adjacency: List[List[int]]
- vertex_weights: np.ndarray (populations)
- index_to_geoid: dict mapping idx -> GEOID
- edge_weights: Optional[Dict[Tuple[int, int], float]]
"""

import sys
from pathlib import Path
import pickle
import numpy as np
import pandas as pd
from scipy import sparse

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

# States for temporal stability analysis
STATES = ['alabama', 'georgia', 'louisiana', 'mississippi', 'south_carolina']

# State name to FIPS code mapping
STATE_TO_FIPS = {
    'alabama': '01',
    'georgia': '13',
    'louisiana': '22',
    'mississippi': '28',
    'south_carolina': '45'
}

# Edge weighting parameters (from VRA papers)
EDGE_WEIGHT_FACTOR = 5
MINORITY_THRESHOLD = 0.40


def convert_state_adjacency(state: str):
    """Convert one state's adjacency from .npz to .pkl format."""
    print(f"\nConverting {state}...")

    project_root = Path(__file__).parent.parent.parent

    # Load adjacency (scipy sparse matrix)
    adj_input = project_root / f'outputs/data/2010/adjacency/{state}_adjacency.npz'
    adjacency_sparse = sparse.load_npz(adj_input)
    print(f"  Loaded adjacency: {adjacency_sparse.shape}")

    # Load demographics (to get GEOIDs, populations, and compute edge weights)
    demographics_file = project_root / f'data/2010/demographics/{state}_demographics_2010.csv'
    demographics = pd.read_csv(demographics_file)
    print(f"  Loaded {len(demographics)} tracts from demographics")

    # Verify shapes match
    assert adjacency_sparse.shape[0] == len(demographics), \
        f"Shape mismatch: adjacency {adjacency_sparse.shape[0]} vs demographics {len(demographics)}"

    # Convert adjacency to List[List[int]]
    adjacency_coo = adjacency_sparse.tocoo()
    n = len(demographics)
    adjacency_list = [[] for _ in range(n)]
    for i, j in zip(adjacency_coo.row, adjacency_coo.col):
        adjacency_list[i].append(int(j))
    print(f"  Converted to adjacency list format")

    # Extract populations (vertex weights)
    vertex_weights = demographics['total_pop'].values.astype(np.int32)
    print(f"  Extracted vertex weights: sum={vertex_weights.sum():,}")

    # Create index to GEOID mapping
    index_to_geoid = {i: str(demographics.iloc[i]['GEOID']) for i in range(n)}

    # Compute edge weights (for VRA-aware edge-weighted partitioning)
    # Calculate minority percentage (total minority coalition)
    demographics['minority_pct'] = (
        demographics['hispanic'] + demographics['black_non_hispanic'] +
        demographics['asian_non_hispanic'] + demographics.get('other', 0)
    ) / demographics['total_pop']

    edge_weights = {}
    edge_count = 0
    weighted_edge_count = 0

    for i, j in zip(adjacency_coo.row, adjacency_coo.col):
        if i < j:  # Only process each edge once
            edge_count += 1
            # Both tracts above threshold? Apply weight
            if (demographics.iloc[i]['minority_pct'] >= MINORITY_THRESHOLD and
                demographics.iloc[j]['minority_pct'] >= MINORITY_THRESHOLD):
                edge_weights[(i, j)] = EDGE_WEIGHT_FACTOR
                edge_weights[(j, i)] = EDGE_WEIGHT_FACTOR
                weighted_edge_count += 1
            else:
                edge_weights[(i, j)] = 1.0
                edge_weights[(j, i)] = 1.0

    print(f"  Computed edge weights: {len(edge_weights):,} edges ({weighted_edge_count:,} weighted)")

    # Create output dict
    graph_data = {
        'adjacency': adjacency_list,
        'vertex_weights': vertex_weights,
        'index_to_geoid': index_to_geoid,
        'edge_weights': edge_weights,
        'metadata': {
            'year': '2010',
            'state': state,
            'num_tracts': n,
            'total_population': int(vertex_weights.sum()),
            'edge_weight_factor': EDGE_WEIGHT_FACTOR,
            'minority_threshold': MINORITY_THRESHOLD
        }
    }

    # Save to pipeline location
    output_dir = project_root / 'outputs/v1/data/2010/adjacency'
    output_dir.mkdir(parents=True, exist_ok=True)
    output_file = output_dir / f'{state}_adjacency_2010.pkl'

    with open(output_file, 'wb') as f:
        pickle.dump(graph_data, f, protocol=pickle.HIGHEST_PROTOCOL)

    print(f"  Saved: {output_file}")
    print(f"  Size: {output_file.stat().st_size / 1024:.1f} KB")


def main():
    """Convert all state adjacency files."""
    print("="*70)
    print("CONVERT ADJACENCY FILES TO PIPELINE FORMAT")
    print("="*70)
    print(f"States: {', '.join(STATES)}")
    print(f"Input: outputs/data/2010/adjacency/*.npz")
    print(f"Output: outputs/v1/data/2010/adjacency/*_adjacency_2010.pkl")
    print()

    for state in STATES:
        try:
            convert_state_adjacency(state)
        except Exception as e:
            print(f"  ERROR: {e}")
            import traceback
            traceback.print_exc()

    print()
    print("="*70)
    print("CONVERSION COMPLETE")
    print("="*70)


if __name__ == '__main__':
    main()
