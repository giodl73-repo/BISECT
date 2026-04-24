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
import colorama

# Initialize colorama for ANSI escape code support on Windows
colorama.init()

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

# Import pipeline orchestrator for unified worker management
from scripts.utils.pipeline_orchestrator import PipelineOrchestrator

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


def check_prerequisites(state_code, year='2020', version='v1'):
    """Check if state has necessary data files for a specific version."""
    # Load data files (version-specific structure)
    tracts_file = get_tract_file(state_code, year, version)
    places_file = get_places_file(state_code, year, version)
    graph_file = get_adjacency_file(state_code, year, version)

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
    parser.add_argument('-pm', '--partition-mode', type=str, default='edge-weighted', choices=['unweighted', 'edge-weighted', 'metis-vra'],
                        help='Partitioning mode: "edge-weighted" (boundary length minimization, default), "unweighted" (edge cut minimization), or "metis-vra" (VRA-aware multi-constraint)')
    parser.add_argument('--processing-mode', type=str, default='streaming', choices=['batch', 'streaming'],
                        help='Processing mode: "streaming" (per-state pipeline, default) or "batch" (all states per stage). Streaming lets each state flow through all stages independently via process_single_state.py.')
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
            # Append-only mode - no cursor movement
            # Just print a separator and current status
            print("=" * 70)
            display_text = coordinator.render()
            print(display_text)
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
        from scripts.data.process_census_data import check_all_stages_complete

        # Default stages for redistricting
        required_stages = ['tracts', 'merge', 'adjacency']  # Core stages needed for redistricting
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

        # =========================================================================
        # UNIFIED PIPELINE ORCHESTRATION (census → states → nation)
        # =========================================================================

        # Define stages to run based on args.stages
        pipeline_stages = []
        if 'data' in args.stages:
            pipeline_stages.append('census')
        if 'states' in args.stages:
            pipeline_stages.append('states')
        if 'nation' in args.stages:
            pipeline_stages.append('nation')

        # Command builders for each stage
        def build_census_command(year):
            """Build command for census data processing."""
            census_script = scripts_dir.parent / 'data' / 'process_census_data.py'
            year_idx = year_queue.index(year)
            worker_count = workers_per_year[year_idx]

            cmd = [
                sys.executable,
                str(census_script),
                '--year', year,
                '--version', args.version,
                '--output-dir', str(year_output_dirs[year]),
                '--stages', 'tracts', 'merge', 'adjacency', 'elections', 'demographics',
                '--workers', str(worker_count),
                '--minimum-boundary-length', str(args.minimum_boundary_length),
                '--processing-mode', args.processing_mode,
                '--position', '999'  # Signal deeply nested child - suppress verbose output
            ]
            if args.partition_mode == 'edge-weighted':
                cmd.append('--compute-boundary-lengths')
            if args.print_only:
                cmd.append('--dry-run')
            return cmd

        def build_states_command(year):
            """Build command for state redistricting - spawns worker manager."""
            # Instead of recursively calling orchestrator, call a dedicated worker manager
            # that will spawn individual state workers with ProcessPoolExecutor
            year_idx = year_queue.index(year)
            worker_count = workers_per_year[year_idx]

            # Delegate to run_states_parallel.py which manages parallel state workers
            cmd = [
                sys.executable,
                str(scripts_dir / 'run_states_parallel.py'),  # Dedicated worker manager
                '--year', year,
                '--version', args.version,
                '--output-dir', str(year_output_dirs[year]),
                '--workers', str(worker_count),
                '--dpi', str(args.dpi),
                '--partition-mode', args.partition_mode,
                '--processing-mode', args.processing_mode
            ]
            if args.reprocess:
                cmd.append('--reprocess')
            if args.print_only:
                cmd.append('--print-only')
            if args.debug:
                cmd.append('--debug')
            if args.states:
                cmd.append('--states')
                cmd.extend(args.states)
            return cmd

        def build_nation_command(year):
            """Build command for national post-processing."""
            year_idx = year_queue.index(year)
            cmd = [
                sys.executable,
                str(scripts_dir / 'process_nation.py'),
                '--year', year,
                '--version', args.version,
                '--output-dir', str(year_output_dirs[year]),
                '--election-year', args.election_year,
                '--dpi', str(args.dpi),
                '--workers', str(workers_per_year[year_idx])
            ]
            if args.run_analysis:
                cmd.append('--run-analysis')
            else:
                cmd.append('--skip-analysis')
            if args.skip_political:
                cmd.append('--skip-political')
            if args.skip_demographic:
                cmd.append('--skip-demographic')
            if args.print_only:
                cmd.append('--print-only')
            if args.debug:
                cmd.append('--debug')
            return cmd

        # Track stage completion counts for redistricting
        # Maps year -> {stage_name: count}
        stage_completion_counts = {year: {} for year in year_queue}

        # Message handlers for each stage
        census_handlers = {
            'CENSUS_STAGE': lambda data: coordinator.update_census_stage(
                data['year'],
                data['stage_name'],
                data['completed'],
                data['total']
            ),
            'CENSUS_STAGE_PROGRESS': lambda data: coordinator.update_census_stage(
                data['year'],
                coordinator.census_progress.get(data['year'], {}).get('stage', 'Processing'),
                data['completed'],
                data['total']
            ),
            'CENSUS_WORKER': lambda data: coordinator.update_census_worker(
                data['year'],
                data['worker_id'],
                data['state_num'],
                data['state_name'],
                data['stage_name']
            )
        }

        def handle_stage_complete(data):
            """Handle STAGE_COMPLETE message - increment count and update coordinator."""
            year = data['year']
            stage_name = data['stage_name']
            if year in stage_completion_counts:
                current_count = stage_completion_counts[year].get(stage_name, 0)
                stage_completion_counts[year][stage_name] = current_count + 1
                coordinator.update_stage_completion(year, stage_name, current_count + 1)

        state_handlers = {
            'YEAR': lambda data: coordinator.update_year_progress(
                data['year'],
                data['completed'],
                data['total']
            ),
            'WORKER': lambda data: coordinator.update_worker_status(
                data['year'],
                data['worker_id'],
                data['state_num'],
                data['state_name'],
                data['stage'],
                data['stage_total'],
                data['stage_desc']
            ),
            'WORKER_TASK': lambda data: coordinator.update_worker_task(
                data['year'],
                data['worker_id'],
                data['task_index'],
                data['task_total'],
                data['task_name']
            ),
            'STAGE_COMPLETE': handle_stage_complete,
            # State stage may also emit census messages if census runs inline
            **census_handlers
        }

        nation_handlers = {
            'YEAR_POSTPROCESS': lambda data: coordinator.update_year_postprocess(
                data['year'],
                data['completed'],
                data['total']
            )
        }

        # Create and configure pipeline orchestrator
        orchestrator = PipelineOrchestrator(
            coordinator=coordinator,
            display_lock=display_lock,
            years=year_queue,
            output_dirs=year_output_dirs,
            processing_mode=args.processing_mode
        )

        # Register stages
        orchestrator.add_stage('census', build_census_command, census_handlers)
        orchestrator.add_stage('states', build_states_command, state_handlers)
        orchestrator.add_stage('nation', build_nation_command, nation_handlers)

        # Run pipeline
        pipeline_results = orchestrator.run_pipeline(
            stages=pipeline_stages,
            skip_stages_if_complete=True,
            reset=args.reset,
            poll_interval=0.5,
            display_update_callback=lambda: clear_and_update_display(coordinator)
        )

        # Process results - flatten stage results into year results
        results = {}
        for year in year_queue:
            year_results = pipeline_results.get(year, {})
            # Check if all stages succeeded
            all_success = all(
                stage_result.get('success', False)
                for stage_result in year_results.values()
            )
            # Find first failure if any
            first_error = None
            for stage_name, stage_result in year_results.items():
                if not stage_result.get('success', False):
                    first_error = stage_result.get('error', f'{stage_name} failed')
                    break

            results[year] = {
                'success': all_success,
                'error': first_error
            }

        # Legacy compatibility: ensure year_phase is set correctly
        for year in year_queue:
            if results[year]['success']:
                year_phase[year] = 'complete'
            else:
                year_phase[year] = 'failed'

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
