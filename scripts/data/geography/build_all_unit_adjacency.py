#!/usr/bin/env python3
"""Build adjacency graphs for all 50 states at any resolution (tract, block_group, or block)."""

import argparse
import subprocess
import sys
import os
from pathlib import Path
import warnings

# Suppress warnings that can disrupt hierarchical display
warnings.filterwarnings('ignore')

from tqdm import tqdm

# Add parent directories to path for imports
sys.path.insert(0, str(Path(__file__).resolve().parents[3]))
from scripts.utils.paths import get_census_data_dir

def report(message, is_standalone=True):
    """Report progress using STATUS protocol when running as child process."""
    if is_standalone:
        print(message, flush=True)
    else:
        # Running as child process - use STATUS protocol
        position = int(os.environ.get('TQDM_POSITION', '-1'))
        if position >= 0:
            print(f"STATUS:{position}:{message}", flush=True)

# All 50 states (no DC for redistricting)
ALL_STATES = [
    'AL', 'AK', 'AZ', 'AR', 'CA', 'CO', 'CT', 'DE', 'FL', 'GA',
    'HI', 'ID', 'IL', 'IN', 'IA', 'KS', 'KY', 'LA', 'ME', 'MD',
    'MA', 'MI', 'MN', 'MS', 'MO', 'MT', 'NE', 'NV', 'NH', 'NJ',
    'NM', 'NY', 'NC', 'ND', 'OH', 'OK', 'OR', 'PA', 'RI', 'SC',
    'SD', 'TN', 'TX', 'UT', 'VT', 'VA', 'WA', 'WV', 'WI', 'WY'
]

def check_existing(state_code, resolution='tract', year='2020', output_dir=None, version=None):
    """Check if adjacency graph already exists."""
    if output_dir is None:
        if version:
            output_dir = get_census_data_dir(version, year) / 'adjacency'
        else:
            output_dir = Path(f'outputs/data/{year}/adjacency')
    graph_file = Path(output_dir) / f'{state_code.lower()}_{resolution}_adjacency_{year}.pkl'
    return graph_file.exists()

def check_units_exist(state_code, resolution='tract', year='2020', input_dir=None, version=None):
    """Check if units file exists."""
    if input_dir is None:
        if version:
            input_dir = get_census_data_dir(version, year) / 'units'
        else:
            input_dir = Path(f'outputs/data/{year}/units')
    units_file = Path(input_dir) / f'{state_code.lower()}_{resolution}s_{year}.parquet'
    return units_file.exists()

def build_adjacency_graph(state_code, resolution='tract', year='2020', compute_boundary_lengths=False, water_distance=1.0,
                          minimum_boundary_length=0.0, input_dir=None, output_dir=None, version=None, is_standalone=True):
    """Build adjacency graph for a single state."""
    mode_str = "with boundary lengths" if compute_boundary_lengths else "without boundary lengths"
    water_str = f" (water distance: {water_distance} km)"
    boundary_filter = f", min boundary: {minimum_boundary_length}m" if minimum_boundary_length > 0 else ""
    report(f"Building {resolution} adjacency graph for {state_code} ({year} Census) {mode_str}{water_str}{boundary_filter}...", is_standalone)

    scripts_dir = Path(__file__).parent

    # Set input and output directories (version-aware)
    if input_dir is None:
        if version:
            input_dir = get_census_data_dir(version, year) / 'units'
        else:
            input_dir = Path(f'outputs/data/{year}/units')
    if output_dir is None:
        if version:
            output_dir = get_census_data_dir(version, year) / 'adjacency'
        else:
            output_dir = Path(f'outputs/data/{year}/adjacency')

    cmd = [sys.executable, str(scripts_dir / 'build_unit_adjacency.py'),
           '--state', state_code, '--year', str(year),
           '--resolution', resolution,
           '--water-distance', str(water_distance),
           '--minimum-boundary-length', str(minimum_boundary_length),
           '--input-dir', str(input_dir),
           '--output-dir', str(output_dir)]

    if version:
        cmd.extend(['--version', version])

    if compute_boundary_lengths:
        cmd.append('--compute-boundary-lengths')

    try:
        result = subprocess.run(
            cmd,
            check=True,
            capture_output=True,
            text=True,
            timeout=1800  # 30 minutes per state (blocks can be large)
        )
        # Only show subprocess output in standalone mode
        if is_standalone:
            if result.stdout:
                print(result.stdout)
            if result.stderr:
                print("STDERR:", result.stderr)
        return True
    except subprocess.CalledProcessError as e:
        report(f"ERROR: Failed to build graph for {state_code}", is_standalone)
        if is_standalone:
            print(f"STDOUT: {e.stdout}")
            print(f"STDERR: {e.stderr}")
        return False
    except subprocess.TimeoutExpired:
        report(f"ERROR: Build timed out for {state_code}", is_standalone)
        return False

def main():
    """Build adjacency graphs for all states."""
    parser = argparse.ArgumentParser(description='Build adjacency graphs for all 50 states at any resolution')
    parser.add_argument('--year', type=str, default='2020', choices=['2020', '2010', '2000'],
                        help='Census year (default: 2020)')
    parser.add_argument('--resolution', type=str, default='tract', choices=['tract', 'block_group', 'block'],
                        help='Geographic resolution (default: tract)')
    parser.add_argument('--version', type=str,
                        help='Version identifier (e.g., v1, test). If not provided, uses shared legacy paths.')
    parser.add_argument('--input-dir', type=str,
                        help='Input directory for unit files (default: outputs/{version}/data/{year}/units)')
    parser.add_argument('--output-dir', type=str,
                        help='Output directory for adjacency files (default: outputs/{version}/data/{year}/adjacency)')
    parser.add_argument('--compute-boundary-lengths', action='store_true',
                        help='Compute boundary lengths for edge-weighted partitioning')
    parser.add_argument('--water-distance', type=float, default=1.0,
                        help='Water distance threshold in km (default: 1.0)')
    parser.add_argument('--minimum-boundary-length', type=float, default=0.0,
                        help='Minimum shared boundary length (meters) to filter tiny adjacencies (default: 0, recommended: 10-25)')
    parser.add_argument('--workers', type=int, default=1,
                        help='Number of parallel workers (default: 1, sequential)')
    parser.add_argument('--reset', action='store_true',
                        help='Delete existing adjacency graphs and rebuild from scratch')
    parser.add_argument('--states', type=str, nargs='*',
                        help='Process specific states (e.g., VT DE CA). If not specified, processes all states.')
    args = parser.parse_args()

    # Detect if running as child process (via TQDM_POSITION)
    position = int(os.environ.get('TQDM_POSITION', '-1'))
    is_standalone = position < 0

    # Set default paths if not provided (version-aware)
    if args.input_dir:
        input_dir = args.input_dir
    elif args.version:
        input_dir = get_census_data_dir(args.version, args.year) / 'units'
    else:
        input_dir = Path(f'outputs/data/{args.year}/units')

    if args.output_dir:
        output_dir = args.output_dir
    elif args.version:
        output_dir = get_census_data_dir(args.version, args.year) / 'adjacency'
    else:
        output_dir = Path(f'outputs/data/{args.year}/adjacency')

    # Create output directory if it doesn't exist
    Path(output_dir).mkdir(parents=True, exist_ok=True)

    # Determine which states to process
    if args.states:
        states_to_process = [s.upper() for s in args.states]
    else:
        states_to_process = ALL_STATES

    # If reset flag is set, delete existing graphs
    if args.reset:
        report("RESET MODE: Deleting existing adjacency graphs...", is_standalone)
        deleted_count = 0
        for state in states_to_process:
            graph_file = Path(output_dir) / f'{state.lower()}_{args.resolution}_adjacency_{args.year}.pkl'

            if graph_file.exists():
                graph_file.unlink()
                deleted_count += 1
                report(f"  [DELETED] {state}", is_standalone)
        report(f"Deleted {deleted_count} existing graphs", is_standalone)

    # Check which states need processing
    to_build = []
    already_exists = []
    missing_units = []

    for state in states_to_process:
        if check_existing(state, args.resolution, args.year, output_dir, args.version):
            already_exists.append(state)
        elif not check_units_exist(state, args.resolution, args.year, input_dir, args.version):
            missing_units.append(state)
        else:
            to_build.append(state)

    # Only show detailed status in standalone mode
    if is_standalone:
        print(f"\n{'='*70}")
        mode_str = " with boundary lengths" if args.compute_boundary_lengths else ""
        water_str = f" (water distance: {args.water_distance} km)"
        print(f"Building {args.resolution.upper()} Adjacency Graphs - {args.year} Census{mode_str}{water_str}")
        print(f"{'='*70}")
        print(f"States with existing graphs: {len(already_exists)}")
        for state in already_exists:
            print(f"  [OK] {state}")

        if missing_units:
            print(f"\nStates missing {args.resolution} data: {len(missing_units)}")
            for state in missing_units:
                print(f"  [MISSING] {state}")

        print(f"\nStates to build: {len(to_build)}")
        for state in to_build:
            print(f"  - {state}")
        print(f"{'='*70}\n")

    if not to_build:
        report(f"All {len(states_to_process)} states already have {args.resolution} adjacency graphs!", is_standalone)
        return 0

    # Build missing graphs
    successful = []
    failed = []

    # Sequential processing (workers=1) or standalone mode
    if args.workers == 1 or is_standalone:
        # Only show progress bar in standalone mode
        if is_standalone:
            with tqdm(to_build,
                      desc=f"Building {args.resolution} adjacency",
                      unit="state",
                      position=0,
                      leave=True,
                      ncols=100) as pbar:
                for state in pbar:
                    pbar.set_description(f"Building {state}")

                    if build_adjacency_graph(state, args.resolution, args.year, args.compute_boundary_lengths, args.water_distance,
                                            args.minimum_boundary_length, input_dir, output_dir, args.version, is_standalone):
                        successful.append(state)
                        pbar.set_postfix_str("OK Built")
                    else:
                        failed.append(state)
                        pbar.set_postfix_str("X Failed")
        else:
            # Running as child process - emit STATUS protocol messages
            for state in to_build:
                if build_adjacency_graph(state, args.resolution, args.year, args.compute_boundary_lengths, args.water_distance,
                                        args.minimum_boundary_length, input_dir, output_dir, args.version, is_standalone):
                    successful.append(state)
                else:
                    failed.append(state)

    # Parallel processing (workers > 1) - not implemented for initial version
    else:
        report("ERROR: Parallel processing not yet implemented for resolution-aware adjacency building", is_standalone)
        return 1

    # Summary - only show detailed output in standalone mode
    if is_standalone:
        print(f"\n\n{'='*70}")
        print(f"BUILD SUMMARY")
        print(f"{'='*70}")
        print(f"Successful: {len(successful)}/{len(to_build)}")
        for state in successful:
            print(f"  [OK] {state}")

        if failed:
            print(f"\nFailed: {len(failed)}")
            for state in failed:
                print(f"  [FAIL] {state}")

        print(f"\nTotal states with graphs: {len(already_exists) + len(successful)}/{len(states_to_process)}")
        print(f"{'='*70}")
    else:
        # In child mode, just report the final result
        total_complete = len(already_exists) + len(successful)
        report(f"{args.resolution.capitalize()} adjacency graphs complete: {total_complete}/{len(states_to_process)} states ({len(successful)} built, {len(failed)} failed)", is_standalone)

    return 0 if not failed else 1

if __name__ == '__main__':
    sys.exit(main())
