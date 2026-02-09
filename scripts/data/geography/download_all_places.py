#!/usr/bin/env python3
"""Download Census places (cities) data for all 50 states."""

import argparse
import subprocess
import sys
import os
from pathlib import Path
from tqdm import tqdm

# All 50 states + DC
ALL_STATES = [
    'AL', 'AK', 'AZ', 'AR', 'CA', 'CO', 'CT', 'DE', 'FL', 'GA',
    'HI', 'ID', 'IL', 'IN', 'IA', 'KS', 'KY', 'LA', 'ME', 'MD',
    'MA', 'MI', 'MN', 'MS', 'MO', 'MT', 'NE', 'NV', 'NH', 'NJ',
    'NM', 'NY', 'NC', 'ND', 'OH', 'OK', 'OR', 'PA', 'RI', 'SC',
    'SD', 'TN', 'TX', 'UT', 'VT', 'VA', 'WA', 'WV', 'WI', 'WY'
]

def check_existing(state_code, year=2020):
    """Check if places file already exists."""
    places_file = Path(f'data/raw/{state_code.lower()}_places_{year}.parquet')
    return places_file.exists()

def download_state_places(state_code, year=2020):
    """Download places for a single state."""
    print(f"\n{'='*70}")
    print(f"Downloading {state_code} places ({year} Census)...")
    print(f"{'='*70}")

    scripts_dir = Path(__file__).parent

    try:
        result = subprocess.run(
            [sys.executable, str(scripts_dir / 'download_places.py'),
             '--state', state_code, '--year', str(year)],
            check=True,
            capture_output=True,
            text=True,
            timeout=300
        )
        print(result.stdout)
        if result.stderr:
            print("STDERR:", result.stderr)
        return True
    except subprocess.CalledProcessError as e:
        print(f"ERROR: Failed to download {state_code}")
        print(f"STDOUT: {e.stdout}")
        print(f"STDERR: {e.stderr}")
        return False
    except subprocess.TimeoutExpired:
        print(f"ERROR: Download timed out for {state_code}")
        return False

def main():
    """Download places for all states."""
    parser = argparse.ArgumentParser(description='Download Census places for all 50 states')
    parser.add_argument('--year', type=int, default=2020, choices=[2020, 2010, 2000],
                        help='Census year (default: 2020)')
    parser.add_argument('--skip', type=str, nargs='*', default=[],
                        help='State codes to skip (e.g., CA TX)')
    args = parser.parse_args()

    skip_states = [s.upper() for s in args.skip]

    # Check which states need downloading
    to_download = []
    already_exists = []

    for state in ALL_STATES:
        if state in skip_states:
            continue
        if check_existing(state, args.year):
            already_exists.append(state)
        else:
            to_download.append(state)

    print(f"\n{'='*70}")
    print(f"Downloading Census Places Data for All States - {args.year} Census")
    print(f"{'='*70}")
    print(f"States with existing data: {len(already_exists)}")
    for state in already_exists:
        print(f"  [OK] {state}")
    print(f"\nStates to download: {len(to_download)}")
    for state in to_download:
        print(f"  - {state}")
    print(f"{'='*70}\n")

    if not to_download:
        print("All states already have places data!")
        return 0

    # Download missing states
    successful = []
    failed = []

    # Get position for stacked progress bars
    position = int(os.environ.get('TQDM_POSITION', '0'))

    with tqdm(to_download,
              desc="Downloading places",
              unit="state",
              position=position,
              leave=(position == 0),
              ncols=100) as pbar:
        for state in pbar:
            pbar.set_description(f"Downloading {state}")

            if download_state_places(state, args.year):
                successful.append(state)
                pbar.set_postfix_str("OK Complete")
            else:
                failed.append(state)
                pbar.set_postfix_str("X Failed")

    # Summary
    print(f"\n\n{'='*70}")
    print(f"DOWNLOAD SUMMARY")
    print(f"{'='*70}")
    print(f"Successful: {len(successful)}/{len(to_download)}")
    for state in successful:
        print(f"  [OK] {state}")

    if failed:
        print(f"\nFailed: {len(failed)}")
        for state in failed:
            print(f"  [FAIL] {state}")

    print(f"\nTotal states with places data: {len(already_exists) + len(successful)}/50")
    print(f"{'='*70}")

    return 0 if not failed else 1

if __name__ == '__main__':
    sys.exit(main())
