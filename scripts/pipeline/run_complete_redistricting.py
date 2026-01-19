#!/usr/bin/env python3
"""
Master script to run the complete US redistricting pipeline.

This orchestrates the entire process:
1. Check for election data (for political analysis)
2. Check for demographic data (for demographic analysis)
3. Process all 50 states (sequential or parallel mode)
4. Generate rounds_hierarchy.csv files
5. Create US aggregate files
6. Create US national maps
7. Run political analysis on all states (analyze partisan lean)
8. Run demographic analysis on all states (analyze demographics)

Usage:
  # Sequential mode (one state at a time)
  python scripts/pipeline/run_complete_redistricting.py --workers 1

  # Parallel mode (4-8 states at once)
  python scripts/pipeline/run_complete_redistricting.py --workers 4

  # With options
  python scripts/pipeline/run_complete_redistricting.py --workers 4 --year 2020 --version v3 --dpi 150

  # Skip political or demographic analysis
  python scripts/pipeline/run_complete_redistricting.py --workers 4 --skip-political
  python scripts/pipeline/run_complete_redistricting.py --workers 4 --skip-demographic

Prerequisites for political analysis:
  1. Download election data: python scripts/data/elections/download_election_data.py --year 2020
  2. Process to parquet: python scripts/data/elections/process_election_data.py --year 2020

Prerequisites for demographic analysis:
  1. Download demographic data: python scripts/data/demographics/download_demographic_data_robust.py --year 2020
  2. Process to parquet: python scripts/data/demographics/process_demographic_data.py --year 2020
"""

import argparse
import subprocess
import sys
import os
import platform
from pathlib import Path
from datetime import datetime
import time
import threading
import multiprocessing
from concurrent.futures import ProcessPoolExecutor, as_completed
from tqdm import tqdm

# Ensure we're running from project root
script_dir = Path(__file__).parent
project_root = script_dir.parent.parent

# If data/ directory doesn't exist in current directory, change to project root
if not Path('data').exists() and (project_root / 'data').exists():
    os.chdir(project_root)
    print(f"Changed working directory to project root: {project_root}\n")

# Add parent directory to path for imports
sys.path.insert(0, str(project_root))
sys.path.insert(0, str(project_root / 'src'))

# Import run config module for config.json generation
from apportionment.config import (
    RunConfig, write_config,
    VersionConfig, write_version_config, read_version_config, update_version_config_with_year
)

# Import utility functions
from scripts.utils import (
    get_state_config, get_state_config_safe,
    get_tract_file, get_places_file, get_adjacency_file,
    get_election_data_file, get_demographic_data_file
)

# Import progress coordinator for hierarchical multi-year display
from scripts.utils.progress_coordinator import ProgressCoordinator

# Import configuration files
try:
    from scripts.config_2020 import STATE_CONFIG_2020
except ImportError:
    STATE_CONFIG_2020 = None

try:
    from scripts.config_2010 import STATE_CONFIG_2010
except ImportError:
    STATE_CONFIG_2010 = None

try:
    from scripts.config_2000 import STATE_CONFIG_2000
except ImportError:
    STATE_CONFIG_2000 = None


def check_prerequisites(state_code, year='2020'):
    """Check if state has necessary data files."""
    # Load data files (unified directory structure for all census years)
    tracts_file = get_tract_file(state_code, year)
    places_file = get_places_file(state_code, year)
    graph_file = get_adjacency_file(state_code, year)

    missing = []
    if not tracts_file.exists():
        missing.append('tracts')
    # Places is optional - only needed for demographic/metro analysis
    # if not places_file.exists():
    #     missing.append('places')
    if not graph_file.exists():
        missing.append('adjacency graph')

    return missing


def build_pipeline_command(base_args, year, states_only=False, skip_states=False):
    """
    Build command for running pipeline with specific year and mode.

    Args:
        base_args: Original parsed arguments
        year: Census year to process
        states_only: Skip post-processing (state processing only)
        skip_states: Skip state processing (post-processing only)

    Returns:
        List of command arguments
    """
    cmd = [sys.executable, str(Path(__file__))]
    cmd.extend(['--year', year])
    cmd.extend(['--version', base_args.version])
    cmd.extend(['--dpi', str(base_args.dpi)])
    cmd.extend(['--election-year', base_args.election_year])
    cmd.extend(['--partition-mode', base_args.partition_mode])
    cmd.extend(['--run-type', base_args.run_type])

    # Mode flags
    if states_only:
        cmd.append('--states-only')
    if skip_states:
        cmd.append('--skip-states')

    # Optional parameters
    if base_args.experiment_name:
        cmd.extend(['--experiment-name', base_args.experiment_name])
    # Pass worker count for both state processing and post-processing
    cmd.extend(['--workers', str(base_args.workers)])
    if base_args.run_analysis:
        cmd.append('--run-analysis')
    else:
        cmd.append('--skip-analysis')
    if base_args.skip_political:
        cmd.append('--skip-political')
    if base_args.skip_demographic:
        cmd.append('--skip-demographic')
    if base_args.reprocess and not skip_states:
        cmd.append('--reprocess')
    if base_args.print_only:
        cmd.append('--print-only')
    if base_args.debug:
        cmd.append('--debug')

    # Add state list if specified
    if base_args.states and not skip_states:
        cmd.append('--states')
        cmd.extend(base_args.states)

    return cmd


def allocate_workers_across_years(total_workers, num_years=3):
    """
    Distribute workers across census years for parallel execution.

    Prioritizes 2020 (newest, highest priority) > 2010 > 2000 for extra workers.

    Args:
        total_workers: Total number of workers available
        num_years: Number of census years (default: 3)

    Returns:
        List of worker counts per year [2020, 2010, 2000]

    Examples:
        allocate_workers_across_years(4) -> [2, 1, 1]  (2020 gets extra)
        allocate_workers_across_years(5) -> [2, 2, 1]  (2020 and 2010 get extra)
        allocate_workers_across_years(6) -> [2, 2, 2]  (all equal)
        allocate_workers_across_years(7) -> [3, 2, 2]  (2020 gets extra)
        allocate_workers_across_years(8) -> [3, 3, 2]  (2020 and 2010 get extra)
    """
    if total_workers < num_years:
        # Minimum 1 worker per year
        return [1] * num_years

    base = total_workers // num_years
    remainder = total_workers % num_years

    # Distribute base to all years
    workers = [base] * num_years

    # Distribute remainder from FIRST to LAST (prioritize 2020 over 2000)
    for i in range(remainder):
        workers[i] += 1  # Start from beginning: workers[0], workers[1], workers[2]

    return workers


def create_argument_parser():
    """Create and configure argument parser for the pipeline."""
    parser = argparse.ArgumentParser(description='Run complete US redistricting pipeline')
    parser.add_argument('--output-dir', type=str, help='Output directory (overrides year and version)')
    parser.add_argument('-y', '--year', type=str, default='all', choices=['2020', '2010', '2000', 'all'],
                        help='Census year: 2020, 2010, 2000, or "all" to run all three in parallel (default: all)')
    parser.add_argument('-v', '--version', type=str, default='v1', help='Version identifier (default: v1)')
    parser.add_argument('-w', '--workers', type=int, default=12,
                        help='Number of parallel workers: 1=sequential, 2-24=parallel (default: 12)')
    parser.add_argument('--dpi', type=int, default=150, choices=[72, 100, 150, 200, 300],
                        help='DPI for output maps (default: 150). Higher = better quality but slower.')
    parser.add_argument('-ey', '--election-year', type=str, default='2020', choices=['2020', '2016'],
                        help='Election year for political analysis (default: 2020)')
    parser.add_argument('--run-analysis', action='store_true', default=True,
                        help='Run per-state analysis (political, demographic, compactness) during state processing (default: True)')
    parser.add_argument('--skip-analysis', dest='run_analysis', action='store_false',
                        help='Skip per-state analysis (use old batch post-processing instead)')
    parser.add_argument('--skip-political', action='store_true',
                        help='Skip political analysis steps')
    parser.add_argument('--skip-demographic', action='store_true',
                        help='Skip demographic analysis steps')
    parser.add_argument('-s', '--stages', type=str, nargs='+', default=['data', 'states', 'nation'],
                        choices=['data', 'states', 'nation'],
                        help='Which pipeline stages to run (default: all three). Examples: -s data (data only), -s states nation (skip data), -s nation (post-processing only)')
    parser.add_argument('--reprocess', action='store_true',
                        help='Reprocess all states (do not skip already processed states)')
    parser.add_argument('-r', '--reset', action='store_true',
                        help='Delete output directory before starting (fresh run, not incremental)')
    parser.add_argument('-p', '--print-only', action='store_true',
                        help='Print commands without executing (debug mode)')
    parser.add_argument('-d', '--debug', action='store_true',
                        help='Enable debug mode with progress delays')
    parser.add_argument('-pm', '--partition-mode', type=str, default='edge-weighted', choices=['unweighted', 'edge-weighted'],
                        help='Partitioning mode: "edge-weighted" (boundary length minimization, default) or "unweighted" (edge cut minimization for comparison)')
    parser.add_argument('--minimum-boundary-length', type=float, default=10.0,
                        help='Minimum shared boundary length (meters) to filter tiny adjacencies. Eliminates unrealistic corner touches. (default: 10, range: 0-100)')
    parser.add_argument('-rt', '--run-type', type=str, default='production', choices=['production', 'experiment', 'test'],
                        help='Run type: "production" (outputs/v{version}/{year}/), "experiment" (outputs/experiments/{experiment_name}/), or "test" (outputs/dev/) (default: production)')
    parser.add_argument('--experiment-name', type=str,
                        help='Experiment name (required when --run-type=experiment)')
    parser.add_argument('-st', '--states', nargs='*', default=None,
                        help='Specific state codes to process (default: all states)')
    parser.add_argument('--states-only', action='store_true',
                        help='Internal: Process states only, skip post-processing')
    parser.add_argument('--skip-states', action='store_true',
                        help='Internal: Skip state processing, run post-processing only')
    return parser


def main():
    # Parse arguments
    parser = create_argument_parser()
    args = parser.parse_args()

    # Validate arguments
    if args.run_type == 'experiment' and not args.experiment_name:
        parser.error("--experiment-name is required when --run-type=experiment")

    # Handle internal flags for hierarchical execution
    if args.states_only:
        # States only: skip nation processing
        args.stages = ['data', 'states']
    elif args.skip_states:
        # Post-processing only: skip states
        args.stages = ['nation']

    # Get the scripts directory
    scripts_dir = Path(__file__).parent

    # Build year queue based on arguments
    if args.year == 'all':
        year_queue = ['2020', '2010', '2000']  # Priority order: newest to oldest
    else:
        year_queue = [args.year]

    # Always use hierarchical parallel approach (whether 1 year or 3 years)
    multi_year_mode = True  # Now always true - single vs multi-year use same code path

    # Determine version directory (same for all years in queue)
    if args.run_type == 'production':
        version_dir = Path(f'outputs/{args.version}')
    elif args.run_type == 'experiment':
        version_dir = Path(f'outputs/experiments/{args.experiment_name}')
    else:  # test
        version_dir = Path('outputs/dev')

    # Create or load version config (once for all years)
    version_config_path = version_dir / 'version.json'
    if 'states' in args.stages and not args.print_only:
        if version_config_path.exists() and not args.reset:
            print(f"\n[OK] Loading existing version config: {version_config_path}")
            version_config = read_version_config(version_config_path)
        else:
            print(f"\n[OK] Creating new version config: {version_config_path}")
            version_config = VersionConfig.create(
                version=args.version,
                partition_mode=args.partition_mode.replace('-', '_'),
                data_level='tract',
                minimum_boundary_length=args.minimum_boundary_length,
                description=f"Redistricting variant: {args.version}",
                skip_political=args.skip_political,
                skip_demographic=args.skip_demographic,
                dpi=args.dpi
            )
            write_version_config(version_config, version_dir)

    # =========================================================================
    # PHASE 0: CENSUS DATA PROCESSING - Now handled by hierarchical parallel mode below
    # =========================================================================
    # Old sequential census data processing removed - now uses hierarchical coordinator
    # with proper progress display and STATUS protocol (see lines 1022+)

    # Always use hierarchical parallel execution
    if True:  # Always run this path now
        print("\n" + "="*70)
        if len(year_queue) > 1:
            print(f"PARALLEL MULTI-YEAR MODE: Running {', '.join(year_queue)} concurrently")
        else:
            print(f"HIERARCHICAL PIPELINE MODE: {year_queue[0]} Census")
        print("="*70)

        # Allocate workers across years
        workers_per_year = allocate_workers_across_years(args.workers, num_years=len(year_queue))
        print(f"\nExecution Model:")
        if len(year_queue) > 1:
            print(f"  - {len(year_queue)} years run in parallel ({', '.join(year_queue)})")
            workers_str = '/'.join(str(w) for w in workers_per_year)
            print(f"  - Each year runs {workers_str} states in parallel")
            print(f"  - Total: {sum(workers_per_year)} concurrent state processes")
            print(f"  - Estimated time: 2-4 hours (vs 7-13 hours sequential)")
        else:
            print(f"  - Single year: {year_queue[0]}")
            print(f"  - Parallel workers: {workers_per_year[0]}")
            print(f"  - Estimated time: ~1 hour")
        print("="*70)

        # Create hierarchical progress coordinator
        coordinator = ProgressCoordinator(
            years=year_queue,
            workers_per_year=workers_per_year
        )

        # Track display lines for in-place updates
        num_display_lines = [0]

        # Shared lock for display updates
        display_lock = threading.Lock()
        last_display_time = [time.time()]  # Mutable container for thread sharing

        def clear_and_update_display(coordinator):
            """Clear previous display and show updated progress."""
            # Move cursor up to overwrite previous display
            if num_display_lines[0] > 0:
                # ANSI escape: move cursor up N lines
                print(f"\033[{num_display_lines[0]}A", end='', flush=True)

            # Render the display
            display_text = coordinator.render()
            lines = display_text.split('\n')

            # Print each line with clear-to-end-of-line code
            for line in lines:
                # \r moves to start of line, \033[K clears to end of line
                print(f"\r\033[K{line}")

            # Track number of lines for next clear
            num_display_lines[0] = len(lines)
            sys.stdout.flush()

        if args.print_only:
            print("\n[PRINT-ONLY MODE] - Demonstrating progress display with mock data")
            print("\nInitial Progress Display:")
            coordinator.print_status()

            # Simulate some progress updates to demonstrate the display
            print("\n\nSimulated Progress Updates (demonstrating the display):")
            print("="*70)

            # Update 1: Some initial progress (only for years in year_queue)
            if '2020' in year_queue:
                coordinator.update_year_progress('2020', 5, 50)
                coordinator.update_worker_status('2020', 0, 3, 'california', 2, 7, 'district_maps')
                if workers_per_year[year_queue.index('2020')] > 1:
                    coordinator.update_worker_status('2020', 1, 2, 'texas', 4, 7, 'round_maps')

            if '2010' in year_queue:
                coordinator.update_year_progress('2010', 8, 50)
                coordinator.update_worker_status('2010', 0, 4, 'florida', 5, 7, 'political_analysis')
                if workers_per_year[year_queue.index('2010')] > 1:
                    coordinator.update_worker_status('2010', 1, 4, 'new_york', 3, 7, 'summary')

            if '2000' in year_queue:
                coordinator.update_year_progress('2000', 3, 50)
                coordinator.update_worker_status('2000', 0, 2, 'pennsylvania', 1, 7, 'redistricting')
                if workers_per_year[year_queue.index('2000')] > 1:
                    coordinator.update_worker_status('2000', 1, 1, 'illinois', 6, 7, 'demographic_analysis')

            print("\nProgress Update 1:")
            coordinator.print_status()

            # Update 2: More progress
            if '2020' in year_queue:
                coordinator.update_year_progress('2020', 15, 50)
                coordinator.update_worker_status('2020', 0, 8, 'ohio', 7, 7, 'demographic_analysis')
                if workers_per_year[year_queue.index('2020')] > 1:
                    coordinator.update_worker_status('2020', 1, 7, 'georgia', 1, 7, 'redistricting')

            if '2010' in year_queue:
                coordinator.update_year_progress('2010', 20, 50)
                coordinator.update_worker_status('2010', 0, 10, 'michigan', 3, 7, 'summary')
                if workers_per_year[year_queue.index('2010')] > 1:
                    coordinator.update_worker_status('2010', 1, 10, 'north_carolina', 5, 7, 'political_analysis')

            if '2000' in year_queue:
                coordinator.update_year_progress('2000', 12, 50)
                coordinator.update_worker_status('2000', 0, 6, 'virginia', 4, 7, 'round_maps')
                if workers_per_year[year_queue.index('2000')] > 1:
                    coordinator.update_worker_status('2000', 1, 6, 'massachusetts', 2, 7, 'district_maps')

            print("\nProgress Update 2:")
            coordinator.print_status()

            # Update 3: Near completion
            if '2020' in year_queue:
                coordinator.update_year_progress('2020', 45, 50)
                coordinator.update_worker_status('2020', 0, 23, 'vermont', 6, 7, 'demographic_analysis')
                if workers_per_year[year_queue.index('2020')] > 1:
                    coordinator.update_worker_status('2020', 1, 22, 'wyoming', 7, 7, 'complete')

            if '2010' in year_queue:
                coordinator.update_year_progress('2010', 48, 50)
                coordinator.update_worker_status('2010', 0, 24, 'delaware', 5, 7, 'political_analysis')
                if workers_per_year[year_queue.index('2010')] > 1:
                    coordinator.update_worker_status('2010', 1, 24, 'rhode_island', 6, 7, 'demographic_analysis')

            if '2000' in year_queue:
                coordinator.update_year_progress('2000', 40, 50)
                coordinator.update_worker_status('2000', 0, 20, 'montana', 3, 7, 'summary')
                if workers_per_year[year_queue.index('2000')] > 1:
                    coordinator.update_worker_status('2000', 1, 20, 'south_dakota', 4, 7, 'round_maps')

            print("\nProgress Update 3 (near completion):")
            coordinator.print_status()

            print("\n" + "="*70)
            print("[PRINT-ONLY MODE] Progress display demonstration complete")
            print("="*70)

            # Don't actually run anything in print-only mode
            return 0
        else:
            # Real execution mode - print initial display
            print("\n")
            display_text = coordinator.render()
            lines = display_text.split('\n')
            for line in lines:
                print(line)
            num_display_lines[0] = len(lines)
            sys.stdout.flush()

        # Start timestamp
        start_time = time.time()

        # Import progress coordinator parser
        from scripts.utils.progress_coordinator import parse_status_message

        # Run all 3 years in parallel using Popen for real-time monitoring
        results = {}
        processes = {}

        # Track phase for each year
        year_phase = {}  # Will be set to: 'data', 'states', 'ready_for_nation', 'nation', 'failed'

        # Determine output directories for each year
        year_output_dirs = {}
        for year in year_queue:
            if args.run_type == 'production':
                year_output_dirs[year] = version_dir / year
            elif args.run_type == 'experiment':
                year_output_dirs[year] = version_dir / year
            else:  # test
                year_output_dirs[year] = version_dir / year

        # =====================================================================
        # PHASE 0: CENSUS DATA PROCESSING (if needed)
        # =====================================================================
        # Check for per-stage census data markers (.tract_tracts_complete, etc.)
        # Import helper to check stage completion
        sys.path.insert(0, str(scripts_dir.parent / 'data'))
        from process_census_data import check_all_stages_complete

        # Default stages for redistricting
        required_stages = ['tracts', 'adjacency']  # Core stages needed for redistricting
        resolution = 'tract'  # Currently only tract-level supported

        year_census_complete = {}
        census_processes = {}

        for year in year_queue:
            # Check if ALL required stages are complete (per-stage markers)
            census_complete = check_all_stages_complete(
                year_output_dirs[year],
                required_stages,
                resolution,
                args.reset  # If reset, treat as not complete
            )
            year_census_complete[year] = census_complete

            # Determine initial phase based on --stages and census data completion
            if 'data' not in args.stages:
                # Skip census data processing
                if 'states' in args.stages:
                    year_phase[year] = 'ready_for_states'
                else:
                    # Skip directly to nation
                    year_phase[year] = 'ready_for_nation'
            elif not census_complete:
                # Need to process census data for this year
                year_phase[year] = 'data'
            elif 'states' in args.stages:
                # Census data already complete, ready for states
                year_phase[year] = 'ready_for_states'
            else:
                # Skip states, go directly to nation
                year_phase[year] = 'ready_for_nation'

        # Launch census data processing for years that need it
        any_census_needed = any(phase == 'data' for phase in year_phase.values())

        if any_census_needed:
            # Launch census data processing processes (hierarchical display will show progress)
            for i, year in enumerate(year_queue):
                if year_phase[year] == 'data':
                    # Build command for process_census_data.py
                    census_script = scripts_dir.parent / 'data' / 'process_census_data.py'

                    cmd_census = [
                        sys.executable,
                        str(census_script),
                        '--year', year,
                        '--output-dir', str(year_output_dirs[year]),
                        '--stages', 'tracts', 'adjacency', 'elections', 'demographics',
                        '--minimum-boundary-length', str(args.minimum_boundary_length),
                        '--position', '999'  # Signal deeply nested child - suppress verbose output
                    ]

                    # Add compute-boundary-lengths flag for edge-weighted mode
                    if args.partition_mode == 'edge-weighted':
                        cmd_census.append('--compute-boundary-lengths')

                    if args.print_only:
                        cmd_census.append('--dry-run')

                    # Set environment for child processes spawned by process_census_data.py
                    env = os.environ.copy()
                    env['TQDM_POSITION'] = '999'  # For deeply nested children (parse scripts, etc.)

                    # Launch process
                    proc = subprocess.Popen(
                        cmd_census,
                        stdout=subprocess.PIPE,
                        stderr=sys.stderr,
                        text=True,
                        bufsize=1,
                        env=env
                    )
                    census_processes[year] = proc

            # Monitor census data processing with hierarchical coordinator
            # Track which years are processing census data
            census_active_years = {year for year, proc in census_processes.items() if proc is not None}

            # Helper function to monitor census processes
            def monitor_census_process(year, proc):
                """Monitor a census data process and update coordinator."""
                try:
                    # Read stdout until process exits
                    while True:
                        line = proc.stdout.readline()
                        if not line:
                            if proc.poll() is not None:
                                break
                            continue

                        line = line.strip()
                        if not line:
                            continue

                        msg_type, data = parse_status_message(line)
                        if msg_type == 'CENSUS_STAGE':
                            with display_lock:
                                coordinator.update_census_stage(
                                    data['year'],
                                    data['stage_name'],
                                    data['completed'],
                                    data['total']
                                )
                                # Note: Main loop handles display updates (no flicker)
                        elif msg_type == 'CENSUS_WORKER':
                            with display_lock:
                                coordinator.update_census_worker(
                                    data['year'],
                                    data['worker_id'],
                                    data['state_num'],
                                    data['state_name'],
                                    data['stage_name']
                                )
                                # Note: Main loop handles display updates (no flicker)

                    # Process has exited
                    if proc.returncode is None:
                        proc.wait()

                    # Mark census as complete for this year
                    with display_lock:
                        coordinator.census_complete(year)
                        if proc.returncode == 0:
                            year_phase[year] = 'ready_for_states'
                        else:
                            year_phase[year] = 'failed'
                        clear_and_update_display(coordinator)

                except Exception as e:
                    sys.stderr.write(f"[ERROR] Census monitor thread for {year} died: {e}\n")
                    sys.stderr.flush()

            # Start monitoring threads for census processes
            census_threads = {}
            for year, proc in census_processes.items():
                thread = threading.Thread(target=monitor_census_process, args=(year, proc), daemon=True)
                thread.start()
                census_threads[year] = thread

            # Display progress while census processes run
            while census_active_years:
                with display_lock:
                    clear_and_update_display(coordinator)
                time.sleep(0.5)

                # Check which years are still active
                still_active = set()
                for year in census_active_years:
                    if census_processes[year].poll() is None:
                        still_active.add(year)
                census_active_years = still_active

            # Wait for all census threads to complete
            for year, thread in census_threads.items():
                thread.join(timeout=5)

            # Final display update
            print()  # Blank line after display (display already updated by loop)

            # Check if any failed
            failed_census = [year for year in census_processes.keys() if year_phase.get(year) == 'failed']
            if failed_census:
                print(f"\n[ERROR] Census data processing failed for: {', '.join(failed_census)}")
                print("Cannot proceed with state processing.")
                return 1

            # Census data processing complete (hierarchical display showed progress)

        # Update year_phase for years that already had complete census data
        for year in year_queue:
            if year_phase[year] == 'ready_for_states':
                year_phase[year] = 'states'  # Ready to start state processing

        # Check for .states_complete marker file or 'states' not in --stages
        year_states_complete = {}
        for year in year_queue:
            # Only check states if not already failed during census processing
            if year_phase.get(year) == 'failed':
                year_states_complete[year] = False
                continue

            # Skip states if not in requested stages OR if marker file exists
            if 'states' not in args.stages:
                states_complete = True
                # Don't print in multi-year mode - would interfere with hierarchical display
                year_phase[year] = 'ready_for_nation'  # Skip to nation processing
            else:
                marker_file = year_output_dirs[year] / '.states_complete'
                states_complete = marker_file.exists() and not args.reset
                if states_complete:
                    # Don't print in multi-year mode - would interfere with hierarchical display
                    year_phase[year] = 'ready_for_nation'  # Skip to nation processing
                elif year_phase.get(year) != 'failed':
                    # Only set to 'states' if not failed
                    year_phase[year] = 'states'  # Need to run state processing
            year_states_complete[year] = states_complete

        # Start year processes (skip if .states_complete marker exists or census failed)
        for i, year in enumerate(year_queue):
            # Skip if census processing failed for this year
            if year_phase.get(year) == 'failed':
                processes[year] = None
                results[year] = {'success': False, 'error': 'Census data processing failed'}
                continue

            if year_states_complete[year]:
                # Skip state processing - go straight to national
                processes[year] = None  # Will launch nation process immediately
                coordinator.update_year_progress(year, 50, 50)  # Show as complete
            else:
                # Launch state processing
                cmd_states = build_pipeline_command(args, year, states_only=True, skip_states=False)
                cmd_states.extend(['--workers', str(workers_per_year[i])])

                env = os.environ.copy()
                env['MULTI_YEAR_SUBPROCESS'] = '1'

                # Use Popen to monitor stdout in real-time
                proc = subprocess.Popen(
                    cmd_states,
                    stdout=subprocess.PIPE,
                    stderr=sys.stderr,  # Don't capture stderr - let it flow through
                    text=True,
                    bufsize=1,  # Line buffering
                    env=env
                )
                processes[year] = proc

        # Monitor all processes in real-time
        import select

        # (display_lock, last_display_time, and clear_and_update_display already defined above)

        def monitor_process(year, proc, coordinator):
            """Monitor a single year process and update coordinator."""
            try:
                # Read stdout until process exits
                while True:
                    line = proc.stdout.readline()
                    if not line:  # EOF or empty
                        # Check if process has exited
                        if proc.poll() is not None:
                            break
                        continue

                    line = line.strip()
                    if not line:
                        continue

                    msg_type, data = parse_status_message(line)
                    if msg_type == 'YEAR':
                        with display_lock:
                            coordinator.update_year_progress(
                                data['year'],
                                data['completed'],
                                data['total']
                            )
                            # Note: Main loop handles display updates (no flicker)
                    elif msg_type == 'YEAR_POSTPROCESS':
                        with display_lock:
                            coordinator.update_year_postprocess(
                                data['year'],
                                data['completed'],
                                data['total']
                            )
                            # Note: Main loop handles display updates (no flicker)
                    elif msg_type == 'WORKER':
                        with display_lock:
                            coordinator.update_worker_status(
                                data['year'],
                                data['worker_id'],
                                data['state_num'],
                                data['state_name'],
                                data['stage'],
                                data['stage_total'],
                                data['stage_desc']
                            )
                            # Note: Main loop handles display updates (no flicker)
                    elif msg_type == 'WORKER_TASK':
                        with display_lock:
                            coordinator.update_worker_task(
                                data['year'],
                                data['worker_id'],
                                data['task_index'],
                                data['task_total'],
                                data['task_name']
                            )
                            # Note: Main loop handles display updates (no flicker)
                    elif msg_type == 'CENSUS_STAGE':
                        with display_lock:
                            coordinator.update_census_stage(
                                data['year'],
                                data['stage_name'],
                                data['completed'],
                                data['total']
                            )
                            # Note: Main loop handles display updates (no flicker)
                    elif msg_type == 'CENSUS_WORKER':
                        with display_lock:
                            coordinator.update_census_worker(
                                data['year'],
                                data['worker_id'],
                                data['state_num'],
                                data['state_name'],
                                data['stage_name']
                            )
                            # Note: Main loop handles display updates (no flicker)

                # Process has exited (detected by poll() in loop above)
                # Ensure we have the return code
                if proc.returncode is None:
                    proc.wait()

                # Mark year as complete based on phase
                with display_lock:
                    phase = year_phase.get(year, 'states')
                    if phase == 'nation':
                        # Post-processing complete - get task count from coordinator
                        progress = coordinator.year_progress.get(year, {})
                        total_tasks = progress.get('total', 7)
                        coordinator.update_year_postprocess(year, total_tasks, total_tasks)
                    else:
                        # State processing complete
                        coordinator.update_year_progress(year, 50, 50)

                    # Final display update
                    clear_and_update_display(coordinator)

            except Exception as e:
                sys.stderr.write(f"[ERROR] Monitor thread for {year} died: {e}\n")
                sys.stderr.flush()

        # Start monitoring threads (only for years that are running states)
        threads = {}  # Changed to dict to track per-year
        for year, proc in processes.items():
            if proc is not None:  # Skip years with marker file
                thread = threading.Thread(target=monitor_process, args=(year, proc, coordinator), daemon=True)
                thread.start()
                threads[year] = thread

        # Helper function to launch national post-processing
        def launch_nation_processing(year):
            """Launch process_nation.py for a given year."""
            output_dir = year_output_dirs[year]

            cmd_nation = [
                sys.executable,
                str(scripts_dir / 'process_nation.py'),
                '--year', year,
                '--version', args.version,
                '--output-dir', str(output_dir),
                '--election-year', args.election_year,
                '--dpi', str(args.dpi),
                '--workers', str(workers_per_year[year_queue.index(year)])
            ]

            # Add optional flags
            if args.run_analysis:
                cmd_nation.append('--run-analysis')
            else:
                cmd_nation.append('--skip-analysis')

            if args.skip_political:
                cmd_nation.append('--skip-political')

            if args.skip_demographic:
                cmd_nation.append('--skip-demographic')

            if args.print_only:
                cmd_nation.append('--print-only')

            if args.debug:
                cmd_nation.append('--debug')

            # Start nation process
            env = os.environ.copy()
            env['MULTI_YEAR_SUBPROCESS'] = '1'

            proc_nation = subprocess.Popen(
                cmd_nation,
                stdout=subprocess.PIPE,
                stderr=sys.stderr,  # Don't capture stderr - let it flow through
                text=True,
                bufsize=1,
                env=env
            )
            processes[year] = proc_nation
            year_phase[year] = 'nation'

            # Start new monitoring thread for nation process
            thread = threading.Thread(target=monitor_process, args=(year, proc_nation, coordinator), daemon=True)
            thread.start()
            threads[year] = thread

        # Immediately launch nation processing for years with .states_complete marker
        for year in year_queue:
            if year_phase[year] == 'ready_for_nation':
                launch_nation_processing(year)

        # Wait for states to complete, then launch national processing
        # This allows each year to start post-processing as soon as its states finish
        while any(phase == 'states' for phase in year_phase.values()):
            # Update display
            with display_lock:
                clear_and_update_display(coordinator)

            # Check for completed processes
            for year in year_queue:
                if year_phase[year] == 'states':
                    proc = processes[year]
                    if proc.poll() is not None:  # Process finished
                        # Wait for monitoring thread to finish reading output
                        threads[year].join(timeout=2)

                        success = (proc.returncode == 0)
                        if success:
                            # Create .states_complete marker file
                            marker_file = year_output_dirs[year] / '.states_complete'
                            marker_file.parent.mkdir(parents=True, exist_ok=True)
                            marker_file.write_text(f"States processing completed: {datetime.now().isoformat()}\n")

                            # Launch national post-processing immediately
                            launch_nation_processing(year)
                        else:
                            # State processing failed
                            results[year] = {'success': False, 'error': f"State processing exit code {proc.returncode}"}
                            year_phase[year] = 'failed'

            time.sleep(0.5)  # Throttle display updates

        # Now wait for all national post-processing to complete
        while any(year_phase.get(year) == 'nation' for year in year_queue):
            # Update display
            with display_lock:
                clear_and_update_display(coordinator)

            # Check for completed processes
            for year in year_queue:
                if year_phase[year] == 'nation':
                    proc = processes[year]
                    if proc.poll() is not None:  # Process finished
                        # Wait for monitoring thread to finish reading output
                        if year in threads:
                            threads[year].join(timeout=2)

                        # Get return code
                        success = (proc.returncode == 0)
                        results[year] = {'success': success, 'error': None if success else f"Post-processing exit code {proc.returncode}"}
                        year_phase[year] = 'complete' if success else 'failed'
                elif year_phase[year] == 'failed':
                    pass  # Already recorded failure

            time.sleep(0.5)  # Throttle display updates

        # Calculate elapsed time
        elapsed = time.time() - start_time
        elapsed_mins = elapsed / 60
        elapsed_hours = elapsed / 3600

        # Update final progress display (one last time, in-place)
        with display_lock:
            for year in year_queue:
                result = results.get(year, {'success': False})
                if result['success']:
                    coordinator.update_year_progress(year, 50, 50)
            clear_and_update_display(coordinator)

        # Move past the hierarchical display for subsequent output
        print("\n")

        # Generate master dashboard (cross-year comparison) if any year succeeded
        any_success = any(results.get(year, {}).get('success', False) for year in year_queue)
        if any_success and not args.print_only:
            print("\n" + "="*70)
            print("GENERATING MASTER DASHBOARD")
            print("="*70)
            master_dash_script = Path('scripts/web/generate_master_dashboard.py')
            if master_dash_script.exists():
                cmd_master = [
                    sys.executable,
                    str(master_dash_script),
                    '--output', str(version_dir / 'index.html')
                ]
                result = subprocess.run(cmd_master, capture_output=True, text=True)
                if result.returncode == 0:
                    print(f"[OK] Master dashboard: {version_dir / 'index.html'}")
                else:
                    print(f"[WARN] Master dashboard generation failed")
                    if result.stderr:
                        print(f"  {result.stderr[:200]}")

        # Add spacing before summary (move past the hierarchical display)
        print("\n" * 2)

        # Print summary
        print("="*70)
        print("PARALLEL MULTI-YEAR PIPELINE COMPLETE")
        print("="*70)
        print(f"Total Time: {elapsed_hours:.2f} hours ({elapsed_mins:.1f} minutes)")
        print(f"\nResults:")

        success_count = 0
        for year in year_queue:
            result = results.get(year, {'success': False, 'error': 'No result'})
            status = "[OK]" if result['success'] else "[FAIL]"
            print(f"  {year}: {status}")
            if not result['success'] and result['error']:
                print(f"         Error: {result['error']}")
            if result['success']:
                success_count += 1
                # Update version config for successful years
                if not args.print_only:
                    update_version_config_with_year(version_dir, int(year))

        print("="*70)

        # Exit with appropriate code
        if success_count == len(year_queue):
            print("\n[SUCCESS] All census years completed successfully")
            return 0
        elif success_count > 0:
            print(f"\n[PARTIAL] {success_count}/{len(year_queue)} census years completed")
            return 1
        else:
            print("\n[FAILURE] All census years failed")
            return 1

    # NOTE: Old single-year sequential/parallel code paths removed
    # Now always use hierarchical parallel approach above (unified for 1 or N years)

if __name__ == '__main__':
    # Required for multiprocessing on Windows
    multiprocessing.freeze_support()
    sys.exit(main())
