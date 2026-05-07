#!/usr/bin/env python3
"""
Aggregate all state rounds_hierarchy.csv files into a single US file.
"""

import pandas as pd
from pathlib import Path
import argparse
import os
from tqdm import tqdm

# State configuration
STATE_CONFIG = {
    'CA': {'name': 'California', 'districts': 52},
    'TX': {'name': 'Texas', 'districts': 38},
    'FL': {'name': 'Florida', 'districts': 28},
    'NY': {'name': 'New York', 'districts': 26},
    'PA': {'name': 'Pennsylvania', 'districts': 17},
    'IL': {'name': 'Illinois', 'districts': 17},
    'OH': {'name': 'Ohio', 'districts': 15},
    'GA': {'name': 'Georgia', 'districts': 14},
    'NC': {'name': 'North Carolina', 'districts': 14},
    'MI': {'name': 'Michigan', 'districts': 13},
    'NJ': {'name': 'New Jersey', 'districts': 12},
    'VA': {'name': 'Virginia', 'districts': 11},
    'WA': {'name': 'Washington', 'districts': 10},
    'AZ': {'name': 'Arizona', 'districts': 9},
    'MA': {'name': 'Massachusetts', 'districts': 9},
    'TN': {'name': 'Tennessee', 'districts': 9},
    'IN': {'name': 'Indiana', 'districts': 9},
    'MD': {'name': 'Maryland', 'districts': 8},
    'MO': {'name': 'Missouri', 'districts': 8},
    'WI': {'name': 'Wisconsin', 'districts': 8},
    'CO': {'name': 'Colorado', 'districts': 8},
    'MN': {'name': 'Minnesota', 'districts': 8},
    'SC': {'name': 'South Carolina', 'districts': 7},
    'AL': {'name': 'Alabama', 'districts': 7},
    'LA': {'name': 'Louisiana', 'districts': 6},
    'KY': {'name': 'Kentucky', 'districts': 6},
    'OR': {'name': 'Oregon', 'districts': 6},
    'OK': {'name': 'Oklahoma', 'districts': 5},
    'CT': {'name': 'Connecticut', 'districts': 5},
    'UT': {'name': 'Utah', 'districts': 4},
    'IA': {'name': 'Iowa', 'districts': 4},
    'NV': {'name': 'Nevada', 'districts': 4},
    'AR': {'name': 'Arkansas', 'districts': 4},
    'MS': {'name': 'Mississippi', 'districts': 4},
    'KS': {'name': 'Kansas', 'districts': 4},
    'NM': {'name': 'New Mexico', 'districts': 3},
    'NE': {'name': 'Nebraska', 'districts': 3},
    'ID': {'name': 'Idaho', 'districts': 2},
    'WV': {'name': 'West Virginia', 'districts': 2},
    'HI': {'name': 'Hawaii', 'districts': 2},
    'NH': {'name': 'New Hampshire', 'districts': 2},
    'ME': {'name': 'Maine', 'districts': 2},
    'RI': {'name': 'Rhode Island', 'districts': 2},
    'MT': {'name': 'Montana', 'districts': 2},
    'DE': {'name': 'Delaware', 'districts': 1},
    'SD': {'name': 'South Dakota', 'districts': 1},
    'ND': {'name': 'North Dakota', 'districts': 1},
    'AK': {'name': 'Alaska', 'districts': 1},
    'VT': {'name': 'Vermont', 'districts': 1},
    'WY': {'name': 'Wyoming', 'districts': 1},
}


def main(output_dir=None, print_only=False, debug=False, force=False):
    """Aggregate all state rounds_hierarchy.csv files into a single US file."""

    # Only print header if running standalone (not called from parent)
    is_standalone = not os.environ.get('TQDM_POSITION')

    if is_standalone:
        print("\n" + "=" * 70)
        print("Creating US Rounds Hierarchy Aggregate")
        print("=" * 70)

    if output_dir is None:
        us_dir = Path('outputs/us_2020_v2')
    else:
        us_dir = Path(output_dir)

    if is_standalone:
        print(f"Output directory: {us_dir}")
        print("=" * 70)

    # Create data directory if it doesn't exist
    data_dir = us_dir / 'data'
    data_dir.mkdir(parents=True, exist_ok=True)

    # Check if output already exists
    us_file = data_dir / 'us_rounds_hierarchy.csv'

    if not force and us_file.exists():
        if is_standalone:
            print("\nUS rounds hierarchy already exists - skipping")
            print(f"  {us_file.name}")
            print("\nUse --force to regenerate")
        return

    # Get position from parent (or 0 if standalone)
    position = int(os.environ.get('TQDM_POSITION', '0'))

    # In print-only mode, skip actual work
    if print_only:
        import time
        # Count multi-district states (43 out of 50)
        num_multi_district = sum(1 for _, cfg in STATE_CONFIG.items() if cfg['districts'] > 1)

        pbar = tqdm(total=num_multi_district,
                   desc="  Collecting hierarchies" if position > 0 else "Collecting hierarchies",
                   unit="state",
                   position=position,
                   ncols=100,
                   leave=False)

        if debug:
            for i in range(num_multi_district):
                time.sleep(0.02)
                pbar.update(1)
        else:
            pbar.update(num_multi_district)

        pbar.close()
        return

    all_hierarchies = []

    # Check if we should send status messages to parent
    send_status = position > 0

    def report_progress(msg):
        if send_status:
            print(f"STATUS:{position}:{msg}", flush=True)

    sorted_states = sorted(STATE_CONFIG.items(), key=lambda x: x[1]['districts'], reverse=True)
    multi_district_states = [(code, cfg) for code, cfg in sorted_states if cfg['districts'] > 1]
    total_states = len(multi_district_states)

    for idx, (state_code, config) in enumerate(multi_district_states, 1):
        state_name = config['name']
        num_districts = config['districts']

        # Report progress to parent
        report_progress(f"Create US rounds hierarchy - Collecting ({idx}/{total_states})")

        state_dir = us_dir / 'states' / state_name.lower().replace(' ', '_')
        hierarchy_file = state_dir / 'data' / 'rounds_hierarchy.csv'

        if hierarchy_file.exists():
            try:
                df = pd.read_csv(hierarchy_file)
                # Add state columns
                df.insert(0, 'state_code', state_code)
                df.insert(0, 'state', state_name)
                all_hierarchies.append(df)
            except Exception as e:
                pass  # Silently skip errors during progress bar operation

    if all_hierarchies:
        # Combine all state hierarchies
        us_hierarchy = pd.concat(all_hierarchies, ignore_index=True)
        us_hierarchy.to_csv(us_file, index=False)

    # Print summary only at the end and only if standalone
    if is_standalone and all_hierarchies:
        print("\n" + "=" * 70)
        print("SUCCESS! US rounds hierarchy aggregate created")
        print("=" * 70)
        print(f"  File: {us_file.name}")
        print(f"  Total rows: {len(us_hierarchy):,}")
        print(f"  States included: {len(all_hierarchies)}")
        print("=" * 70)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Create US rounds hierarchy aggregate')
    parser.add_argument('--output-dir', type=str, help='Output directory (default: outputs/us_2020_v2)')
    parser.add_argument('--print-only', action='store_true',
                        help='Print what would be done without executing')
    parser.add_argument('--debug', action='store_true',
                        help='Enable debug mode with progress delays')
    parser.add_argument('--force', action='store_true',
                        help='Force regeneration even if outputs exist')
    args = parser.parse_args()

    main(args.output_dir, args.print_only, args.debug, args.force)
