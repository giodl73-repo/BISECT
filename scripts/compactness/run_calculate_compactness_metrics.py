#!/usr/bin/env python3
"""
Run compactness metrics calculation on all states.

Calculates Polsby-Popper, Reock, and convex hull ratio for each district
and adds them to district_summary.csv files.

Usage:
    python scripts/compactness/run_analyze_district_compactness.py --census-year 2020 --version v1
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
    parser = argparse.ArgumentParser(description='Run compactness metrics calculation on all states')
    parser.add_argument('--census-year', type=str, required=True,
                       choices=['2010', '2020'],
                       help='Census year')
    parser.add_argument('--version', type=str, required=True,
                       help='Version (e.g., v1, v2)')
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
        print(f"Calculating compactness metrics for 50 states...")
        print(f"Census Year: {args.census_year}")
        print()

    calculated = 0
    skipped = 0
    failed = []

    # Report initial progress
    report_progress("Calculating compactness metrics - Starting")

    # Process all states
    pbar = tqdm(total=len(STATES), desc="Compactness metrics", unit="state", disable=send_status)

    for i, (state_name, num_districts) in enumerate(STATES.items()):
        pbar.set_description(f"Compactness metrics: {state_name.title()}")

        # Report progress periodically
        if send_status and i % 10 == 0:
            report_progress(f"Compactness metrics ({i}/{len(STATES)}) - {state_name.title()}")

        state_dir = base_dir / 'states' / state_name

        # Check if state directory exists
        if not state_dir.exists():
            skipped += 1
            pbar.update(1)
            continue

        # Run calculation
        try:
            result = subprocess.run(
                [sys.executable, 'scripts/compactness/analyze_district_compactness.py',
                 str(state_dir)],
                capture_output=True,
                text=True,
                timeout=300
            )

            if result.returncode == 0:
                calculated += 1
            else:
                failed.append(state_name.title())
                if is_standalone:
                    print(f"\n{state_name.title()}: {result.stderr.strip()}")
        except Exception as e:
            failed.append(state_name.title())
            if is_standalone:
                print(f"\n{state_name.title()}: {e}")

        pbar.update(1)

    pbar.close()

    # Report final progress
    report_progress(f"Compactness metrics - {calculated} calculated, {skipped} skipped, {len(failed)} failed")

    # Print summary (only in standalone mode)
    if is_standalone:
        print()
        print("=" * 70)
        print("COMPACTNESS METRICS CALCULATION COMPLETE")
        print("=" * 70)
        print(f"Calculated: {calculated} states")
        print(f"Skipped: {skipped} states (no redistricting data)")

        if failed:
            print(f"Failed: {len(failed)} states")
            print()
            print(f"Failed states: {', '.join(failed)}")

    if failed:
        return 1

    return 0

if __name__ == '__main__':
    sys.exit(main())
