#!/usr/bin/env python3
"""
Create individual detailed maps for each district.
"""

import sys
import os
import warnings
import time
import contextlib

# Suppress ALL warnings including matplotlib
warnings.filterwarnings('ignore')
os.environ['MPLBACKEND'] = 'Agg'

from pathlib import Path

# Import utility functions
sys.path.insert(0, str(Path(__file__).parent.parent.parent))
from scripts.utils import get_state_config, get_tract_file, get_places_file

import pickle
import pandas as pd
import geopandas as gpd
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
plt.rcParams['figure.max_open_warning'] = 0  # Completely disable figure warning
import matplotlib.patheffects as path_effects
import matplotlib.cm as cm
import matplotlib.colors as mcolors
from tqdm import tqdm
import argparse
from concurrent.futures import ProcessPoolExecutor, as_completed
import multiprocessing


def create_single_district_map(args_tuple):
    """Worker function to create a single district map (must be picklable for multiprocessing)."""
    (district_id, tracts_file, assignments_file, cities_file, summary_file,
     output_dir, num_districts, colors_list, dpi) = args_tuple

    # Import inside worker to avoid issues with multiprocessing
    import warnings
    import geopandas as gpd
    import pandas as pd
    import pickle
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    plt.rcParams['figure.max_open_warning'] = 0  # Disable warning in worker
    warnings.filterwarnings('ignore')
    import matplotlib.patheffects as path_effects
    import matplotlib.colors as mcolors
    import contextlib
    import os

    # Load data
    tracts = gpd.read_parquet(tracts_file)
    with open(assignments_file, 'rb') as f:
        assignments = pickle.load(f)
    tracts['district'] = [assignments[i] for i in range(len(tracts))]

    cities_df = pd.read_csv(cities_file)
    summary_df = pd.read_csv(summary_file)

    # Color mapping
    district_colors = {i: colors_list[(i - 1) % len(colors_list)] for i in range(1, num_districts + 1)}

    def darken_color(color, factor=0.7):
        rgb = mcolors.to_rgb(color)
        return tuple(c * factor for c in rgb)

    # Get district data
    district_tracts = tracts[tracts['district'] == district_id]
    district_info = summary_df[summary_df['district'] == district_id].iloc[0]
    city_info = cities_df[cities_df['district'] == district_id].iloc[0]

    if len(district_tracts) == 0:
        return district_id

    # Create figure
    fig, ax = plt.subplots(1, 1, figsize=(12, 10))

    # Get bounding box from focus district only with healthy margin
    bounds = district_tracts.total_bounds
    x_range = bounds[2] - bounds[0]
    y_range = bounds[3] - bounds[1]
    padding = max(x_range, y_range) * 0.20  # Healthy 20% margin

    view_bounds = (
        bounds[0] - padding,
        bounds[1] - padding,
        bounds[2] + padding,
        bounds[3] + padding
    )

    # Filter tracts to only those in view
    tracts_in_view = tracts.cx[view_bounds[0]:view_bounds[2], view_bounds[1]:view_bounds[3]]

    district_color = district_colors.get(district_id, 'steelblue')

    # Plot neighboring districts in view (lighter colors)
    neighbors_in_view = tracts_in_view[tracts_in_view['district'] != district_id]
    if len(neighbors_in_view) > 0:
        # Plot each neighbor in its own color
        for neighbor_id in neighbors_in_view['district'].unique():
            neighbor_tracts = tracts_in_view[tracts_in_view['district'] == neighbor_id]
            if len(neighbor_tracts) > 0:
                neighbor_color = district_colors.get(neighbor_id, 'lightgray')
                neighbor_tracts.plot(
                    ax=ax,
                    color=neighbor_color,
                    edgecolor='white',
                    linewidth=0.3,
                    alpha=0.3,
                    zorder=1
                )

    # Plot focus district (full color, prominent)
    district_tracts.plot(
        ax=ax,
        color=district_color,
        edgecolor='white',
        linewidth=0.5,
        alpha=0.8,
        zorder=2
    )

    # Add district boundary outline
    district_boundary = district_tracts.geometry.union_all()
    gpd.GeoSeries([district_boundary]).boundary.plot(
        ax=ax,
        color=darken_color(district_color, 0.5),
        linewidth=2.5,
        zorder=3
    )

    # Add district number to this district (prominent)
    try:
        district_centroid = district_tracts.geometry.union_all().representative_point()
        text = ax.text(
            district_centroid.x, district_centroid.y,
            str(district_id),
            fontsize=48,
            fontweight='bold',
            ha='center',
            va='center',
            color='white',
            zorder=5
        )
        text.set_path_effects([
            path_effects.Stroke(linewidth=4, foreground=darken_color(district_color, 0.5)),
            path_effects.Normal()
        ])
    except:
        pass

    # Add district numbers to neighboring districts (smaller, lighter)
    if len(neighbors_in_view) > 0:
        for neighbor_id in neighbors_in_view['district'].unique():
            neighbor_tracts = neighbors_in_view[neighbors_in_view['district'] == neighbor_id]
            neighbor_color = district_colors.get(neighbor_id, 'lightgray')
            try:
                neighbor_centroid = neighbor_tracts.geometry.union_all().representative_point()
                text = ax.text(
                    neighbor_centroid.x, neighbor_centroid.y,
                    str(int(neighbor_id)),
                    fontsize=18,
                    fontweight='bold',
                    ha='center',
                    va='center',
                    color='white',
                    alpha=0.6,
                    zorder=4
                )
                text.set_path_effects([
                    path_effects.Stroke(linewidth=2, foreground=darken_color(neighbor_color, 0.5)),
                    path_effects.Normal()
                ])
            except:
                pass

    # Add city marker for this district (larger, prominent)
    if city_info['city_population'] > 0 and pd.notna(city_info['city_lon']):
        city_x = city_info['city_lon']
        city_y = city_info['city_lat']
        if (view_bounds[0] <= city_x <= view_bounds[2] and
            view_bounds[1] <= city_y <= view_bounds[3]):
            marker_color = darken_color(district_color, 0.6)

            # City dot
            ax.plot(city_x, city_y, 'o',
                   color=marker_color, markersize=10, zorder=15,
                   markeredgecolor='white', markeredgewidth=1.5)

            # City label
            label = city_info['largest_city']
            text = ax.text(
                city_x,
                city_y + y_range * 0.03,
                label,
                fontsize=11,
                fontweight='bold',
                ha='center',
                va='bottom',
                color=marker_color,
                bbox=dict(boxstyle='round,pad=0.4', facecolor='white',
                         edgecolor=marker_color, linewidth=1.5, alpha=0.9),
                zorder=16
            )

    # Add city markers and labels for neighboring districts (smaller, lighter)
    if len(neighbors_in_view) > 0:
        for neighbor_id in neighbors_in_view['district'].unique():
            neighbor_city_info = cities_df[cities_df['district'] == neighbor_id]
            if len(neighbor_city_info) > 0:
                neighbor_city_info = neighbor_city_info.iloc[0]
                if neighbor_city_info['city_population'] > 0 and pd.notna(neighbor_city_info['city_lon']):
                    # Check if city is within view bounds
                    city_x = neighbor_city_info['city_lon']
                    city_y = neighbor_city_info['city_lat']
                    if (view_bounds[0] <= city_x <= view_bounds[2] and
                        view_bounds[1] <= city_y <= view_bounds[3]):
                        neighbor_color = district_colors.get(neighbor_id, 'lightgray')
                        marker_color = darken_color(neighbor_color, 0.6)

                        # City dot (smaller, lighter)
                        ax.plot(city_x, city_y, 'o',
                               color=marker_color, markersize=5, zorder=14,
                               markeredgecolor='white', markeredgewidth=0.7, alpha=0.5)

                        # City label (smaller, lighter)
                        label = neighbor_city_info['largest_city']
                        text = ax.text(
                            city_x,
                            city_y + y_range * 0.02,
                            label,
                            fontsize=7,
                            ha='center',
                            va='bottom',
                            color=marker_color,
                            alpha=0.6,
                            bbox=dict(boxstyle='round,pad=0.3', facecolor='white',
                                     edgecolor=marker_color, linewidth=0.7, alpha=0.7),
                            zorder=13
                        )

    # Set view bounds
    ax.set_xlim(view_bounds[0], view_bounds[2])
    ax.set_ylim(view_bounds[1], view_bounds[3])
    ax.set_aspect('equal')
    ax.set_axis_off()

    # Title with district info
    city_name = city_info['largest_city'] if city_info['city_population'] > 0 else 'Rural District'
    title = f"District {district_id}: {city_name}"
    ax.set_title(title, fontsize=16, fontweight='bold', pad=15)

    # Add statistics text box
    pop = int(district_info['population'])
    ideal_pop = int(district_info['ideal_population'])
    deviation = district_info['deviation_percent']
    n_tracts = int(district_info['num_tracts'])

    stats_text = f"Population: {pop:,}\n"
    stats_text += f"Ideal: {ideal_pop:,}\n"
    stats_text += f"Deviation: {deviation:+.2f}%\n"
    stats_text += f"Tracts: {n_tracts}"

    props = dict(boxstyle='round', facecolor='white', edgecolor=district_color,
                alpha=0.9, linewidth=2)
    ax.text(0.02, 0.98, stats_text, transform=ax.transAxes, fontsize=10,
           verticalalignment='top', bbox=props, family='monospace')

    # Suppress tight_layout warnings to stderr
    with contextlib.redirect_stderr(open(os.devnull, 'w')):
        plt.tight_layout()

    # Save with clean filename (no city slug)
    filename = f"district_{district_id:02d}.png"
    output_file = output_dir / filename

    plt.savefig(output_file, dpi=dpi, bbox_inches='tight')
    plt.close(fig)  # Explicitly close the figure object
    del fig, ax  # Help garbage collection

    return district_id


def create_district_maps(
    tracts_file: str,
    places_file: str,
    assignments_file: str,
    cities_file: str,
    summary_file: str,
    output_dir: Path,
    num_districts: int = 52,
    debug: bool = False,
    progress_bar = None,
    position: int = 2
):
    """
    Create individual map for each district.
    """
    if debug:
        print("\nLoading data...")

    # Create output directory
    output_dir.mkdir(parents=True, exist_ok=True)

    # Color mapping
    colors_list = list(cm.tab20.colors) + list(cm.tab20b.colors) + list(cm.tab20c.colors)

    if debug:
        print(f"\nGenerating individual district maps in parallel...")

    # Position is now passed as parameter (999 in parallel mode)
    # No need to derive from progress_bar

    # Prepare tasks for parallel execution
    tasks = [
        (district_id, tracts_file, assignments_file, cities_file, summary_file,
         output_dir, num_districts, colors_list, args.dpi)
        for district_id in range(1, num_districts + 1)
    ]

    # Use passed-in progress bar if available, otherwise create a new one (but not if position==999)
    if progress_bar:
        pbar = progress_bar
        pbar.total = num_districts
        pbar.refresh()
        pbar_context = None
    elif position == 999:
        # Parallel mode - don't show progress bar, use dummy
        pbar_context = tqdm(total=num_districts, disable=True)
        pbar = pbar_context
    else:
        pbar_context = tqdm(total=num_districts,
                          desc="    Creating maps",
                          position=position,
                          leave=False,
                          ncols=100)
        pbar = pbar_context

    try:
        # When in PARALLEL_MODE, run sequentially to avoid nested parallelism overhead
        # Otherwise use ProcessPoolExecutor for parallel map creation
        if os.environ.get('PARALLEL_MODE'):
            # Sequential execution in main process (avoid multiprocessing overhead)
            completed = 0
            for task in tasks:
                try:
                    district_id = create_single_district_map(task)
                    completed += 1
                    pbar.update(1)
                    # Print progress for parent process to read
                    if position == 999:
                        print(f"PROGRESS:{completed}/{num_districts}", flush=True)
                except Exception as e:
                    completed += 1
                    if debug:
                        print(f"Error creating map for district: {e}")
                    pbar.update(1)
                    if position == 999:
                        print(f"PROGRESS:{completed}/{num_districts}", flush=True)
        else:
            # Parallel execution with ProcessPoolExecutor
            max_workers = min(4, multiprocessing.cpu_count())
            with ProcessPoolExecutor(max_workers=max_workers) as executor:
                futures = [executor.submit(create_single_district_map, task) for task in tasks]

                completed = 0
                for future in as_completed(futures):
                    try:
                        district_id = future.result()
                        completed += 1
                        pbar.update(1)
                        # Print progress for parent process to read
                        if position == 999:
                            print(f"PROGRESS:{completed}/{num_districts}", flush=True)
                    except Exception as e:
                        completed += 1
                        if debug:
                            print(f"Error creating map for district: {e}")
                        pbar.update(1)
                        if position == 999:
                            print(f"PROGRESS:{completed}/{num_districts}", flush=True)
    finally:
        # Only close if we created it (not passed-in progress_bar)
        if pbar_context:
            pbar_context.close()

    if debug:
        print(f"\n{num_districts} district maps saved to: {output_dir}")




if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Create individual district maps')
    parser.add_argument('run_dir', type=str, help='Run directory (e.g., outputs/us_2020_v1/states/california)')
    parser.add_argument('--state', type=str, required=True, help='State code (e.g., CA, NY)')
    parser.add_argument('--year', type=str, default='2020', choices=['2020', '2010', '2000'],
                       help='Census year (default: 2020)')
    parser.add_argument('--version', type=str, default='v1',
                       help='Version identifier (default: v1)')
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

    # Load tract and places files (unified directory structure)
    tracts_file = str(get_tract_file(state_code, args.year, args.version))
    places_file = str(get_places_file(state_code, args.year, args.version))

    assignments_file = run_dir / 'data' / 'final_assignments.pkl'
    cities_file = run_dir / 'data' / 'district_cities.csv'
    summary_file = run_dir / 'data' / 'district_summary.csv'
    output_dir = run_dir / 'maps' / 'districts'

    # Show progress bars for integration with parent script
    operation_pos = args.position
    file_pos = operation_pos + 1

    stage_pbar = None
    file_pbar = None

    # Get number of districts from config (needed for skip logic)
    try:
        STATE_CONFIG = get_state_config(args.year)
        config = STATE_CONFIG.get(state_code.upper(), {})
        num_districts = config.get('districts', 1)
    except:
        num_districts = 1

    # Create progress bars if position is set (for integration with parent scripts)
    # In parallel mode: show stage progress but not file displays
    if args.print_only or args.position != 2 and args.position != 999:

        # Create progress bar at operation_pos (number of district maps)
        stage_pbar = tqdm(total=num_districts,
                         desc=f"{state_name} [{num_districts}D] Creating maps",
                         unit="map",
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

    # Check if output already exists - verify ALL expected district maps are present
    skip_exists = False
    if not args.print_only and output_dir.exists():
        # Check if all expected district maps exist (district_01.png through district_NN.png)
        expected_files = [output_dir / f'district_{i+1:02d}.png' for i in range(num_districts)]
        all_exist = all(f.exists() for f in expected_files)

        if all_exist:
            skip_exists = True
            if args.debug:
                print(f"[DEBUG] All {num_districts} district maps already exist, skipping", file=sys.stderr, flush=True)
        else:
            # Some maps are missing - need to regenerate
            missing_count = sum(1 for f in expected_files if not f.exists())
            if args.debug:
                print(f"[DEBUG] {missing_count} of {num_districts} district maps missing, regenerating all", file=sys.stderr, flush=True)

    # Print-only mode or skip if output exists
    if args.print_only or skip_exists:
        # Update progress bar description to show skipping status
        if skip_exists and not args.print_only and stage_pbar:
            stage_pbar.set_description("    Creating maps (skipped - output exists)")

        if args.debug:
            # Debug mode: show progress with delays
            if stage_pbar:
                for i in range(num_districts):
                    time.sleep(0.05)  # Simulate map creation
                    stage_pbar.update(1)
            else:
                time.sleep(1)  # Just wait without progress
        else:
            # Non-debug mode: instant completion
            if stage_pbar:
                stage_pbar.update(num_districts)

        # Close progress bars
        if stage_pbar:
            stage_pbar.close()
        if file_pbar:
            file_pbar.clear()
            file_pbar.close()

        sys.exit(0)

    # Auto-detect number of districts from assignments
    with open(assignments_file, 'rb') as f:
        assignments = pickle.load(f)
    num_districts = max(assignments.values())


    create_district_maps(
        tracts_file,
        places_file,
        str(assignments_file),
        str(cities_file),
        str(summary_file),
        output_dir,
        num_districts=num_districts,
        debug=args.debug,
        progress_bar=stage_pbar,
        position=args.position
    )

    # Close progress bars if they were created
    if stage_pbar:
        stage_pbar.close()
    if file_pbar:
        file_pbar.clear()
        file_pbar.close()

    if args.debug:
        print("\n" + "=" * 70)
        print("=" * 70)
