#!/usr/bin/env python3
"""
Calculate compactness metrics for all states.

Runs calculate_compactness_metrics.py on each state directory.
"""

import subprocess
import sys
from pathlib import Path

def calculate_compactness_for_all_states(output_dir='outputs/us_2020_v1'):
    """Calculate compactness for all states."""

    output_dir = Path(output_dir)
    states_dir = output_dir / 'states'

    if not states_dir.exists():
        print(f"ERROR: {states_dir} not found")
        return 1

    # Find calculate_compactness_metrics.py
    calc_script = Path('scripts/pipeline/calculate_compactness_metrics.py')
    if not calc_script.exists():
        print(f"ERROR: {calc_script} not found")
        return 1

    # Get all state directories
    state_dirs = sorted([d for d in states_dir.iterdir() if d.is_dir()])

    print("="*70)
    print(f"CALCULATING COMPACTNESS FOR {len(state_dirs)} STATES")
    print("="*70)

    successful = []
    failed = []
    skipped = []

    for i, state_dir in enumerate(state_dirs, 1):
        state_name = state_dir.name.replace('_', ' ').title()

        # Check if already calculated
        compactness_file = state_dir / 'district_compactness.csv'
        if compactness_file.exists():
            print(f"[{i}/{len(state_dirs)}] {state_name} - Already calculated, skipping")
            skipped.append(state_name)
            continue

        print(f"[{i}/{len(state_dirs)}] {state_name} - Calculating...", end=' ', flush=True)

        try:
            result = subprocess.run(
                [sys.executable, str(calc_script), str(state_dir)],
                capture_output=True,
                text=True,
                timeout=300
            )

            if result.returncode == 0:
                print("OK")
                successful.append(state_name)
            else:
                print("FAILED")
                print(f"  Error: {result.stderr[:200]}")
                failed.append(state_name)

        except subprocess.TimeoutExpired:
            print("TIMEOUT")
            failed.append(state_name)
        except Exception as e:
            print(f"ERROR: {e}")
            failed.append(state_name)

    # Summary
    print("\n" + "="*70)
    print("COMPACTNESS CALCULATION COMPLETE")
    print("="*70)
    print(f"Successful: {len(successful)}")
    print(f"Skipped: {len(skipped)}")
    print(f"Failed: {len(failed)}")

    if failed:
        print(f"\nFailed states: {', '.join(failed)}")

    return 0 if len(failed) == 0 else 1

def main():
    """Main function."""
    import argparse

    parser = argparse.ArgumentParser(description='Calculate compactness for all states')
    parser.add_argument('--output-dir', type=str, default='outputs/us_2020_v1',
                       help='Output directory with redistricting results')
    args = parser.parse_args()

    return calculate_compactness_for_all_states(args.output_dir)

if __name__ == '__main__':
    sys.exit(main())
