#!/usr/bin/env python3
"""
Process national-level aggregation and post-processing.

This script handles all nation-wide operations after individual states are processed:
1. Create US national maps
2. Create US aggregate files
3. Create rounds hierarchy
4. Create national round progression maps
5. Run batch political/demographic analysis (fallback mode)
6. Create metro area maps
7. Generate static dashboard
8. Validate pipeline outputs
9. Update version config

Usage:
  # After state processing completes
  python scripts/pipeline/process_nation.py --year 2020 --version v1 --output-dir outputs/v1/2020

  # Standalone post-processing only
  python scripts/pipeline/process_nation.py --year 2020 --version v1 --output-dir outputs/v1/2020
"""

import argparse
import subprocess
import sys
import os
import threading
import multiprocessing
from pathlib import Path
from tqdm import tqdm
from concurrent.futures import ProcessPoolExecutor

# Import utility functions
sys.path.insert(0, str(Path(__file__).parent.parent.parent))
from scripts.utils import (
    get_election_data_file,
    get_demographic_data_file,
    get_error_logger,
    get_stage_tracker,
)

# Import config helpers
try:
    from scripts.pipeline.run_complete_redistricting import (
        read_version_config,
        update_version_config_with_year
    )
except ImportError:
    # Fallback if circular import
    pass


def run_postprocessing_task(args_tuple):
    """
    Run a single post-processing task in parallel.
    Must be at module level for Windows multiprocessing pickling.

    Args:
        args_tuple: (task_name, command, worker_id, year, task_index, total_tasks, is_multi_year)

    Returns:
        (task_name, success_bool)
    """
    task_name, command, worker_id, year, task_index, total_tasks, is_multi_year = args_tuple

    # Don't print STATUS messages from worker processes on Windows
    # Windows multiprocessing doesn't properly inherit stdout, causing OSError
    # Year-level progress is tracked via return values instead

    try:
        # Run the command (suppress verbose output, keep only critical errors)
        result = subprocess.run(command, shell=True, timeout=900,
                               capture_output=True, text=True)

        # On completion, return success (progress tracked via return values)
        if result.returncode == 0:
            return (task_name, True)
        else:
            # On failure, print stderr for debugging
            if result.stderr:
                sys.stderr.write(f"[ERROR] {task_name} failed:\n{result.stderr}\n")
                sys.stderr.flush()
            return (task_name, False)

    except subprocess.TimeoutExpired:
        return (task_name, False)
    except Exception as e:
        return (task_name, False)


def main():
    parser = argparse.ArgumentParser(description='Process national-level redistricting aggregation')
    parser.add_argument('--year', type=str, required=True, choices=['2020', '2010', '2000'],
                        help='Census year')
    parser.add_argument('--version', type=str, required=True,
                        help='Version identifier (e.g., v1)')
    parser.add_argument('--output-dir', type=str, required=True,
                        help='Output directory (e.g., outputs/v1/2020)')
    parser.add_argument('--election-year', type=str, default='2020', choices=['2020', '2016'],
                        help='Election year for political analysis (default: 2020)')
    parser.add_argument('--dpi', type=int, default=150, choices=[72, 100, 150, 200, 300],
                        help='DPI for output maps (default: 150)')
    parser.add_argument('--run-analysis', action='store_true', default=True,
                        help='Per-state analysis was run (skip batch fallback mode)')
    parser.add_argument('--skip-analysis', dest='run_analysis', action='store_false',
                        help='Per-state analysis was skipped (use batch fallback mode)')
    parser.add_argument('--skip-political', action='store_true',
                        help='Skip political analysis steps')
    parser.add_argument('--skip-demographic', action='store_true',
                        help='Skip demographic analysis steps')
    parser.add_argument('--print-only', action='store_true',
                        help='Print commands without executing (debug mode)')
    parser.add_argument('--debug', action='store_true',
                        help='Enable debug mode')
    parser.add_argument('--workers', type=int, default=2,
                        help='Number of workers for parallel Phase 2 visualization (default: 2)')
    args = parser.parse_args()

    output_dir = Path(args.output_dir)
    scripts_dir = Path(__file__).parent

    # Check if running in multi-year mode
    is_multi_year = os.environ.get('MULTI_YEAR_SUBPROCESS') == '1'

    # Initialize error logger and stage tracker
    error_logger = None
    stage_tracker = None
    if not args.print_only:
        try:
            error_logger = get_error_logger(output_dir, args.version, int(args.year))
            stage_tracker = get_stage_tracker(output_dir)
        except Exception as e:
            # If logger initialization fails, continue without logging (non-fatal)
            if not is_multi_year:
                print(f"[WARNING] Could not initialize error logger: {e}")

    if not is_multi_year:
        print("\n" + "="*70)
        print("NATIONAL POST-PROCESSING")
        print("="*70)
        print(f"Census Year: {args.year}")
        print(f"Output directory: {output_dir}")
        print(f"Version: {args.version}")
        if args.print_only:
            print("Mode: PRINT-ONLY (debug mode - no execution)")
        print("="*70)

    # Collect all post-processing tasks as sequential stages
    # Workers will dynamically pick up tasks as they become available
    all_tasks = []

    # Common flags for all commands
    flags = []
    if args.print_only:
        flags.append('--print-only')
    if args.debug:
        flags.append('--debug')
    flags_str = ' '.join(flags)

    # ========== ALL POST-PROCESSING TASKS (numbered sequentially) ==========

    # Task 1: Create US aggregate files
    if output_dir.exists() or args.print_only:
        all_tasks.append({
            'name': 'US_aggregates',
            'command': f'{sys.executable} {scripts_dir}/create_us_aggregate.py --year {args.year} --version {args.version} --dpi {args.dpi} --output-dir {output_dir} --skip-maps {flags_str}'.strip(),
            'critical': False
        })

    # Task 2: Create US rounds hierarchy aggregate
    if output_dir.exists() or args.print_only:
        all_tasks.append({
            'name': 'Rounds_hierarchy',
            'command': f'{sys.executable} {scripts_dir}/create_us_rounds_hierarchy.py --output-dir {output_dir} {flags_str}'.strip(),
            'critical': False
        })

    # Task 3: National district maps
    national_map_script = scripts_dir / 'visualize_national_districts.py'
    if national_map_script.exists():
        all_tasks.append({
            'name': 'National_district_map',
            'command': f'{sys.executable} {scripts_dir}/visualize_national_districts.py --year {args.year} --output-dir {output_dir} --dpi {args.dpi} {flags_str}'.strip(),
            'critical': False
        })

    # Task 4: National round progression maps
    national_rounds_script = scripts_dir / 'visualize_national_rounds.py'
    if national_rounds_script.exists() and (output_dir.exists() or args.print_only):
        all_tasks.append({
            'name': 'Round_progression_maps',
            'command': f'{sys.executable} {national_rounds_script} --year {args.year} --version {args.version} --output-dir {output_dir} --dpi {args.dpi} --max-rounds 6'.strip(),
            'critical': False
        })

    # Check data availability for optional analysis
    # Political analysis requires election data from same time period as census
    # 2020 census -> use 2020 election, 2010 census -> would need 2010/2012 election (not available)
    election_data_file = get_election_data_file(args.election_year, args.version)
    election_data_available = (args.year == '2020' and election_data_file.exists())
    demographic_data_available = get_demographic_data_file(args.year, args.version).exists()

    # Log data availability status (only in standalone mode)
    if not is_multi_year:
        if not election_data_available and not args.skip_political:
            if args.year != '2020':
                print(f"\n[INFO] Political analysis will be skipped: Census year {args.year} requires {args.year}/2012 election data (not available)")
            else:
                print(f"\n[INFO] Political analysis will be skipped: No {args.election_year} election data found")
                print(f"       Expected: data/processed/elections/{args.election_year}_president_tract.parquet")
        if not demographic_data_available and not args.skip_demographic:
            print(f"[INFO] Demographic analysis will be skipped: No {args.year} demographic data found")
            print(f"       Expected: data/processed/demographics/{args.year}_demographics_tract.parquet\n")

    # Task 5: National political map
    if not args.skip_political and election_data_available and (output_dir.exists() or args.print_only):
        all_tasks.append({
            'name': 'Political_map',
            'command': f'{sys.executable} scripts/pipeline/visualize_partisan_lean.py --scope national --output-dir {output_dir} --version {args.version} --election-year {args.election_year} --census-year {args.year} --dpi {args.dpi}'.strip(),
            'critical': False
        })

    # Task 6: National demographic map
    if not args.skip_demographic and demographic_data_available and (output_dir.exists() or args.print_only):
        all_tasks.append({
            'name': 'Demographic_map',
            'command': f'{sys.executable} scripts/pipeline/visualize_district_demographics.py --scope national --output-dir {output_dir} --version {args.version} --census-year {args.year} --dpi {args.dpi}'.strip(),
            'critical': False
        })

    # Task 7: National compactness map
    if output_dir.exists() or args.print_only:
        compactness_script = scripts_dir / 'visualize_compactness.py'
        all_tasks.append({
            'name': 'Compactness_map',
            'command': f'{sys.executable} {compactness_script} --scope national --output-dir {output_dir} --version {args.version} --census-year {args.year} --dpi {args.dpi}'.strip(),
            'critical': False
        })

    # Task 8: Metro area national aggregation
    if output_dir.exists() or args.print_only:
        metro_viz_script = Path('scripts/pipeline/visualize_metro_areas.py')
        if metro_viz_script.exists():
            all_tasks.append({
                'name': 'Metro_areas',
                'command': f'{sys.executable} {metro_viz_script} --scope national --output-dir {output_dir} --version {args.version} --year {args.year}'.strip(),
                'critical': False
            })

    # Task 9: Generate static dashboard - MOVED TO FINAL STAGE after all years complete
    # Dashboard generation now happens once after all years finish
    # See run_complete_redistricting.py final stage

    # ========== EXECUTE ALL TASKS (Workers pick up tasks dynamically) ==========
    if all_tasks:
        if not is_multi_year:
            print("\n" + "="*70)
            print(f"POST-PROCESSING ({len(all_tasks)} tasks)")
            print("="*70)

        # Log stage start
        if error_logger:
            error_logger.log_stage_start('national_post_processing')

        total_tasks = len(all_tasks)

        # Determine number of workers (use allocated workers, or number of tasks if fewer)
        max_workers = min(total_tasks, args.workers)

        # Prepare task arguments (tasks numbered sequentially, workers assigned round-robin)
        task_args = [
            (task['name'], task['command'], i % max_workers, args.year, i, total_tasks, is_multi_year)
            for i, task in enumerate(all_tasks, 1)
        ]

        if args.print_only:
            # Print-only mode - just show what would run
            for task_name, command, worker_id, year, task_index, total_tasks, _ in task_args:
                if not is_multi_year:
                    print(f"[Worker {worker_id+1}] [Task {task_index}/{total_tasks}] {task_name}")
        else:
            # Emit initial Idle status for all workers (only in multi-year mode)
            if is_multi_year:
                for worker_id in range(max_workers):
                    print(f"STATUS:WORKER:{args.year}:{worker_id}:TASK:0/{total_tasks}:Idle", flush=True)
                sys.stdout.flush()

            # Run tasks in parallel using ProcessPoolExecutor
            # Workers dynamically pick up next task as they finish
            try:
                # Emit initial progress for multi-year mode
                if is_multi_year:
                    print(f"STATUS:YEAR_POSTPROCESS:{args.year}:0/{total_tasks}", flush=True)

                # Submit all tasks and track completion
                failed_tasks = []
                completed = 0

                with ProcessPoolExecutor(max_workers=max_workers) as executor:
                    # Use as_completed to track progress
                    from concurrent.futures import as_completed
                    futures = {executor.submit(run_postprocessing_task, task_arg): task_arg[0]
                              for task_arg in task_args}

                    # Process results as they complete
                    for future in as_completed(futures):
                        task_name = futures[future]
                        try:
                            result_name, success = future.result()
                            completed += 1

                            # Emit progress update for multi-year mode
                            if is_multi_year:
                                print(f"STATUS:YEAR_POSTPROCESS:{args.year}:{completed}/{total_tasks}", flush=True)

                            if not success:
                                failed_tasks.append(task_name)
                                if error_logger:
                                    # Log as warning since tasks are non-critical
                                    error_logger.log_warning(
                                        f"Task '{task_name}' failed but continuing",
                                        context={'year': args.year, 'task_count': f'{len(failed_tasks)}/{total_tasks}'}
                                    )
                                if not is_multi_year:
                                    print(f"[WARNING] {task_name} failed but continuing...")
                        except Exception as e:
                            # Future failed to execute
                            failed_tasks.append(task_name)
                            if error_logger:
                                error_logger.log_exception('post_processing_task', e, context={'task': task_name})

                # Log stage complete (with or without failures)
                if error_logger:
                    if failed_tasks:
                        error_logger.log_warning(
                            f"National post-processing completed with {len(failed_tasks)} failed tasks",
                            context={'failed_tasks': ', '.join(failed_tasks)}
                        )
                    error_logger.log_stage_complete('national_post_processing')

                # Mark stage as complete
                if stage_tracker and not failed_tasks:
                    stage_tracker.mark_stage_complete(
                        'national_post_processing',
                        metadata={'tasks_completed': total_tasks, 'year': args.year}
                    )

            except Exception as e:
                # Log critical failure
                if error_logger:
                    error_logger.log_stage_failed('national_post_processing')
                    error_logger.log_exception('national_post_processing', e, context={
                        'total_tasks': total_tasks,
                        'max_workers': max_workers
                    })
                raise

    # Validate pipeline outputs
    if not args.print_only and not is_multi_year:
        print("\n" + "="*70)
        print("  Validating Pipeline Outputs")
        print("="*70)

        validation_script = Path('scripts/validation/validate_pipeline_outputs.py')
        if validation_script.exists():
            validation_cmd = [
                sys.executable,
                str(validation_script),
                '--year', args.year,
                '--version', args.version,
                '--output-dir', str(output_dir)
            ]

            validation_result = subprocess.run(validation_cmd)

            # Validation script handles its own output:
            # - Brief summary printed to console
            # - Detailed report written to outputs/us_{year}_{version}_validation.txt
            # - Exit code: 0 = all outputs present, 1 = some outputs missing

            if validation_result.returncode != 0 and not is_multi_year:
                print(f"\nWARNING: Some pipeline outputs are missing.")
                print(f"Review detailed report at: {output_dir.name}_validation.txt")

    # Update version config with completed year
    if not args.print_only:
        version_dir = output_dir.parent  # e.g., outputs/v1/ (parent of outputs/v1/2020/)
        version_config_path = version_dir / 'version.json'
        if version_config_path.exists():
            try:
                update_version_config_with_year(version_dir, int(args.year))
                if not is_multi_year:
                    version_config = read_version_config(version_config_path)
                    print(f"\n[OK] Updated version config: completed years = {version_config.completed_years}")
            except Exception as e:
                if not is_multi_year:
                    print(f"\n[WARNING] Could not update version config: {e}")

    # Brief final summary (only in standalone mode)
    if not is_multi_year:
        summary_bar = tqdm(total=1,
                          desc=f"[3] National post-processing complete - Results in: {output_dir}",
                          unit="step",
                          position=3,
                          ncols=120,
                          leave=True,
                          bar_format="{desc}",
                          dynamic_ncols=False,
                          file=sys.stderr)
        summary_bar.update(1)
        summary_bar.close()

    # Write error log summary
    if error_logger:
        try:
            error_logger.write_summary()
        except Exception:
            pass  # Non-fatal if summary write fails

    return 0


if __name__ == '__main__':
    sys.exit(main())
