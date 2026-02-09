#!/usr/bin/env python3
"""
Run redistricting for a state at multiple resolutions.

This script runs the same state through the redistricting algorithm
at tract, block_group, and block resolutions to test MAUP sensitivity.
"""

import sys
import subprocess
from pathlib import Path
import argparse

def run_resolution(state, year, version, resolution, workers=1):
    """Run redistricting for one resolution."""
    print(f"\n{'='*70}")
    print(f"RUNNING {state} at {resolution.upper()} resolution")
    print(f"{'='*70}\n")

    script_path = Path(__file__).parent / 'run_state_redistricting.py'
    cmd = [
        sys.executable,
        str(script_path),
        '--state', state,
        '--year', year,
        '--resolution', resolution,
        '--version', version,
        '--dpi', '150'
    ]

    try:
        result = subprocess.run(cmd, check=True, text=True)
        print(f"\n[OK] {state} {resolution} completed successfully")
        return True
    except subprocess.CalledProcessError as e:
        print(f"\n[FAIL] {state} {resolution} failed with exit code {e.returncode}")
        return False

def main():
    parser = argparse.ArgumentParser(
        description='Run redistricting at multiple resolutions for MAUP validation'
    )
    parser.add_argument('--states', type=str, nargs='+', required=True,
                       help='State codes (e.g., AL CO)')
    parser.add_argument('--year', type=str, default='2020',
                       choices=['2020', '2010', '2000'],
                       help='Census year (default: 2020)')
    parser.add_argument('--version', type=str, default='maup_test',
                       help='Version identifier (default: maup_test)')
    parser.add_argument('--resolutions', type=str, nargs='+',
                       default=['tract', 'block_group', 'block'],
                       choices=['tract', 'block_group', 'block'],
                       help='Resolutions to run (default: all three)')

    args = parser.parse_args()

    print(f"\n{'='*70}")
    print(f"MULTI-RESOLUTION REDISTRICTING VALIDATION")
    print(f"{'='*70}")
    print(f"States: {', '.join(args.states)}")
    print(f"Year: {args.year}")
    print(f"Version: {args.version}")
    print(f"Resolutions: {', '.join(args.resolutions)}")
    print(f"{'='*70}\n")

    results = {}

    for state in args.states:
        state_code = state.upper()
        results[state_code] = {}

        for resolution in args.resolutions:
            success = run_resolution(state_code, args.year, args.version, resolution)
            results[state_code][resolution] = 'OK' if success else 'FAIL'

    # Summary
    print(f"\n\n{'='*70}")
    print(f"VALIDATION SUMMARY")
    print(f"{'='*70}\n")

    for state, res_results in results.items():
        print(f"{state}:")
        for resolution, status in res_results.items():
            status_icon = '[OK]' if status == 'OK' else '[FAIL]'
            print(f"  {status_icon} {resolution:12s} ... {status}")
        print()

    # Check if all passed
    all_passed = all(
        status == 'OK'
        for state_results in results.values()
        for status in state_results.values()
    )

    if all_passed:
        print("All validations PASSED!")
        return 0
    else:
        print("Some validations FAILED - see details above")
        return 1

if __name__ == '__main__':
    sys.exit(main())
