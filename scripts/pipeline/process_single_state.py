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

    # Load state config
    sys.path.insert(0, str(Path(__file__).parent.parent.parent))
    try:
        if args.year == '2020':
            from config_2020 import STATE_CONFIG_2020
            STATE_CONFIG = STATE_CONFIG_2020
        elif args.year == '2010':
            from config_2010 import STATE_CONFIG_2010
            STATE_CONFIG = STATE_CONFIG_2010
        else:
            print(f"ERROR: Year {args.year} not supported")
            sys.exit(1)
    except ImportError:
        print(f"ERROR: Could not load config for year {args.year}")
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
    flags = []
    if args.print_only:
        flags.append('--print-only')
    if args.debug:
        flags.append('--debug')
    flags.append(f'--dpi {args.dpi}')
    flags_str = ' '.join(flags)

    # In parallel mode, suppress child progress bars (use position 999)
    # Only the wrapper shows a status indicator at the assigned position
    child_position = 999  # 999 = hide progress bars

    steps = [
        ("Redistricting", f'{sys.executable} {scripts_dir}/run_state_redistricting.py --state {state_code} --year {args.year} --output-dir {state_dir} --position {child_position} {flags_str}'.strip()),
        ("Cities", f'{sys.executable} {scripts_dir}/add_cities_to_districts.py {state_dir} --year {args.year} --position {child_position} {flags_str}'.strip()),
        ("Summary", f'{sys.executable} {scripts_dir}/create_final_district_summary.py {state_dir} --year {args.year} --position {child_position} {flags_str}'.strip()),
        ("Round maps", f'{sys.executable} {scripts_dir}/visualize_all_rounds.py {state_dir} --year {args.year} --position {child_position} {flags_str}'.strip()),
        ("District maps", f'{sys.executable} {scripts_dir}/create_individual_district_maps.py {state_dir} --year {args.year} --position {child_position} {flags_str}'.strip())
    ]

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

        # Show completion
        send_status(f"{state_name} [{num_districts}D] COMPLETE")

    return 0


if __name__ == '__main__':
    sys.exit(main())
