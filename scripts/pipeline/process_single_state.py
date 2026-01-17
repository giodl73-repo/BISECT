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
)


def main():
    import argparse
    parser = argparse.ArgumentParser(description='Process a single state')
    parser.add_argument('--state', type=str, required=True)
    parser.add_argument('--year', type=str, default='2020')
    parser.add_argument('--output-dir', type=str, required=True)
    parser.add_argument('--position', type=int, default=1, help='Progress bar position (1-50)')
    parser.add_argument('--dpi', type=int, default=150, help='DPI for output maps')
    parser.add_argument('--print-only', action='store_true')
    parser.add_argument('--debug', action='store_true')
    parser.add_argument('--run-analysis', action='store_true',
                       help='Run per-state analysis (compactness, political, demographic)')
    parser.add_argument('--partition-mode', type=str, default='edge-weighted', choices=['unweighted', 'edge-weighted'],
                       help='Partitioning mode: "edge-weighted" (boundary length minimization, default) or "unweighted" (edge cut minimization for comparison)')
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

    steps = [
        ("Redistricting", f'{sys.executable} {scripts_dir}/run_state_redistricting.py --state {state_code} --year {args.year} --output-dir {state_dir} --position {child_position} {redistricting_flags_str}'.strip()),
        ("Cities", f'{sys.executable} {scripts_dir}/add_cities_to_districts.py {state_dir} --state {state_code} --year {args.year} --position {child_position} {common_flags_str}'.strip()),
        ("Summary", f'{sys.executable} {scripts_dir}/create_final_district_summary.py {state_dir} --state {state_code} --year {args.year} --position {child_position} {common_flags_str}'.strip()),
        ("Round maps", f'{sys.executable} {scripts_dir}/visualize_all_rounds.py {state_dir} --state {state_code} --year {args.year} --position {child_position} {common_flags_str}'.strip()),
        ("District maps", f'{sys.executable} {scripts_dir}/create_individual_district_maps.py {state_dir} --state {state_code} --year {args.year} --position {child_position} {common_flags_str}'.strip())
    ]

    # Add optional analysis steps
    if args.run_analysis:
        # Check data availability using utility functions
        # Political analysis requires election data from same time period as census
        # 2020 census -> use 2020 election, 2010 census -> would need 2010/2012 election (not available)
        election_data_2020 = get_election_data_file('2020')
        demographic_data = get_demographic_data_file(args.year)

        # Political analysis (only if compatible election data exists for this census year)
        can_do_political = (args.year == '2020' and election_data_2020.exists())
        if can_do_political:
            political_analyze = Path(__file__).parent.parent / 'political' / 'analyze_districts.py'
            political_visualize = Path(__file__).parent.parent / 'political' / 'visualize_partisan_lean.py'

            steps.append((
                "Political analysis",
                f'{sys.executable} {political_analyze} {state_dir} --state {state_code} --year 2020 --census-year {args.year}'.strip()
            ))
            steps.append((
                "Political visualization",
                f'{sys.executable} {political_visualize} --scope state --state {state_code} --state-dir {state_dir} --election-year 2020 --census-year {args.year} --dpi {args.dpi} --skip-rounds --position {child_position}'.strip()
            ))

        # Demographic analysis (only if demographic data exists for this census year)
        if demographic_data.exists():
            demographic_analyze = Path(__file__).parent.parent / 'demographic' / 'analyze_district_demographics.py'
            demographic_visualize = Path(__file__).parent.parent / 'demographic' / 'visualize_district_demographics.py'

            steps.append((
                "Demographic analysis",
                f'{sys.executable} {demographic_analyze} {state_dir} --state {state_code} --census-year {args.year}'.strip()
            ))
            steps.append((
                "Demographic visualization",
                f'{sys.executable} {demographic_visualize} --scope state --state {state_code} --state-dir {state_dir} --census-year {args.year} --dpi {args.dpi} --position {child_position}'.strip()
            ))

        # Compactness visualization (metrics already calculated)
        compactness_script = Path(__file__).parent.parent / 'compactness' / 'visualize_compactness.py'
        steps.append((
            "Compactness visualization",
            f'{sys.executable} {compactness_script} --scope state --state {state_code} --state-dir {state_dir} --census-year {args.year} --dpi {args.dpi} --position {child_position}'.strip()
        ))

        # Metro area visualization (if state has major metros)
        metro_script = Path(__file__).parent.parent / 'visualization' / 'create_metro_area_maps.py'
        steps.append((
            "Metro area maps",
            f'{sys.executable} {metro_script} --scope state --state {state_code} --state-dir {state_dir} --year {args.year} --dpi {args.dpi}'.strip()
        ))

    # Set up environment for child processes (inherit PARALLEL_MODE)
    env = os.environ.copy()
    if 'PARALLEL_MODE' not in env:
        env['PARALLEL_MODE'] = '1'

    # Don't create tqdm bars in subprocess - just send status messages to parent
    # Parent will create and manage the bars
    def send_status(msg):
        """Send status update to parent process."""
        print(f"STATUS:{position}:{msg}", flush=True)

    # Send starting message
    send_status(f"{state_name} [{num_districts}D] Starting...")

    # Process without tqdm
    if True:

        # Process each step
        for i, (step_label, cmd) in enumerate(steps, 1):
            # Send status update to parent
            send_status(f"{state_name} [{num_districts}D] {i}/{len(steps)}: {step_label}")

            try:
                # Use Popen to read progress messages in real-time
                process = subprocess.Popen(
                    cmd, shell=True, env=env,
                    stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                    text=True, bufsize=1
                )

                # Read output line by line and look for PROGRESS: messages
                for line in process.stdout:
                    if line.startswith("PROGRESS:"):
                        # Parse format: "PROGRESS:15/52"
                        progress = line.split(":", 1)[1].strip()
                        # Send updated status with progress to parent
                        send_status(f"{state_name} [{num_districts}D] {i}/{len(steps)}: {step_label} ({progress})")

                process.wait(timeout=3600)

                if process.returncode != 0:
                    send_status(f"{state_name} [{num_districts}D] FAILED at {step_label}")
                    print(f"\n[ERROR] {state_name} failed at {step_label}", file=sys.stderr)
                    print(f"Command: {cmd}", file=sys.stderr)
                    print(f"Return code: {process.returncode}", file=sys.stderr)
                    sys.exit(1)

            except subprocess.TimeoutExpired:
                send_status(f"{state_name} [{num_districts}D] TIMEOUT at {step_label}")
                print(f"\n[TIMEOUT] {state_name} timed out at {step_label} (60 min limit)", file=sys.stderr)
                print(f"Command: {cmd}", file=sys.stderr)
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

    return 0


if __name__ == '__main__':
    sys.exit(main())
