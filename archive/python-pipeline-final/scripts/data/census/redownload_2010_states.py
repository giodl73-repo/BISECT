#!/usr/bin/env python3
"""
Re-download incomplete 2010 Census states with improved rate limiting.

This script re-downloads states that have incomplete population data,
using the improved downloader with proper rate limiting and validation.
"""

import argparse
import sys
from pathlib import Path
import time
from tqdm import tqdm

sys.path.insert(0, str(Path(__file__).parent.parent / 'src'))

from download_tracts_improved import download_tracts_improved

# States that need re-downloading based on validation results
# These states have less than 95% of their expected population
INCOMPLETE_STATES_2010 = [
    # Major issues (< 95%)
    'IA', 'IN', 'KS', 'KY', 'LA', 'MA', 'MD', 'ME', 'MI', 'MN',
    'MO', 'MS', 'MT', 'NC', 'ND', 'NE', 'NH', 'NJ', 'NM', 'NY',
    'OH', 'OK', 'OR', 'PA', 'RI', 'SC', 'SD', 'TN', 'TX', 'UT',
    'VA', 'VT', 'WA', 'WI', 'WV', 'WY',
    # Incomplete (95-99%)
    'NV',
    # Minor issues (99%+, but not perfect)
    'IL',
]


def main():
    parser = argparse.ArgumentParser(
        description='Re-download incomplete 2010 Census states'
    )
    parser.add_argument('--states', type=str, nargs='*',
                        help='Specific state codes to download (default: all incomplete states)')
    parser.add_argument('--delay', type=float, default=3.0,
                        help='Delay between API requests in seconds (default: 3.0)')
    parser.add_argument('--delay-between-states', type=float, default=10.0,
                        help='Delay between states in seconds (default: 10.0)')
    parser.add_argument('--max-retries', type=int, default=5,
                        help='Maximum retries for failed requests (default: 5)')
    parser.add_argument('--no-resume', action='store_true',
                        help='Do not resume from partial downloads')
    parser.add_argument('--dry-run', action='store_true',
                        help='Show what would be downloaded without downloading')
    parser.add_argument('--yes', '-y', action='store_true',
                        help='Skip confirmation prompt and start immediately')

    args = parser.parse_args()

    # Determine which states to download
    if args.states:
        states_to_download = [s.upper() for s in args.states]
    else:
        states_to_download = INCOMPLETE_STATES_2010

    print("=" * 80)
    print(f"Re-downloading 2010 Census Data for {len(states_to_download)} States")
    print("=" * 80)
    print(f"\nSettings:")
    print(f"  - API request delay: {args.delay}s")
    print(f"  - Delay between states: {args.delay_between_states}s")
    print(f"  - Max retries: {args.max_retries}")
    print(f"  - Resume: {'No' if args.no_resume else 'Yes'}")
    print()
    print(f"States to download: {', '.join(states_to_download)}")
    print("=" * 80)

    if args.dry_run:
        print("\nDRY RUN - No downloads will be performed")
        return 0

    # Estimate time
    avg_counties_per_state = 60  # rough estimate
    avg_time_per_county = args.delay + 0.5  # request delay + processing
    estimated_minutes = (len(states_to_download) * avg_counties_per_state * avg_time_per_county) / 60
    print(f"\nEstimated time: {estimated_minutes:.0f} minutes")
    print("(This is a rough estimate; actual time may vary)")
    print()

    if not args.yes:
        input("Press Enter to start downloading, or Ctrl+C to cancel...")
        print()

    success_count = 0
    failed_states = []
    start_time = time.time()

    for i, state in enumerate(tqdm(states_to_download, desc="Overall progress", unit="state")):
        print(f"\n{'=' * 80}")
        print(f"[{i+1}/{len(states_to_download)}] Downloading {state}")
        print(f"{'=' * 80}")

        try:
            download_tracts_improved(
                state=state,
                year=2010,
                cache_dir='cache',
                output_dir='data/raw',
                delay_between_requests=args.delay,
                max_retries=args.max_retries,
                resume=not args.no_resume
            )
            success_count += 1
            print(f"\n[OK] {state} completed successfully")

        except KeyboardInterrupt:
            print("\n\nDownload interrupted by user")
            print(f"Completed: {success_count}/{i+1} states")
            return 1

        except Exception as e:
            print(f"\n[FAIL] ERROR: {state} failed - {e}")
            failed_states.append(state)

        # Delay between states (except after last state)
        if i < len(states_to_download) - 1:
            print(f"\nWaiting {args.delay_between_states}s before next state...")
            time.sleep(args.delay_between_states)

    # Summary
    elapsed_time = time.time() - start_time
    print("\n" + "=" * 80)
    print("DOWNLOAD SUMMARY")
    print("=" * 80)
    print(f"  Total time: {elapsed_time/60:.1f} minutes")
    print(f"  Successful: {success_count}/{len(states_to_download)}")
    if failed_states:
        print(f"  Failed: {len(failed_states)} - {', '.join(failed_states)}")
    else:
        print(f"  Failed: 0")
    print("=" * 80)
    print()

    if success_count == len(states_to_download):
        print("[SUCCESS] All states downloaded successfully!")
        print("\nNext step: Run validation to confirm:")
        print("  python scripts/validate_2010_census_data.py")
        return 0
    else:
        print(f"[FAIL] {len(failed_states)} states failed")
        print("\nTo retry failed states, run:")
        print(f"  python scripts/redownload_2010_states.py --states {' '.join(failed_states)}")
        return 1


if __name__ == '__main__':
    sys.exit(main())
