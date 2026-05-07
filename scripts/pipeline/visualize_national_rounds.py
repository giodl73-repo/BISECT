#!/usr/bin/env python3
"""
Create national-level maps showing recursive bisection progression across all states.

Uses the 3-panel layout (Continental US + Alaska + Hawaii insets) to visualize
how districts are progressively created through recursive bisection:
- Round 1: 2 regions
- Round 2: 4 regions
- Round 3: 8 regions
- Round 4: 16 regions
- Round 5: 32 regions
- Round 6: 64 regions
- Continue through later rounds as states complete divisions
"""

import warnings
warnings.filterwarnings('ignore')

import geopandas as gpd
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.cm as cm
import matplotlib.patheffects as path_effects
import json
import os
from pathlib import Path
import argparse
import sys
import numpy as np
from tqdm import tqdm

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))
from scripts.utils import get_state_config, get_error_logger


def load_state_round_data(state_dir, round_num, tracts_file, state_code, state_name):
    """
    Load round data for a single state.

    If the requested round doesn't exist (state completed earlier),
    loads the last available round to keep completed districts visible.

    For single-district states with no intermediate rounds, loads final assignments.

    Returns None if no round data exists for this state.
    """
    # Look for round metadata file
    intermediate_dir = state_dir / 'intermediate'

    # Handle single-district states (no intermediate directory)
    if not intermediate_dir.exists():
        # Load final assignments directly
        assignments_file = state_dir / 'data' / 'final_assignments.pkl'
        if not assignments_file.exists():
            return None

        if not Path(tracts_file).exists():
            return None

        tracts = gpd.read_parquet(tracts_file)

        with open(assignments_file, 'rb') as f:
            assignments = pickle.load(f)

        # Assign all tracts to district 1
        tracts['region'] = [assignments[i] for i in range(len(tracts))]
        tracts['state_code'] = state_code
        tracts['state_name'] = state_name
        tracts['unique_region_id'] = tracts['state_code'] + '_0'  # Single region

        return {
            'tracts': tracts,
            'num_regions': 1,
            'total_population': int(tracts['population'].sum()) if 'population' in tracts.columns else 0,
            'region_targets': {f"{state_code}_0": 1}
        }

    # Find all round files
    round_files = sorted(intermediate_dir.glob('round_*_metadata.json'))

    if not round_files:
        return None

    # Build a map of round depth to file
    rounds_map = {}
    for f in round_files:
        with open(f, 'r') as fp:
            metadata = json.load(fp)
            rounds_map[metadata['depth']] = f

    # Find the specific round we want, or the highest available round if it doesn't exist
    target_file = None
    if round_num in rounds_map:
        target_file = rounds_map[round_num]
    else:
        # State completed earlier - use the last available round
        available_rounds = sorted(rounds_map.keys())
        if available_rounds and available_rounds[-1] < round_num:
            target_file = rounds_map[available_rounds[-1]]

    if not target_file:
        return None

    # Load metadata
    with open(target_file, 'r') as f:
        metadata = json.load(f)

    # Load assignments
    assignments_file = target_file.parent / target_file.name.replace('_metadata.json', '_assignments.json')
    if not assignments_file.exists():
        return None

    with open(assignments_file, 'r') as f:
        assignments_dict = json.load(f)

    # Load tracts
    if not Path(tracts_file).exists():
        return None

    tracts = gpd.read_parquet(tracts_file)

    # Convert assignments to array
    try:
        assignments = np.array([assignments_dict[str(i)] for i in range(len(tracts))], dtype=np.int32)
    except KeyError:
        return None  # Incomplete round

    # Add region assignments
    tracts['region'] = assignments
    tracts['state_code'] = state_code
    tracts['state_name'] = state_name

    # Create unique region IDs across states (state_code + region number)
    tracts['unique_region_id'] = tracts['state_code'] + '_' + tracts['region'].astype(str)

    # Extract target districts for each region
    regions_info = metadata.get('regions', [])
    region_targets = {}
    for r in regions_info:
        region_id = r.get('region_id')
        target_districts = r.get('target_districts')
        if region_id is not None and target_districts is not None:
            unique_id = f"{state_code}_{region_id}"
            region_targets[unique_id] = target_districts

    return {
        'tracts': tracts,
        'num_regions': metadata['num_regions'],
        'total_population': metadata.get('total_population', 0),
        'region_targets': region_targets
    }


def create_national_round_map(round_num, all_states_data, output_file, dpi=150, progress_callback=None):
    """
    Create national map for a specific round showing regions across all states.

    Uses 3-panel layout: Continental US (large) + Alaska inset + Hawaii inset
    """
    if not all_states_data:
        print(f"No data for round {round_num}")
        return False

    def report(msg):
        if progress_callback:
            progress_callback(msg)

    report(f"Round {round_num} - Combining tract data...")

    # Combine all state tracts
    all_tracts_list = [data['tracts'] for data in all_states_data]
    us_tracts = pd.concat(all_tracts_list, ignore_index=True)

    # Combine all region targets
    all_region_targets = {}
    for data in all_states_data:
        all_region_targets.update(data.get('region_targets', {}))

    # Calculate total regions
    total_regions = us_tracts['unique_region_id'].nunique()

    # Separate continental US, Alaska, and Hawaii
    continental = us_tracts[~us_tracts['state_code'].isin(['AK', 'HI'])].copy()
    alaska = us_tracts[us_tracts['state_code'] == 'AK'].copy()
    hawaii = us_tracts[us_tracts['state_code'] == 'HI'].copy()

    # Create figure with 3 panels (same layout as visualize_national_districts.py)
    fig = plt.figure(figsize=(28, 18))

    # Main continental US (takes most of the space)
    ax_main = plt.subplot2grid((4, 4), (0, 0), colspan=4, rowspan=3)

    # Alaska inset (bottom left)
    ax_alaska = plt.subplot2grid((4, 4), (3, 0), colspan=1, rowspan=1)

    # Hawaii inset (bottom left-center)
    ax_hawaii = plt.subplot2grid((4, 4), (3, 1), colspan=1, rowspan=1)

    # Use extended color palette
    colors = list(cm.tab20.colors) + list(cm.tab20b.colors) + list(cm.tab20c.colors)

    def plot_region(tracts_data, ax, region_name, show_labels=True):
        """Plot a region with colored tracts and boundaries."""
        if len(tracts_data) == 0:
            ax.set_axis_off()
            return

        # Get unique regions in this subset
        unique_regions = tracts_data['unique_region_id'].unique()

        # Create a color mapping for these regions
        region_to_color = {}
        for i, region_id in enumerate(sorted(unique_regions)):
            region_to_color[region_id] = colors[i % len(colors)]

        # Plot tracts with thin white boundaries
        for region_id in unique_regions:
            region_data = tracts_data[tracts_data['unique_region_id'] == region_id]
            region_data.plot(
                ax=ax,
                color=region_to_color[region_id],
                edgecolor='white',
                linewidth=0.05,
                alpha=0.8
            )

        # Add thick black boundaries around regions
        regions_dissolved = tracts_data.dissolve(by='unique_region_id', as_index=False)
        regions_dissolved.boundary.plot(
            ax=ax,
            edgecolor='black',
            linewidth=1.0,
            zorder=10
        )

        # Add state boundaries (lighter gray, dashed)
        states_dissolved = tracts_data.dissolve(by='state_code', as_index=False)
        states_dissolved.boundary.plot(
            ax=ax,
            edgecolor='darkgray',
            linewidth=0.5,
            linestyle='--',
            zorder=9,
            alpha=0.7
        )

        # Add region labels with target district counts
        if show_labels:
            # Scale font size based on number of regions
            if total_regions <= 8:
                fontsize = 10
            elif total_regions <= 16:
                fontsize = 8
            elif total_regions <= 32:
                fontsize = 6
            elif total_regions <= 64:
                fontsize = 5
            else:
                fontsize = 4

            for region_id in unique_regions:
                region_data = tracts_data[tracts_data['unique_region_id'] == region_id]
                if len(region_data) > 0:
                    try:
                        centroid = region_data.geometry.union_all().representative_point()

                        # Parse region_id (format: "CA_0", "TX_1", etc.)
                        parts = region_id.split('_')
                        if len(parts) == 2:
                            state_code = parts[0]
                            region_num = int(parts[1]) + 1  # 1-based

                            # Create label with target districts if available
                            if region_id in all_region_targets:
                                label = f"{state_code}-{region_num} ({all_region_targets[region_id]})"
                            else:
                                label = f"{state_code}-{region_num}"

                            text = ax.text(centroid.x, centroid.y, label,
                                    fontsize=fontsize, fontweight='bold',
                                    ha='center', va='center',
                                    color='white', zorder=11)
                            text.set_path_effects([path_effects.Stroke(linewidth=1.5, foreground='black'),
                                                  path_effects.Normal()])
                    except:
                        pass

        ax.set_axis_off()

    # Plot each region (labels only on continental US to avoid clutter on small insets)
    report(f"Round {round_num} - Plotting continental US...")
    plot_region(continental, ax_main, "Continental US", show_labels=True)

    report(f"Round {round_num} - Plotting Alaska & Hawaii...")
    plot_region(alaska, ax_alaska, "Alaska", show_labels=False)
    plot_region(hawaii, ax_hawaii, "Hawaii", show_labels=False)

    # Add title
    report(f"Round {round_num} - Finalizing map...")
    expected_regions = 2 ** round_num
    title = f'U.S. Congressional Districts - Round {round_num}\n'
    title += f'{total_regions} regions nationwide (target: {expected_regions} from 2^{round_num} bisections)'

    fig.suptitle(title, fontsize=20, fontweight='bold', y=0.98)

    # Stats text box removed for cleaner visualization


    # Information available in CSV data files

    plt.tight_layout()

    # Save
    output_file.parent.mkdir(parents=True, exist_ok=True)
    plt.savefig(output_file, dpi=dpi, bbox_inches='tight')
    plt.close(fig)

    return True


def main():
    parser = argparse.ArgumentParser(description='Create national round progression maps')
    parser.add_argument('--year', type=str, default='2020', choices=['2000', '2010', '2020'],
                       help='Census year (default: 2020)')
    parser.add_argument('--version', type=str, default='v1',
                       help='Version identifier (default: v1)')
    parser.add_argument('--output-dir', type=str, default=None,
                       help='Output directory (default: outputs/us_YEAR_VERSION)')
    parser.add_argument('--dpi', type=int, default=150,
                       help='DPI for output maps (default: 150)')
    parser.add_argument('--max-rounds', type=int, default=10,
                       help='Maximum rounds to generate (default: 10)')
    args = parser.parse_args()

    # Determine output directory
    if args.output_dir:
        output_dir = Path(args.output_dir)
    else:
        output_dir = Path(f'outputs/us_{args.year}_{args.version}')

    # Get state config
    state_config = get_state_config(args.year)

    # Initialize error logger
    error_logger = None
    try:
        error_logger = get_error_logger(output_dir, args.version, int(args.year))
    except Exception:
        # If logger initialization fails, continue without logging (non-fatal)
        pass

    # Check if called from parent (progress reporting protocol)
    position = int(os.environ.get('TQDM_POSITION', '-1'))
    send_status = position >= 0
    is_standalone = not send_status

    def report_progress(msg):
        """Send progress to parent process."""
        if send_status:
            print(f"STATUS:{position}:{msg}", flush=True)

    # Only show banners if standalone
    if is_standalone:
        print(f"\n{'='*80}")
        print(f"Creating National Round Progression Maps")
        print(f"{'='*80}")
        print(f"Year: {args.year}")
        print(f"Version: {args.version}")
        print(f"Output: {output_dir}")
        print(f"DPI: {args.dpi}")
        print(f"Max rounds: {args.max_rounds}\n")

    # Check which rounds are available across all states
    available_rounds = {}

    report_progress("Scanning states for round data...")

    # Progress bar for scanning (disabled if called from parent)
    scan_iter = tqdm(state_config.items(),
                     desc="Scanning states",
                     disable=send_status,
                     leave=False)

    for state_code, config in scan_iter:
        state_name = config['name']
        state_dir = output_dir / 'states' / state_name.lower().replace(' ', '_')

        if not state_dir.exists():
            continue

        intermediate_dir = state_dir / 'intermediate'
        if not intermediate_dir.exists():
            continue

        # Find all rounds for this state
        round_files = sorted(intermediate_dir.glob('round_*_metadata.json'))
        for f in round_files:
            with open(f, 'r') as fp:
                metadata = json.load(fp)
                round_num = metadata['depth']

                if round_num not in available_rounds:
                    available_rounds[round_num] = []

                available_rounds[round_num].append((state_code, state_name, state_dir))

    if not available_rounds:
        if is_standalone:
            print("No round data found!")
        return 1

    # Generate maps for each round
    rounds_to_generate = [r for r in sorted(available_rounds.keys()) if r <= args.max_rounds]

    if is_standalone:
        print(f"\nFound {len(available_rounds)} rounds: {sorted(available_rounds.keys())}")
        print(f"\nGenerating {len(rounds_to_generate)} national maps...")

    skipped = 0
    created = 0

    # Create maps/rounds directory if it doesn't exist
    rounds_dir = output_dir / 'maps' / 'rounds'
    rounds_dir.mkdir(parents=True, exist_ok=True)

    for idx, round_num in enumerate(rounds_to_generate, 1):
        states_in_round = available_rounds[round_num]

        # Check if output file already exists (zero-padded, no year suffix)
        output_file = rounds_dir / f'round_{round_num:02d}.png'

        if output_file.exists():
            skipped += 1
            if is_standalone:
                print(f"\nRound {round_num}: [SKIP] Already exists")
            continue

        report_progress(f"Creating round {round_num} map ({idx}/{len(rounds_to_generate)})")

        if is_standalone:
            print(f"\nRound {round_num}: {len(states_in_round)} states")

        # Load data for all states in this round
        all_states_data = []

        load_iter = tqdm(states_in_round,
                        desc=f"  Loading round {round_num}",
                        disable=send_status,
                        leave=False)

        for state_code, state_name, state_dir in load_iter:
            # Load tracts (unified directory structure)
            state_code_lower = state_code.lower()
            tracts_file = f'data/tracts/{args.year}/{state_code_lower}_tracts_{args.year}.parquet'

            data = load_state_round_data(state_dir, round_num, tracts_file,
                                        state_code, state_name)

            if data:
                all_states_data.append(data)

        report_progress(f"Rendering round {round_num} map ({idx}/{len(rounds_to_generate)}) - Loading data...")

        success = create_national_round_map(round_num, all_states_data,
                                           output_file, dpi=args.dpi,
                                           progress_callback=report_progress)

        if success:
            created += 1
            report_progress(f"Round {round_num} complete ({idx}/{len(rounds_to_generate)})")
            if is_standalone:
                print(f"  [OK] Created: {output_file.name}")
        else:
            report_progress(f"Round {round_num} FAILED ({idx}/{len(rounds_to_generate)})")
            if is_standalone:
                print(f"  [ERROR] Failed: round {round_num}")

    if is_standalone:
        print(f"\n{'='*80}")
        print(f"National round progression maps complete!")
        print(f"Created: {created}, Skipped: {skipped}")
        print(f"Output directory: {output_dir}")
        print(f"{'='*80}\n")

    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as e:
        print(f"FATAL ERROR: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
