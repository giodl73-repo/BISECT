#!/usr/bin/env python3
"""Build adjacency graphs for all 50 states."""

import argparse
import subprocess
import sys
import os
from pathlib import Path
from tqdm import tqdm

# All 50 states (no DC for redistricting)
ALL_STATES = [
    'AL', 'AK', 'AZ', 'AR', 'CA', 'CO', 'CT', 'DE', 'FL', 'GA',
    'HI', 'ID', 'IL', 'IN', 'IA', 'KS', 'KY', 'LA', 'ME', 'MD',
    'MA', 'MI', 'MN', 'MS', 'MO', 'MT', 'NE', 'NV', 'NH', 'NJ',
    'NM', 'NY', 'NC', 'ND', 'OH', 'OK', 'OR', 'PA', 'RI', 'SC',
    'SD', 'TN', 'TX', 'UT', 'VT', 'VA', 'WA', 'WV', 'WI', 'WY'
]

def check_existing(state_code, year='2020', output_dir=None):
    """Check if adjacency graph already exists."""
    if output_dir is None:
        output_dir = f'outputs/data/{year}/adjacency'
    graph_file = Path(output_dir) / f'{state_code.lower()}_adjacency_{year}.pkl'
    return graph_file.exists()

def check_tracts_exist(state_code, year='2020', input_dir=None):
    """Check if tracts file exists."""
    if input_dir is None:
        input_dir = f'outputs/data/{year}/units'
    tracts_file = Path(input_dir) / f'{state_code.lower()}_tracts_{year}.parquet'
    return tracts_file.exists()

def build_adjacency_graph(state_code, year='2020', compute_boundary_lengths=False, water_distance=1.0,
                          minimum_boundary_length=0.0, input_dir=None, output_dir=None):
    """Build adjacency graph for a single state."""
    print(f"\n{'='*70}")
    mode_str = "with boundary lengths" if compute_boundary_lengths else "without boundary lengths"
    water_str = f" (water distance: {water_distance} km)"
    boundary_filter = f", min boundary: {minimum_boundary_length}m" if minimum_boundary_length > 0 else ""
    print(f"Building adjacency graph for {state_code} ({year} Census) {mode_str}{water_str}{boundary_filter}...")
    print(f"{'='*70}")

    scripts_dir = Path(__file__).parent

    # Set input and output directories (use new default paths)
    if input_dir is None:
        input_dir = f'outputs/data/{year}/units'
    if output_dir is None:
        output_dir = f'outputs/data/{year}/adjacency'

    cmd = [sys.executable, str(scripts_dir / 'build_tract_adjacency.py'),
           '--state', state_code, '--year', str(year),
           '--water-distance', str(water_distance),
           '--minimum-boundary-length', str(minimum_boundary_length),
           '--input-dir', input_dir,
           '--output-dir', output_dir]

    if compute_boundary_lengths:
        cmd.append('--compute-boundary-lengths')

    try:
        result = subprocess.run(
            cmd,
            check=True,
            capture_output=True,
            text=True,
            timeout=600  # 10 minutes per state
        )
        print(result.stdout)
        if result.stderr:
            print("STDERR:", result.stderr)
        return True
    except subprocess.CalledProcessError as e:
        print(f"ERROR: Failed to build graph for {state_code}")
        print(f"STDOUT: {e.stdout}")
        print(f"STDERR: {e.stderr}")
        return False
    except subprocess.TimeoutExpired:
        print(f"ERROR: Build timed out for {state_code}")
        return False

def main():
    """Build adjacency graphs for all states."""
    parser = argparse.ArgumentParser(description='Build adjacency graphs for all 50 states')
    parser.add_argument('--year', type=str, default='2020', choices=['2020', '2010', '2000'],
                        help='Census year (default: 2020)')
    parser.add_argument('--input-dir', type=str,
                        help='Input directory for tract files (default: outputs/data/{year}/units)')
    parser.add_argument('--output-dir', type=str,
                        help='Output directory for adjacency files (default: outputs/data/{year}/adjacency)')
    parser.add_argument('--compute-boundary-lengths', action='store_true',
                        help='Compute boundary lengths for edge-weighted partitioning')
    parser.add_argument('--water-distance', type=float, default=1.0,
                        help='Water distance threshold in km (default: 1.0)')
    parser.add_argument('--minimum-boundary-length', type=float, default=0.0,
                        help='Minimum shared boundary length (meters) to filter tiny adjacencies (default: 0, recommended: 10-25)')
    parser.add_argument('--reset', action='store_true',
                        help='Delete existing adjacency graphs and rebuild from scratch')
    args = parser.parse_args()

    # Set default paths if not provided
    input_dir = args.input_dir if args.input_dir else f'outputs/data/units/{args.year}'
    output_dir = args.output_dir if args.output_dir else f'outputs/data/adjacency/{args.year}'

    # Create output directory if it doesn't exist
    Path(output_dir).mkdir(parents=True, exist_ok=True)

    # If reset flag is set, delete existing graphs
    if args.reset:
        print(f"\n{'='*70}")
        print("RESET MODE: Deleting existing adjacency graphs...")
        print(f"{'='*70}")
        deleted_count = 0
        for state in ALL_STATES:
            graph_file = Path(output_dir) / f'{state.lower()}_adjacency_{args.year}.pkl'

            if graph_file.exists():
                graph_file.unlink()
                deleted_count += 1
                print(f"  [DELETED] {state}")
        print(f"\nDeleted {deleted_count} existing graphs")
        print(f"{'='*70}\n")

    # Check which states need processing
    to_build = []
    already_exists = []
    missing_tracts = []

    for state in ALL_STATES:
        if check_existing(state, args.year, output_dir):
            already_exists.append(state)
        elif not check_tracts_exist(state, args.year, input_dir):
            missing_tracts.append(state)
        else:
            to_build.append(state)

    print(f"\n{'='*70}")
    mode_str = " with boundary lengths" if args.compute_boundary_lengths else ""
    water_str = f" (water distance: {args.water_distance} km)"
    print(f"Building Adjacency Graphs for All States - {args.year} Census{mode_str}{water_str}")
    print(f"{'='*70}")
    print(f"States with existing graphs: {len(already_exists)}")
    for state in already_exists:
        print(f"  [OK] {state}")

    if missing_tracts:
        print(f"\nStates missing tract data: {len(missing_tracts)}")
        for state in missing_tracts:
            print(f"  [MISSING] {state}")

    print(f"\nStates to build: {len(to_build)}")
    for state in to_build:
        print(f"  - {state}")
    print(f"{'='*70}\n")

    if not to_build:
        print("All states already have adjacency graphs!")
        return 0

    # Build missing graphs
    successful = []
    failed = []

    # Get position for stacked progress bars
    position = int(os.environ.get('TQDM_POSITION', '0'))

    with tqdm(to_build,
              desc="Building adjacency graphs",
              unit="state",
              position=position,
              leave=(position == 0),
              ncols=100) as pbar:
        for state in pbar:
            pbar.set_description(f"Building {state}")

            if build_adjacency_graph(state, args.year, args.compute_boundary_lengths, args.water_distance,
                                    args.minimum_boundary_length, input_dir, output_dir):
                successful.append(state)
                pbar.set_postfix_str("OK Built")
            else:
                failed.append(state)
                pbar.set_postfix_str("X Failed")

    # Summary
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

    print(f"\nTotal states with graphs: {len(already_exists) + len(successful)}/50")
    print(f"{'='*70}")

    return 0 if not failed else 1

if __name__ == '__main__':
    sys.exit(main())
