"""
Convert adjacency pkl files to native .adj.bin format for the Rust CLI.

Run this after `redist fetch --release` has downloaded adjacency pkls.
The .adj.bin files are read natively by the Rust CLI without any Python subprocess.

Usage:
    python scripts/data/generate_adj_bin.py --year 2020
    python scripts/data/generate_adj_bin.py --year 2020 --states VT AL
    python scripts/data/generate_adj_bin.py --year 2020 --version V3

Output:
    outputs/{version}/data/{year}/adjacency/{state}_adjacency_{year}.adj.bin
    outputs/{version}/data/{year}/adjacency/{state}_adjacency_{year}_geoids.json
"""

import argparse
import json
import pickle
import sys
from pathlib import Path

try:
    import redist_py
except ImportError:
    print("ERROR: redist_py not available.")
    print("Build with: cd redist/python/redist_py && maturin develop")
    sys.exit(1)


def convert_state(pkl_path: Path) -> None:
    with open(pkl_path, 'rb') as f:
        d = pickle.load(f)

    adj = d['adjacency']
    vw = [int(x) for x in d['vertex_weights']]
    ew = d.get('edge_weights') or {}
    ew_canonical = {(min(i, j), max(i, j)): float(w) for (i, j), w in ew.items()}
    ig = {int(k): str(v) for k, v in d.get('index_to_geoid', {}).items()}

    bin_data = redist_py.adjacency_to_bin(adj, vw, ew_canonical, len(adj), len(ew_canonical))
    bin_path = pkl_path.with_suffix('.adj.bin')
    bin_path.write_bytes(bin_data)

    geoid_path = pkl_path.with_name(pkl_path.stem + '_geoids.json')
    geoid_path.write_text(json.dumps({str(k): v for k, v in ig.items()}))

    n = len(adj)
    ne = len(ew_canonical)
    print(f"  [OK] {pkl_path.stem}: {n} tracts, {ne} edges -> {bin_path.name}")


def main():
    parser = argparse.ArgumentParser(description='Generate .adj.bin from pkl files')
    parser.add_argument('--year', default='2020', choices=['2020', '2010', '2000'])
    parser.add_argument('--version', default='V3')
    parser.add_argument('--states', nargs='*', help='State codes (default: all)')
    args = parser.parse_args()

    adj_dir = Path(f'outputs/{args.version}/data/{args.year}/adjacency')
    if not adj_dir.exists():
        print(f"ERROR: adjacency directory not found: {adj_dir}")
        print(f"Run: redist fetch --type adjacency --year {args.year} --release")
        sys.exit(1)

    pattern = f'*_adjacency_{args.year}.pkl'
    pkls = sorted(adj_dir.glob(pattern))

    if args.states:
        codes = {s.upper() for s in args.states}
        pkls = [p for p in pkls if any(
            p.name.startswith(code.lower()) for code in codes
        )]

    print(f"Converting {len(pkls)} state(s) to .adj.bin format...")
    errors = []
    for pkl in pkls:
        try:
            convert_state(pkl)
        except Exception as e:
            errors.append(f"{pkl.name}: {e}")
            print(f"  [FAIL] {pkl.name}: {e}")

    print(f"\nDone: {len(pkls) - len(errors)}/{len(pkls)} converted.")
    if errors:
        print(f"Errors: {len(errors)}")
        sys.exit(1)


if __name__ == '__main__':
    main()
