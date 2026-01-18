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


def process_state_for_multi_year(args_tuple):
    """
    Process a single state in multi-year mode.
    Must be at module level for Windows multiprocessing pickling.

    Args:
        args_tuple: (state_code, state_number, year, output_dir, script_args_dict, worker_id, completed_states)

    Returns:
        (state_code, success_bool)
    """
    state_code, state_number, year, output_dir, args_dict, worker_id = args_tuple

    # Reconstruct paths and config
    from scripts.utils import get_state_config
    STATE_CONFIG = get_state_config(year)

    config = STATE_CONFIG[state_code]
    state_name = config['name']
    states_dir = Path(output_dir) / 'states'
    state_dir = states_dir / state_name.lower().replace(' ', '_')

    scripts_dir = Path(__file__).parent
    flags = []
    if args_dict.get('print_only'):
        flags.append('--print-only')
    if args_dict.get('debug'):
        flags.append('--debug')
    if args_dict.get('run_analysis'):
        flags.append('--run-analysis')
    if args_dict.get('partition_mode') != 'edge-weighted':
        flags.append(f"--partition-mode {args_dict['partition_mode']}")
    flags_str = ' '.join(flags)

    cmd = f'{sys.executable} {scripts_dir}/process_single_state.py --state {state_code} --year {year} --output-dir {state_dir} --dpi {args_dict["dpi"]} {flags_str}'.strip()

    env = os.environ.copy()
    env['DPI'] = str(args_dict['dpi'])
    env['STATE_NUMBER'] = str(state_number)
    env['WORKER_ID'] = str(worker_id)
    env['CENSUS_YEAR'] = year  # Pass year for YEAR completion messages

    # Use Popen to forward STATUS messages
    proc = subprocess.Popen(cmd, shell=True, env=env,
                           stdout=subprocess.PIPE,
                           stderr=subprocess.PIPE,
                           text=True, bufsize=1)

    # Forward stdout (STATUS messages) to our stdout
    for line in proc.stdout:
        print(line, end='', flush=True)

    proc.wait()

    return (state_code, proc.returncode == 0)


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
        start_time = time.time()

        # Build commands for this year
        # Pass 1: State processing
        cmd_states = build_pipeline_command(args, year, states_only=True, skip_states=False)
        cmd_states.extend(['--workers', str(workers_for_year)])

        # Pass 2: Post-processing (override worker count for this year's allocation)
        cmd_post = build_pipeline_command(args, year, states_only=False, skip_states=True)
        # Override the --workers argument with this year's allocation
        # Find and replace the --workers value
        for i, arg in enumerate(cmd_post):
            if arg == '--workers' and i + 1 < len(cmd_post):
                cmd_post[i + 1] = str(workers_for_year)
                break

        # Run state processing
        if not args.print_only:
            # Set environment variable to suppress child progress bars
            env = os.environ.copy()
            env['MULTI_YEAR_SUBPROCESS'] = '1'

            # Use Popen to forward STATUS messages
            cmd_states_str = ' '.join(cmd_states)
            proc = subprocess.Popen(cmd_states_str, shell=True, env=env,
                                   stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE,
                                   text=True, bufsize=1)

            # Forward STATUS messages to parent
            for line in proc.stdout:
                print(line, end='', flush=True)

            proc.wait()
            if proc.returncode != 0:
                error_msg = f"State processing failed with code {proc.returncode}"
                sys.stderr.write(f"[{year}] FAILED: {error_msg}\n")
                sys.stderr.flush()
                return (year, False, error_msg)

            # Run post-processing (also forward STATUS messages)
            cmd_post_str = ' '.join(cmd_post)
            proc = subprocess.Popen(cmd_post_str, shell=True, env=env,
                                   stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE,
                                   text=True, bufsize=1)

            # Forward STATUS messages to parent
            for line in proc.stdout:
                print(line, end='', flush=True)

            proc.wait()
            if proc.returncode != 0:
                error_msg = f"Post-processing failed with code {proc.returncode}"
                sys.stderr.write(f"[{year}] FAILED: {error_msg}\n")
                sys.stderr.flush()
                return (year, False, error_msg)

        elapsed = time.time() - start_time
        elapsed_mins = elapsed / 60
        # Success - don't print anything to keep display clean
        return (year, True, None)

    except Exception as e:
        error_msg = str(e)
        sys.stderr.write(f"[{year}] EXCEPTION: {error_msg}\n")
        sys.stderr.flush()
        return (year, False, error_msg)


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
    parser.add_argument('--stages', type=str, nargs='+', default=['census_data', 'states', 'nation'],
                        choices=['census_data', 'states', 'nation'],
                        help='Which pipeline stages to run (default: all three). Examples: --stages census_data (data only), --stages states nation (skip data processing), --stages nation (post-processing only)')
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
    parser.add_argument('-s', '--states', nargs='*', default=None,
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
        minimum_boundary_length=args.minimum_boundary_length,
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
            minimum_boundary_length=args.minimum_boundary_length,
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
        year_queue = ['2020', '2010', '2000']  # Priority order: newest to oldest
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

    # Handle multi-year mode with PARALLEL execution
    if multi_year_mode:
        print("\n" + "="*70)
        print("PARALLEL MULTI-YEAR MODE: Running 2020, 2010, 2000 concurrently")
        print("="*70)

        # Allocate workers across years
        workers_per_year = allocate_workers_across_years(args.workers, num_years=3)
        print(f"\nExecution Model:")
        print(f"  - 3 years run in parallel (2020, 2010, 2000)")
        print(f"  - Each year runs {workers_per_year[0]}/{workers_per_year[1]}/{workers_per_year[2]} states in parallel")
        print(f"  - Total: {sum(workers_per_year)} concurrent state processes")
        print(f"  - Estimated time: 2-4 hours (vs 7-13 hours sequential)")
        print("="*70)

        # Create hierarchical progress coordinator
        coordinator = ProgressCoordinator(
            years=['2020', '2010', '2000'],
            workers_per_year=workers_per_year
        )

        # Track display lines for in-place updates
        num_display_lines = [0]

        if args.print_only:
            print("\n[PRINT-ONLY MODE] - Demonstrating progress display with mock data")
            print("\nInitial Progress Display:")
            coordinator.print_status()

            # Simulate some progress updates to demonstrate the display
            print("\n\nSimulated Progress Updates (demonstrating the display):")
            print("="*70)

            # Update 1: Some initial progress
            coordinator.update_year_progress('2020', 5, 50)
            coordinator.update_worker_status('2020', 0, 3, 'california', 2, 7, 'district_maps')
            coordinator.update_worker_status('2020', 1, 2, 'texas', 4, 7, 'round_maps')

            coordinator.update_year_progress('2010', 8, 50)
            coordinator.update_worker_status('2010', 0, 4, 'florida', 5, 7, 'political_analysis')
            coordinator.update_worker_status('2010', 1, 4, 'new_york', 3, 7, 'summary')

            coordinator.update_year_progress('2000', 3, 50)
            coordinator.update_worker_status('2000', 0, 2, 'pennsylvania', 1, 7, 'redistricting')
            coordinator.update_worker_status('2000', 1, 1, 'illinois', 6, 7, 'demographic_analysis')

            print("\nProgress Update 1:")
            coordinator.print_status()

            # Update 2: More progress
            coordinator.update_year_progress('2020', 15, 50)
            coordinator.update_worker_status('2020', 0, 8, 'ohio', 7, 7, 'demographic_analysis')
            coordinator.update_worker_status('2020', 1, 7, 'georgia', 1, 7, 'redistricting')

            coordinator.update_year_progress('2010', 20, 50)
            coordinator.update_worker_status('2010', 0, 10, 'michigan', 3, 7, 'summary')
            coordinator.update_worker_status('2010', 1, 10, 'north_carolina', 5, 7, 'political_analysis')

            coordinator.update_year_progress('2000', 12, 50)
            coordinator.update_worker_status('2000', 0, 6, 'virginia', 4, 7, 'round_maps')
            coordinator.update_worker_status('2000', 1, 6, 'massachusetts', 2, 7, 'district_maps')

            print("\nProgress Update 2:")
            coordinator.print_status()

            # Update 3: Near completion
            coordinator.update_year_progress('2020', 45, 50)
            coordinator.update_worker_status('2020', 0, 23, 'vermont', 6, 7, 'demographic_analysis')
            coordinator.update_worker_status('2020', 1, 22, 'wyoming', 7, 7, 'complete')

            coordinator.update_year_progress('2010', 48, 50)
            coordinator.update_worker_status('2010', 0, 24, 'delaware', 5, 7, 'political_analysis')
            coordinator.update_worker_status('2010', 1, 24, 'rhode_island', 6, 7, 'demographic_analysis')

            coordinator.update_year_progress('2000', 40, 50)
            coordinator.update_worker_status('2000', 0, 20, 'montana', 3, 7, 'summary')
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
        year_phase = {}  # Will be set to: 'census_data', 'states', 'ready_for_nation', 'nation', 'failed'

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
            if 'census_data' not in args.stages:
                # Skip census data processing
                if 'states' in args.stages:
                    year_phase[year] = 'ready_for_states'
                else:
                    # Skip directly to nation
                    year_phase[year] = 'ready_for_nation'
            elif not census_complete:
                # Need to process census data for this year
                year_phase[year] = 'census_data'
            elif 'states' in args.stages:
                # Census data already complete, ready for states
                year_phase[year] = 'ready_for_states'
            else:
                # Skip states, go directly to nation
                year_phase[year] = 'ready_for_nation'

        # Launch census data processing for years that need it
        any_census_needed = any(phase == 'census_data' for phase in year_phase.values())

        if any_census_needed:
            print("\n")
            print("="*70)
            print("CENSUS DATA PROCESSING")
            print("="*70)

            for year in year_queue:
                if year_phase[year] == 'census_data':
                    print(f"  [{year}] Census data processing needed")
                else:
                    print(f"  [{year}] Census data already complete (all {resolution}-level stages done)")
            print("="*70)
            print()

            # Launch census data processing processes
            for i, year in enumerate(year_queue):
                if year_phase[year] == 'census_data':
                    # Build command for process_census_data.py
                    census_script = scripts_dir.parent / 'data' / 'process_census_data.py'

                    cmd_census = [
                        sys.executable,
                        str(census_script),
                        '--year', year,
                        '--output-dir', str(year_output_dirs[year]),
                        '--stages', 'tracts', 'adjacency', 'elections', 'demographics',
                        '--minimum-boundary-length', str(args.minimum_boundary_length)
                    ]

                    # Add compute-boundary-lengths flag for edge-weighted mode
                    if args.partition_mode == 'edge-weighted':
                        cmd_census.append('--compute-boundary-lengths')

                    if args.print_only:
                        cmd_census.append('--dry-run')

                    # Set environment for worker subprocess
                    env = os.environ.copy()
                    env['TQDM_POSITION'] = str(i)  # Assign position for STATUS protocol
                    env['CENSUS_YEAR'] = year

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

            # Monitor census data processing with simple output
            census_results = {}
            for year, proc in census_processes.items():
                # Read and forward stdout
                for line in proc.stdout:
                    print(f"[{year}] {line}", end='', flush=True)

                proc.wait()
                success = (proc.returncode == 0)
                census_results[year] = success

                if success:
                    # Per-stage markers already created by process_census_data.py
                    year_phase[year] = 'ready_for_states'
                    print(f"\n[{year}] [OK] Census data processing complete (per-stage markers created)\n")
                else:
                    year_phase[year] = 'failed'
                    print(f"\n[{year}] [FAIL] Census data processing failed\n")

            print("\n" + "="*70)
            print("CENSUS DATA PROCESSING COMPLETE")
            print("="*70)

            # Check if any failed
            failed_census = [year for year, success in census_results.items() if not success]
            if failed_census:
                print(f"\n[ERROR] Census data processing failed for: {', '.join(failed_census)}")
                print("Cannot proceed with state processing.")
                return 1

            print("\nProceeding to state processing...")
            print("="*70)
            print()

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
        import threading

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

            # Print each line, clearing to end of line
            for line in lines:
                print(f"\r\033[K{line}")

            # Track number of lines for next clear
            num_display_lines[0] = len(lines)
            sys.stdout.flush()

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
                            # Refresh display (throttled to 0.5s for better responsiveness)
                            now = time.time()
                            if now - last_display_time[0] >= 0.5:
                                clear_and_update_display(coordinator)
                                last_display_time[0] = now
                    elif msg_type == 'YEAR_POSTPROCESS':
                        with display_lock:
                            coordinator.update_year_postprocess(
                                data['year'],
                                data['completed'],
                                data['total']
                            )
                            # Refresh display (throttled)
                            now = time.time()
                            if now - last_display_time[0] >= 0.5:
                                clear_and_update_display(coordinator)
                                last_display_time[0] = now
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
                            # Refresh display (throttled)
                            now = time.time()
                            if now - last_display_time[0] >= 0.5:
                                clear_and_update_display(coordinator)
                                last_display_time[0] = now
                    elif msg_type == 'WORKER_TASK':
                        with display_lock:
                            coordinator.update_worker_task(
                                data['year'],
                                data['worker_id'],
                                data['task_index'],
                                data['task_total'],
                                data['task_name']
                            )
                            # Refresh display (throttled)
                            now = time.time()
                            if now - last_display_time[0] >= 0.5:
                                clear_and_update_display(coordinator)
                                last_display_time[0] = now

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

            time.sleep(0.1)  # Avoid busy-waiting

        # Now wait for all national post-processing to complete
        for year in year_queue:
            if year_phase[year] == 'nation':
                proc = processes[year]

                # IMPORTANT: Don't call proc.wait() here! The monitoring thread already called it.
                # On Windows, calling wait() twice on the same process causes the second call to hang.
                # Instead, just wait for the monitoring thread to finish.

                # Wait for monitoring thread to finish (it handles proc.wait())
                if year in threads:
                    threads[year].join(timeout=180)  # 3 minutes - enough for post-processing

                # Now get return code (process has been waited on by monitoring thread)
                success = (proc.returncode == 0)
                results[year] = {'success': success, 'error': None if success else f"Post-processing exit code {proc.returncode}"}
            elif year_phase[year] == 'failed':
                pass  # Already recorded failure

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
    if not args.skip_political and 'states' in args.stages:
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
    if not args.skip_demographic and 'states' in args.stages:
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
    if 'states' in args.stages:
        # Check if we're running as a subprocess of multi-year mode
        is_multi_year_subprocess = os.environ.get('MULTI_YEAR_SUBPROCESS') == '1'

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

        if not is_multi_year_subprocess:
            print(f"\nProcessing {len(states_to_process)} states in {mode} mode...")
            print()
            sys.stdout.flush()
        # else: subprocess runs silently to avoid interfering with parent progress display

        # Track results
        successful = []
        failed = []
        skipped_states = []
        results = {}

        if mode == 'sequential':
            if not is_multi_year_subprocess:
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
                # Running as subprocess - sequential without progress bars or output
                # (silent to avoid interfering with parent's hierarchical display)
                for i, state_code in enumerate(states_to_process, 1):
                    config = STATE_CONFIG[state_code]
                    state_name = config['name']

                    scripts_dir = Path(__file__).parent
                    flags = []
                    if args.print_only:
                        flags.append('--print-only')
                    if args.debug:
                        flags.append('--debug')
                    if args.run_analysis:
                        flags.append('--run-analysis')
                    if args.partition_mode != 'edge-weighted':
                        flags.append(f'--partition-mode {args.partition_mode}')
                    flags_str = ' '.join(flags)

                    states_dir = output_dir / 'states'
                    state_dir = states_dir / state_name.lower().replace(' ', '_')
                    cmd = f'{sys.executable} {scripts_dir}/process_single_state.py --state {state_code} --year {args.year} --output-dir {state_dir} --dpi {args.dpi} {flags_str}'.strip()

                    env = os.environ.copy()
                    env['DPI'] = str(args.dpi)

                    result = subprocess.run(cmd, shell=True, env=env, capture_output=True)
                    if result.returncode == 0:
                        successful.append(state_code)
                    else:
                        failed.append(state_code)
                        # Only print errors to stderr
                        sys.stderr.write(f"[{args.year}] Failed: {state_name}\n")
                        sys.stderr.flush()

        elif is_multi_year_subprocess:
            # Running as subprocess of multi-year mode
            # Run multiple states in parallel using ProcessPoolExecutor
            max_workers = min(args.workers, 8)

            # Prepare arguments as dict (picklable)
            args_dict = {
                'print_only': args.print_only,
                'debug': args.debug,
                'run_analysis': args.run_analysis,
                'partition_mode': args.partition_mode,
                'dpi': args.dpi
            }

            # Pre-assign states to workers in round-robin fashion
            # This ensures we can track which worker is processing which state
            state_args_list = [
                (state_code, i, args.year, str(output_dir), args_dict, i % max_workers)
                for i, state_code in enumerate(states_to_process, 1)
            ]

            with ProcessPoolExecutor(max_workers=max_workers) as executor:
                for state_code, success in executor.map(process_state_for_multi_year, state_args_list):
                    if success:
                        successful.append(state_code)
                        # Emit year-level progress using sys.stdout to ensure proper forwarding
                        sys.stdout.write(f"STATUS:YEAR:{args.year}:COMPLETE:{len(successful)}/50\n")
                        sys.stdout.flush()
                    else:
                        failed.append(state_code)
                        sys.stderr.write(f"[{args.year}] Failed: {STATE_CONFIG[state_code]['name']}\n")
                        sys.stderr.flush()

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

    # Skip post-processing if 'nation' not in --stages
    if 'nation' not in args.stages:
        print("\n[OK] Skipping nation post-processing (not in --stages)")
        print(f"[OK] Requested stages complete. Run with --stages nation to do post-processing later.")
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
        '--dpi', str(args.dpi),
        '--workers', str(args.workers)
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
    # Check if running in multi-year mode - if so, forward STATUS messages
    is_multi_year = os.environ.get('MULTI_YEAR_SUBPROCESS') == '1'

    if is_multi_year:
        # Use Popen to forward STATUS messages to parent
        proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                               text=True, bufsize=1)

        # Forward STATUS messages
        for line in proc.stdout:
            print(line, end='', flush=True)

        proc.wait()
        result_code = proc.returncode
    else:
        # Standalone mode - use regular subprocess.run
        result = subprocess.run(cmd)
        result_code = result.returncode

    return 0 if (result_code == 0 and (not 'failed' in locals() or not failed)) else 1


if __name__ == '__main__':
    # Required for multiprocessing on Windows
    multiprocessing.freeze_support()
    sys.exit(main())
