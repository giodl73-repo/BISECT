#!/usr/bin/env python3
"""
Process a single state through the entire pipeline.
Used by parallel execution to run one state per worker.
"""

import sys
import subprocess
import os
from pathlib import Path
from tqdm import tqdm

# Import utility functions
sys.path.insert(0, str(Path(__file__).parent.parent.parent))
from scripts.utils import (
    get_state_config,
    get_election_data_file,
    get_demographic_data_file,
    get_error_logger,
)


def main():
    import argparse
    parser = argparse.ArgumentParser(description='Process a single state')
    parser.add_argument('--state', type=str, required=True)
    parser.add_argument('--year', type=str, default='2020')
    parser.add_argument('--version', type=str, default='v1', help='Version identifier (e.g., v1, test)')
    parser.add_argument('--output-dir', type=str, required=True)
    parser.add_argument('--position', type=int, default=1, help='Progress bar position (1-50)')
    parser.add_argument('--dpi', type=int, default=150, help='DPI for output maps')
    parser.add_argument('--print-only', action='store_true')
    parser.add_argument('--debug', action='store_true')
    parser.add_argument('--run-analysis', action='store_true',
                       help='Run per-state analysis (compactness, political, demographic)')
    parser.add_argument('--partition-mode', type=str, default='edge-weighted', choices=['unweighted', 'edge-weighted', 'metis-vra'],
                       help='Partitioning mode: "edge-weighted" (boundary length minimization, default), "unweighted" (edge cut minimization), or "metis-vra" (VRA-aware multi-constraint)')
    args = parser.parse_args()

    state_code = args.state.upper()

    # If output_dir looks like a state-specific directory already, use it
    # Otherwise, create states/state_name subdirectory structure
    output_path = Path(args.output_dir)

    # Check if this is already a full path (e.g., outputs/us_2020_v1/states/california)
    # or just a base path (e.g., outputs/us_2020_v1)
    if output_path.name in ['states'] or output_path.parent.name == 'states':
        state_dir = output_path
    else:
        # This is the full path to the state directory passed from parent
        state_dir = output_path

    # Load state config using utility function
    try:
        STATE_CONFIG = get_state_config(args.year)
    except (ValueError, ImportError) as e:
        print(f"ERROR: Could not load config for year {args.year}: {e}")
        sys.exit(1)

    config = STATE_CONFIG.get(state_code)
    if not config:
        print(f"ERROR: Unknown state code {state_code}")
        sys.exit(1)

    state_name = config['name']
    num_districts = config['districts']

    # Get position from argument (not environment - more reliable in parallel)
    position = args.position

    # Initialize error logger (try to extract version and year from output path)
    error_logger = None
    try:
        # Extract version and year from output path (e.g., outputs/V9/2020/states/california)
        parts = output_path.parts
        version = None
        year_int = None

        # Look for version (like V9, v1, etc.)
        for part in parts:
            if part.startswith('v') or part.startswith('V'):
                version = part
                break

        # Year is the argument we got
        year_int = int(args.year)

        # Determine base output directory (parent of states/)
        if 'states' in parts:
            states_index = parts.index('states')
            base_output_dir = Path(*parts[:states_index])
        else:
            base_output_dir = output_path

        if version and year_int:
            error_logger = get_error_logger(base_output_dir, version, year_int)
    except Exception:
        # If logger initialization fails, continue without logging (non-fatal)
        pass

    scripts_dir = Path(__file__).parent

    # Define pipeline steps
    # Common flags for all scripts
    common_flags = []
    if args.print_only:
        common_flags.append('--print-only')
    if args.debug:
        common_flags.append('--debug')
    common_flags.append(f'--dpi {args.dpi}')
    common_flags_str = ' '.join(common_flags)

    # Redistricting-specific flags
    redistricting_flags = common_flags.copy()
    if args.partition_mode != 'edge-weighted':
        redistricting_flags.append(f'--partition-mode {args.partition_mode}')
    redistricting_flags_str = ' '.join(redistricting_flags)

    # In parallel mode, suppress child progress bars (use position 999)
    # Only the wrapper shows a status indicator at the assigned position
    child_position = 999  # 999 = hide progress bars

    # Core steps (always run)
    steps = [
        ("Redistricting", f'{sys.executable} {scripts_dir}/run_state_redistricting.py --state {state_code} --year {args.year} --version {args.version} --output-dir {state_dir} --position {child_position} {redistricting_flags_str}'.strip()),
        ("Summary", f'{sys.executable} {scripts_dir}/create_final_district_summary.py {state_dir} --state {state_code} --year {args.year} --version {args.version} --position {child_position} {common_flags_str}'.strip()),
        ("Cities", f'{sys.executable} {scripts_dir}/add_cities_to_districts.py {state_dir} --state {state_code} --year {args.year} --position {child_position} {common_flags_str}'.strip()),
        ("Round maps", f'{sys.executable} {scripts_dir}/visualize_all_rounds.py {state_dir} --state {state_code} --year {args.year} --position {child_position} {common_flags_str}'.strip()),
        ("District maps", f'{sys.executable} {scripts_dir}/visualize_individual_districts.py {state_dir} --state {state_code} --year {args.year} --position {child_position} {common_flags_str}'.strip())
    ]

    # Add optional analysis steps (in order: metro areas, compactness, demographics, political)
    if args.run_analysis:
        # Check data availability using utility functions
        election_data_2020 = get_election_data_file('2020', args.version)
        demographic_data = get_demographic_data_file(args.year, args.version)

        # Metro area visualization (if state has major metros)
        metro_script = Path(__file__).parent / 'visualize_metro_areas.py'
        steps.append((
            "Metro area maps",
            f'{sys.executable} {metro_script} --scope state --state {state_code} --state-dir {state_dir} --year {args.year} --dpi {args.dpi}'.strip()
        ))

        # Compactness visualization (metrics already calculated)
        compactness_script = Path(__file__).parent / 'visualize_compactness.py'
        steps.append((
            "Compactness",
            f'{sys.executable} {compactness_script} --scope state --state {state_code} --state-dir {state_dir} --census-year {args.year} --version {args.version} --dpi {args.dpi} --position {child_position}'.strip()
        ))

        # Demographic analysis (only if demographic data exists for this census year)
        if demographic_data.exists():
            demographic_analyze = Path(__file__).parent / 'analyze_district_demographics.py'
            demographic_visualize = Path(__file__).parent / 'visualize_district_demographics.py'

            steps.append((
                "Demographics analysis",
                f'{sys.executable} {demographic_analyze} {state_dir} --state {state_code} --census-year {args.year}'.strip()
            ))
            steps.append((
                "Demographics visualization",
                f'{sys.executable} {demographic_visualize} --scope state --state {state_code} --state-dir {state_dir} --census-year {args.year} --dpi {args.dpi} --position {child_position}'.strip()
            ))

        # Political analysis (only if compatible election data exists for this census year)
        # Always last in the pipeline
        can_do_political = (args.year == '2020' and election_data_2020.exists())
        if can_do_political:
            political_analyze = Path(__file__).parent / 'analyze_districts.py'
            political_visualize = Path(__file__).parent / 'visualize_partisan_lean.py'

            steps.append((
                "Political analysis",
                f'{sys.executable} {political_analyze} {state_dir} --state {state_code} --year 2020 --census-year {args.year}'.strip()
            ))
            steps.append((
                "Political visualization",
                f'{sys.executable} {political_visualize} --scope state --state {state_code} --state-dir {state_dir} --election-year 2020 --census-year {args.year} --dpi {args.dpi} --skip-rounds --position {child_position}'.strip()
            ))

    # Set up environment for child processes (inherit PARALLEL_MODE)
    env = os.environ.copy()
    env['PYTHONUNBUFFERED'] = '1'  # Ensure STATUS messages reach parent in real-time
    if 'PARALLEL_MODE' not in env:
        env['PARALLEL_MODE'] = '1'

    # Don't create tqdm bars in subprocess - just send status messages to parent
    # Parent will create and manage the bars

    # Check if running in multi-year mode
    is_multi_year = os.environ.get('MULTI_YEAR_SUBPROCESS') == '1'
    census_year = args.year

    # State counter and worker ID for multi-year mode (get from environment if set)
    state_number = int(os.environ.get('STATE_NUMBER', '0'))
    worker_id = int(os.environ.get('WORKER_ID', '0'))

    def send_status(msg, stage=None, stage_total=None, stage_desc=None):
        """Send status update to parent process."""
        if is_multi_year:
            # Hierarchical format for multi-year mode
            if stage is not None:
                # WORKER status
                print(f"STATUS:WORKER:{census_year}:{worker_id}:STATE:{state_number}/50:{state_name}:STAGE:{stage}/{stage_total}:{stage_desc}", flush=True)
            else:
                # Simple message - send as year progress
                pass
        else:
            # Original format for single-year mode
            print(f"STATUS:{position}:{msg}", flush=True)

    # Send starting message
    if not is_multi_year:
        send_status(f"{state_name} [{num_districts}D] Starting...")

    # Process without tqdm
    if True:

        # Process each step
        for i, (step_label, cmd) in enumerate(steps, 1):
            # Send status update to parent
            if is_multi_year:
                send_status(None, stage=i, stage_total=len(steps), stage_desc=step_label.lower().replace(' ', '_'))
            else:
                send_status(f"{state_name} [{num_districts}D] {i}/{len(steps)}: {step_label}")

            try:
                # Use Popen to read progress messages in real-time
                process = subprocess.Popen(
                    cmd, shell=True, env=env,
                    stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                    text=True, bufsize=1
                )

                # Buffer ALL output (not just PROGRESS messages) for error logging
                output_lines = []

                # Read output line by line and look for PROGRESS: messages
                for line in process.stdout:
                    output_lines.append(line)  # Buffer all output

                    if line.startswith("PROGRESS:"):
                        # Parse format: "PROGRESS:15/52"
                        progress = line.split(":", 1)[1].strip()
                        # Send updated status with progress to parent
                        if is_multi_year:
                            send_status(None, stage=i, stage_total=len(steps), stage_desc=f"{step_label.lower().replace(' ', '_')}_{progress}")
                        else:
                            send_status(f"{state_name} [{num_districts}D] {i}/{len(steps)}: {step_label} ({progress})")

                process.wait(timeout=3600)

                if process.returncode == 0:
                    # Emit stage completion message for aggregation (include state_code for tracking)
                    # Normalize step_label to stage_name format (lowercase with underscores)
                    stage_name = step_label.lower().replace(' ', '_')
                    if is_multi_year:
                        print(f"STATUS:STAGE_COMPLETE:{census_year}:{stage_name}:{state_code}", flush=True)

                if process.returncode != 0:
                    send_status(f"{state_name} [{num_districts}D] FAILED at {step_label}")
                    print(f"\n[ERROR] {state_name} failed at {step_label}", file=sys.stderr)
                    print(f"Command: {cmd}", file=sys.stderr)
                    print(f"Return code: {process.returncode}", file=sys.stderr)

                    # Log to error log with full output
                    if error_logger:
                        full_output = ''.join(output_lines)
                        context = {
                            'state': state_name,
                            'state_code': state_code,
                            'step': step_label,
                            'step_number': f'{i}/{len(steps)}'
                        }
                        error_logger.log_command_failure(
                            command=cmd,
                            return_code=process.returncode,
                            output=full_output,
                            task_name=f'{state_name} - {step_label}',
                            context=context
                        )
                        error_logger.write_summary()
                        error_logger.close()

                    sys.exit(1)

            except subprocess.TimeoutExpired:
                send_status(f"{state_name} [{num_districts}D] TIMEOUT at {step_label}")
                print(f"\n[TIMEOUT] {state_name} timed out at {step_label} (60 min limit)", file=sys.stderr)
                print(f"Command: {cmd}", file=sys.stderr)

                # Log timeout to error log
                if error_logger:
                    context = {
                        'state': state_name,
                        'state_code': state_code,
                        'step': step_label,
                        'step_number': f'{i}/{len(steps)}',
                        'timeout_minutes': '60'
                    }
                    error_logger.log_warning(
                        f"Command timed out after 60 minutes: {step_label}",
                        context=context
                    )
                    error_logger.write_summary()
                    error_logger.close()

                # Graceful shutdown: terminate first, force-kill if needed
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                sys.exit(1)

        # Validate outputs after all steps complete
        if not args.print_only:
            from scripts.validation.validate_pipeline_outputs import validate_state_outputs

            validation_result = validate_state_outputs(
                state_dir,
                state_code,
                state_name,
                num_districts,
                args.year,
                check_optional=False,  # Only check required outputs
                verbose=False  # Don't print file-by-file details
            )

            # Determine if we should print validation results
            is_standalone = (args.position == 2)
            is_quiet = (args.position == 999)

            missing_count = sum(1 for r in validation_result['results'] if not r['exists'] and r['required'])

            if missing_count > 0:
                # Always warn about missing files (even in quiet mode)
                if is_standalone or is_quiet:
                    send_status(f"{state_name} [{num_districts}D] WARNING: {missing_count} files missing")
            else:
                # All required outputs exist
                if is_standalone:
                    send_status(f"{state_name} [{num_districts}D] All outputs verified")

        # Show completion
        send_status(f"{state_name} [{num_districts}D] COMPLETE")

    # Close error logger on success (if no errors were logged, file remains minimal)
    if error_logger:
        error_logger.close()

    return 0


if __name__ == '__main__':
    sys.exit(main())
