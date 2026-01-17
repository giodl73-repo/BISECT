#!/usr/bin/env python3
"""
Run compactness visualization on all states.

Creates Polsby-Popper and Reock compactness maps for all districts
in all states.

Usage:
    python scripts/pipeline/run_compactness_visualization.py --census-year 2020 --version v1 --dpi 150
"""

import argparse
import os
import subprocess
import sys
from pathlib import Path
from tqdm import tqdm

# State configuration (2020 apportionment)
STATES = {
    'alabama': 7, 'alaska': 1, 'arizona': 9, 'arkansas': 4, 'california': 52,
    'colorado': 8, 'connecticut': 5, 'delaware': 1, 'florida': 28, 'georgia': 14,
    'hawaii': 2, 'idaho': 2, 'illinois': 17, 'indiana': 9, 'iowa': 4,
    'kansas': 4, 'kentucky': 6, 'louisiana': 6, 'maine': 2, 'maryland': 8,
    'massachusetts': 9, 'michigan': 13, 'minnesota': 8, 'mississippi': 4, 'missouri': 8,
    'montana': 2, 'nebraska': 3, 'nevada': 4, 'new_hampshire': 2, 'new_jersey': 12,
    'new_mexico': 3, 'new_york': 26, 'north_carolina': 14, 'north_dakota': 1, 'ohio': 15,
    'oklahoma': 5, 'oregon': 6, 'pennsylvania': 17, 'rhode_island': 2, 'south_carolina': 7,
    'south_dakota': 1, 'tennessee': 9, 'texas': 38, 'utah': 4, 'vermont': 1,
    'virginia': 11, 'washington': 10, 'west_virginia': 2, 'wisconsin': 8, 'wyoming': 1
}

def main():
    parser = argparse.ArgumentParser(description='Run compactness visualization on all states')
    parser.add_argument('--census-year', type=str, required=True,
                       choices=['2010', '2020'],
                       help='Census year')
    parser.add_argument('--version', type=str, required=True,
                       help='Version (e.g., v1, v2)')
    parser.add_argument('--dpi', type=int, default=150,
                       choices=[72, 100, 150, 200, 300],
                       help='DPI for output maps')
    args = parser.parse_args()

    # Get position from parent (or None if standalone)
    position = int(os.environ.get('TQDM_POSITION', '-1'))
    send_status = position >= 0
    is_standalone = not send_status

    def report_progress(msg):
        """Send progress update to parent if running in parallel mode."""
        if send_status:
            print(f"STATUS:{position}:{msg}", flush=True)

    base_dir = Path(f'outputs/us_{args.census_year}_{args.version}')

    if not base_dir.exists():
        if is_standalone:
            print(f"ERROR: Output directory not found: {base_dir}")
        return 1

    if is_standalone:
        print(f"Running compactness visualization on 50 states...")
        print(f"Census Year: {args.census_year}")
        print(f"DPI: {args.dpi}")
        print()

    visualized = 0
    skipped = 0
    failed = []

    # Report initial progress
    report_progress("Compactness visualization - Starting")

    # Process all states
    pbar = tqdm(total=len(STATES), desc="Compactness visualization", unit="state", disable=send_status)

    for i, (state_name, num_districts) in enumerate(STATES.items()):
        pbar.set_description(f"Compactness visualization: {state_name.title()}")

        # Report progress periodically
        if send_status and i % 10 == 0:
            report_progress(f"Compactness visualization ({i}/{len(STATES)}) - {state_name.title()}")

        state_dir = base_dir / 'states' / state_name

        # Check if state directory exists
        if not state_dir.exists():
            skipped += 1
            pbar.update(1)
            continue

        # Check if visualizations already exist
        maps_dir = state_dir / 'compactness_analysis' / 'maps'
        pp_map = maps_dir / f'polsby_popper_districts_{args.census_year}.png'
        reock_map = maps_dir / f'reock_districts_{args.census_year}.png'

        if pp_map.exists() and reock_map.exists():
            skipped += 1
            pbar.update(1)
            continue

        # Run visualization
        try:
            result = subprocess.run(
                [sys.executable, 'scripts/pipeline/visualize_compactness.py',
                 str(state_dir), '--census-year', args.census_year, '--dpi', str(args.dpi)],
                capture_output=True,
                text=True,
                timeout=300
            )

            if result.returncode == 0:
                visualized += 1
            else:
                failed.append(state_name.title())
        except Exception as e:
            failed.append(state_name.title())

        pbar.update(1)

    pbar.close()

    # Report final progress
    report_progress(f"Compactness visualization - {visualized} visualized, {skipped} skipped, {len(failed)} failed")

    # Print summary (only in standalone mode)
    if is_standalone:
        print()
        print("=" * 70)
        print("COMPACTNESS VISUALIZATION COMPLETE")
        print("=" * 70)
        print(f"Visualized: {visualized} states")
        print(f"Skipped: {skipped} states (already done or no redistricting data)")

        if failed:
            print(f"Failed: {len(failed)} states")
            print()
            print(f"Failed states: {', '.join(failed)}")

    if failed:
        return 1

    return 0

if __name__ == '__main__':
    sys.exit(main())
