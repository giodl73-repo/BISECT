#!/usr/bin/env python3
"""
Download orchestrator for parallel multi-year census data downloads.

Follows the same architecture as scripts/pipeline/run_complete_redistricting.py:
- Parallel execution with multiple workers
- Hierarchical progress display (year -> worker -> state)
- STATUS protocol for real-time updates
- Multi-year support

Usage:
    # Download for redistricting pipeline stages (recommended)
    python download_orchestrator.py --stages redistricting demographics --year 2020 --workers 4

    # Download specific data types (legacy mode)
    python download_orchestrator.py --type demographics --year 2020 --workers 4
    python download_orchestrator.py --type redistricting --year all --workers 12

    # Test with specific states
    python download_orchestrator.py --stages redistricting --year 2020 --workers 2 --states VT DE

Created: 2026-01-18 (Enhancement #48)
Updated: 2026-01-18 (Added stage-aware downloading)
"""

import sys
import os
import argparse
import subprocess
import threading
import time
import select
from pathlib import Path
from concurrent.futures import ProcessPoolExecutor, as_completed

# Add project root to path
script_dir = Path(__file__).parent
project_root = script_dir.parent.parent
sys.path.insert(0, str(project_root))

# Import configuration
from scripts.config.download_sources import (
    ALL_STATES, validate_state_code, validate_year
)

# Import progress coordinator and stage checking
from scripts.data.download_progress import DownloadCoordinator, parse_download_status
from scripts.data.download_stages import (
    get_download_plan, print_download_status, STAGE_REQUIREMENTS
)


def allocate_workers_across_years(total_workers, num_years=3):
    """
    Distribute workers across census years for parallel execution.

    Prioritizes 2020 (newest) > 2010 > 2000 for extra workers.
    Same pattern as pipeline.

    Args:
        total_workers: Total number of workers available
        num_years: Number of census years (default: 3)

    Returns:
        List of worker counts per year [2020, 2010, 2000]
    """
    if total_workers < num_years:
        return [1] * num_years

    base = total_workers // num_years
    remainder = total_workers % num_years

    workers = [base] * num_years

    # Distribute remainder from first to last (prioritize 2020)
    for i in range(remainder):
        workers[i] += 1

    return workers


def download_state_worker(args_tuple):
    """
    Worker function to download data for a single state.

    Must be at module level for Windows multiprocessing pickling.

    Args:
        args_tuple: (state_code, state_number, download_type, year, output_dir, worker_id)

    Returns:
        (state_code, success_bool)
    """
    state_code, state_number, download_type, year, output_dir, worker_id = args_tuple

    scripts_dir = Path(__file__).parent

    # Build command for download_worker.py
    cmd = [
        sys.executable,
        str(scripts_dir / 'download_worker.py'),
        '--state', state_code,
        '--type', download_type,
        '--year', year,
        '--output-dir', str(output_dir)
    ]

    # Set environment for worker subprocess
    env = os.environ.copy()
    env['STATE_NUMBER'] = str(state_number)
    env['WORKER_ID'] = str(worker_id)
    env['CENSUS_YEAR'] = year
    env['MULTI_YEAR_SUBPROCESS'] = '1'

    # Use Popen to forward STATUS messages
    proc = subprocess.Popen(
        cmd,
        shell=False,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1
    )

    # Forward stdout (STATUS messages)
    for line in proc.stdout:
        print(line, end='', flush=True)

    proc.wait()

    return (state_code, proc.returncode == 0)


def main():
    parser = argparse.ArgumentParser(description='Parallel download orchestrator for census data')

    # Stage-aware mode (new, recommended)
    parser.add_argument('--stages', type=str, nargs='*',
                       choices=['redistricting', 'adjacency', 'elections', 'demographics', 'places', 'metros', 'enacted_districts'],
                       help='Pipeline stages to download data for (recommended)')

    # Legacy type mode (backward compatible)
    parser.add_argument('--type', type=str,
                       choices=['redistricting', 'demographics', 'places', 'elections', 'metros', 'enacted_districts', 'all'],
                       help='Data type to download (legacy mode, use --stages instead)')

    parser.add_argument('--year', type=str, default='2020',
                       choices=['2020', '2010', '2000', 'all'],
                       help='Census year or "all" for multi-year parallel')
    parser.add_argument('--workers', type=int, default=4,
                       help='Number of parallel workers (default: 4)')
    parser.add_argument('--states', type=str, nargs='*',
                       help='Specific states to download (default: all 50)')
    parser.add_argument('--output-dir', type=str, default='outputs/data',
                       help='Output directory (default: outputs/data)')
    parser.add_argument('--force', action='store_true',
                       help='Force redownload even if files exist')
    parser.add_argument('--print-only', action='store_true',
                       help='Print what would be done without executing')
    parser.add_argument('--check-only', action='store_true',
                       help='Check what data exists and what needs downloading (no download)')
    args = parser.parse_args()

    # Default behavior: if no stages or type specified, check all stages for all years
    if not args.stages and not args.type:
        print("\n" + "="*70)
        print("DOWNLOAD ORCHESTRATOR - Checking All Data")
        print("="*70)
        print("No --stages specified. Defaulting to: --stages all --year all --check-only")
        print("(To download: dl --stages redistricting demographics --year 2020)")
        print("="*70 + "\n")

        # Set defaults
        args.stages = ['redistricting', 'demographics', 'elections', 'places', 'metros', 'enacted_districts']
        args.year = 'all'
        args.check_only = True

    if args.stages and args.type:
        parser.error("Cannot specify both --stages and --type (use --stages for pipeline integration)")

    # Build year queue
    if args.year == 'all':
        year_queue = ['2020', '2010', '2000']
    else:
        year_queue = [args.year]

    # Build state list
    if args.states:
        states_to_download = [s.upper() for s in args.states]
        # Validate states
        for state in states_to_download:
            try:
                validate_state_code(state)
            except ValueError as e:
                print(f"[ERROR] {e}", file=sys.stderr)
                return 1
    else:
        states_to_download = ALL_STATES

    # =============================================================================
    # STAGE-AWARE CACHE CHECKING
    # =============================================================================
    # If using --stages, check what's in local cache vs what needs downloading

    if args.stages:
        print("\n" + "="*70)
        if args.force:
            print("CHECKING LOCAL CACHE (FORCE MODE - will redownload)")
        else:
            print("CHECKING LOCAL CACHE")
        print("="*70)

        for year in year_queue:
            year_output_dir = Path(args.output_dir) / year
            print(f"\n[{year}] Checking what's available locally...")

            # Get download plan for this year
            plan = get_download_plan(int(year), args.stages, year_output_dir, force=args.force)

            # Report status
            if plan['complete_stages']:
                print(f"  [OK] Already processed (skip download):")
                for stage in plan['complete_stages']:
                    print(f"    - {stage}: {STAGE_REQUIREMENTS[stage]['description']}")

            if plan['available_data']:
                print(f"  [CACHE] Data in local cache (skip download):")
                for stage in plan['available_data']:
                    print(f"    - {stage}: {STAGE_REQUIREMENTS[stage]['description']}")

            if plan['needed_downloads']:
                print(f"  [DOWNLOAD] Needed from remote:")
                for stage, missing_files in plan['needed_downloads'].items():
                    print(f"    - {stage}: {STAGE_REQUIREMENTS[stage]['description']}")
                    for file in missing_files[:2]:  # Show first 2 missing files
                        print(f"        Missing: {file}")
                    if len(missing_files) > 2:
                        print(f"        ... and {len(missing_files) - 2} more")

            # If everything is available, we can skip download entirely
            if not plan['needed_downloads']:
                print(f"  [OK] All data available! No download needed for {year}")

        print("="*70)
        print()

        # Check-only mode: exit after reporting with helpful commands
        if args.check_only:
            print("[CHECK-ONLY MODE] Download check complete.")

            # Collect missing data per year
            missing_by_year = {}
            for year in year_queue:
                year_output_dir = Path(args.output_dir) / year
                plan = get_download_plan(int(year), args.stages, year_output_dir, force=False)
                if plan['needed_downloads']:
                    missing_by_year[year] = list(plan['needed_downloads'].keys())

            if missing_by_year:
                print("\n" + "="*70)
                print("MISSING DATA DETECTED")
                print("="*70)

                # Group years by same missing stages
                stages_to_years = {}
                download_commands = []
                for year, stages in missing_by_year.items():
                    stages_key = tuple(sorted(stages))
                    if stages_key not in stages_to_years:
                        stages_to_years[stages_key] = []
                    stages_to_years[stages_key].append(year)

                # Generate commands
                for idx, (stages_tuple, years) in enumerate(stages_to_years.items(), 1):
                    stages_list = list(stages_tuple)
                    stages_str = ' '.join(stages_list)

                    # Determine year argument
                    if len(years) == 1:
                        year_arg = years[0]
                        year_desc = years[0]
                    elif len(years) == len(year_queue) and len(year_queue) > 1:
                        # All requested years need this
                        year_arg = 'all'
                        year_desc = f"{len(years)} years"
                    else:
                        # Multiple but not all years
                        year_arg = 'all'
                        year_desc = f"{len(years)} years ({', '.join(years)})"

                    cmd = f"dl --stages {stages_str} --year {year_arg}"
                    download_commands.append(cmd)

                    print(f"\n{idx}. Download {', '.join(stages_list)} for {year_desc}:")
                    print(f"   {cmd}")

                print("\n" + "="*70)

                # Interactive prompt
                print("\nWhat would you like to do?")
                for i in range(len(download_commands)):
                    print(f"  {i+1}) Run download command {i+1}")
                print(f"  0) Exit")
                print()

                try:
                    choice = input("Enter choice [0-{}]: ".format(len(download_commands))).strip()

                    if choice == '0' or choice == '':
                        print("\n[EXIT] No downloads performed.")
                        return 0

                    choice_num = int(choice)
                    if 1 <= choice_num <= len(download_commands):
                        # Re-run orchestrator with the selected command in a new process
                        selected_cmd = download_commands[choice_num - 1]
                        print(f"\n[RUNNING] {selected_cmd}")
                        print("="*70 + "\n")

                        # Build full command
                        import shlex
                        cmd_parts = shlex.split(selected_cmd)
                        # Remove 'dl' and replace with full script path
                        if cmd_parts[0] == 'dl':
                            cmd_parts = cmd_parts[1:]

                        full_cmd = [sys.executable, str(Path(__file__))] + cmd_parts

                        # Run in subprocess
                        result = subprocess.run(full_cmd)
                        return result.returncode
                    else:
                        print(f"\n[ERROR] Invalid choice: {choice}")
                        return 1

                except ValueError:
                    print(f"\n[ERROR] Invalid input. Please enter a number.")
                    return 1
                except KeyboardInterrupt:
                    print("\n\n[CANCELLED] User interrupted.")
                    return 0
            else:
                print("\n[OK] All requested data is available! No downloads needed.")
                return 0

        # Filter year_queue to only years that need downloads
        years_needing_download = []
        for year in year_queue:
            year_output_dir = Path(args.output_dir) / year
            plan = get_download_plan(int(year), args.stages, year_output_dir, force=args.force)
            if plan['needed_downloads']:
                years_needing_download.append(year)

        if not years_needing_download:
            print("[OK] All data available in local cache. No downloads needed!")
            print("\nTo process existing data, run:")
            print(f"  python scripts/data/process_census_data.py --year {year_queue[0]} --stages {' '.join(args.stages)}")
            return 0

        # Update year_queue to only download what's needed
        year_queue = years_needing_download
        print(f"[DOWNLOAD] Proceeding to download for: {', '.join(year_queue)}\n")

    # Determine download types based on stages or legacy type
    download_types = []
    if args.stages:
        # Stages mode: download types are stage names (except adjacency which is generated)
        for stage in args.stages:
            if stage == 'adjacency':
                # Skip adjacency - it's generated from redistricting data, not downloaded
                continue
            if stage not in download_types:
                download_types.append(stage)
        if not download_types:
            print("[INFO] No downloads needed for specified stages (adjacency is generated, not downloaded)")
            return 0
    else:
        # Legacy mode - single type
        download_types = [args.type]

    # Allocate workers across years
    workers_per_year = allocate_workers_across_years(args.workers, num_years=len(year_queue))

    print("\n" + "="*70)
    if len(year_queue) > 1:
        print(f"PARALLEL MULTI-YEAR DOWNLOAD: {', '.join(year_queue)}")
    else:
        print(f"DOWNLOAD: {year_queue[0]} Census")
    print("="*70)
    if args.stages:
        print(f"Stages: {', '.join(args.stages)}")
        print(f"Download types: {', '.join(download_types)}")
    else:
        print(f"Data type: {args.type}")
    print(f"States: {len(states_to_download)} states")
    print(f"Workers: {args.workers} total")
    if len(year_queue) > 1:
        workers_str = '/'.join(str(w) for w in workers_per_year)
        print(f"  Per year: {workers_str}")
    print("="*70)
    print()

    if args.print_only:
        print("[PRINT-ONLY MODE] Would download:")
        for year in year_queue:
            print(f"  [{year}] {len(states_to_download)} states with {workers_per_year[year_queue.index(year)]} workers")
        print()
        return 0

    # Create progress coordinator
    coordinator = DownloadCoordinator(
        download_type=args.type,
        years=year_queue,
        workers_per_year=workers_per_year
    )

    # Display lock for thread-safe updates
    display_lock = threading.Lock()
    last_display_time = [time.time()]
    num_display_lines = [0]

    def clear_and_update_display(coordinator):
        """Clear and redraw display (copy from pipeline)."""
        if num_display_lines[0] > 0:
            print(f"\033[{num_display_lines[0]}A", end='', flush=True)

        display_text = coordinator.render()
        lines = display_text.split('\n')
        for line in lines:
            print(f"\r\033[K{line}")

        num_display_lines[0] = len(lines)
        sys.stdout.flush()

    def monitor_process(year, proc, coordinator):
        """Monitor subprocess and update coordinator."""
        try:
            while True:
                line = proc.stdout.readline()
                if not line and proc.poll() is not None:
                    break

                line = line.strip()
                if not line:
                    continue

                msg_type, data = parse_download_status(line)
                if msg_type == 'YEAR':
                    with display_lock:
                        coordinator.update_year_progress(
                            data['year'],
                            data['completed'],
                            data['total']
                        )
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
                            data['step'],
                            data['step_total'],
                            data['step_desc']
                        )
                        now = time.time()
                        if now - last_display_time[0] >= 0.5:
                            clear_and_update_display(coordinator)
                            last_display_time[0] = now

                elif msg_type == 'ERROR':
                    print(f"\n[ERROR] {data['year']} {data['state_name']}: {data['error']}", file=sys.stderr)

        except Exception as e:
            sys.stderr.write(f"[ERROR] Monitor thread for {year} died: {e}\n")
            sys.stderr.flush()

    # Initial display
    display_text = coordinator.render()
    lines = display_text.split('\n')
    for line in lines:
        print(line)
    num_display_lines[0] = len(lines)
    sys.stdout.flush()

    # Start timestamp
    start_time = time.time()

    # Process states with concurrent workers
    # Use direct Popen with threading for STATUS message parsing
    results = {}

    from scripts.data.download_progress import parse_download_status

    for year_idx, year in enumerate(year_queue):
        year_workers = workers_per_year[year_idx]
        year_output_dir = Path(args.output_dir) / year
        year_results = []

        # Loop over each download type needed (usually 1 in legacy mode, multiple in stages mode)
        for download_type in download_types:
            print(f"\n[{year}] Downloading {download_type}...")

            # Build all worker commands (one per state, limited by worker count)
            scripts_dir = Path(__file__).parent
            worker_procs = []
            worker_threads = []

            # Process states in batches of year_workers
            for batch_start in range(0, len(states_to_download), year_workers):
                batch_end = min(batch_start + year_workers, len(states_to_download))
                batch_states = states_to_download[batch_start:batch_end]

                batch_procs = []
                batch_threads = []

                for state_offset, state_code in enumerate(batch_states):
                    state_idx = batch_start + state_offset
                    state_number = state_idx + 1
                    worker_id = state_offset  # 0 to year_workers-1
                    state_number = state_idx + 1
                    worker_id = state_offset  # 0 to year_workers-1

                    cmd = [
                        sys.executable,
                        str(scripts_dir / 'download_worker.py'),
                        '--state', state_code,
                        '--type', download_type,
                        '--year', year,
                        '--output-dir', str(year_output_dir)
                    ]

                    env = os.environ.copy()
                    env['STATE_NUMBER'] = str(state_number)
                    env['WORKER_ID'] = str(worker_id)
                    env['CENSUS_YEAR'] = year
                    env['MULTI_YEAR_SUBPROCESS'] = '1'

                    proc = subprocess.Popen(
                        cmd,
                        shell=False,
                        env=env,
                        stdout=subprocess.PIPE,
                        stderr=subprocess.PIPE,
                        text=True,
                        bufsize=1
                    )

                    # Create monitoring thread for this worker
                    def monitor_worker(proc, state_code, year):
                        try:
                            for line in proc.stdout:
                                line = line.strip()
                                if not line:
                                    continue

                                msg_type, data = parse_download_status(line)

                                if msg_type == 'WORKER':
                                    with display_lock:
                                        coordinator.update_worker_status(
                                            data['year'],
                                            data['worker_id'],
                                            data['state_num'],
                                            data['state_name'],
                                            data['step'],
                                            data['step_total'],
                                            data['step_desc']
                                        )
                                        now = time.time()
                                        if now - last_display_time[0] >= 0.5:
                                            clear_and_update_display(coordinator)
                                            last_display_time[0] = now
                        except Exception as e:
                            pass  # Silently ignore thread errors

                    thread = threading.Thread(target=monitor_worker, args=(proc, state_code, year), daemon=True)
                    thread.start()

                    batch_procs.append((proc, state_code))
                    batch_threads.append(thread)

                # Wait for this batch to complete
                for proc, state_code in batch_procs:
                    proc.wait()
                    success = (proc.returncode == 0)
                    year_results.append((state_code, success))

                    # Update progress
                    completed = len(year_results)
                    coordinator.update_year_progress(year, completed, len(states_to_download))
                    with display_lock:
                        clear_and_update_display(coordinator)

                # Wait for monitoring threads
                for thread in batch_threads:
                    thread.join(timeout=1.0)

            type_completed = len([s for s, success in year_results if success])
            print(f"\n[{year}] {download_type}: {type_completed}/{len(states_to_download)} states completed")

        # Final results for this year (all types combined)
        unique_states = set(state for state, _ in year_results)
        successful_states = set(state for state, success in year_results if success)

        results[year] = {
            'success': len(successful_states) == len(unique_states),
            'completed': len(successful_states),
            'total': len(unique_states)
        }

    # Calculate elapsed time
    elapsed = time.time() - start_time
    elapsed_mins = elapsed / 60

    # Final display update
    print("\nAll downloads complete!")

    # Move past the hierarchical display
    print("\n\n")

    # Print summary
    print("="*70)
    print("DOWNLOAD COMPLETE")
    print("="*70)
    print(f"Total Time: {elapsed_mins:.1f} minutes")
    print(f"\nResults:")

    success_count = 0
    for year in year_queue:
        result = results[year]
        status = "[OK]" if result['success'] else "[PARTIAL]"
        print(f"  {year}: {status} ({result['completed']}/{result['total']} states)")
        if result['success']:
            success_count += 1

    print("="*70)

    if success_count == len(year_queue):
        print("\n[SUCCESS] All years completed successfully")
        return 0
    elif success_count > 0:
        print(f"\n[PARTIAL] {success_count}/{len(year_queue)} years completed")
        return 1
    else:
        print("\n[FAILURE] All years failed")
        return 1


if __name__ == '__main__':
    import multiprocessing
    multiprocessing.freeze_support()
    sys.exit(main())
