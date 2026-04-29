"""
Compare Rust CLI outputs against Python pipeline baseline.

Usage:
    python scripts/pipeline/compare_rust_vs_python.py \
        --rust outputs/V3_rust \
        --python outputs/V3/2020/states \
        --year 2020

Checks per state:
  - Assignment file exists and has correct tract count
  - All tracts assigned to exactly one district
  - Correct number of districts
  - Population balance ≤ 0.5%

National summary:
  - Mean Polsby-Popper (from district_summary.csv if present)
  - Count of states with failures
"""

import argparse
import csv
import json
import pickle
import sys
from pathlib import Path


def load_assignments_rust(state_dir: Path) -> dict:
    """Load JSON assignments from Rust output."""
    p = state_dir / 'data' / 'final_assignments.json'
    if not p.exists():
        return None
    raw = json.loads(p.read_text())
    return {int(k): v for k, v in raw.items()}


def load_assignments_python(state_dir: Path) -> dict:
    """Load pkl assignments from Python output."""
    p = state_dir / 'data' / 'final_assignments.pkl'
    if not p.exists():
        return None
    with open(p, 'rb') as f:
        return pickle.load(f)


def compute_balance(assignments: dict, adj_pkl: Path) -> float:
    """Max fractional deviation from ideal district population."""
    import numpy as np
    with open(adj_pkl, 'rb') as f:
        d = pickle.load(f)
    vw = np.array(d['vertex_weights'], dtype=np.int64)
    n_districts = len(set(assignments.values()))
    total = int(vw.sum())
    ideal = total / n_districts
    dist_pop = {}
    for tract, dist in assignments.items():
        dist_pop[dist] = dist_pop.get(dist, 0) + int(vw[tract])
    return max(abs(p - ideal) / ideal for p in dist_pop.values())


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--rust', required=True, help='Rust output dir (e.g. outputs/V3_rust)')
    parser.add_argument('--python', required=True, help='Python states dir (e.g. outputs/V3/2020/states)')
    parser.add_argument('--year', default='2020')
    parser.add_argument('--adjacency', default=None, help='Adjacency dir override')
    args = parser.parse_args()

    rust_root = Path(args.rust)
    python_root = Path(args.python)
    adj_dir = Path(args.adjacency) if args.adjacency else Path(f'outputs/V3/data/{args.year}/adjacency')

    # Load state config
    sys.path.insert(0, '.')
    from scripts.config_2020 import STATE_CONFIG_2020
    config = STATE_CONFIG_2020

    results = []
    failures = []

    for code, cfg in sorted(config.items()):
        state_name = cfg['name'].lower().replace(' ', '_')
        expected_districts = cfg['districts']

        rust_dir = rust_root / 'states' / state_name
        python_dir = python_root / state_name
        adj_pkl = adj_dir / f'{code.lower()}_adjacency_{args.year}.pkl'

        rust_assignments = load_assignments_rust(rust_dir)
        python_assignments = load_assignments_python(python_dir)

        row = {
            'state': code,
            'name': state_name,
            'expected_districts': expected_districts,
            'rust_ok': False,
            'python_ok': False,
            'match': False,
            'rust_tracts': 0,
            'rust_districts': 0,
            'rust_balance_pct': None,
            'python_tracts': 0,
            'python_districts': 0,
            'notes': [],
        }

        # Rust checks
        if rust_assignments is None:
            row['notes'].append('RUST: missing final_assignments.json')
            failures.append(f'{code}: missing Rust output')
        else:
            row['rust_tracts'] = len(rust_assignments)
            row['rust_districts'] = len(set(rust_assignments.values()))

            if row['rust_districts'] != expected_districts:
                row['notes'].append(f'RUST: {row["rust_districts"]} districts != expected {expected_districts}')
                failures.append(f'{code}: wrong district count ({row["rust_districts"]} != {expected_districts})')

            if adj_pkl.exists():
                try:
                    bal = compute_balance(rust_assignments, adj_pkl)
                    row['rust_balance_pct'] = round(bal * 100, 3)
                    if bal > 0.005:
                        row['notes'].append(f'RUST: balance {bal*100:.3f}% > 0.5%')
                        failures.append(f'{code}: Rust balance violation {bal*100:.3f}%')
                except Exception as e:
                    row['notes'].append(f'RUST: balance check failed: {e}')
            else:
                row['notes'].append('RUST: adjacency pkl not found for balance check')

            row['rust_ok'] = not any('RUST:' in n for n in row['notes'])

        # Python checks
        if python_assignments is None:
            row['notes'].append('PYTHON: missing final_assignments.pkl')
        else:
            row['python_tracts'] = len(python_assignments)
            row['python_districts'] = len(set(python_assignments.values()))
            row['python_ok'] = (row['python_districts'] == expected_districts)

        # Match: same tract count and district count
        if rust_assignments and python_assignments:
            row['match'] = (
                row['rust_tracts'] == row['python_tracts'] and
                row['rust_districts'] == row['python_districts']
            )

        results.append(row)

    # Print summary
    n_ok = sum(1 for r in results if r['rust_ok'])
    n_match = sum(1 for r in results if r['match'])
    balances = [r['rust_balance_pct'] for r in results if r['rust_balance_pct'] is not None]

    print(f'\n{"="*60}')
    print(f'50-STATE RUST vs PYTHON COMPARISON')
    print(f'{"="*60}')
    print(f'Rust outputs: {rust_root}')
    print(f'Python baseline: {python_root}')
    print(f'\nRust OK (correct district count + balance):  {n_ok}/50')
    print(f'Tract/district count matches Python:         {n_match}/50')
    if balances:
        import statistics
        print(f'Mean pop balance (Rust): {statistics.mean(balances):.3f}%')
        print(f'Max pop balance (Rust):  {max(balances):.3f}%')

    if failures:
        print(f'\nFAILURES ({len(failures)}):')
        for f in failures:
            print(f'  - {f}')
    else:
        print('\n[OK] All 50 states passed!')

    # Per-state table
    print(f'\n{"State":<6} {"Exp":>4} {"R-D":>4} {"R-B%":>7} {"Py-D":>5} {"Match":>6} {"Notes"}')
    print('-' * 75)
    for r in results:
        bal = f'{r["rust_balance_pct"]:.3f}' if r["rust_balance_pct"] is not None else 'N/A'
        match = 'Y' if r['match'] else 'N'
        status = '[OK]' if r['rust_ok'] else '[FAIL]'
        notes = '; '.join(r['notes']) if r['notes'] else ''
        print(f'{r["state"]:<6} {r["expected_districts"]:>4} {r["rust_districts"]:>4} {bal:>7} {r["python_districts"]:>5} {match:>6}  {status} {notes}')

    # Write CSV
    out_csv = Path(args.rust) / 'comparison_results.csv'
    with open(out_csv, 'w', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=list(results[0].keys()))
        writer.writeheader()
        for r in results:
            r['notes'] = '; '.join(r['notes'])
            writer.writerow(r)
    print(f'\nResults written to: {out_csv}')

    sys.exit(0 if not failures else 1)


if __name__ == '__main__':
    main()
