#!/usr/bin/env python3
"""Download tract data for all 50 US states."""

import argparse
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent / 'src'))

from download_tracts import download_tracts
import time
import os
from tqdm import tqdm

# All 50 states + DC
STATES = [
    'AL', 'AK', 'AZ', 'AR', 'CA', 'CO', 'CT', 'DE', 'FL', 'GA',
    'HI', 'ID', 'IL', 'IN', 'IA', 'KS', 'KY', 'LA', 'ME', 'MD',
    'MA', 'MI', 'MN', 'MS', 'MO', 'MT', 'NE', 'NV', 'NH', 'NJ',
    'NM', 'NY', 'NC', 'ND', 'OH', 'OK', 'OR', 'PA', 'RI', 'SC',
    'SD', 'TN', 'TX', 'UT', 'VT', 'VA', 'WA', 'WV', 'WI', 'WY', 'DC'
]

def main():
    parser = argparse.ArgumentParser(description='Download tract data for all 50 US states')
    parser.add_argument('--year', type=int, default=2020, choices=[2020, 2010, 2000],
                        help='Census year (default: 2020)')
    parser.add_argument('--skip', type=str, nargs='*', default=[],
                        help='State codes to skip (e.g., CA TX)')
    args = parser.parse_args()

    skip_states = [s.upper() for s in args.skip]

    print(f"Downloading {args.year} tract data for {len(STATES) - len(skip_states)} states...")
    if skip_states:
        print(f"Skipping: {', '.join(skip_states)}")
    print("=" * 70)

    success_count = 0
    failed_states = []

    # Filter out skipped states
    states_to_download = [s for s in STATES if s not in skip_states]

    # Get position for stacked progress bars
    position = int(os.environ.get('TQDM_POSITION', '0'))

    # Use progress bar
    with tqdm(states_to_download,
              desc="Downloading tracts",
              unit="state",
              position=position,
              leave=(position == 0),
              ncols=100) as pbar:
        for state in pbar:
            pbar.set_description(f"Downloading {state}")

            try:
                download_tracts(state=state, year=args.year)
                success_count += 1
                pbar.set_postfix_str("OK Complete")
            except Exception as e:
                tqdm.write(f"ERROR: {state} failed - {e}")
                failed_states.append(state)
                pbar.set_postfix_str("X Failed")

            # Small delay to avoid overwhelming Census servers
            time.sleep(2)

    print("\n" + "=" * 70)
    print(f"SUMMARY:")
    print(f"  Successful: {success_count}/{len(STATES) - len(skip_states)}")
    print(f"  Skipped: {len(skip_states)}")
    if failed_states:
        print(f"  Failed: {len(failed_states)} - {', '.join(failed_states)}")
    else:
        print(f"  Failed: 0")
    print("=" * 70)

if __name__ == '__main__':
    main()
