#!/usr/bin/env python3
"""
Run demographic visualization on all states in a redistricting run.

This script:
1. Finds all state directories in the output
2. For each state, runs visualize_district_demographics.py
3. Tracks progress and reports results
"""

import argparse
import subprocess
import sys
import os
from pathlib import Path
from tqdm import tqdm


def main():
    parser = argparse.ArgumentParser(description='Run demographic visualization on all states')
    parser.add_argument('--census-year', type=str, default='2020', choices=['2020'],
                       help='Census year (default: 2020)')
    parser.add_argument('--version', type=str, default='v1',
                       help='Version identifier (default: v1)')
    parser.add_argument('--output-dir', type=str, default=None,
                       help='Override output directory (default: outputs/us_{census_year}_{version})')
    parser.add_argument('--dpi', type=int, default=150,
                       help='DPI for output maps (default: 150)')
    args = parser.parse_args()

    # Determine output directory
    if args.output_dir:
        output_dir = Path(args.output_dir)
    else:
        output_dir = Path(f'outputs/us_{args.census_year}_{args.version}')

    states_dir = output_dir / 'states'

    if not states_dir.exists():
        print(f"ERROR: States directory not found: {states_dir}")
        return 1

    # Find all state directories
    state_dirs = sorted([d for d in states_dir.iterdir() if d.is_dir()])

    if len(state_dirs) == 0:
        print(f"ERROR: No state directories found in: {states_dir}")
        return 1

    # Get position from parent (or None if standalone)
    position = int(os.environ.get('TQDM_POSITION', '-1'))
    send_status = position >= 0

    def report_progress(msg):
        """Report progress to parent pipeline."""
        if send_status:
            print(f"STATUS:{position}:{msg}", flush=True)

    # Only show detailed output if running standalone
    if not send_status:
        print(f"\nRunning demographic visualization on {len(state_dirs)} states...")
        print(f"Census Year: {args.census_year}")
        print()

    # Get scripts directory
    scripts_dir = Path(__file__).parent

    successful = []
    failed = []
    skipped = []

    # Process each state
    show_progress = not send_status
    pbar = tqdm(state_dirs, desc="Demographic visualization", unit="state", ncols=100, disable=send_status)

    for i, state_dir in enumerate(pbar):
        state_name = state_dir.name.replace('_', ' ').title()

        if show_progress:
            pbar.set_description(f"Demographic visualization: {state_name}")
        else:
            # Report to parent
            report_progress(f"Demographic Visualization ({i+1}/{len(state_dirs)}) - {state_name}")

        # Check if demographic analysis exists (required)
        demo_dir = state_dir / 'demographic'
        demo_file = demo_dir / 'district_demographics.csv'

        if not demo_file.exists():
            skipped.append(state_name)
            continue

        # Check if demographic maps already exist
        maps_dir = demo_dir / 'maps'
        gender_map = maps_dir / 'gender_balance.png'
        majority_map = maps_dir / 'majority_race.png'
        diversity_map = maps_dir / 'diversity_index.png'

        if gender_map.exists() and majority_map.exists() and diversity_map.exists():
            # Skip if already done
            skipped.append(state_name)
            continue

        try:
            # Run visualize_district_demographics.py
            visualize_cmd = [
                sys.executable,
                str(scripts_dir / 'visualize_district_demographics.py'),
                str(state_dir),
                '--dpi', str(args.dpi)
            ]
            result = subprocess.run(visualize_cmd, capture_output=True, text=True)
            if result.returncode != 0:
                failed.append(state_name)
                continue

            successful.append(state_name)

        except Exception as e:
            if not send_status:
                print(f"\n  ERROR processing {state_name}: {e}")
            failed.append(state_name)

    pbar.close()

    # Report final status
    if send_status:
        report_progress(f"Demographic Visualization - {len(successful)} visualized, {len(skipped)} skipped, {len(failed)} failed")
    else:
        # Summary (standalone mode only)
        print(f"\n{'='*70}")
        print(f"DEMOGRAPHIC VISUALIZATION COMPLETE")
        print(f"{'='*70}")
        print(f"Visualized: {len(successful)} states")
        print(f"Skipped: {len(skipped)} states (already done or no demographic analysis)")
        print(f"Failed: {len(failed)} states")

        if failed:
            print(f"\nFailed states: {', '.join(failed)}")

    return 0 if len(failed) == 0 else 1


if __name__ == '__main__':
    exit(main())
