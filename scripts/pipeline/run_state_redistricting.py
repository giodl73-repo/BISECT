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

# Import utility functions
from scripts.utils import get_state_config, get_tract_file, get_unit_file, get_adjacency_file

from apportionment.partition.recursive_bisection import RecursiveBisection
import geopandas as gpd
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import matplotlib.patheffects as path_effects
import matplotlib.cm as cm

def run_state_redistricting(state_code: str, state_config: dict, year: str = '2020', version: str = 'v1',
                           output_dir: str = None, print_only: bool = False, debug: bool = False, dpi: int = 150, position: int = 2, reset: bool = False,
                           ufactor: int = 5, niter: int = 100, objtype: str = 'cut', seed: int = None, partition_mode: str = 'edge-weighted', target_mm_districts: int = None, tree_structure: str = None, resolution: str = 'tract'):
    """Run redistricting for a specific state with version-specific data."""

    state_code = state_code.upper()
    if state_code not in state_config:
        print(f"ERROR: Unknown state code '{state_code}'")
        print(f"Valid codes: {', '.join(sorted(state_config.keys()))}")
        sys.exit(1)

    config = state_config[state_code]
    state_name = config['name']
    num_districts = config['districts']

    # File paths (version-specific directory structure)
    graph_file = str(get_adjacency_file(state_code, year, version, resolution))
    units_file = str(get_unit_file(state_code, year, version, resolution))
    # Backward compatibility alias
    tracts_file = units_file

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
        print(f"[RESET] Deleting existing output directory: {output_dir}")
        shutil.rmtree(output_dir)
        print(f"[RESET] Deleted. Starting fresh run.")

    # Create data directory if needed
    data_dir = output_dir / 'data'
    data_dir.mkdir(parents=True, exist_ok=True)
    final_file = data_dir / 'final_assignments.pkl'

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

    # Load edge weights by default (edge-weighted is default mode)
    edge_weights = None
    if partition_mode == 'edge-weighted':
        edge_weights = graph_data.get('edge_weights', None)
        if edge_weights is None:
            print(f"WARNING: edge_weights not found in graph file, falling back to unweighted mode")
            print(f"         To use edge-weighted mode, run:")
            print(f"         python scripts/data/compute_tract_adjacencies_{year}.py --states {state_code}")
            print(f"         Falling back to unweighted mode (edge cut minimization)")
            partition_mode = 'unweighted'
        else:
            print(f"[OK] Using edge-weighted mode (boundary length minimization, default)")
            print(f"     Loaded {len(edge_weights):,} edge weights from graph")
    else:
        # Unweighted mode: explicitly set edge_weights to None to ensure they're not used
        edge_weights = None
        if position == 2:  # Only print in standalone mode, not when called from pipeline
            print(f"[OK] Using unweighted mode (edge cut minimization)")

    total_pop = int(vertex_weights.sum())

    if load_pbar:
        load_pbar.update(1)
        load_pbar.close()

    # output_dir was already created earlier for skip check
    output_dir.mkdir(parents=True, exist_ok=True)

    # VRA Mode: Load demographics and create multi-constraint weights
    vra_mode = False
    vra_target_weights = None

    if partition_mode == 'metis-vra' and num_districts == 1:
        print(f"[VRA] Skipping VRA mode for single-district state (no split needed)")
        partition_mode = 'edge-weighted'
        edge_weights = graph_data.get('edge_weights', None)

    if partition_mode == 'metis-vra':
        from apportionment.partition import vra_utils
        from apportionment.partition import vra_targets

        vra_mode = True
        print(f"[VRA] Enabling VRA-aware partitioning mode")

        # Load tract geometries for merging with demographics
        tracts_gdf = gpd.read_parquet(tracts_file)

        # Load demographics
        try:
            demographics = vra_utils.load_tract_demographics(state_name.lower().replace(' ', '_'), year=int(year))
            print(f"[VRA] Loaded demographics for {len(demographics)} tracts")
        except FileNotFoundError as e:
            print(f"ERROR: Demographics not found for {state_name}: {e}")
            print(f"       VRA mode requires demographic data. Falling back to standard mode.")
            vra_mode = False

        if vra_mode:
            # Create multi-constraint vertex weights (population + minority VAP)
            vertex_weights_vra, tracts_with_demo = vra_utils.create_vra_vertex_weights(tracts_gdf, demographics)

            total_minority = vertex_weights_vra[:, 1].sum()
            state_minority_pct = (total_minority / total_pop)

            print(f"[VRA] State minority population: {state_minority_pct*100:.1f}%")

            # Get target MM districts (auto-detect or use specified)
            if target_mm_districts is None:
                target_mm_districts = vra_utils.get_vra_target_mm_districts(state_name.lower().replace(' ', '_'))
                if target_mm_districts is None:
                    # Default: Create MM districts proportional to minority population
                    target_mm_districts = max(1, int(num_districts * state_minority_pct))
                print(f"[VRA] Auto-detected target: {target_mm_districts} majority-minority districts")
            else:
                print(f"[VRA] User-specified target: {target_mm_districts} majority-minority districts")

            # Parse tree structure if provided
            first_split = None
            if tree_structure is not None:
                try:
                    left, right = map(int, tree_structure.split(','))
                    if left + right != num_districts:
                        print(f"[WARNING] Tree structure {tree_structure} doesn't sum to {num_districts}, using default")
                    else:
                        first_split = (left, right)
                        print(f"[VRA] Using custom tree structure: {num_districts} -> [{left}, {right}]")
                except ValueError:
                    print(f"[WARNING] Invalid tree structure format: {tree_structure}, using default")

            # Create VRA target tree (bottom-up target calculation)
            vra_target_tree = vra_targets.create_vra_target_tree(
                num_districts=num_districts,
                target_mm_districts=target_mm_districts,
                state_minority_pct=state_minority_pct,
                mm_target_pct=0.60,  # 60% minority in MM districts
                first_split=first_split
            )

            print(f"[VRA] Created VRA target tree for recursive bisection")
            print(f"[VRA] Target: {target_mm_districts} districts at 60%+ minority")

            # Show root split targets
            root_targets = vra_target_tree['target_weights']
            left_dists = vra_target_tree['left_count']
            right_dists = vra_target_tree['right_count']
            print(f"[VRA] Root split ({num_districts} -> {left_dists}|{right_dists}): " +
                  f"Left=[{root_targets[0][0]:.3f} pop, {root_targets[0][1]:.3f} min], " +
                  f"Right=[{root_targets[1][0]:.3f} pop, {root_targets[1][1]:.3f} min]")

            # Replace vertex_weights with multi-constraint version
            vertex_weights = vertex_weights_vra

    partitioner = RecursiveBisection(
        adjacency=adjacency,
        vertex_weights=vertex_weights,
        num_districts=num_districts,
        save_intermediate=True,
        intermediate_dir=str(output_dir / 'intermediate'),
        state_code=state_code,
        tqdm_position=tqdm_pos,
        debug=debug,
        edge_weights=edge_weights,
        ufactor=ufactor,
        niter=niter,
        objtype=objtype,
        seed=seed,
        vra_mode=vra_mode,
        vra_target_weights=vra_target_weights
    )

    # Run the algorithm
    final_assignments = partitioner.partition()

    # Calculate overall statistics (no prints during progress)
    # Handle VRA mode (2D vertex_weights) vs standard mode (1D vertex_weights)
    if vra_mode:
        # VRA mode: vertex_weights is 2D, use first column for population
        populations = [sum(vertex_weights[block_idx, 0] for block_idx, assigned_dist in final_assignments.items() if assigned_dist == district_id)
                       for district_id in range(1, num_districts + 1)]
    else:
        # Standard mode: vertex_weights is 1D
        populations = [sum(vertex_weights[block_idx] for block_idx, assigned_dist in final_assignments.items() if assigned_dist == district_id)
                       for district_id in range(1, num_districts + 1)]
    ideal = total_pop / num_districts
    deviations = [(p - ideal) / ideal * 100 for p in populations]
    max_dev = max(abs(d) for d in deviations)

    # Analyze VRA compliance if in VRA mode
    if vra_mode:
        from apportionment.partition import vra_utils
        # Convert assignments dict to array
        assignments_array = np.array([final_assignments[i] for i in range(len(tracts_with_demo))])
        vra_analysis = vra_utils.analyze_mm_districts(tracts_with_demo, assignments_array)

        mm_count = vra_analysis['mm_count']
        mm_target = target_mm_districts if target_mm_districts else 0

        print(f"\n[VRA] Majority-Minority Districts Created: {mm_count} (target: {mm_target})")
        print(f"[VRA] District Demographics:")
        for dist_info in vra_analysis['districts']:
            mm_marker = "  [MM]" if dist_info['is_mm'] else "      "
            print(f"{mm_marker} District {dist_info['district']}: {dist_info['pct_minority']*100:.1f}% minority "
                  f"({dist_info['pct_black']*100:.1f}% Black, {dist_info['pct_hispanic']*100:.1f}% Hispanic)")

        if mm_count >= mm_target:
            print(f"[VRA] [OK] Target met: {mm_count} >= {mm_target} majority-minority districts")
        else:
            print(f"[VRA] [FAIL] Target not met: {mm_count} < {mm_target} majority-minority districts")

        # Save VRA analysis
        vra_file = data_dir / 'vra_analysis.pkl'
        with open(vra_file, 'wb') as f:
            pickle.dump(vra_analysis, f)
        print(f"[VRA] Saved VRA analysis to: {vra_file}")

    # Save final assignments to data/ subdirectory
    final_file = data_dir / 'final_assignments.pkl'
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

    # Stats text box removed for cleaner visualization
    # Information available in CSV data files

    plt.tight_layout()

    # Create maps directory if it doesn't exist
    maps_dir = output_dir / 'maps'
    maps_dir.mkdir(parents=True, exist_ok=True)

    final_map = maps_dir / 'all_districts.png'
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
    parser.add_argument('--resolution', type=str, default='tract', choices=['tract', 'block_group', 'block'],
                       help='Geographic resolution: tract (default), block_group, or block')
    parser.add_argument('--version', type=str, default='v1',
                       help='Version identifier (default: v1)')
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
    parser.add_argument('--partition-mode', type=str, default='edge-weighted', choices=['unweighted', 'edge-weighted', 'metis-vra'],
                       help='Partitioning mode: "edge-weighted" (default), "unweighted" (edge cut), or "metis-vra" (VRA-aware with minority constraints)')
    parser.add_argument('--target-mm-districts', type=int, default=None,
                       help='Target number of majority-minority districts (VRA mode only, default: auto-detect from state)')
    parser.add_argument('--tree-structure', type=str, default=None,
                       help='Binary tree first split for VRA mode (e.g., "6,1" or "3,4"). Default: balanced split')
    parser.add_argument('--ufactor', type=int, default=5,
                       help='METIS imbalance tolerance factor (default: 5 = 0.5%%)')
    parser.add_argument('--niter', type=int, default=100,
                       help='METIS refinement iterations (default: 100)')
    parser.add_argument('--objtype', type=str, choices=['cut', 'vol'], default='cut',
                       help='METIS objective function: cut (edge-cut) or vol (volume)')
    parser.add_argument('--seed', type=int, default=None,
                       help='METIS random seed for reproducibility (default: None = random)')

    args = parser.parse_args()

    # Load state configuration for the specified year
    try:
        STATE_CONFIG = get_state_config(args.year)
    except (ValueError, ImportError) as e:
        print(f"ERROR: Could not load config for year {args.year}: {e}")
        sys.exit(1)

    run_state_redistricting(args.state, STATE_CONFIG, args.year, args.version, args.output_dir, args.print_only, args.debug, args.dpi, args.position, args.reset,
                            args.ufactor, args.niter, args.objtype, args.seed, args.partition_mode, args.target_mm_districts, args.tree_structure, args.resolution)
