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
    Run a single post-processing task (for parallel execution).
    Must be at module level for Windows multiprocessing pickling.

    Args:
        args_tuple: (task_name, command, worker_id, year, task_index, total_tasks)

    Returns:
        (task_name, success_bool)
    """
    task_name, command, worker_id, year, task_index, total_tasks = args_tuple

    # Emit starting status
    print(f"STATUS:WORKER:{year}:{worker_id}:TASK:{task_index}/{total_tasks}:{task_name}:PROGRESS:0/100", flush=True)

    try:
        # Run the command
        proc = subprocess.Popen(command, shell=True,
                               stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE,
                               text=True, bufsize=1)

        # Monitor stdout for progress (if child emits PROGRESS messages)
        for line in proc.stdout:
            if line.startswith("PROGRESS:"):
                # Forward progress updates
                try:
                    progress_str = line.split(":", 1)[1].strip()
                    current, total = progress_str.split('/')
                    percent = int((int(current) / int(total)) * 100)
                    print(f"STATUS:WORKER:{year}:{worker_id}:TASK:{task_index}/{total_tasks}:{task_name}:PROGRESS:{percent}/100", flush=True)
                except:
                    pass

        proc.wait(timeout=900)  # 15 minute timeout

        if proc.returncode == 0:
            # Emit completion status
            print(f"STATUS:WORKER:{year}:{worker_id}:TASK:{task_index}/{total_tasks}:{task_name}:PROGRESS:100/100", flush=True)
            return (task_name, True)
        else:
            return (task_name, False)

    except subprocess.TimeoutExpired:
        proc.kill()
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
    parser.add_argument('--workers', type=int, default=6,
                        help='Number of workers for parallel Phase 2 visualization (default: 6)')
    args = parser.parse_args()

    output_dir = Path(args.output_dir)
    scripts_dir = Path(__file__).parent

    # Check if running in multi-year mode
    is_multi_year = os.environ.get('MULTI_YEAR_SUBPROCESS') == '1'

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

    # Organize tasks into 3 phases:
    # Phase 1: Setup (sequential - creates prerequisite files)
    # Phase 2: Visualization (parallel - independent visualization tasks)
    # Phase 3: Finalization (sequential - dashboard and validation)

    phase1_steps = []  # Setup
    phase2_steps = []  # Parallel visualization
    phase3_steps = []  # Finalization

    # Common flags for all commands
    flags = []
    if args.print_only:
        flags.append('--print-only')
    if args.debug:
        flags.append('--debug')
    flags_str = ' '.join(flags)

    # ========== PHASE 1: SETUP (Sequential - creates prerequisite files) ==========

    # Create US aggregate files (required by visualization tasks)
    if output_dir.exists() or args.print_only:
        phase1_steps.append({
            'name': 'US aggregates',
            'command': f'{sys.executable} {scripts_dir}/create_us_aggregate.py --year {args.year} --version {args.version} --dpi {args.dpi} --output-dir {output_dir} --skip-maps {flags_str}'.strip(),
            'critical': False
        })

    # Create US rounds hierarchy aggregate (required by round progression maps)
    if output_dir.exists() or args.print_only:
        phase1_steps.append({
            'name': 'Rounds hierarchy',
            'command': f'{sys.executable} {scripts_dir}/create_us_rounds_hierarchy.py --output-dir {output_dir} {flags_str}'.strip(),
            'critical': False
        })

    # ========== PHASE 2: VISUALIZATION (Parallel - independent tasks) ==========

    # National district maps
    national_map_script = scripts_dir / 'visualize_national_districts.py'
    if national_map_script.exists():
        phase2_steps.append({
            'name': 'National district map',
            'command': f'{sys.executable} {scripts_dir}/visualize_national_districts.py --year {args.year} --output-dir {output_dir} --dpi {args.dpi} {flags_str}'.strip(),
            'critical': False
        })

    # National round progression maps
    national_rounds_script = scripts_dir / 'visualize_national_rounds.py'
    if national_rounds_script.exists() and (output_dir.exists() or args.print_only):
        phase2_steps.append({
            'name': 'Round progression maps',
            'command': f'{sys.executable} {national_rounds_script} --year {args.year} --version {args.version} --output-dir {output_dir} --dpi {args.dpi} --max-rounds 6'.strip(),
            'critical': False
        })

    # Check data availability for optional analysis
    # Political analysis requires election data from same time period as census
    # 2020 census -> use 2020 election, 2010 census -> would need 2010/2012 election (not available)
    election_data_file = get_election_data_file(args.election_year)
    election_data_available = (args.year == '2020' and election_data_file.exists())
    demographic_data_available = get_demographic_data_file(args.year).exists()

    # Log data availability status
    if not election_data_available and not args.skip_political:
        if args.year != '2020':
            print(f"\n[INFO] Political analysis will be skipped: Census year {args.year} requires {args.year}/2012 election data (not available)")
        else:
            print(f"\n[INFO] Political analysis will be skipped: No {args.election_year} election data found")
            print(f"       Expected: data/processed/elections/{args.election_year}_president_tract.parquet")
    if not demographic_data_available and not args.skip_demographic:
        print(f"[INFO] Demographic analysis will be skipped: No {args.year} demographic data found")
        print(f"       Expected: data/processed/demographics/{args.year}_demographics_tract.parquet\n")

    # National political map
    if not args.skip_political and election_data_available and (output_dir.exists() or args.print_only):
        phase2_steps.append({
            'name': 'Political map',
            'command': f'{sys.executable} scripts/pipeline/visualize_partisan_lean.py --scope national --output-dir {output_dir} --version {args.version} --election-year {args.election_year} --census-year {args.year} --dpi {args.dpi}'.strip(),
            'critical': False
        })

    # National demographic map
    if not args.skip_demographic and demographic_data_available and (output_dir.exists() or args.print_only):
        phase2_steps.append({
            'name': 'Demographic map',
            'command': f'{sys.executable} scripts/pipeline/visualize_district_demographics.py --scope national --output-dir {output_dir} --version {args.version} --census-year {args.year} --dpi {args.dpi}'.strip(),
            'critical': False
        })

    # National compactness map
    if output_dir.exists() or args.print_only:
        compactness_script = scripts_dir / 'visualize_compactness.py'
        phase2_steps.append({
            'name': 'Compactness map',
            'command': f'{sys.executable} {compactness_script} --scope national --output-dir {output_dir} --version {args.version} --census-year {args.year} --dpi {args.dpi}'.strip(),
            'critical': False
        })

    # Metro area national aggregation
    if output_dir.exists() or args.print_only:
        metro_viz_script = Path('scripts/pipeline/visualize_metro_areas.py')
        if metro_viz_script.exists():
            phase2_steps.append({
                'name': 'Metro areas',
                'command': f'{sys.executable} {metro_viz_script} --scope national --output-dir {output_dir} --version {args.version} --year {args.year}'.strip(),
                'critical': False
            })

    # ========== PHASE 3: FINALIZATION (Sequential - requires all visualizations complete) ==========

    # Generate static dashboard with all district data
    if output_dir.exists() or args.print_only:
        dashboard_script = Path('scripts/web/generate_dashboard.py')
        if dashboard_script.exists():
            phase3_steps.append({
                'name': 'Dashboard',
                'command': f'{sys.executable} {dashboard_script} --year {args.year} --version {args.version} --output-dir {output_dir}'.strip(),
                'critical': False
            })

    # Helper function to run a single step sequentially
    def run_sequential_step(step, phase_name, step_index, total_steps):
        if is_multi_year:
            print(f"STATUS:YEAR:{args.year}:POSTPROCESS:PHASE:{phase_name}:STEP:{step_index}/{total_steps}:{step['name']}", flush=True)
        else:
            print(f"\n[{step_index}/{total_steps}] {step['name']}...")

        proc = subprocess.run(step['command'], shell=True, capture_output=True)
        return proc.returncode == 0

    # ========== PHASE 1: SETUP (Sequential) ==========
    if phase1_steps:
        if is_multi_year:
            print(f"STATUS:YEAR:{args.year}:POSTPROCESS:PHASE:1/3:Setup", flush=True)
        else:
            print("\n" + "="*70)
            print("PHASE 1: SETUP")
            print("="*70)

        for i, step in enumerate(phase1_steps, 1):
            success = run_sequential_step(step, "1/3:Setup", i, len(phase1_steps))
            if not success and step['critical'] and not args.print_only:
                return 1

    # ========== PHASE 2: VISUALIZATION (Parallel) ==========
    if phase2_steps:
        if is_multi_year:
            print(f"STATUS:YEAR:{args.year}:POSTPROCESS:PHASE:2/3:Visualization", flush=True)
        else:
            print("\n" + "="*70)
            print("PHASE 2: VISUALIZATION (Parallel)")
            print("="*70)

        # Determine number of workers (use allocated workers, or number of tasks if fewer)
        max_workers = min(len(phase2_steps), args.workers)

        # Prepare task arguments with worker ID assignment
        task_args = [
            (step['name'], step['command'], i % max_workers, args.year, i, len(phase2_steps))
            for i, step in enumerate(phase2_steps, 1)
        ]

        if not args.print_only:
            with ProcessPoolExecutor(max_workers=max_workers) as executor:
                for task_name, success in executor.map(run_postprocessing_task, task_args):
                    if not success:
                        if not is_multi_year:
                            print(f"[WARNING] {task_name} failed but continuing...")
        else:
            # Print-only mode - just show what would run
            for task_name, command, worker_id, year, task_index, total_tasks in task_args:
                if not is_multi_year:
                    print(f"[{task_index}/{total_tasks}] {task_name}: {command}")

    # ========== PHASE 3: FINALIZATION (Sequential) ==========
    if phase3_steps:
        if is_multi_year:
            print(f"STATUS:YEAR:{args.year}:POSTPROCESS:PHASE:3/3:Finalization", flush=True)
        else:
            print("\n" + "="*70)
            print("PHASE 3: FINALIZATION")
            print("="*70)

        for i, step in enumerate(phase3_steps, 1):
            success = run_sequential_step(step, "3/3:Finalization", i, len(phase3_steps))
            if not success and step['critical'] and not args.print_only:
                return 1

    # Validate pipeline outputs
    if not args.print_only:
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

            if validation_result.returncode != 0:
                print(f"\nWARNING: Some pipeline outputs are missing.")
                print(f"Review detailed report at: {output_dir.name}_validation.txt")

    # Update version config with completed year
    if not args.print_only:
        version_dir = output_dir.parent  # e.g., outputs/v1/ (parent of outputs/v1/2020/)
        version_config_path = version_dir / 'version.json'
        if version_config_path.exists():
            try:
                update_version_config_with_year(version_dir, int(args.year))
                version_config = read_version_config(version_config_path)
                print(f"\n[OK] Updated version config: completed years = {version_config.completed_years}")
            except Exception as e:
                print(f"\n[WARNING] Could not update version config: {e}")

    # Brief final summary at position 3 (after the 3 post-processing steps at 0-2)
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

    return 0


if __name__ == '__main__':
    sys.exit(main())
