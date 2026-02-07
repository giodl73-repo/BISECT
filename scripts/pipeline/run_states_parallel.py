#!/usr/bin/env python3
"""
Parallel state worker manager for redistricting pipeline.

This script spawns multiple state workers in parallel using ProcessPoolExecutor.
It's called by the pipeline orchestrator for the "states" stage.

Usage:
    python scripts/pipeline/run_states_parallel.py --year 2020 --workers 4 --output-dir outputs/v1/2020
"""

import argparse
import sys
import os
import subprocess
from pathlib import Path
from concurrent.futures import ProcessPoolExecutor
from typing import List, Tuple, Dict, Any

# Add project root to path
project_root = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(project_root))
sys.path.insert(0, str(project_root / 'src'))

from scripts.utils import get_state_config


def run_batch_mode(
    states_to_process: List[str],
    year: str,
    version: str,
    output_dir: Path,
    args_dict: Dict[str, Any],
    max_workers: int,
    run_analysis: bool
) -> Tuple[List[str], List[str]]:
    """
    Run redistricting in batch mode: all states complete stage N before any state starts N+1.

    Args:
        states_to_process: List of state codes to process
        year: Census year
        version: Version identifier
        output_dir: Output directory
        args_dict: Arguments dict
        max_workers: Number of parallel workers
        run_analysis: Whether to run analysis stages

    Returns:
        (successful_states, failed_states)
    """
    # Define stages to run
    core_stages = ['metis', 'summary', 'cities', 'round_maps', 'district_maps']
    analysis_stages = []

    if run_analysis:
        analysis_stages = [
            'metro_areas',
            'compactness',
            'demographics_analysis',
            'demographics_visualization',
            'political_analysis',
            'political_visualization'
        ]

    all_stages = core_stages + analysis_stages

    # Build command for run_redistricting_stage.py
    scripts_dir = Path(__file__).parent
    stage_runner = scripts_dir / 'run_redistricting_stage.py'

    successful_states = set(states_to_process)
    failed_states = []

    # Run each stage for all states
    for stage_idx, stage_name in enumerate(all_stages, 1):
        print(f"[{year}] Stage {stage_idx}/{len(all_stages)}: {stage_name}", flush=True)

        # Build command
        cmd = [
            sys.executable,
            str(stage_runner),
            '--stage', stage_name,
            '--year', year,
            '--version', version,
            '--output-dir', str(output_dir),
            '--workers', str(max_workers),
            '--dpi', str(args_dict['dpi']),
            '--partition-mode', args_dict['partition_mode']
        ]

        if args_dict.get('print_only'):
            cmd.append('--print-only')
        if args_dict.get('debug'):
            cmd.append('--debug')
        if args_dict.get('reprocess'):
            cmd.append('--reprocess')

        # Add states that haven't failed yet
        if successful_states:
            cmd.append('--states')
            cmd.extend(successful_states)

        # Run stage
        env = os.environ.copy()
        env['TQDM_POSITION'] = '999'

        proc = subprocess.Popen(
            cmd,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1
        )

        # Forward stdout (STATUS messages)
        for line in proc.stdout:
            sys.stdout.write(line)
            sys.stdout.flush()

        proc.wait()

        if proc.returncode != 0:
            # Stage failed - read stderr
            stderr_output = proc.stderr.read() if proc.stderr else ""
            sys.stderr.write(f"[{year}] Stage {stage_name} failed\n")
            if stderr_output:
                sys.stderr.write(stderr_output)
            sys.stderr.flush()

            # Mark all remaining states as failed
            failed_states.extend(successful_states)
            successful_states = set()
            break

    return (list(successful_states), failed_states)


def process_single_state(args_tuple: Tuple) -> Tuple[str, bool]:
    """
    Process a single state in parallel mode.
    Must be at module level for Windows multiprocessing pickling.

    Args:
        args_tuple: (state_code, state_number, year, output_dir, script_args_dict, worker_id)

    Returns:
        (state_code, success_bool)
    """
    state_code, state_number, year, output_dir, args_dict, worker_id = args_tuple

    # Reconstruct paths and config
    STATE_CONFIG = get_state_config(year)
    config = STATE_CONFIG[state_code]
    state_name = config['name']

    states_dir = Path(output_dir) / 'states'
    state_dir = states_dir / state_name.lower().replace(' ', '_')

    # Build command for process_single_state.py
    scripts_dir = Path(__file__).parent
    cmd = [
        sys.executable,
        str(scripts_dir / 'process_single_state.py'),
        '--state', state_code,
        '--year', year,
        '--output-dir', str(state_dir),
        '--dpi', str(args_dict['dpi'])
    ]

    if args_dict.get('print_only'):
        cmd.append('--print-only')
    if args_dict.get('debug'):
        cmd.append('--debug')
    if args_dict.get('run_analysis'):
        cmd.append('--run-analysis')
    if args_dict.get('partition_mode') != 'edge-weighted':
        cmd.extend(['--partition-mode', args_dict['partition_mode']])
    if args_dict.get('reprocess'):
        cmd.append('--reprocess')

    # Set environment variables for STATUS protocol
    env = os.environ.copy()
    env['DPI'] = str(args_dict['dpi'])
    env['STATE_NUMBER'] = str(state_number)
    env['WORKER_ID'] = str(worker_id)
    env['CENSUS_YEAR'] = year  # For YEAR completion messages
    env['TQDM_POSITION'] = str(999)  # Suppress verbose output from child

    # Spawn subprocess and forward STATUS messages
    # Suppress stderr to avoid flickering in hierarchical display
    proc = subprocess.Popen(
        cmd,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,  # Suppress stderr to prevent flicker
        text=True,
        bufsize=1
    )

    # Forward stdout (STATUS messages) to our stdout for orchestrator parsing
    for line in proc.stdout:
        sys.stdout.write(line)
        sys.stdout.flush()

    proc.wait()

    return (state_code, proc.returncode == 0)


def main():
    parser = argparse.ArgumentParser(description='Run state redistricting workers in parallel')
    parser.add_argument('--year', type=str, required=True, choices=['2020', '2010', '2000'],
                       help='Census year')
    parser.add_argument('--version', type=str, required=True,
                       help='Version identifier')
    parser.add_argument('--output-dir', type=str, required=True,
                       help='Output directory for this year')
    parser.add_argument('--workers', type=int, default=4,
                       help='Number of parallel workers')
    parser.add_argument('--dpi', type=int, default=150,
                       help='DPI for output maps')
    parser.add_argument('--partition-mode', type=str, default='edge-weighted',
                       choices=['unweighted', 'edge-weighted'],
                       help='Partitioning mode')
    parser.add_argument('--states', nargs='*', default=None,
                       help='Specific states to process (default: all)')
    parser.add_argument('--reprocess', action='store_true',
                       help='Reprocess all states (ignore completion markers)')
    parser.add_argument('--print-only', action='store_true',
                       help='Print commands without executing')
    parser.add_argument('--debug', action='store_true',
                       help='Enable debug mode')
    parser.add_argument('--run-analysis', action='store_true', default=True,
                       help='Run per-state analysis')
    parser.add_argument('--processing-mode', type=str, default='batch',
                       choices=['batch', 'streaming'],
                       help='Processing mode: "batch" (all states per stage) or "streaming" (per-state pipeline)')

    args = parser.parse_args()

    # Get state configuration for this year
    STATE_CONFIG = get_state_config(args.year)

    # Determine which states to process
    if args.states:
        states_to_process = [s.upper() for s in args.states]
    else:
        states_to_process = sorted(STATE_CONFIG.keys())

    # Check for already completed states (skip logic)
    output_dir = Path(args.output_dir)
    states_complete_marker = output_dir / '.states_complete'

    if states_complete_marker.exists() and not args.reprocess:
        # Read list of completed states
        completed_states = set()
        if states_complete_marker.exists():
            with open(states_complete_marker, 'r') as f:
                completed_states = {line.strip() for line in f if line.strip()}

        # Filter out completed states
        states_to_process = [s for s in states_to_process if s not in completed_states]

        if not states_to_process:
            print(f"[OK] All states already processed for {args.year}")
            return 0

    # Prepare arguments as dict (picklable for ProcessPoolExecutor)
    args_dict = {
        'print_only': args.print_only,
        'debug': args.debug,
        'run_analysis': args.run_analysis,
        'partition_mode': args.partition_mode,
        'dpi': args.dpi,
        'reprocess': args.reprocess
    }

    # Determine max workers
    max_workers = min(args.workers, len(states_to_process), 8)

    # Choose processing mode
    if args.processing_mode == 'batch':
        # Batch mode: all states complete stage N before any state starts N+1
        print(f"[{args.year}] Running in BATCH mode: all states per stage", flush=True)
        successful, failed = run_batch_mode(
            states_to_process,
            args.year,
            args.version,
            output_dir,
            args_dict,
            max_workers,
            args.run_analysis
        )

        # Update completion markers for successful states
        for state_code in successful:
            with open(states_complete_marker, 'a') as f:
                f.write(f"{state_code}\n")

    else:
        # Streaming mode: each state flows through all stages independently
        print(f"[{args.year}] Running in STREAMING mode: per-state pipeline", flush=True)

        # Pre-assign states to workers in round-robin fashion
        state_args_list = [
            (state_code, i, args.year, str(output_dir), args_dict, i % max_workers)
            for i, state_code in enumerate(states_to_process, 1)
        ]

        successful = []
        failed = []

        # Process states in parallel
        with ProcessPoolExecutor(max_workers=max_workers) as executor:
            for state_code, success in executor.map(process_single_state, state_args_list):
                if success:
                    successful.append(state_code)
                    # Emit year-level progress for orchestrator
                    sys.stdout.write(f"STATUS:YEAR:{args.year}:COMPLETE:{len(successful)}/{len(states_to_process)}\n")
                    sys.stdout.flush()

                    # Update completion marker
                    with open(states_complete_marker, 'a') as f:
                        f.write(f"{state_code}\n")
                else:
                    failed.append(state_code)
                    state_name = STATE_CONFIG[state_code]['name']
                    sys.stderr.write(f"[{args.year}] Failed: {state_name}\n")
                    sys.stderr.flush()

    # Report final status
    if failed:
        sys.stderr.write(f"\n[{args.year}] {len(failed)} states failed: {', '.join(failed)}\n")
        sys.stderr.flush()
        return 1
    else:
        sys.stdout.write(f"[{args.year}] All {len(successful)} states completed successfully\n")
        sys.stdout.flush()
        return 0


if __name__ == '__main__':
    sys.exit(main())
