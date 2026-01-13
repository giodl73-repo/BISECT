#!/usr/bin/env python3
"""Run full recursive bisection for any state."""

import sys
import os
import warnings

# Suppress matplotlib warnings
warnings.filterwarnings('ignore', category=UserWarning, module='matplotlib')
from pathlib import Path
import pickle
import numpy as np
from datetime import datetime
import argparse

sys.path.insert(0, str(Path(__file__).parent.parent.parent / 'src'))
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

# Import configuration files
try:
    from scripts.config_2020 import STATE_CONFIG_2020
except ImportError:
    STATE_CONFIG_2020 = None

try:
    from scripts.config_2010 import STATE_CONFIG_2010
except ImportError:
    STATE_CONFIG_2010 = None

from apportionment.partition.recursive_bisection import RecursiveBisection
import geopandas as gpd
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import matplotlib.patheffects as path_effects
import matplotlib.cm as cm

def run_state_redistricting(state_code: str, state_config: dict, year: str = '2020',
                           output_dir: str = None, print_only: bool = False, debug: bool = False, dpi: int = 150, position: int = 2, reset: bool = False):
    """Run redistricting for a specific state."""

    state_code = state_code.upper()
    if state_code not in state_config:
        print(f"ERROR: Unknown state code '{state_code}'")
        print(f"Valid codes: {', '.join(sorted(state_config.keys()))}")
        sys.exit(1)

    config = state_config[state_code]
    state_name = config['name']
    num_districts = config['districts']

    # File paths
    graph_file = f'data/adjacency/{state_code.lower()}_adjacency_{year}.pkl'
    tracts_file = f'data/raw/{state_code.lower()}_tracts_{year}.parquet'

    # Show progress bars for integration with parent script
    operation_pos = position

    stage_pbar = None
    file_pbar = None

    # Only create progress bars for print-only mode
    # In real mode, RecursiveBisection creates its own progress bar
    if print_only and position != 2 and position != 999:
        from tqdm import tqdm

        # Create position for progress bar (splits count)
        # num_districts - 1 splits needed (binary tree structure)
        total_splits = num_districts - 1

        stage_pbar = tqdm(total=total_splits,
                         desc=f"{state_name} [{num_districts}D] Splitting",
                         unit="split",
                         position=operation_pos,
                         leave=False,
                         ncols=120)

    # Check if output already exists
    if output_dir is None:
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        output_dir = Path(f'outputs/{state_name.lower().replace(" ", "_")}_full_{timestamp}')
    else:
        output_dir = Path(output_dir)

    # Handle --reset flag: delete output directory for fresh run
    if reset and output_dir.exists() and not print_only:
        import shutil
        print(f"\n[RESET] Deleting existing output directory: {output_dir}")
        shutil.rmtree(output_dir)
        print(f"[RESET] Deleted. Starting fresh run.\n")

    final_file = output_dir / 'final_assignments.pkl'

    # In print-only mode or if output exists, skip processing
    if print_only or final_file.exists():
        # Update progress bar description to show skipping status
        if final_file.exists() and not print_only and stage_pbar:
            stage_pbar.set_description("    Splitting (skipped - output exists)")

        if debug:
            import time
            # Debug mode: show progress with delays
            if num_districts == 1:
                # Single district, no splitting needed
                time.sleep(0.1)
            else:
                # Use stage_pbar if it exists, otherwise don't show progress
                if stage_pbar:
                    for i in range(num_districts - 1):
                        time.sleep(0.05)  # Delay to see progress bar update
                        stage_pbar.update(1)
                else:
                    time.sleep(2)  # Just wait without progress
        else:
            # Non-debug mode: instant completion
            if stage_pbar:
                stage_pbar.update(num_districts - 1)

        # Close progress bar if it was created
        if stage_pbar:
            stage_pbar.close()

        return output_dir  # Return early, don't check files or run METIS

    if not Path(graph_file).exists():
        print(f"ERROR: Adjacency graph not found: {graph_file}")
        print(f"Please run: python scripts/build_tract_adjacency.py --state {state_code}")
        sys.exit(1)

    if not Path(tracts_file).exists():
        print(f"ERROR: Tracts file not found: {tracts_file}")
        print(f"Please download tracts first")
        sys.exit(1)

    # Get tqdm position from parameter
    tqdm_pos = position

    # Show loading progress bar
    load_pbar = None
    if position != 2 and position != 999:
        from tqdm import tqdm
        load_pbar = tqdm(total=1,
                        desc=f"{state_name} [{num_districts}D] Loading graph",
                        unit="file",
                        position=tqdm_pos,
                        leave=False,
                        ncols=120)

    # Load tract graph
    with open(graph_file, 'rb') as f:
        graph_data = pickle.load(f)

    adjacency = graph_data['adjacency']
    vertex_weights = graph_data['vertex_weights']
    index_to_geoid = graph_data['index_to_geoid']

    # Load edge weights only if using edge-weighted mode
    edge_weights = None
    if args.partition_mode == 'edge-weighted':
        edge_weights = graph_data.get('edge_weights', None)
        if edge_weights is None:
            print(f"WARNING: --partition-mode edge-weighted specified but no edge_weights in graph file!")
            print(f"         Run: python scripts/data/geography/build_tract_adjacency.py --state {state_code} --year {args.year} --compute-boundary-lengths")
            print(f"         Falling back to normal mode (edge cut minimization)")
            args.partition_mode = 'normal'
        else:
            print(f"[OK] Using edge-weighted mode (boundary length minimization)")
            print(f"     Loaded {len(edge_weights):,} edge weights from graph")
    else:
        # Normal mode: explicitly set edge_weights to None to ensure they're not used
        edge_weights = None
        if position == 2:  # Only print in standalone mode, not when called from pipeline
            print(f"[OK] Using normal mode (edge cut minimization)")

    total_pop = int(vertex_weights.sum())

    if load_pbar:
        load_pbar.update(1)
        load_pbar.close()

    # output_dir was already created earlier for skip check
    output_dir.mkdir(parents=True, exist_ok=True)

    partitioner = RecursiveBisection(
        adjacency=adjacency,
        vertex_weights=vertex_weights,
        num_districts=num_districts,
        save_intermediate=True,
        intermediate_dir=str(output_dir / 'intermediate'),
        state_code=state_code,
        tqdm_position=tqdm_pos,
        debug=debug,
        edge_weights=edge_weights
    )

    # Run the algorithm
    final_assignments = partitioner.partition()

    # Calculate overall statistics (no prints during progress)
    populations = [sum(vertex_weights[block_idx] for block_idx, assigned_dist in final_assignments.items() if assigned_dist == district_id)
                   for district_id in range(1, num_districts + 1)]
    ideal = total_pop / num_districts
    deviations = [(p - ideal) / ideal * 100 for p in populations]
    max_dev = max(abs(d) for d in deviations)

    # Save final assignments
    final_file = output_dir / 'final_assignments.pkl'
    with open(final_file, 'wb') as f:
        pickle.dump({i: int(final_assignments[i]) for i in range(len(final_assignments))}, f)

    # Show map generation progress
    if position != 2 and position != 999:
        from tqdm import tqdm
        map_pbar = tqdm(total=3,
                       desc=f"{state_name} [{num_districts}D] Creating map",
                       unit="step",
                       position=tqdm_pos,
                       leave=False,
                       ncols=120)
    else:
        map_pbar = None

    # Generate final map
    tracts = gpd.read_parquet(tracts_file)
    if map_pbar:
        map_pbar.update(1)  # Loaded tracts

    # Map tract indices to district IDs
    district_assignments = [final_assignments[i] for i in range(len(tracts))]
    tracts['district'] = district_assignments

    fig, ax = plt.subplots(1, 1, figsize=(16, 14))

    # Use colormap with enough distinct colors
    colors = cm.tab20.colors + cm.tab20b.colors + cm.tab20c.colors

    for district_id in range(1, num_districts + 1):
        district_data = tracts[tracts['district'] == district_id]
        if len(district_data) > 0:
            district_data.plot(
                ax=ax,
                color=colors[(district_id - 1) % len(colors)],
                edgecolor='white',
                linewidth=0.05,
                alpha=0.8
            )

    # Add district numbers
    for district_id in range(1, num_districts + 1):
        district_data = tracts[tracts['district'] == district_id]
        if len(district_data) > 0:
            try:
                centroid = district_data.geometry.union_all().representative_point()
                text = ax.text(centroid.x, centroid.y, str(district_id),
                        fontsize=8, fontweight='bold', ha='center', va='center',
                        color='white', zorder=10)
                text.set_path_effects([path_effects.Stroke(linewidth=2, foreground='black'),
                                      path_effects.Normal()])
            except:
                pass

    if map_pbar:
        map_pbar.update(1)  # Plotted districts

    ax.set_axis_off()
    ax.set_title(f'{state_name} Congressional Districts - {num_districts} Districts\nTract-Level Redistricting (2020 Census)',
                 fontsize=16, fontweight='bold', pad=20)

    # Add stats text box
    textstr = f'Total Population: {total_pop:,}\n'
    textstr += f'Districts: {num_districts}\n'
    textstr += f'Ideal per district: {ideal:,.0f}\n'
    textstr += f'Max Deviation: {max_dev:.2f}%'

    props = dict(boxstyle='round', facecolor='wheat', alpha=0.8)
    ax.text(0.02, 0.98, textstr, transform=ax.transAxes, fontsize=11,
            verticalalignment='top', bbox=props)

    plt.tight_layout()

    final_map = output_dir / f'{state_name.lower().replace(" ", "_")}_{num_districts}_districts.png'
    plt.savefig(final_map, dpi=dpi, bbox_inches='tight')
    plt.close()

    if map_pbar:
        map_pbar.update(1)  # Saved map
        map_pbar.close()

    # Close progress bar if it was created
    if stage_pbar:
        stage_pbar.close()

    return output_dir


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Run congressional redistricting for a state')
    parser.add_argument('--state', type=str, required=True,
                       help='Two-letter state code (e.g., CA, TX, FL)')
    parser.add_argument('--year', type=str, default='2020', choices=['2020', '2010', '2000'],
                       help='Census year (default: 2020)')
    parser.add_argument('--output-dir', type=str, default=None,
                       help='Output directory (default: auto-generated timestamped directory)')
    parser.add_argument('--dpi', type=int, default=150,
                       help='DPI for output maps (default: 150)')
    parser.add_argument('--position', type=int, default=2,
                       help='Progress bar position (for parallel mode)')
    parser.add_argument('--print-only', action='store_true',
                       help='Print what would be done without executing')
    parser.add_argument('--debug', action='store_true',
                       help='Enable debug mode with progress delays and file display')
    parser.add_argument('--reset', action='store_true',
                       help='Delete output directory before starting (fresh run, not incremental)')
    parser.add_argument('--partition-mode', type=str, default='normal', choices=['normal', 'edge-weighted'],
                       help='Partitioning mode: "normal" (edge cut minimization) or "edge-weighted" (boundary length minimization for better compactness)')

    args = parser.parse_args()

    # Select the correct STATE_CONFIG based on year
    if args.year == '2020':
        if STATE_CONFIG_2020 is None:
            print("ERROR: config_2020.py not found. Cannot process 2020 data.")
            sys.exit(1)
        STATE_CONFIG = STATE_CONFIG_2020
    elif args.year == '2010':
        if STATE_CONFIG_2010 is None:
            print("ERROR: config_2010.py not found. Cannot process 2010 data.")
            sys.exit(1)
        STATE_CONFIG = STATE_CONFIG_2010
    elif args.year == '2000':
        print("ERROR: 2000 census configuration not yet implemented.")
        sys.exit(1)
    else:
        print(f"ERROR: Unknown year {args.year}")
        sys.exit(1)

    run_state_redistricting(args.state, STATE_CONFIG, args.year, args.output_dir, args.print_only, args.debug, args.dpi, args.position, args.reset)
