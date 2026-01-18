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
from pathlib import Path
from tqdm import tqdm

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
    args = parser.parse_args()

    output_dir = Path(args.output_dir)
    scripts_dir = Path(__file__).parent

    print("\n" + "="*70)
    print("NATIONAL POST-PROCESSING")
    print("="*70)
    print(f"Census Year: {args.year}")
    print(f"Output directory: {output_dir}")
    print(f"Version: {args.version}")
    if args.print_only:
        print("Mode: PRINT-ONLY (debug mode - no execution)")
    print("="*70)

    pipeline_steps = []

    # Create US national maps FIRST (most important visualization)
    national_map_script = scripts_dir / 'visualize_national_districts.py'
    if national_map_script.exists():
        flags = []
        if args.print_only:
            flags.append('--print-only')
        if args.debug:
            flags.append('--debug')
        flags_str = ' '.join(flags)
        pipeline_steps.append({
            'name': 'Create US national maps',
            'command': f'{sys.executable} {scripts_dir}/visualize_national_districts.py --year {args.year} --output-dir {output_dir} --dpi {args.dpi} {flags_str}'.strip(),
            'critical': False
        })

    # Create US aggregate files
    if output_dir.exists() or args.print_only:
        flags = []
        if args.print_only:
            flags.append('--print-only')
        if args.debug:
            flags.append('--debug')
        flags_str = ' '.join(flags)
        pipeline_steps.append({
            'name': 'Create US aggregate files',
            'command': f'{sys.executable} {scripts_dir}/create_us_aggregate.py --year {args.year} --version {args.version} --dpi {args.dpi} --output-dir {output_dir} --skip-maps {flags_str}'.strip(),
            'critical': False
        })

    # Create US rounds hierarchy aggregate
    if output_dir.exists() or args.print_only:
        flags = []
        if args.print_only:
            flags.append('--print-only')
        if args.debug:
            flags.append('--debug')
        flags_str = ' '.join(flags)
        pipeline_steps.append({
            'name': 'Create US rounds hierarchy',
            'command': f'{sys.executable} {scripts_dir}/create_us_rounds_hierarchy.py --output-dir {output_dir} {flags_str}'.strip(),
            'critical': False
        })

    # Create national round progression maps
    national_rounds_script = scripts_dir / 'visualize_national_rounds.py'
    if national_rounds_script.exists() and (output_dir.exists() or args.print_only):
        pipeline_steps.append({
            'name': 'Create national round progression maps',
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

    # Run political analysis on all states (batch mode - fallback only)
    # Note: Only runs if --skip-analysis was used (per-state analysis didn't run)
    if not args.skip_political and not args.run_analysis and election_data_available and (output_dir.exists() or args.print_only):
        political_scripts = scripts_dir
        analyze_script = political_scripts / 'analyze_districts.py'
        visualize_script = political_scripts / 'visualize_partisan_lean.py'

        if analyze_script.exists() and visualize_script.exists():
            # Get list of state directories
            states_dir = output_dir / 'states'
            if states_dir.exists() or args.print_only:
                # Add step to run political analysis on all states (legacy batch mode)
                pipeline_steps.append({
                    'name': 'Political analysis (batch fallback)',
                    'command': f'{sys.executable} {political_scripts}/run_political_analysis.py --census-year {args.year} --version {args.version} --election-year {args.election_year}'.strip(),
                    'critical': False
                })

    # Create national political map (after per-state analysis completes)
    if not args.skip_political and election_data_available and (output_dir.exists() or args.print_only):
        pipeline_steps.append({
            'name': 'Create national political map',
            'command': f'{sys.executable} scripts/pipeline/visualize_partisan_lean.py --scope national --output-dir {output_dir} --version {args.version} --election-year {args.election_year} --census-year {args.year} --dpi {args.dpi}'.strip(),
            'critical': False
        })

    # Run demographic analysis on all states (batch mode - fallback only)
    # Note: Only runs if --skip-analysis was used (per-state analysis didn't run)
    if not args.skip_demographic and not args.run_analysis and demographic_data_available and (output_dir.exists() or args.print_only):
        demographic_scripts = scripts_dir
        demographic_script = demographic_scripts / 'run_demographic_analysis.py'

        if demographic_script.exists():
            # Get list of state directories
            states_dir = output_dir / 'states'
            if states_dir.exists() or args.print_only:
                # Add step to run demographic analysis on all states (legacy batch mode)
                pipeline_steps.append({
                    'name': 'Demographic analysis (batch fallback)',
                    'command': f'{sys.executable} {demographic_scripts}/run_demographic_analysis.py --census-year {args.year} --version {args.version}'.strip(),
                    'critical': False
                })

    # Run demographic visualization on all states (batch mode - fallback only)
    # Note: Only runs if --skip-analysis was used (per-state visualization didn't run)
    if not args.skip_demographic and not args.run_analysis and demographic_data_available and (output_dir.exists() or args.print_only):
        demographic_scripts = scripts_dir
        demographic_viz_script = demographic_scripts / 'run_demographic_visualization.py'

        if demographic_viz_script.exists():
            # Get list of state directories
            states_dir = output_dir / 'states'
            if states_dir.exists() or args.print_only:
                # Add step to run demographic visualization on all states (legacy batch mode)
                pipeline_steps.append({
                    'name': 'Demographic visualization (batch fallback)',
                    'command': f'{sys.executable} {demographic_scripts}/run_demographic_visualization.py --census-year {args.year} --version {args.version} --dpi {args.dpi}'.strip(),
                    'critical': False
                })

    # Create national demographic map (after per-state analysis completes)
    if not args.skip_demographic and demographic_data_available and (output_dir.exists() or args.print_only):
        pipeline_steps.append({
            'name': 'Create national demographic map',
            'command': f'{sys.executable} scripts/pipeline/visualize_district_demographics.py --scope national --output-dir {output_dir} --version {args.version} --census-year {args.year} --dpi {args.dpi}'.strip(),
            'critical': False
        })

    # Run compactness visualization
    # Note: If --run-analysis is enabled, per-state visualizations already ran during state processing
    # This step creates the national aggregation map
    if output_dir.exists() or args.print_only:
        compactness_script = scripts_dir / 'visualize_compactness.py'
        pipeline_steps.append({
            'name': 'Create national compactness map',
            'command': f'{sys.executable} {compactness_script} --scope national --output-dir {output_dir} --version {args.version} --census-year {args.year} --dpi {args.dpi}'.strip(),
            'critical': False
        })

    # Create metro area maps for major MSAs
    if not args.run_analysis and (output_dir.exists() or args.print_only):
        # Batch fallback mode (legacy) - only if --skip-analysis was used
        metro_viz_script = Path('scripts/pipeline/visualize_metro_areas.py')
        if metro_viz_script.exists():
            pipeline_steps.append({
                'name': 'Create metro area district maps (batch fallback)',
                'command': f'{sys.executable} {metro_viz_script} --scope all --year {args.year} --version {args.version} --output-dir {output_dir} --dpi {args.dpi}'.strip(),
                'critical': False
            })
    elif args.run_analysis and (output_dir.exists() or args.print_only):
        # Per-state mode - metros already created, just report completion
        metro_viz_script = Path('scripts/pipeline/visualize_metro_areas.py')
        if metro_viz_script.exists():
            pipeline_steps.append({
                'name': 'Metro area maps (completed per-state)',
                'command': f'{sys.executable} {metro_viz_script} --scope national --output-dir {output_dir} --version {args.version} --year {args.year}'.strip(),
                'critical': False
            })

    # Generate static dashboard with all district data
    if output_dir.exists() or args.print_only:
        dashboard_script = Path('scripts/web/generate_dashboard.py')
        if dashboard_script.exists():
            pipeline_steps.append({
                'name': 'Generate static dashboard',
                'command': f'{sys.executable} {dashboard_script} --year {args.year} --version {args.version} --output-dir {output_dir}'.strip(),
                'critical': False
            })

    # Run post-processing steps with progress bars
    if pipeline_steps:
        # Create progress bars for each post-processing step at positions 0-3
        step_bars = []
        for i, step in enumerate(pipeline_steps):
            bar = tqdm(total=1,
                      desc=f"[{i}] {step['name']} - Waiting...",
                      unit="step",
                      position=i,
                      ncols=120,
                      leave=True,
                      bar_format="{desc}",
                      dynamic_ncols=False,
                      file=sys.stderr)
            step_bars.append(bar)

        # Run each step (set TQDM_POSITION to suppress child banners)
        for i, step in enumerate(pipeline_steps):
            # Update bar to show starting
            step_bars[i].set_description_str(f"[{i}] {step['name']} - Starting...".ljust(120))
            step_bars[i].refresh()

            env = os.environ.copy()
            env['TQDM_POSITION'] = str(i)  # Pass position so child can report progress

            # Use Popen to monitor output in real-time
            proc = subprocess.Popen(step['command'], shell=True, env=env,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                   text=True, bufsize=1)

            # Non-blocking monitor thread for STATUS messages
            def monitor_output(proc, bar_index):
                try:
                    for line in proc.stdout:
                        line = line.strip()
                        if line.startswith("STATUS:"):
                            # Parse: STATUS:position:message
                            parts = line.split(":", 2)
                            if len(parts) >= 3:
                                msg = parts[2]
                                step_bars[bar_index].set_description_str(f"[{bar_index}] {msg}".ljust(120))
                                step_bars[bar_index].refresh()
                except:
                    pass

            thread = threading.Thread(target=monitor_output, args=(proc, i), daemon=True)
            thread.start()

            # Wait with timeout (15 minutes for slow operations like national map rendering)
            try:
                proc.wait(timeout=900)
            except subprocess.TimeoutExpired:
                proc.kill()
                step_bars[i].set_description_str(f"[{i}] {step['name']} - TIMEOUT (15 min)".ljust(120))
                step_bars[i].refresh()
                if step['critical'] and not args.print_only:
                    print(f"\n[ERROR] {step['name']} timed out after 15 minutes", file=sys.stderr)
                    return 1
                continue

            if proc.returncode == 0:
                step_bars[i].set_description_str(f"[{i}] {step['name']} - COMPLETE".ljust(120))
            else:
                step_bars[i].set_description_str(f"[{i}] {step['name']} - FAILED".ljust(120))
                if step['critical'] and not args.print_only:
                    print(f"\n[ERROR] {step['name']} failed", file=sys.stderr)
                    stderr_output = proc.stderr.read()
                    print(stderr_output[-500:] if stderr_output else "", file=sys.stderr)
                    return 1

            step_bars[i].refresh()
            step_bars[i].update(1)

        # Close bars
        for bar in step_bars:
            bar.close()

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
