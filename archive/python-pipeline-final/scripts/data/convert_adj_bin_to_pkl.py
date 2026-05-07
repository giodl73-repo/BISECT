"""Convert .adj.bin (Rust binary format) to .pkl (Python pickle) adjacency files.

This is a standalone conversion script — NOT a PyO3 function. It keeps
PyO3 Rule 2 (no I/O through bindings) clean: Rust writes .bin, Python
converts to .pkl for backward compatibility with the existing pipeline.

Usage:
    python scripts/data/convert_adj_bin_to_pkl.py \
        outputs/V3/data/2020/adjacency/vt_adjacency_2020.adj.bin \
        outputs/V3/data/2020/adjacency/vt_adjacency_2020.pkl

    # Batch convert all .adj.bin files in a directory:
    python scripts/data/convert_adj_bin_to_pkl.py --dir outputs/V3/data/2020/adjacency/
"""

import argparse
import pickle
import sys
from pathlib import Path

import numpy as np

try:
    from redist_py import adjacency_from_bin
except ImportError:
    print("ERROR: redist_py not available.")
    print("Build with: cd redist/python/redist_py && maturin develop")
    sys.exit(1)


def convert_bin_to_pkl(bin_path: Path, pkl_path: Path) -> None:
    """Convert a single .adj.bin file to .pkl format."""
    data = bin_path.read_bytes()
    graph = adjacency_from_bin(data)

    # Build the pkl dict matching the format expected by the pipeline
    # (adjacency.py build_adjacency_graph output format)
    adj = graph['adjacency']
    ew = graph['edge_weights']

    # Build index_to_geoid and geoid_to_index as empty (geometry info not in .bin)
    # These are populated during the adjacency build phase; the .bin format
    # stores the graph topology only.
    pkl_data = {
        'adjacency': adj,
        'vertex_weights': np.zeros(graph['n_vertices'], dtype=np.int32),
        'edge_weights': {k: float(v) for k, v in ew.items()},
        'index_to_geoid': {},
        'geoid_to_index': {},
        'n_components': 1,  # assumed connected after bridging
        'mean_neighbors': sum(len(n) for n in adj) / max(len(adj), 1),
    }

    with open(pkl_path, 'wb') as f:
        pickle.dump(pkl_data, f)

    n_tracts = graph['n_vertices']
    n_edges = graph['n_edges']
    print(f"[OK] {bin_path.name} -> {pkl_path.name} ({n_tracts} tracts, {n_edges} edges)")


def main() -> None:
    parser = argparse.ArgumentParser(description='Convert .adj.bin to .pkl')
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument('src', nargs='?', help='.adj.bin input file')
    group.add_argument('--dir', metavar='DIR',
                       help='Convert all .adj.bin files in directory')
    parser.add_argument('dst', nargs='?', help='.pkl output file (only with src)')
    args = parser.parse_args()

    if args.dir:
        d = Path(args.dir)
        bins = list(d.glob('*.adj.bin'))
        if not bins:
            print(f"No .adj.bin files found in {d}")
            sys.exit(0)
        for bin_path in sorted(bins):
            pkl_path = bin_path.with_suffix('').with_suffix('.pkl')
            convert_bin_to_pkl(bin_path, pkl_path)
    else:
        src = Path(args.src)
        if args.dst:
            dst = Path(args.dst)
        else:
            dst = src.with_suffix('').with_suffix('.pkl')
        convert_bin_to_pkl(src, dst)


if __name__ == '__main__':
    main()
