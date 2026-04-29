"""
Validate state house redistricting across all 50 states.

Runs redist state for each state at the correct resolution (block_group for
states with < 20 tracts/district, tract otherwise). Uses 10% balance tolerance
and ufactor=50 appropriate for state legislative maps.

Usage:
    python scripts/pipeline/validate_state_legislative.py
    python scripts/pipeline/validate_state_legislative.py --workers 8 --min-districts 80
    python scripts/pipeline/validate_state_legislative.py --states TX CA NY

Excludes NH (400 floterial multi-member districts — not single-member).
"""

import argparse
import json
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

# State house district counts (2020)
HOUSE_DISTRICTS = {
    'AL': 105, 'AK': 40, 'AZ': 60, 'AR': 100, 'CA': 80,
    'CO': 65, 'CT': 151, 'DE': 41, 'FL': 120, 'GA': 180,
    'HI': 51, 'ID': 70, 'IL': 118, 'IN': 100, 'IA': 100,
    'KS': 125, 'KY': 100, 'LA': 105, 'ME': 151, 'MD': 141,
    'MA': 160, 'MI': 110, 'MN': 134, 'MS': 122, 'MO': 163,
    'MT': 100, 'NE': 49, 'NV': 42, 'NJ': 80, 'NM': 70,
    'NY': 150, 'NC': 120, 'ND': 94, 'OH': 99, 'OK': 101,
    'OR': 60, 'PA': 203, 'RI': 75, 'SC': 124, 'SD': 70,
    'TN': 99, 'TX': 150, 'UT': 75, 'VT': 150, 'VA': 100,
    'WA': 98, 'WV': 100, 'WI': 99, 'WY': 60,
    # Excluded: NH (400 multi-member floterial), AK single-member ambiguity
}


def find_binary():
    for p in ['redist/target/release/redist.exe', 'redist/target/release/redist']:
        if Path(p).exists():
            return str(Path(p).resolve())
    sys.exit("ERROR: redist binary not found.")


def count_tracts(state_code):
    geoid_f = Path(f'outputs/V3/data/2020/adjacency/{state_code.lower()}_adjacency_2020_geoids.json')
    if not geoid_f.exists():
        return 0
    return len(json.load(open(geoid_f)))


def choose_resolution(state_code, house_districts):
    tracts = count_tracts(state_code)
    if tracts == 0:
        return 'tract', 0
    tpd = tracts / house_districts
    res = 'block_group' if tpd < 20 else 'tract'
    return res, round(tpd, 1)


def run_state(binary, state_code, house_districts, version, seed):
    resolution, tpd = choose_resolution(state_code, house_districts)
    label = f'{state_code.lower()}_house_validation'

    cmd = [
        binary, 'state',
        '--state', state_code,
        '--year', '2020',
        '--version', version,
        '--districts', str(house_districts),
        '--chamber', 'house',
        '--label', label,
        '--balance-tolerance', '10.0',
        '--ufactor', '50',
        '--seed', str(seed),
        '--resolution', resolution,
        '--force',
    ]

    t0 = time.perf_counter()
    r = subprocess.run(cmd, capture_output=True, text=True)
    elapsed = time.perf_counter() - t0

    manifest_path = Path(f'outputs/{version}/2020/plans/{label}/manifest.json')
    balance_ok = False
    if manifest_path.exists():
        m = json.load(open(manifest_path))
        balance_ok = m.get('population_balance_valid', False)

    return {
        'state': state_code,
        'house_districts': house_districts,
        'resolution': resolution,
        'tracts_per_district': tpd,
        'success': r.returncode == 0,
        'balance_ok': balance_ok,
        'elapsed_s': round(elapsed, 1),
        'stderr': r.stderr[-200:] if r.returncode != 0 else '',
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--workers', type=int, default=6)
    parser.add_argument('--min-districts', type=int, default=0,
                        help='Only run states with >= N house districts')
    parser.add_argument('--states', nargs='+', help='Specific states only')
    parser.add_argument('--version', default='StateHouseValidation')
    parser.add_argument('--seed', type=int, default=42)
    args = parser.parse_args()

    binary = find_binary()

    # Filter states
    targets = {k: v for k, v in HOUSE_DISTRICTS.items()
               if v >= args.min_districts}
    if args.states:
        targets = {k: v for k, v in targets.items()
                   if k.upper() in [s.upper() for s in args.states]}

    # Skip states without tract data
    targets = {k: v for k, v in targets.items() if count_tracts(k) > 0}

    print(f"Validating {len(targets)} states ({args.workers} workers, seed={args.seed})")
    print(f"Version: {args.version}")
    print()

    results = []
    t0 = time.perf_counter()

    with ThreadPoolExecutor(max_workers=args.workers) as pool:
        futures = {
            pool.submit(run_state, binary, code, districts, args.version, args.seed): code
            for code, districts in sorted(targets.items())
        }
        for fut in as_completed(futures):
            r = fut.result()
            status = 'OK' if r['balance_ok'] else ('FAIL' if r['success'] else 'ERR')
            res_short = 'BG' if r['resolution'] == 'block_group' else 'TR'
            print(f"  {r['state']:<3} {r['house_districts']:>3}HD  {res_short}  "
                  f"{r['tracts_per_district']:>5.1f}t/d  "
                  f"{r['elapsed_s']:>5.1f}s  {status}"
                  + (f"  {r['stderr'][:80]}" if status != 'OK' else ''))
            results.append(r)

    total_s = time.perf_counter() - t0

    # Summary
    ok = [r for r in results if r['balance_ok']]
    fail = [r for r in results if not r['balance_ok']]
    bg_states = [r for r in results if r['resolution'] == 'block_group']

    print(f"\n{'='*60}")
    print(f"RESULTS: {len(ok)}/{len(results)} states balanced")
    print(f"Block group: {len(bg_states)} states | Tract: {len(results)-len(bg_states)} states")
    print(f"Total time: {total_s:.1f}s")

    if fail:
        print(f"\nFAILURES ({len(fail)}):")
        for r in sorted(fail, key=lambda x: x['state']):
            print(f"  {r['state']} {r['house_districts']}HD "
                  f"({r['resolution']}, {r['tracts_per_district']}t/d): {r['stderr'][:100]}")

    # Write CSV
    csv_path = f'outputs/state_legislative_validation_{args.version}.csv'
    with open(csv_path, 'w') as f:
        f.write('state,house_districts,resolution,tracts_per_district,balance_ok,elapsed_s\n')
        for r in sorted(results, key=lambda x: x['state']):
            f.write(f"{r['state']},{r['house_districts']},{r['resolution']},"
                    f"{r['tracts_per_district']},{r['balance_ok']},{r['elapsed_s']}\n")
    print(f"\nResults written to: {csv_path}")

    sys.exit(0 if not fail else 1)


if __name__ == '__main__':
    main()
