#!/usr/bin/env python3
"""
Parameter Sensitivity Analysis for P1.1

Systematic parameter sweeps to demonstrate robustness of redistricting results.

Tests variations in:
- ufactor: 1, 5, 10, 20 (population imbalance tolerance)
- niter: 10, 50, 100, 200 (refinement iterations)
- objtype: 'cut', 'vol' (edge-cut vs volume minimization)
- seed: 100 random seeds (reproducibility ensemble)

Usage:
    python parameter_sensitivity.py --year 2020 --states AL GA MN WI PA --output outputs/sensitivity/
"""

import sys
import os
from pathlib import Path
import argparse
import json
import pickle
import numpy as np
import pandas as pd
from datetime import datetime
from tqdm import tqdm
import itertools

# Add project root to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent / 'src'))
sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent))

from scripts.utils import get_state_config
from scripts.pipeline.run_state_redistricting import run_state_redistricting


def compute_compactness_polsby_popper(tracts_gdf, assignments):
    """
    Compute Polsby-Popper compactness for each district.

    Parameters
    ----------
    tracts_gdf : GeoDataFrame
        Census tracts with geometry
    assignments : dict
        Mapping from tract index to district ID

    Returns
    -------
    dict
        District ID -> Polsby-Popper score
    """
    import geopandas as gpd

    # Add district assignments to tracts
    tracts = tracts_gdf.copy()
    tracts['district'] = [assignments[i] for i in range(len(tracts))]

    compactness = {}
    for district_id in sorted(tracts['district'].unique()):
        district_tracts = tracts[tracts['district'] == district_id]

        # Dissolve to single polygon
        district_geom = district_tracts.geometry.union_all()

        # Compute Polsby-Popper: 4π * area / perimeter²
        area = district_geom.area
        perimeter = district_geom.length

        if perimeter > 0:
            pp = (4 * np.pi * area) / (perimeter ** 2)
        else:
            pp = 0.0

        compactness[district_id] = pp

    return compactness


def compute_population_balance(vertex_weights, assignments, num_districts):
    """
    Compute population balance statistics.

    Parameters
    ----------
    vertex_weights : np.ndarray
        Census tract populations
    assignments : dict
        Mapping from tract index to district ID
    num_districts : int
        Number of districts

    Returns
    -------
    dict
        Population balance metrics
    """
    populations = []
    for district_id in range(1, num_districts + 1):
        pop = sum(vertex_weights[i] for i, d in assignments.items() if d == district_id)
        populations.append(pop)

    total_pop = sum(populations)
    ideal_pop = total_pop / num_districts

    deviations = [(p - ideal_pop) / ideal_pop * 100 for p in populations]

    return {
        'populations': populations,
        'ideal_pop': ideal_pop,
        'deviations': deviations,
        'max_deviation': max(abs(d) for d in deviations),
        'mean_deviation': np.mean([abs(d) for d in deviations]),
        'std_deviation': np.std([abs(d) for d in deviations])
    }


def run_parameter_sweep(states, year, output_dir, state_config, data_version, params_to_vary='all'):
    """
    Run redistricting with parameter variations.

    Parameters
    ----------
    states : list
        List of state codes (e.g., ['AL', 'GA', 'MN'])
    year : str
        Census year
    output_dir : Path
        Output directory for results
    state_config : dict
        State configuration
    data_version : str
        Version to use for input data (e.g., 'v1')
    params_to_vary : str
        Which parameters to vary: 'all', 'ufactor', 'niter', 'objtype', 'seed'
    """

    # Define parameter ranges
    if params_to_vary == 'all' or params_to_vary == 'ufactor':
        ufactor_values = [1, 5, 10, 20]
    else:
        ufactor_values = [5]  # Default

    if params_to_vary == 'all' or params_to_vary == 'niter':
        niter_values = [10, 50, 100, 200]
    else:
        niter_values = [100]  # Default

    if params_to_vary == 'all' or params_to_vary == 'objtype':
        objtype_values = ['cut', 'vol']
    else:
        objtype_values = ['cut']  # Default

    if params_to_vary == 'seed':
        # Random seed ensemble: 100 runs with different seeds
        seed_values = list(range(1, 101))
        ufactor_values = [5]
        niter_values = [100]
        objtype_values = ['cut']
    else:
        seed_values = [42]  # Fixed seed for non-seed sweeps

    # Create all parameter combinations
    param_combinations = list(itertools.product(
        ufactor_values, niter_values, objtype_values, seed_values
    ))

    print(f"\n{'='*80}")
    print(f"PARAMETER SENSITIVITY ANALYSIS")
    print(f"{'='*80}")
    print(f"States: {', '.join(states)}")
    print(f"Year: {year}")
    print(f"Output: {output_dir}")
    print(f"Parameter combinations: {len(param_combinations)}")
    print(f"  ufactor: {ufactor_values}")
    print(f"  niter: {niter_values}")
    print(f"  objtype: {objtype_values}")
    print(f"  seed: {len(seed_values)} values")
    print(f"{'='*80}\n")

    # Create output directory
    output_dir.mkdir(parents=True, exist_ok=True)

    # Results storage
    results = []

    # Total runs
    total_runs = len(states) * len(param_combinations)
    pbar = tqdm(total=total_runs, desc="Parameter sweep", unit="run")

    for state_code in states:
        state_name = state_config[state_code]['name']
        num_districts = state_config[state_code]['districts']

        print(f"\n{state_name} ({state_code}) - {num_districts} districts")

        for ufactor, niter, objtype, seed in param_combinations:
            # Create run-specific output directory
            run_name = f"{state_code}_uf{ufactor}_ni{niter}_{objtype}_s{seed}"
            run_output_dir = output_dir / 'runs' / run_name

            # Check if already completed
            final_file = run_output_dir / 'data' / 'final_assignments.pkl'
            if final_file.exists():
                pbar.set_description(f"{state_code} {run_name} (cached)")
                pbar.update(1)

                # Load cached results
                with open(final_file, 'rb') as f:
                    assignments = pickle.load(f)

                # Store metadata
                results.append({
                    'state': state_code,
                    'ufactor': ufactor,
                    'niter': niter,
                    'objtype': objtype,
                    'seed': seed,
                    'run_name': run_name,
                    'output_dir': str(run_output_dir),
                    'cached': True
                })
                continue

            # Run redistricting
            pbar.set_description(f"{state_code} uf={ufactor} ni={niter} {objtype} s={seed}")

            try:
                run_state_redistricting(
                    state_code=state_code,
                    state_config=state_config,
                    year=year,
                    version=data_version,  # Use existing data version
                    output_dir=str(run_output_dir),
                    print_only=False,
                    debug=False,
                    dpi=150,
                    position=999,  # Hide progress bars
                    reset=False,
                    ufactor=ufactor,
                    niter=niter,
                    objtype=objtype,
                    seed=seed,
                    partition_mode='unweighted'  # Use unweighted for cleaner parameter isolation
                )

                # Store metadata
                results.append({
                    'state': state_code,
                    'ufactor': ufactor,
                    'niter': niter,
                    'objtype': objtype,
                    'seed': seed,
                    'run_name': run_name,
                    'output_dir': str(run_output_dir),
                    'cached': False
                })

            except Exception as e:
                print(f"\n  ERROR: {run_name} failed: {e}")
                results.append({
                    'state': state_code,
                    'ufactor': ufactor,
                    'niter': niter,
                    'objtype': objtype,
                    'seed': seed,
                    'run_name': run_name,
                    'output_dir': str(run_output_dir),
                    'error': str(e),
                    'cached': False
                })

            pbar.update(1)

    pbar.close()

    # Save metadata
    metadata_file = output_dir / 'sensitivity_metadata.json'
    with open(metadata_file, 'w') as f:
        json.dump({
            'timestamp': datetime.now().isoformat(),
            'states': states,
            'year': year,
            'params_varied': params_to_vary,
            'total_runs': len(results),
            'successful_runs': sum(1 for r in results if 'error' not in r),
            'failed_runs': sum(1 for r in results if 'error' in r),
            'cached_runs': sum(1 for r in results if r.get('cached', False)),
            'results': results
        }, f, indent=2)

    print(f"\n{'='*80}")
    print(f"SWEEP COMPLETE")
    print(f"{'='*80}")
    print(f"Total runs: {len(results)}")
    print(f"Successful: {sum(1 for r in results if 'error' not in r)}")
    print(f"Failed: {sum(1 for r in results if 'error' in r)}")
    print(f"Cached: {sum(1 for r in results if r.get('cached', False))}")
    print(f"Metadata saved: {metadata_file}")
    print(f"{'='*80}\n")

    return results


def main():
    parser = argparse.ArgumentParser(description='Parameter sensitivity analysis for redistricting')
    parser.add_argument('--year', type=str, default='2020', choices=['2020', '2010', '2000'],
                       help='Census year (default: 2020)')
    parser.add_argument('--states', type=str, nargs='+', required=True,
                       help='State codes to analyze (e.g., AL GA MN WI PA)')
    parser.add_argument('--output', type=str, default='outputs/sensitivity',
                       help='Output directory (default: outputs/sensitivity)')
    parser.add_argument('--data-version', type=str, default='v1',
                       help='Version to use for input data (default: v1)')
    parser.add_argument('--params', type=str, default='all',
                       choices=['all', 'ufactor', 'niter', 'objtype', 'seed'],
                       help='Which parameters to vary (default: all)')

    args = parser.parse_args()

    # Load state configuration
    try:
        state_config = get_state_config(args.year)
    except (ValueError, ImportError) as e:
        print(f"ERROR: Could not load config for year {args.year}: {e}")
        sys.exit(1)

    # Validate states
    invalid_states = [s for s in args.states if s.upper() not in state_config]
    if invalid_states:
        print(f"ERROR: Invalid state codes: {', '.join(invalid_states)}")
        print(f"Valid codes: {', '.join(sorted(state_config.keys()))}")
        sys.exit(1)

    states = [s.upper() for s in args.states]
    output_dir = Path(args.output)

    # Run parameter sweep
    run_parameter_sweep(
        states=states,
        year=args.year,
        output_dir=output_dir,
        state_config=state_config,
        data_version=args.data_version,
        params_to_vary=args.params
    )


if __name__ == '__main__':
    main()
