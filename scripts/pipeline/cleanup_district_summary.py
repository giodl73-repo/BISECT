#!/usr/bin/env python3
"""
Clean up duplicate compactness columns in district_summary.csv files.

Removes _x and _y suffixed columns created by pandas merge.
"""

import pandas as pd
from pathlib import Path
import sys

def cleanup_district_summary(state_dir):
    """Remove duplicate compactness columns from district_summary.csv."""
    summary_file = Path(state_dir) / 'district_summary.csv'

    if not summary_file.exists():
        return False

    df = pd.read_csv(summary_file)

    # Check for duplicate columns
    duplicate_cols = [col for col in df.columns if col.endswith('_x') or col.endswith('_y')]

    if not duplicate_cols:
        return False  # Already clean

    print(f"  {state_dir.name}: Removing {len(duplicate_cols)} duplicate columns")

    # Drop all _x and _y suffixed columns
    df = df.drop(columns=duplicate_cols)

    # Save cleaned version
    df.to_csv(summary_file, index=False)

    return True

def main():
    base_dir = Path(sys.argv[1]) if len(sys.argv) > 1 else Path('outputs/us_2020_v1')
    states_dir = base_dir / 'states'

    if not states_dir.exists():
        print(f"ERROR: {states_dir} not found")
        return 1

    print(f"Cleaning up district_summary.csv files in {base_dir}")
    print()

    cleaned = 0
    already_clean = 0

    for state_dir in sorted(states_dir.iterdir()):
        if not state_dir.is_dir():
            continue

        if cleanup_district_summary(state_dir):
            cleaned += 1
        else:
            already_clean += 1

    print()
    print(f"Cleaned: {cleaned} states")
    print(f"Already clean: {already_clean} states")

    return 0

if __name__ == '__main__':
    sys.exit(main())
