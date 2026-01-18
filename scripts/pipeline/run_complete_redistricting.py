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


def process_state_sequential(state_code, us_dir, state_config, year='2020', skip_existing=True,
                             print_only=False, debug=False, position=1, dpi=150, run_analysis=True,
                             partition_mode='edge-weighted'):
    """
    Process a single state through the full pipeline (sequential mode).
    Uses ONLY 1 progress bar at the specified position.
    """
    config = state_config[state_code]
    state_name = config['name']
    num_districts = config['districts']

    # Use states subdirectory for organization
    states_dir = us_dir / 'states'
    state_dir = states_dir / state_name.lower().replace(' ', '_')

    # Check if already processed
    if not print_only:
        states_dir.mkdir(parents=True, exist_ok=True)

        # NOTE: Removed blanket skip check - now using per-stage skip logic
        # Each stage (redistricting, cities, summary, round maps, district maps, analysis)
        # checks its own outputs and skips if complete. This allows incremental updates
        # (e.g., adding new analysis stages without reprocessing everything)

        # Check prerequisites (only for multi-district states)
        if num_districts > 1:
            missing = check_prerequisites(state_code, year)
            if missing:
                print(f"ERROR: Missing data for {state_name}: {', '.join(missing)}")
                return False

    # Single-district states: simplified processing
    if num_districts == 1:
        with tqdm(total=1,
                  desc=f"{state_name} [1D] At-Large",
                  unit="step",
                  ncols=120,
                  position=position,
                  leave=False) as pbar:
            if debug:
                time.sleep(0.3)
            pbar.update(1)
        return True

    # Multi-district states: run full pipeline with ONE progress bar
    scripts_dir = Path(__file__).parent

    # Define all commands
    # Common flags for all scripts
    common_flags = []
    if print_only:
        common_flags.append('--print-only')
    if debug:
        common_flags.append('--debug')
    common_flags.append(f'--dpi {dpi}')
    common_flags_str = ' '.join(common_flags)

    # Redistricting-specific flags
    redistricting_flags = common_flags.copy()
    if partition_mode != 'edge-weighted':
        redistricting_flags.append(f'--partition-mode {partition_mode}')
    redistricting_flags_str = ' '.join(redistricting_flags)

    # Pipeline steps - pass position 999 to hide child progress bars
    steps = [
        ("Redistricting", f'{sys.executable} {scripts_dir}/run_state_redistricting.py --state {state_code} --year {year} --output-dir {state_dir} --position 999 {redistricting_flags_str}'.strip(), 1800),
        ("Cities", f'{sys.executable} {scripts_dir}/add_cities_to_districts.py {state_dir} --state {state_code} --year {year} --position 999 {common_flags_str}'.strip(), 600),
        ("Summary", f'{sys.executable} {scripts_dir}/create_final_district_summary.py {state_dir} --state {state_code} --year {year} --position 999 {common_flags_str}'.strip(), 300),
        ("Round maps", f'{sys.executable} {scripts_dir}/visualize_all_rounds.py {state_dir} --state {state_code} --year {year} --position 999 {common_flags_str}'.strip(), 600),
        ("District maps", f'{sys.executable} {scripts_dir}/visualize_individual_districts.py {state_dir} --state {state_code} --year {year} --position 999 {common_flags_str}'.strip(), 1800)
    ]

    # ONE progress bar for entire state pipeline
    with tqdm(total=len(steps),
              desc=f"{state_name} [{num_districts}D] Starting...",
              unit="step",
              ncols=120,
              position=position,
              leave=False) as pbar:

        for i, (step_label, cmd, timeout_val) in enumerate(steps, 1):
            # Update description with current step
            pbar.set_description(f"{state_name} [{num_districts}D] {i}/{len(steps)}: {step_label}")

            # Run command (capture output to hide child progress)
            result = subprocess.run(cmd, shell=True, timeout=timeout_val, capture_output=True, text=True)

            if result.returncode != 0:
                pbar.set_description(f"{state_name} [{num_districts}D] FAILED at {step_label}")
                print(f"\nERROR in {state_name} - {step_label}:")
                print(result.stderr[-500:] if result.stderr else "No error output")
                return False

            pbar.update(1)

        # Final description
        pbar.set_description(f"{state_name} [{num_districts}D] DONE")

    return True


def process_state_worker(args):
    """
    Worker function to process a single state (parallel mode).
    Runs in separate process with its own progress bar position.
    """
    state_code, us_dir, state_config, year, skip_existing, print_only, debug, position, dpi = args

    config = state_config[state_code]
    state_name = config['name']
    num_districts = config['districts']

    # Use states subdirectory for organization
    states_dir = us_dir / 'states'
    state_dir = states_dir / state_name.lower().replace(' ', '_')

    try:
        # Check if already processed
        if not print_only:
            states_dir.mkdir(parents=True, exist_ok=True)

            # NOTE: Removed blanket skip check - now using per-stage skip logic
            # Each stage checks its own outputs and skips if complete

            # Check prerequisites (only for multi-district states)
            if num_districts > 1:
                missing = check_prerequisites(state_code, year)
                if missing:
                    return (state_code, False, f"Missing: {', '.join(missing)}")

        # Single-district states: simplified processing
        if num_districts == 1:
            with tqdm(total=1,
                      desc=f"{state_name} [1D] At-Large",
                      unit="step",
                      ncols=120,
                      position=position,
                      leave=False) as pbar:
                if debug:
                    time.sleep(0.3)
                pbar.update(1)
            return (state_code, True, "SUCCESS")

        # Multi-district states: run through single wrapper script
        scripts_dir = Path(__file__).parent

        # Build command
        flags = []
        if print_only:
            flags.append('--print-only')
        if debug:
            flags.append('--debug')
        if run_analysis:
            flags.append('--run-analysis')
        flags_str = ' '.join(flags)

        cmd = f'{sys.executable} {scripts_dir}/process_single_state.py --state {state_code} --year {year} --output-dir {state_dir} --position {position} --dpi {os.environ.get("DPI", str(dpi))} {flags_str}'.strip()

        # Set up environment for child process
        env = os.environ.copy()
        env['TQDM_POSITION'] = str(position)
        env['PARALLEL_MODE'] = '1'
        env['DPI'] = str(dpi)

        # Run the wrapper script with explicit environment
        result = subprocess.run(cmd, shell=True, timeout=3600, env=env)

        if result.returncode != 0:
            return (state_code, False, f"Failed during processing")

        return (state_code, True, "SUCCESS")

    except Exception as e:
        return (state_code, False, f"Exception: {str(e)}")


def run_command(description, command, critical=True, print_only=False, pbar=None):
    """Run a command and handle errors."""
    def pwrite(msg):
        if pbar:
            pbar.write(msg)
        else:
            print(msg)

    result = subprocess.run(command, shell=True)

    if result.returncode != 0:
        pwrite(f"\n[ERROR] {description} failed with exit code {result.returncode}")
        if critical and not print_only:
            pwrite("Stopping pipeline due to critical error.")
            sys.exit(1)
        elif not critical:
            pwrite("Continuing despite error...")

    return result.returncode == 0


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
    if states_only:
        cmd.extend(['--workers', str(base_args.workers)])  # Only for state processing
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
        cmd.extend(base_args.states)

    return cmd


def allocate_workers_across_years(total_workers, num_years=3):
    """
    Distribute workers across census years for parallel execution.

    Args:
        total_workers: Total number of workers available
        num_years: Number of census years (default: 3)

    Returns:
        List of worker counts per year

    Examples:
        allocate_workers_across_years(6) -> [2, 2, 2]
        allocate_workers_across_years(8) -> [3, 3, 2]
        allocate_workers_across_years(9) -> [3, 3, 3]
    """
    if total_workers < num_years:
        # Minimum 1 worker per year
        return [1] * num_years

    base = total_workers // num_years
    remainder = total_workers % num_years

    # Distribute base + remainder to first years
    workers = [base] * num_years
    for i in range(remainder):
        workers[i] += 1

    return workers


def run_single_year_pipeline(year, workers_for_year, args):
    """
    Run the complete pipeline for a single census year.

    This function is designed to be called in parallel for multiple years.

    Args:
        year: Census year string ('2020', '2010', '2000')
        workers_for_year: Number of workers to allocate to this year
        args: Command-line arguments object

    Returns:
        Tuple of (year, success_bool, error_message)
    """
    try:
        print(f"\n[{year}] Starting with {workers_for_year} workers...")
        start_time = time.time()

        # Build commands for this year
        # Pass 1: State processing
        cmd_states = build_pipeline_command(args, year, states_only=True, skip_states=False)
        cmd_states.extend(['--workers', str(workers_for_year)])

        # Pass 2: Post-processing
        cmd_post = build_pipeline_command(args, year, states_only=False, skip_states=True)

        # Run state processing
        if not args.print_only:
            result = subprocess.run(cmd_states, capture_output=False)
            if result.returncode != 0:
                error_msg = f"State processing failed with code {result.returncode}"
                print(f"\n[{year}] FAILED: {error_msg}")
                return (year, False, error_msg)

            # Run post-processing
            result = subprocess.run(cmd_post, capture_output=False)
            if result.returncode != 0:
                error_msg = f"Post-processing failed with code {result.returncode}"
                print(f"\n[{year}] FAILED: {error_msg}")
                return (year, False, error_msg)
        else:
            print(f"\n[{year}] Would execute:")
            print(f"  States: {' '.join(cmd_states)}")
            print(f"  Post:   {' '.join(cmd_post)}")

        elapsed = time.time() - start_time
        elapsed_mins = elapsed / 60
        print(f"\n[{year}] COMPLETE in {elapsed_mins:.1f} minutes")
        return (year, True, None)

    except Exception as e:
        error_msg = str(e)
        print(f"\n[{year}] EXCEPTION: {error_msg}")
        return (year, False, error_msg)


def create_argument_parser():
    """Create and configure argument parser for the pipeline."""
    parser = argparse.ArgumentParser(description='Run complete US redistricting pipeline')
    parser.add_argument('--output-dir', type=str, help='Output directory (overrides year and version)')
    parser.add_argument('--year', type=str, default='2020', choices=['2020', '2010', '2000', 'all'],
                        help='Census year: 2020, 2010, 2000, or "all" to run all three in parallel (default: 2020)')
    parser.add_argument('--version', type=str, default='v1', help='Version identifier (default: v1)')
    parser.add_argument('--workers', type=int, default=4,
                        help='Number of parallel workers: 1=sequential, 2-8=parallel (default: 4)')
    parser.add_argument('--dpi', type=int, default=150, choices=[72, 100, 150, 200, 300],
                        help='DPI for output maps (default: 150). Higher = better quality but slower.')
    parser.add_argument('--election-year', type=str, default='2020', choices=['2020', '2016'],
                        help='Election year for political analysis (default: 2020)')
    parser.add_argument('--run-analysis', action='store_true', default=True,
                        help='Run per-state analysis (political, demographic, compactness) during state processing (default: True)')
    parser.add_argument('--skip-analysis', dest='run_analysis', action='store_false',
                        help='Skip per-state analysis (use old batch post-processing instead)')
    parser.add_argument('--skip-political', action='store_true',
                        help='Skip political analysis steps')
    parser.add_argument('--skip-demographic', action='store_true',
                        help='Skip demographic analysis steps')
    parser.add_argument('--skip-states', action='store_true',
                        help='Skip state processing (for post-processing only)')
    parser.add_argument('--states-only', action='store_true',
                        help='Process states only, skip post-processing (useful for multi-year runs)')
    parser.add_argument('--reprocess', action='store_true',
                        help='Reprocess all states (do not skip already processed states)')
    parser.add_argument('--reset', action='store_true',
                        help='Delete output directory before starting (fresh run, not incremental)')
    parser.add_argument('--print-only', action='store_true',
                        help='Print commands without executing (debug mode)')
    parser.add_argument('--debug', action='store_true',
                        help='Enable debug mode with progress delays')
    parser.add_argument('--partition-mode', type=str, default='edge-weighted', choices=['unweighted', 'edge-weighted'],
                        help='Partitioning mode: "edge-weighted" (boundary length minimization, default) or "unweighted" (edge cut minimization for comparison)')
    parser.add_argument('--run-type', type=str, default='production', choices=['production', 'experiment', 'test'],
                        help='Run type: "production" (outputs/v{version}/{year}/), "experiment" (outputs/experiments/{experiment_name}/), or "test" (outputs/dev/) (default: production)')
    parser.add_argument('--experiment-name', type=str,
                        help='Experiment name (required when --run-type=experiment)')
    parser.add_argument('states', nargs='*',
                        help='Specific state codes to process (default: all states)')
    return parser


def setup_output_directory(args):
    """
    Determine output directory and handle reset logic.

    Returns:
        Path: Output directory
    """
    # Determine output directory based on run type
    if args.output_dir:
        output_dir = Path(args.output_dir)
    else:
        if args.run_type == 'production':
            # Production: outputs/v{version}/{year}/
            output_dir = Path(f'outputs/{args.version}/{args.year}')
        elif args.run_type == 'experiment':
            # Experiment: outputs/experiments/{experiment_name}/{version}_{year}/
            output_dir = Path(f'outputs/experiments/{args.experiment_name}/{args.version}_{args.year}')
        else:  # test
            # Test/Dev: outputs/dev/{version}_{year}/
            output_dir = Path(f'outputs/dev/{args.version}_{args.year}')

    # Handle --reset flag: delete output directory for fresh run
    if args.reset and output_dir.exists() and not args.print_only:
        import shutil

        print(f"\n[RESET] Deleting existing output directory: {output_dir}")

        # Windows-compatible deletion with retry logic
        max_retries = 5
        for attempt in range(max_retries):
            try:
                # On Windows, use rmdir /s /q for more reliable deletion
                if platform.system() == 'Windows':
                    result = subprocess.run(
                        ['cmd', '/c', 'rmdir', '/s', '/q', str(output_dir)],
                        capture_output=True,
                        text=True
                    )
                    if result.returncode == 0:
                        break
                    elif attempt < max_retries - 1:
                        print(f"  Retry {attempt + 1}/{max_retries}... (waiting for file handles to close)")
                        time.sleep(2)
                    else:
                        # Fall back to shutil if cmd fails
                        shutil.rmtree(output_dir, ignore_errors=True)
                else:
                    shutil.rmtree(output_dir)
                    break
            except PermissionError as e:
                if attempt < max_retries - 1:
                    print(f"  Retry {attempt + 1}/{max_retries}... (waiting for file handles to close)")
                    time.sleep(2)
                else:
                    print(f"[WARNING] Could not fully delete directory: {e}")
                    print(f"[WARNING] Some files may still exist. Continuing anyway...")
            except Exception as e:
                print(f"[ERROR] Unexpected error during deletion: {e}")
                break

        # Verify deletion
        if not output_dir.exists():
            print(f"[RESET] Deleted successfully. Starting fresh run.\n")
        else:
            print(f"[RESET] Directory may not be fully deleted. Continuing...\n")

    return output_dir


def create_config_files(args, output_dir):
    """
    Create config.json and version.json files.

    Args:
        args: Parsed command-line arguments
        output_dir: Path to output directory
    """
    if args.print_only:
        return

    output_dir.mkdir(parents=True, exist_ok=True)

    # Generate run-level config.json
    config = RunConfig.create(
        version=args.version,
        census_year=int(args.year),
        election_year=int(args.election_year),
        partition_mode=args.partition_mode.replace('-', '_'),  # 'edge-weighted' -> 'edge_weighted'
        data_level='tract',  # Currently always tract-level
        run_type=args.run_type,
        scope='us' if not args.states else 'state',
        states=args.states if args.states else ['all'],
        experiment_name=args.experiment_name if args.run_type == 'experiment' else None,
        skip_political=args.skip_political,
        skip_demographic=args.skip_demographic,
        dpi=args.dpi
    )
    write_config(config, output_dir)
    print(f"[OK] Generated run config: {output_dir}/config.json")

    # Handle version-level config
    version_dir = output_dir.parent  # e.g., outputs/v1/ (parent of outputs/v1/2020/)
    version_config_path = version_dir / 'version.json'

    if version_config_path.exists():
        # Version config already exists - this is a subsequent year run
        print(f"[OK] Version config already exists: {version_config_path}")
    else:
        # Create new version config for this version
        print(f"[OK] Creating version config: {version_config_path}")
        version_config = VersionConfig.create(
            version=args.version,
            partition_mode=args.partition_mode.replace('-', '_'),
            data_level='tract',
            description=f"Redistricting variant: {args.version}",
            skip_political=args.skip_political,
            skip_demographic=args.skip_demographic,
            dpi=args.dpi
        )
        write_version_config(version_config, version_dir)


def main():
    # Parse arguments
    parser = create_argument_parser()
    args = parser.parse_args()

    # Validate arguments
    if args.run_type == 'experiment' and not args.experiment_name:
        parser.error("--experiment-name is required when --run-type=experiment")

    # Get the scripts directory
    scripts_dir = Path(__file__).parent

    # Build year queue based on arguments
    if args.year == 'all':
        year_queue = ['2000', '2010', '2020']
        multi_year_mode = True
        # Banner will be printed in parallel multi-year section
    else:
        year_queue = [args.year]
        multi_year_mode = False

    # Determine version directory (same for all years in queue)
    if args.run_type == 'production':
        version_dir = Path(f'outputs/{args.version}')
    elif args.run_type == 'experiment':
        version_dir = Path(f'outputs/experiments/{args.experiment_name}')
    else:  # test
        version_dir = Path('outputs/dev')

    # Create or load version config (once for all years)
    version_config_path = version_dir / 'version.json'
    if not args.skip_states and not args.print_only:
        if version_config_path.exists() and not args.reset:
            print(f"\n[OK] Loading existing version config: {version_config_path}")
            version_config = read_version_config(version_config_path)
        else:
            print(f"\n[OK] Creating new version config: {version_config_path}")
            version_config = VersionConfig.create(
                version=args.version,
                partition_mode=args.partition_mode.replace('-', '_'),
                data_level='tract',
                description=f"Redistricting variant: {args.version}",
                skip_political=args.skip_political,
                skip_demographic=args.skip_demographic,
                dpi=args.dpi
            )
            write_version_config(version_config, version_dir)

    # Handle multi-year mode with PARALLEL execution
    if multi_year_mode:
        print("\n" + "="*70)
        print("PARALLEL MULTI-YEAR MODE: Running 2020, 2010, 2000 concurrently")
        print("="*70)

        # Allocate workers across years
        workers_per_year = allocate_workers_across_years(args.workers, num_years=3)
        print(f"\nWorker Allocation:")
        print(f"  2020: {workers_per_year[0]} workers")
        print(f"  2010: {workers_per_year[1]} workers")
        print(f"  2000: {workers_per_year[2]} workers")
        print(f"  Total: {sum(workers_per_year)} workers")
        print("="*70)

        if args.print_only:
            print("\n[PRINT-ONLY MODE] - No execution")

        # Start timestamp
        start_time = time.time()

        # Run all 3 years in parallel using ProcessPoolExecutor
        results = {}
        with ProcessPoolExecutor(max_workers=3) as executor:
            # Submit all year processes
            future_to_year = {
                executor.submit(run_single_year_pipeline, year, workers_per_year[i], args): year
                for i, year in enumerate(year_queue)
            }

            # Collect results as they complete
            for future in as_completed(future_to_year):
                year = future_to_year[future]
                try:
                    year_result, success, error = future.result()
                    results[year_result] = {'success': success, 'error': error}
                except Exception as e:
                    print(f"\n[{year}] EXCEPTION during execution: {e}")
                    results[year] = {'success': False, 'error': str(e)}

        # Calculate elapsed time
        elapsed = time.time() - start_time
        elapsed_mins = elapsed / 60
        elapsed_hours = elapsed / 3600

        # Print summary
        print("\n" + "="*70)
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

    # Determine execution mode
    mode = 'parallel' if args.workers > 1 else 'sequential'

    # Load state configuration for the specified year
    try:
        STATE_CONFIG = get_state_config(args.year)
    except (ValueError, ImportError) as e:
        print(f"ERROR: Could not load config for year {args.year}: {e}")
        sys.exit(1)

    # Setup output directory (determine path and handle --reset)
    output_dir = setup_output_directory(args)

    # Create config files (config.json and version.json)
    create_config_files(args, output_dir)

    print("\n" + "="*70)
    print("US CONGRESSIONAL REDISTRICTING - COMPLETE PIPELINE")
    print("="*70)
    print(f"Census Year: {args.year}")
    print(f"Output directory: {output_dir}")
    print(f"Version: {args.version}")
    print(f"Execution Mode: {mode.upper()}")
    if mode == 'parallel':
        print(f"Parallel Workers: {min(args.workers, 8)} (max 8 recommended)")
    if args.print_only:
        print("Mode: PRINT-ONLY (debug mode - no execution)")
    print("="*70)
    sys.stdout.flush()

    # =========================================================================
    # STEP 0: CHECK ELECTION DATA (for political analysis)
    # =========================================================================
    if not args.skip_political and not args.skip_states:
        election_data_file = get_election_data_file(args.election_year)

        if not election_data_file.exists():
            print(f"\n[WARNING] Election data not found: {election_data_file}")
            print(f"Run these commands to prepare election data:")
            print(f"  python scripts/pipeline/download_election_data.py --year {args.election_year}")
            print(f"  python scripts/pipeline/process_election_data.py --year {args.election_year}")
            print(f"\nPolitical analysis will be skipped.")
            args.skip_political = True
            sys.stdout.flush()
        else:
            print(f"\nElection data found: {election_data_file}")
            sys.stdout.flush()

    # =========================================================================
    # STEP 0B: CHECK DEMOGRAPHIC DATA (for demographic analysis)
    # =========================================================================
    if not args.skip_demographic and not args.skip_states:
        demographic_data_file = get_demographic_data_file(args.year)

        if not demographic_data_file.exists():
            print(f"\n[WARNING] Demographic data not found: {demographic_data_file}")
            print(f"Run these commands to prepare demographic data:")
            print(f"  python scripts/data/demographics/download_demographic_data_robust.py --year {args.year}")
            print(f"  python scripts/data/demographics/process_demographic_data.py --year {args.year}")
            print(f"\nDemographic analysis will be skipped.")
            args.skip_demographic = True
            sys.stdout.flush()
        else:
            print(f"\nDemographic data found: {demographic_data_file}")
            sys.stdout.flush()

    # =========================================================================
    # STEP 1: PROCESS ALL 50 STATES
    # =========================================================================
    if not args.skip_states:
        # Get list of states to process
        if args.states:
            states_to_process = [s.upper() for s in args.states]
            invalid = [s for s in states_to_process if s not in STATE_CONFIG]
            if invalid:
                print(f"ERROR: Invalid state codes: {', '.join(invalid)}")
                sys.exit(1)
        else:
            # Process all states (sorted by number of districts, descending)
            states_to_process = sorted(
                STATE_CONFIG.keys(),
                key=lambda s: STATE_CONFIG[s]['districts'],
                reverse=True
            )

        print(f"\nProcessing {len(states_to_process)} states in {mode} mode...")
        print()
        sys.stdout.flush()

        # Track results
        successful = []
        failed = []
        skipped_states = []
        results = {}

        if mode == 'sequential':
            # SEQUENTIAL MODE: Process one state at a time
            # USA-level progress bar at position 0
            mode_label = " [Edge-Weighted]" if args.partition_mode == 'edge-weighted' else ""
            with tqdm(states_to_process,
                      desc=f"USA Redistricting{mode_label} - {args.year} Census",
                      unit="state",
                      ncols=120,
                      position=0,
                      leave=True,
                      dynamic_ncols=False,
                      file=sys.stderr) as usa_pbar:

                for state_code in usa_pbar:
                    config = STATE_CONFIG[state_code]
                    state_name = config['name']

                    # Update USA progress bar
                    usa_pbar.set_description(f"USA Redistricting - {args.year} Census: {state_name}")

                    # NOTE: Removed blanket skip check - now using per-stage skip logic
                    # process_single_state.py handles per-stage checks

                    # Process state at position 1 (below USA bar)
                    success = process_state_sequential(
                        state_code, output_dir, STATE_CONFIG,
                        year=args.year,
                        skip_existing=not args.reprocess,
                        print_only=args.print_only,
                        debug=args.debug,
                        position=1,
                        dpi=args.dpi,
                        run_analysis=args.run_analysis,
                        partition_mode=args.partition_mode
                    )

                    if success:
                        successful.append(state_code)
                        usa_pbar.set_postfix_str(f"✓ {len(successful)}/{len(states_to_process)}")
                    else:
                        failed.append(state_code)
                        usa_pbar.set_postfix_str(f"✗ {len(failed)} failed")

        else:
            # PARALLEL MODE: Process multiple states at once
            # Parent creates and manages all progress bars
            start_time = time.time()

            import select
            import threading
            from queue import Queue

            # Create tqdm bars for each state at their assigned positions
            state_bars = {}
            for i, state_code in enumerate(states_to_process):
                config = STATE_CONFIG[state_code]
                state_name = config['name']
                num_districts = config['districts']
                bar = tqdm(total=1,
                          desc=f"[{i+1}] {state_name} [{num_districts}D] Waiting...",
                          unit="step",
                          position=i + 1,
                          ncols=120,
                          leave=True,
                          bar_format="{desc}",
                          dynamic_ncols=False,
                          file=sys.stderr)
                state_bars[i + 1] = bar  # Position is i+1

            # USA-level progress bar at position 0
            mode_label = " [Edge-Weighted]" if args.partition_mode == 'edge-weighted' else ""
            usa_pbar = tqdm(total=len(states_to_process),
                           desc=f"USA Redistricting (Parallel){mode_label} - {args.year} Census",
                           unit="state",
                           ncols=120,
                           position=0,
                           leave=True,
                           dynamic_ncols=False,
                           file=sys.stderr)

            # Start subprocess for each state
            processes = {}
            for i, state_code in enumerate(states_to_process):
                config = STATE_CONFIG[state_code]
                state_name = config['name']
                num_districts = config['districts']
                position = i + 1

                states_dir = output_dir / 'states'
                state_dir = states_dir / state_name.lower().replace(' ', '_')

                # NOTE: Removed blanket skip check - now using per-stage skip logic
                # Each stage in process_single_state.py checks its own outputs

                # Build command
                scripts_dir = Path(__file__).parent
                flags = []
                if args.print_only:
                    flags.append('--print-only')
                if args.debug:
                    flags.append('--debug')
                if args.run_analysis:
                    flags.append('--run-analysis')
                if args.partition_mode != 'normal':
                    flags.append(f'--partition-mode {args.partition_mode}')
                flags_str = ' '.join(flags)

                cmd = f'{sys.executable} {scripts_dir}/process_single_state.py --state {state_code} --year {args.year} --output-dir {state_dir} --position {position} --dpi {args.dpi} {flags_str}'.strip()

                # Set up environment
                env = os.environ.copy()
                env['PARALLEL_MODE'] = '1'
                env['DPI'] = str(args.dpi)

                # Start process (limit to max_workers at a time)
                if len([p for p in processes.values() if p.poll() is None]) >= min(args.workers, 8):
                    # Wait for a slot to open up
                    while True:
                        for state_code2, proc in list(processes.items()):
                            if proc.poll() is not None:
                                break
                        if len([p for p in processes.values() if p.poll() is None]) < min(args.workers, 8):
                            break
                        time.sleep(0.1)

                proc = subprocess.Popen(cmd, shell=True, env=env,
                                       stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                       text=True, bufsize=1)
                processes[state_code] = proc

                # Start thread to monitor this process's output
                def monitor_output(proc, position, state_code):
                    try:
                        for line in proc.stdout:
                            line = line.strip()
                            if line.startswith("STATUS:"):
                                # Parse: STATUS:position:message
                                parts = line.split(":", 2)
                                if len(parts) >= 3:
                                    pos = int(parts[1])
                                    msg = parts[2]
                                    if pos in state_bars:
                                        state_bars[pos].set_description_str(f"[{pos}] {msg}".ljust(120))
                                        state_bars[pos].refresh()
                    except:
                        pass

                thread = threading.Thread(target=monitor_output, args=(proc, position, state_code), daemon=True)
                thread.start()

            # Wait for all processes to complete
            for state_code, proc in processes.items():
                proc.wait()
                config = STATE_CONFIG[state_code]
                state_name = config['name']

                if proc.returncode == 0:
                    successful.append(state_code)
                    results[state_code] = (True, "SUCCESS")
                else:
                    failed.append(state_code)
                    results[state_code] = (False, f"Failed with code {proc.returncode}")
                    print(f"\n[FAIL] {state_name}: exit code {proc.returncode}", file=sys.stderr)

                usa_pbar.update(1)
                usa_pbar.set_postfix_str(f"✓ {len(successful)}/{len(states_to_process)}")

            # Close all bars
            usa_pbar.close()
            for bar in state_bars.values():
                bar.close()

            # Clear visual space after closing many bars
            print("\n" + "="*70, file=sys.stderr)
            print("POST-PROCESSING", file=sys.stderr)
            print("="*70, file=sys.stderr)

        # Show brief summary
        if failed:
            print(f"\n[WARN] Failed states: {len(failed)}/{len(states_to_process)}", file=sys.stderr)
            for s in failed:
                state_name = STATE_CONFIG[s]['name']
                print(f"  [FAIL] {state_name} ({s})", file=sys.stderr)
            print(file=sys.stderr)

    # =========================================================================
    # STEP 2: POST-PROCESSING
    # =========================================================================

    # Skip post-processing if --states-only is set
    if args.states_only:
        print("\n[OK] States-only mode: Skipping post-processing steps")
        print(f"[OK] State processing complete. Run with --skip-states to do post-processing later.")
        return 0

    # Call process_nation.py to handle all national post-processing
    process_nation_script = scripts_dir / 'process_nation.py'

    # Build command with all arguments
    cmd = [
        sys.executable,
        str(process_nation_script),
        '--year', args.year,
        '--version', args.version,
        '--output-dir', str(output_dir),
        '--election-year', args.election_year,
        '--dpi', str(args.dpi)
    ]

    # Add optional flags
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

    # Run national post-processing
    result = subprocess.run(cmd)

    return 0 if (result.returncode == 0 and (not 'failed' in locals() or not failed)) else 1


if __name__ == '__main__':
    # Required for multiprocessing on Windows
    multiprocessing.freeze_support()
    sys.exit(main())
