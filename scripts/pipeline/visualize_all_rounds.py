#!/usr/bin/env python3
"""Visualize all intermediate rounds from state redistricting."""

import warnings
import json
import os
import sys
from pathlib import Path

# Import utility functions
sys.path.insert(0, str(Path(__file__).parent.parent.parent))
from scripts.utils import get_state_config, get_tract_file

# Suppress ALL warnings including matplotlib
warnings.filterwarnings('ignore')
os.environ['MPLBACKEND'] = 'Agg'

import geopandas as gpd
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
plt.rcParams['figure.max_open_warning'] = 0  # Completely disable figure warning
import matplotlib.cm as cm
import numpy as np
from datetime import datetime
import sys
import argparse
import os
import time
from tqdm import tqdm
from concurrent.futures import ProcessPoolExecutor, as_completed
import multiprocessing


def visualize_single_round(args_tuple):
    """Worker function to visualize a single round (must be picklable for multiprocessing)."""
    (metadata_file, tracts_file, state_name, total_districts, output_dir, colors_list, dpi) = args_tuple

    # Import inside worker
    import json
    import warnings
    import geopandas as gpd
    import numpy as np
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    plt.rcParams['figure.max_open_warning'] = 0  # Disable warning in worker
    warnings.filterwarnings('ignore')
    import matplotlib.patheffects as path_effects
    from pathlib import Path

    # Load metadata
    with open(metadata_file, 'r') as f:
        metadata = json.load(f)

    round_num = metadata['depth']
    num_regions = metadata['num_regions']
    total_pop = metadata.get('total_population', 0)
    ideal_pop = total_pop / total_districts if total_districts > 0 else 0

    # Load assignments
    assignments_file = metadata_file.parent / metadata_file.name.replace('_metadata.json', '_assignments.json')
    with open(assignments_file, 'r') as f:
        assignments_dict = json.load(f)

    # Load tracts
    tracts = gpd.read_parquet(tracts_file)

    # Convert to array - handle incomplete rounds
    try:
        assignments = np.array([assignments_dict[str(i)] for i in range(len(tracts))], dtype=np.int32)
    except KeyError:
        return None  # Skip incomplete round

    # Add to GeoDataFrame
    tracts['region'] = assignments

    # Create figure
    fig, ax = plt.subplots(1, 1, figsize=(16, 14))

    # Plot each region with thin white tract boundaries
    for region_id in range(num_regions):
        region_data = tracts[tracts['region'] == region_id]
        if len(region_data) > 0:
            region_data.plot(
                ax=ax,
                color=colors_list[region_id % len(colors_list)],
                edgecolor='white',
                linewidth=0.1,
                alpha=0.8
            )

    # Add thick region boundaries on top
    regions_dissolved = tracts.dissolve(by='region', as_index=False)
    regions_dissolved.boundary.plot(
        ax=ax,
        edgecolor='black',
        linewidth=1.5,
        zorder=10
    )

    # Add region numbers (1-based) with target district counts - white text with black outline
    if num_regions <= 4:
        fontsize = 24  # Reduced from 40
    elif num_regions <= 8:
        fontsize = 20  # Reduced from 28
    elif num_regions <= 16:
        fontsize = 18
    elif num_regions <= 32:
        fontsize = 12
    else:  # 52+ regions
        fontsize = 8

    # Get region info for target districts
    regions_info = metadata.get('regions', [])
    region_targets = {}
    for r in regions_info:
        region_id = r.get('region_id')
        target_districts = r.get('target_districts')
        if region_id is not None and target_districts is not None:
            region_targets[region_id] = target_districts

    for region_id in range(num_regions):
        region_data = tracts[tracts['region'] == region_id]
        if len(region_data) > 0:
            try:
                centroid = region_data.geometry.union_all().representative_point()

                # Create label with target districts if available
                # Hide "(1)" when region only has 1 district (redundant and distracting)
                if region_id in region_targets:
                    target_districts = region_targets[region_id]
                    if target_districts == 1:
                        label = str(region_id + 1)
                    else:
                        label = f"{region_id + 1} ({target_districts})"
                else:
                    label = str(region_id + 1)

                # Use black text with white outline for better contrast
                # (white text hard to read on light-colored regions)
                text = ax.text(centroid.x, centroid.y, label,
                        fontsize=fontsize, fontweight='bold', ha='center', va='center',
                        color='black', zorder=10)
                text.set_path_effects([path_effects.Stroke(linewidth=3, foreground='white'),
                                      path_effects.Normal()])
            except:
                pass

    ax.set_axis_off()

    # Calculate max deviation
    regions_info = metadata.get('regions', [])
    if regions_info:
        districts_per_region = total_districts / num_regions
        deviations = []
        for r in regions_info:
            pop = r.get('population', 0)
            per_district = pop / districts_per_region
            dev = abs((per_district - ideal_pop) / ideal_pop * 100) if ideal_pop > 0 else 0
            deviations.append(dev)
        max_dev = max(deviations) if deviations else 0
    else:
        max_dev = 0

    title = f'{state_name} Round {round_num}: {num_regions} Regions\n'
    if num_regions <= 8:
        dprs = total_districts // num_regions
        title += f'({dprs}-{dprs + 1} districts each)'
    else:
        title += f'Tract-Level Redistricting'

    ax.set_title(title, fontsize=16, fontweight='bold', pad=20)

    # Stats text box removed for cleaner visualization
    # Information available in CSV data files

    plt.tight_layout()

    # Create maps/rounds directory if it doesn't exist
    rounds_dir = output_dir / 'maps' / 'rounds'
    rounds_dir.mkdir(parents=True, exist_ok=True)

    # Save with zero-padded round number, no region count
    map_file = rounds_dir / f'round_{round_num:02d}.png'
    plt.savefig(map_file, dpi=dpi, bbox_inches='tight')
    plt.close(fig)  # Explicitly close the figure object
    del fig, ax  # Help garbage collection

    return round_num


def main():
    parser = argparse.ArgumentParser(description='Visualize intermediate redistricting rounds')
    parser.add_argument('run_dir', type=str, help='Run directory (e.g., outputs/us_2020_v1/states/california)')
    parser.add_argument('--state', type=str, required=True, help='State code (e.g., CA, NY)')
    parser.add_argument('--year', type=str, default='2020', choices=['2020', '2010', '2000'],
                       help='Census year (default: 2020)')
    parser.add_argument('--print-only', action='store_true',
                       help='Print what would be done without executing')
    parser.add_argument('--dpi', type=int, default=150, help='DPI for output maps')
    parser.add_argument('--position', type=int, default=2, help='Progress bar position (for parallel mode)')
    parser.add_argument('--debug', action='store_true',
                       help='Enable debug mode with progress delays and file display')
    args = parser.parse_args()

    run_dir = Path(args.run_dir)

    # Get state info from arguments
    state_code = args.state.upper()
    STATE_CONFIG = get_state_config(args.year)
    config = STATE_CONFIG.get(state_code)
    if not config:
        print(f"ERROR: Unknown state code {state_code}")
        sys.exit(1)
    state_name = config['name']

    # Load tracts (unified directory structure)
    tracts_file = str(get_tract_file(state_code, args.year))

    intermediate_dir = run_dir / 'intermediate'
    output_dir = run_dir  # Function will add maps/rounds subdirectory

    # Get number of districts (needed for progress bar and task list)
    try:
        STATE_CONFIG = get_state_config(args.year)
        config = STATE_CONFIG.get(state_code.upper(), {})
        num_districts = config.get('districts', 1)
    except:
        num_districts = 1

    # Show progress bars for integration with parent script
    operation_pos = args.position
    file_pos = operation_pos + 1

    stage_pbar = None
    file_pbar = None

    # Create progress bars if position is set (for integration with parent scripts)
    # In parallel mode: show stage progress but not file displays
    if args.print_only or args.position != 2 and args.position != 999:
        import math
        num_rounds = max(1, int(math.ceil(math.log2(num_districts))))

        # Create progress bar at operation_pos (number of rounds)
        stage_pbar = tqdm(total=num_rounds,
                         desc=f"{state_name} [{num_districts}D] Visualizing rounds",
                         unit="round",
                         position=operation_pos,
                         leave=True,
                         ncols=120)

        # Only show file display in sequential mode (not parallel)
        if not os.environ.get('PARALLEL_MODE'):
            # Check if files exist and color-code them
            project_root = Path(__file__).parent.parent.parent
            tracts_path = project_root / tracts_file

            # ANSI color codes
            GREEN = '\033[32m'
            RED = '\033[31m'
            RESET = '\033[0m'

            tracts_display = f"{GREEN}{tracts_file}{RESET}" if tracts_path.exists() else f"{RED}{tracts_file}{RESET}"

            file_pbar = tqdm(total=0,
                            desc=f"      Files: {tracts_display}",
                            bar_format="{desc}",
                            position=file_pos,
                            leave=False,
                            ncols=100)

    # Check if output already exists
    rounds_output_dir = output_dir / 'maps' / 'rounds'
    if not args.print_only and rounds_output_dir.exists():
        # Check if there are already PNG files in the output directory
        existing_pngs = list(rounds_output_dir.glob('*.png'))
        if len(existing_pngs) > 0:
            skip_exists = True
        else:
            skip_exists = False
    else:
        skip_exists = False

    # Print-only mode or skip if output exists
    if args.print_only or skip_exists:
        # Update progress bar description to show skipping status
        if skip_exists and not args.print_only and stage_pbar:
            stage_pbar.set_description("    Visualizing rounds (skipped - output exists)")

        if args.debug:
            # Debug mode: show progress with delays
            if stage_pbar:
                for i in range(num_rounds):
                    time.sleep(0.05)  # Simulate visualization
                    stage_pbar.update(1)
            else:
                time.sleep(0.5)  # Just wait without progress
        else:
            # Non-debug mode: instant completion
            if stage_pbar:
                stage_pbar.update(num_rounds)

        # Close progress bars
        if stage_pbar:
            stage_pbar.close()
        if file_pbar:
            file_pbar.clear()
            file_pbar.close()

        sys.exit(0)

    # Create maps/rounds subdirectory (no need to create output_dir as it's run_dir)
    rounds_output_dir.mkdir(parents=True, exist_ok=True)

    # Find all round files
    round_files = sorted(intermediate_dir.glob('round_*_metadata.json'))

    colors_list = list(cm.tab20.colors) + list(cm.tab20b.colors) + list(cm.tab20c.colors)

    # Get position for stacked progress bars
    position = args.position

    # Prepare tasks for parallel execution
    tasks = [
        (metadata_file, tracts_file, state_name, num_districts, output_dir, colors_list, args.dpi)
        for metadata_file in round_files
    ]

    # Use stage_pbar if it exists, otherwise create a new one (but not if position==999)
    if stage_pbar:
        pbar = stage_pbar
        pbar.total = len(tasks)
        pbar.refresh()
    elif position == 999:
        # Parallel mode - don't show progress bar, use dummy
        pbar = tqdm(total=len(tasks), disable=True)
    else:
        pbar = tqdm(total=len(tasks),
                    desc="    Visualizing rounds",
                    unit="round",
                    position=position,
                    leave=False,
                    ncols=100)

    try:
        # When in PARALLEL_MODE, run sequentially to avoid nested parallelism overhead
        # Otherwise use ProcessPoolExecutor for parallel visualization
        if os.environ.get('PARALLEL_MODE'):
            # Sequential execution in main process (avoid multiprocessing overhead)
            completed = 0
            for task in tasks:
                try:
                    round_num = visualize_single_round(task)
                    if round_num is None:
                        pass  # Skip incomplete round
                    completed += 1
                    pbar.update(1)
                    # Print progress for parent process to read
                    if args.position == 999:
                        print(f"PROGRESS:{completed}/{len(tasks)}", flush=True)
                except Exception as e:
                    completed += 1
                    pbar.update(1)
                    if args.position == 999:
                        print(f"PROGRESS:{completed}/{len(tasks)}", flush=True)
        else:
            # Parallel execution with ProcessPoolExecutor
            max_workers = min(4, multiprocessing.cpu_count())
            with ProcessPoolExecutor(max_workers=max_workers) as executor:
                futures = [executor.submit(visualize_single_round, task) for task in tasks]

                completed = 0
                for future in as_completed(futures):
                    try:
                        round_num = future.result()
                        if round_num is None:
                            pass  # Skip incomplete round
                        completed += 1
                        pbar.update(1)
                        # Print progress for parent process to read
                        if args.position == 999:
                            print(f"PROGRESS:{completed}/{len(tasks)}", flush=True)
                    except Exception as e:
                        completed += 1
                        pbar.update(1)
                        if args.position == 999:
                            print(f"PROGRESS:{completed}/{len(tasks)}", flush=True)
    finally:
        # Only close if we created it (not stage_pbar)
        if not stage_pbar:
            pbar.close()


    # Close progress bars if they were created
    if stage_pbar:
        stage_pbar.close()
    if file_pbar:
        file_pbar.clear()
        file_pbar.close()


if __name__ == "__main__":
    main()
