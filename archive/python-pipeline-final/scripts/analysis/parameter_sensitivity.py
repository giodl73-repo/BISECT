#!/usr/bin/env python3
"""
Parameter sensitivity analysis for recursive bisection redistricting.

Runs systematic parameter sweeps across multiple states to characterize
outcome variation and validate impossibility defense.

Expected runtime: 10-12 hours (parallelized with 8 workers)
"""

import subprocess
import pandas as pd
from pathlib import Path
import sys
import multiprocessing as mp
from itertools import product
from datetime import datetime
import pickle
import numpy as np

# Add project root to path
project_root = Path(__file__).parents[2]
sys.path.insert(0, str(project_root))
sys.path.insert(0, str(project_root / 'src'))

from scripts.utils import get_state_config

# Parameter ranges to test
UFACTOR_VALUES = [1, 5, 10, 20]      # Imbalance tolerance
NITER_VALUES = [10, 50, 100, 200]     # Refinement iterations
OBJTYPE_VALUES = ['cut', 'vol']       # Objective types
SEED_VALUES = range(1, 101)           # Random seeds 1-100

# Representative states for analysis
TEST_STATES = [
    'VT',   # Vermont - 1 district (single-member)
    'DE',   # Delaware - 1 district (single-member)
    'MN',   # Minnesota - 8 districts (even)
    'AL',   # Alabama - 7 districts (odd)
    'FL',   # Florida - 28 districts (large)
    'CA',   # California - 52 districts (largest)
    'PA',   # Pennsylvania - 17 districts (composite)
    'NC',   # North Carolina - 14 districts
    'MI',   # Michigan - 13 districts
    'GA'    # Georgia - 14 districts
]


def run_single_configuration(state, year, input_version, params):
    """Run redistricting with specific parameter configuration."""
    ufactor, niter, objtype, seed = params

    # Create unique output directory for this configuration
    config_id = f"uf{ufactor}_ni{niter}_{objtype}_s{seed if seed else 'rand'}"
    output_dir = Path(f'outputs/sensitivity/{year}/{state}/{config_id}')
    output_dir.mkdir(parents=True, exist_ok=True)

    # Build command - use input_version (e.g., V1) for data loading
    cmd = [
        sys.executable,  # Use current Python interpreter
        'scripts/pipeline/run_state_redistricting.py',
        '--state', state,
        '--year', str(year),
        '--version', input_version,  # Use existing data version (V1)
        '--output-dir', str(output_dir),
        '--ufactor', str(ufactor),
        '--niter', str(niter),
        '--objtype', objtype,
        '--partition-mode', 'edge-weighted',  # Use edge-weighted mode
        '--dpi', '150',
        '--position', '999'  # Suppress progress bars for parallel runs
    ]

    if seed is not None:
        cmd.extend(['--seed', str(seed)])

    print(f"[{state}] Running config: ufactor={ufactor}, niter={niter}, objtype={objtype}, seed={seed}")

    # Run redistricting
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=3600)  # 1 hour timeout

        if result.returncode != 0:
            print(f"[{state}] Config failed: {config_id}")
            print(f"  stdout: {result.stdout[:200]}")
            print(f"  stderr: {result.stderr[:200]}")
            return None

        # Extract metrics from output
        metrics = extract_metrics(output_dir, state, year, input_version)
        metrics.update({
            'ufactor': ufactor,
            'niter': niter,
            'objtype': objtype,
            'seed': seed,
            'state': state,
            'year': year,
            'input_version': input_version,
            'config_id': config_id
        })

        print(f"[{state}] Config complete: {config_id}")
        return metrics

    except subprocess.TimeoutExpired:
        print(f"[{state}] Config timeout: {config_id}")
        return None
    except Exception as e:
        print(f"[{state}] Config error: {config_id} - {e}")
        return None


def extract_metrics(output_dir, state, year, version):
    """Extract compactness, population deviation, partisan outcomes from redistricting output."""

    # Get state config to know number of districts
    STATE_CONFIG = get_state_config(year)
    num_districts = STATE_CONFIG[state]['districts']

    metrics = {
        'num_districts': num_districts,
        'mean_polsby_popper': None,
        'mean_pop_deviation': None,
        'max_pop_deviation': None,
        'total_edge_cuts': None,
        'dem_district_pct': None,
        'dem_district_count': None
    }

    # For single-district states, skip analysis
    if num_districts == 1:
        metrics['mean_polsby_popper'] = 1.0  # Single district = entire state
        metrics['mean_pop_deviation'] = 0.0
        metrics['max_pop_deviation'] = 0.0
        metrics['total_edge_cuts'] = 0
        return metrics

    # Load final assignments
    data_dir = output_dir / 'data'
    final_file = data_dir / 'final_assignments.pkl'

    if not final_file.exists():
        print(f"  [WARNING] No final assignments found: {final_file}")
        return metrics

    try:
        with open(final_file, 'rb') as f:
            assignments = pickle.load(f)

        # Calculate population deviations
        # Load vertex weights to get tract populations
        from scripts.utils import get_adjacency_file
        graph_file = get_adjacency_file(state, year, version)

        with open(graph_file, 'rb') as f:
            graph_data = pickle.load(f)

        vertex_weights = graph_data['vertex_weights']
        total_pop = int(vertex_weights.sum())
        ideal_pop = total_pop / num_districts

        # Calculate per-district populations and deviations
        district_pops = []
        for district_id in range(1, num_districts + 1):
            district_tracts = [i for i, d in assignments.items() if d == district_id]
            district_pop = sum(vertex_weights[i] for i in district_tracts)
            district_pops.append(district_pop)

        deviations = [(p - ideal_pop) / ideal_pop * 100 for p in district_pops]
        metrics['mean_pop_deviation'] = np.mean(np.abs(deviations))
        metrics['max_pop_deviation'] = np.max(np.abs(deviations))

        # Calculate compactness (Polsby-Popper) if we have geometries
        # This requires loading tract geometries and computing district geometries
        # For now, skip this to save time - focus on population and partisan metrics

        # Calculate partisan outcomes if election data available
        # This requires loading election results and aggregating by district
        # Skip for now - focus on core metrics

        print(f"  Extracted metrics: pop_dev={metrics['mean_pop_deviation']:.2f}%")

    except Exception as e:
        print(f"  [ERROR] Failed to extract metrics: {e}")

    return metrics


def parameter_sweep_ufactor(states, year='2020', input_version='V1'):
    """Sweep ufactor values while holding others constant."""
    print("\n" + "="*70)
    print("PARAMETER SWEEP: ufactor (imbalance tolerance)")
    print("="*70)
    print(f"Testing values: {UFACTOR_VALUES}")
    print(f"States: {', '.join(states)}")
    print(f"Fixed: niter=100, objtype=cut, seed=42")
    print()

    results = []

    for state in states:
        for ufactor in UFACTOR_VALUES:
            params = (ufactor, 100, 'cut', 42)  # Fixed niter, objtype, seed
            metrics = run_single_configuration(state, year, input_version, params)
            if metrics:
                results.append(metrics)

    df = pd.DataFrame(results)
    output_file = Path('outputs/sensitivity/ufactor_sweep.csv')
    output_file.parent.mkdir(parents=True, exist_ok=True)
    df.to_csv(output_file, index=False)
    print(f"\n[OK] Saved results: {output_file}")
    print(f"     {len(df)} configurations completed")
    return df


def parameter_sweep_niter(states, year='2020', input_version='V1'):
    """Sweep niter values while holding others constant."""
    print("\n" + "="*70)
    print("PARAMETER SWEEP: niter (refinement iterations)")
    print("="*70)
    print(f"Testing values: {NITER_VALUES}")
    print(f"States: {', '.join(states)}")
    print(f"Fixed: ufactor=5, objtype=cut, seed=42")
    print()

    results = []

    for state in states:
        for niter in NITER_VALUES:
            params = (5, niter, 'cut', 42)  # Fixed ufactor, objtype, seed
            metrics = run_single_configuration(state, year, input_version, params)
            if metrics:
                results.append(metrics)

    df = pd.DataFrame(results)
    output_file = Path('outputs/sensitivity/niter_sweep.csv')
    output_file.parent.mkdir(parents=True, exist_ok=True)
    df.to_csv(output_file, index=False)
    print(f"\n[OK] Saved results: {output_file}")
    print(f"     {len(df)} configurations completed")
    return df


def parameter_sweep_objtype(states, year='2020', input_version='V1'):
    """Compare cut vs. vol objectives."""
    print("\n" + "="*70)
    print("PARAMETER SWEEP: objtype (objective function)")
    print("="*70)
    print(f"Testing values: {OBJTYPE_VALUES}")
    print(f"States: {', '.join(states)}")
    print(f"Fixed: ufactor=5, niter=100, seed=42")
    print()

    results = []

    for state in states:
        for objtype in OBJTYPE_VALUES:
            params = (5, 100, objtype, 42)  # Fixed ufactor, niter, seed
            metrics = run_single_configuration(state, year, input_version, params)
            if metrics:
                results.append(metrics)

    df = pd.DataFrame(results)
    output_file = Path('outputs/sensitivity/objtype_sweep.csv')
    output_file.parent.mkdir(parents=True, exist_ok=True)
    df.to_csv(output_file, index=False)
    print(f"\n[OK] Saved results: {output_file}")
    print(f"     {len(df)} configurations completed")
    return df


def random_seed_ensemble(states, year='2020', input_version='V1', n_runs=100, n_workers=8):
    """Generate ensemble with different random seeds (parallelized)."""
    print("\n" + "="*70)
    print("PARAMETER SWEEP: Random seed ensemble")
    print("="*70)
    print(f"Testing {n_runs} random seeds per state")
    print(f"States: {', '.join(states)}")
    print(f"Fixed: ufactor=5, niter=100, objtype=cut")
    print(f"Parallel workers: {n_workers}")
    print()

    results = []

    # Build all parameter configurations
    all_configs = []
    for state in states:
        for seed in range(1, n_runs + 1):
            all_configs.append((state, year, input_version, (5, 100, 'cut', seed)))

    print(f"Total configurations: {len(all_configs)}")
    print(f"Estimated time: {len(all_configs) * 3 / 60:.1f} minutes (assuming 3 min/config)")
    print()

    # Run in parallel
    with mp.Pool(n_workers) as pool:
        results = pool.starmap(run_single_configuration, all_configs)

    # Filter out None results (failed runs)
    results = [r for r in results if r is not None]

    df = pd.DataFrame(results)
    output_file = Path('outputs/sensitivity/seed_ensemble.csv')
    output_file.parent.mkdir(parents=True, exist_ok=True)
    df.to_csv(output_file, index=False)
    print(f"\n[OK] Saved results: {output_file}")
    print(f"     {len(df)} configurations completed ({len(df)/len(all_configs)*100:.1f}% success rate)")
    return df


def main():
    """Run all parameter sensitivity analyses."""
    import argparse

    parser = argparse.ArgumentParser(description='Parameter sensitivity analysis')
    parser.add_argument('--year', type=str, default='2020', choices=['2020', '2010', '2000'],
                       help='Census year (default: 2020)')
    parser.add_argument('--input-version', type=str, default='V1',
                       help='Input data version to use (default: V1)')
    parser.add_argument('--workers', type=int, default=8,
                       help='Number of parallel workers for seed ensemble (default: 8)')
    parser.add_argument('--sweep', type=str, choices=['ufactor', 'niter', 'objtype', 'seeds', 'all'],
                       default='all', help='Which parameter sweep to run (default: all)')
    parser.add_argument('--states', type=str, nargs='+', default=TEST_STATES,
                       help='States to test (default: 10 representative states)')
    parser.add_argument('--n-seeds', type=int, default=100,
                       help='Number of random seeds for ensemble (default: 100)')

    args = parser.parse_args()

    print("\n" + "="*70)
    print("PARAMETER SENSITIVITY ANALYSIS")
    print("Congressional Redistricting via Recursive Bisection")
    print("="*70)
    print(f"Census year: {args.year}")
    print(f"Input data version: {args.input_version}")
    print(f"States: {', '.join(args.states)} ({len(args.states)} total)")
    print(f"Timestamp: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print("="*70)

    # Validate states exist
    STATE_CONFIG = get_state_config(args.year)
    for state in args.states:
        if state not in STATE_CONFIG:
            print(f"ERROR: Unknown state code '{state}'")
            print(f"Valid codes: {', '.join(sorted(STATE_CONFIG.keys()))}")
            sys.exit(1)

    # Run requested sweeps
    if args.sweep in ['ufactor', 'all']:
        ufactor_df = parameter_sweep_ufactor(args.states, args.year, args.input_version)

    if args.sweep in ['niter', 'all']:
        niter_df = parameter_sweep_niter(args.states, args.year, args.input_version)

    if args.sweep in ['objtype', 'all']:
        objtype_df = parameter_sweep_objtype(args.states, args.year, args.input_version)

    if args.sweep in ['seeds', 'all']:
        seed_df = random_seed_ensemble(args.states, args.year, args.input_version,
                                       n_runs=args.n_seeds, n_workers=args.workers)

    print("\n" + "="*70)
    print("ANALYSIS COMPLETE")
    print("="*70)
    print(f"Results saved to: outputs/sensitivity/")
    print("Next steps:")
    print("  1. Run: python scripts/analysis/visualize_sensitivity.py")
    print("  2. Review figures in: research/gerry-recursive-bisection/figures/")
    print("  3. Update paper Section 4.5 with results")
    print("="*70 + "\n")


if __name__ == '__main__':
    main()
