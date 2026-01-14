#!/usr/bin/env python3
"""
Generate unified master dashboard landing page for multiple census years and versions.

Scans outputs directory for all us_{year}_{version} directories and creates
a simple landing page with a dropdown selector to navigate to each run's dashboard.
"""

import json
import sys
from pathlib import Path
import re


def find_all_runs(outputs_dir='outputs'):
    """Find all us_{year}_{version} directories."""
    outputs_path = Path(outputs_dir)
    if not outputs_path.exists():
        return []

    runs = []
    pattern = re.compile(r'us_(\d{4})_(v\d+)(_noedge)?$')

    for item in outputs_path.iterdir():
        if not item.is_dir():
            continue

        match = pattern.match(item.name)
        if match:
            year = match.group(1)
            version = match.group(2)
            noedge_suffix = match.group(3) or ''

            # Check if this run has states
            states_dir = item / 'states'
            if not states_dir.exists():
                continue

            num_states = sum(1 for state_dir in states_dir.iterdir()
                           if state_dir.is_dir() and (state_dir / 'district_summary.csv').exists())

            if num_states == 0:
                continue

            # Determine mode
            if noedge_suffix:
                mode = 'Unweighted'
            else:
                mode = 'Edge-Weighted'

            runs.append({
                'year': year,
                'version': version,
                'version_full': version + noedge_suffix,
                'mode': mode,
                'num_states': num_states,
                'path': item.name,
                'sort_key': (year, version, noedge_suffix)
            })

    # Sort by year (desc), then version, then mode
    runs.sort(key=lambda r: r['sort_key'], reverse=True)

    return runs


def generate_master_dashboard(output_file='outputs/index.html', template_file='web/master_dashboard.html'):
    """Generate master dashboard landing page."""

    template_path = Path(template_file)
    output_path = Path(output_file)

    # Check template exists
    if not template_path.exists():
        print(f"ERROR: Template not found: {template_path}")
        return 1

    # Find all runs
    print("Scanning for runs...")
    runs = find_all_runs()

    if not runs:
        print("ERROR: No valid runs found in outputs directory")
        return 1

    # Print found runs
    print(f"\nFound {len(runs)} runs:")
    for run in runs:
        print(f"  {run['year']} {run['version_full']:<12} - {run['mode']:<15} ({run['num_states']} states)")

    # Read template
    with open(template_path, 'r', encoding='utf-8') as f:
        html = f.read()

    # Generate dropdown options HTML
    options_html = []

    # Group by year
    current_year = None
    for run in runs:
        if run['year'] != current_year:
            if current_year is not None:
                options_html.append('')  # Empty line between years (for readability)
            current_year = run['year']

        # Format label
        label = f"{run['year']} - {run['version_full']}"
        if run['mode']:
            label += f" ({run['mode']})"
        label += f" - {run['num_states']} states"

        options_html.append(f'                <option value="{run["path"]}">{label}</option>')

    # Insert options into template
    html = html.replace('<!-- OPTIONS_PLACEHOLDER -->', '\n'.join(options_html))

    # Insert run data JSON
    runs_json = json.dumps(runs, indent=8)
    html = html.replace('/* RUNS_DATA_PLACEHOLDER */', runs_json)

    # Load and embed comparison data if it exists
    comparison_path = output_path.parent / 'comparison.json'
    if comparison_path.exists():
        with open(comparison_path, 'r', encoding='utf-8') as f:
            comparison_data = json.load(f)
        comparison_json = json.dumps(comparison_data, indent=8)
        html = html.replace('/* COMPARISON_DATA_PLACEHOLDER */', comparison_json)
        print(f"  Embedded comparison data from {comparison_path}")
    else:
        # Use empty object if no comparison data
        html = html.replace('/* COMPARISON_DATA_PLACEHOLDER */', '{}')
        print(f"  Warning: comparison.json not found at {comparison_path}")

    # Write output
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write(html)

    # Also write runs.json for dynamic loading by individual dashboards
    runs_json_path = output_path.parent / 'runs.json'
    with open(runs_json_path, 'w', encoding='utf-8') as f:
        json.dump(runs, f, indent=2)

    print(f"\nSUCCESS: Master dashboard generated: {output_path}")
    print(f"  Years: {', '.join(sorted(set(r['year'] for r in runs)))}")
    print(f"  Total runs: {len(runs)}")
    print(f"  Runs list: {runs_json_path}")

    return 0


def regenerate_all_dashboards(runs):
    """Regenerate dashboards for all runs."""
    import subprocess

    print("\nRegenerating all dashboards...")

    for run in runs:
        year = run['year']
        version = run['version']
        mode = 'unweighted' if run['mode'] == 'Unweighted' else 'edge-weighted'

        print(f"  Generating {year} {run['version_full']} ({run['mode']})...")

        cmd = [
            sys.executable,
            'scripts/web/generate_dashboard.py',
            '--year', year,
            '--version', version,
            '--partition-mode', mode
        ]

        result = subprocess.run(cmd, capture_output=True, text=True)

        if result.returncode != 0:
            print(f"    Warning: Failed to generate {year} {run['version_full']}")
            if result.stderr:
                print(f"    Error: {result.stderr}")

    print(f"  Regenerated {len(runs)} dashboards")


if __name__ == '__main__':
    import argparse

    parser = argparse.ArgumentParser(description='Generate master dashboard landing page and regenerate all individual dashboards')
    parser.add_argument('--output', type=str, default='outputs/index.html',
                       help='Output file path (default: outputs/index.html)')
    parser.add_argument('--template', type=str, default='web/master_dashboard.html',
                       help='Dashboard template file (default: web/master_dashboard.html)')
    parser.add_argument('--skip-dashboards', action='store_true',
                       help='Skip regenerating individual dashboards (faster, only updates master)')

    args = parser.parse_args()

    # First, generate master dashboard and runs.json
    result = generate_master_dashboard(args.output, args.template)

    if result != 0:
        sys.exit(result)

    # Then regenerate all individual dashboards (unless --skip-dashboards)
    if not args.skip_dashboards:
        runs = find_all_runs()
        if runs:
            regenerate_all_dashboards(runs)
        else:
            print("\nNo runs found to regenerate")
    else:
        print("\nSkipped dashboard regeneration (--skip-dashboards flag)")

    sys.exit(0)
