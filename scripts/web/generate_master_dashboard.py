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

# Import scan_artifacts from generate_dashboard
from generate_dashboard import scan_artifacts


def find_all_runs(outputs_dir='outputs'):
    """Find all {version}/{year} directories in new structure."""
    outputs_path = Path(outputs_dir)
    if not outputs_path.exists():
        return []

    runs = []

    # Scan for version directories (v1, V2, V3, etc.)
    version_pattern = re.compile(r'^[vV]\d+$', re.IGNORECASE)

    for version_dir in outputs_path.iterdir():
        if not version_dir.is_dir():
            continue

        # Skip special directories
        if version_dir.name in ['artifacts', 'experiments', 'dev', 'test', 'test_post']:
            continue

        # Check if this is a version directory
        if not version_pattern.match(version_dir.name):
            continue

        version = version_dir.name.lower()  # Normalize to lowercase (v1, v2, v3)

        # Read version.json if it exists
        version_json_path = version_dir / 'version.json'
        version_metadata = {}
        if version_json_path.exists():
            try:
                with open(version_json_path, 'r', encoding='utf-8') as f:
                    version_metadata = json.load(f)
            except Exception as e:
                print(f"Warning: Could not read {version_json_path}: {e}")

        # Scan for year subdirectories (2000, 2010, 2020)
        year_pattern = re.compile(r'^(2000|2010|2020)$')

        for year_dir in version_dir.iterdir():
            if not year_dir.is_dir():
                continue

            # Check if this is a year directory
            if not year_pattern.match(year_dir.name):
                continue

            year = year_dir.name

            # Check if this run has states
            states_dir = year_dir / 'states'
            if not states_dir.exists():
                continue

            # Count states with district_summary.csv
            num_states = 0
            for state_dir in states_dir.iterdir():
                if not state_dir.is_dir():
                    continue
                # Check for district_summary.csv in data/ subdirectory
                if (state_dir / 'data' / 'district_summary.csv').exists():
                    num_states += 1

            if num_states == 0:
                continue

            # Read config.json to get partition mode
            config_path = year_dir / 'config.json'
            partition_mode = 'edge_weighted'  # Default
            if config_path.exists():
                try:
                    with open(config_path, 'r', encoding='utf-8') as f:
                        config = json.load(f)
                        partition_mode = config.get('algorithm', {}).get('partition_mode', 'edge_weighted')
                except Exception as e:
                    print(f"Warning: Could not read {config_path}: {e}")

            # Determine mode label
            if partition_mode == 'unweighted':
                mode = 'Unweighted'
            else:
                mode = 'Edge-Weighted'

            # Build relative path for linking
            relative_path = f"{version}/{year}"

            runs.append({
                'year': year,
                'version': version,
                'version_full': version,  # No more _noedge suffix
                'mode': mode,
                'num_states': num_states,
                'path': relative_path,
                'partition_mode': partition_mode,
                'sort_key': (year, version, partition_mode)
            })

    # Sort by year (desc), then version
    runs.sort(key=lambda r: (r['year'], r['version']), reverse=True)

    return runs


def build_versions_data(runs):
    """Build VERSIONS data structure from runs list."""
    versions_dict = {}

    for run in runs:
        version = run['version']

        if version not in versions_dict:
            versions_dict[version] = {
                'name': version,
                'description': f'Redistricting variant: {version}',
                'partition_mode': run['partition_mode'],
                'data_level': 'tract',  # Default, can be extended
                'num_runs': 0,
                'completed_years': [],
                'runs': []
            }

        versions_dict[version]['num_runs'] += 1
        if run['year'] not in versions_dict[version]['completed_years']:
            versions_dict[version]['completed_years'].append(run['year'])

        versions_dict[version]['runs'].append({
            'year': run['year'],
            'mode': run['mode'],
            'num_states': run['num_states'],
            'path': run['path'] + '/index.html',
            'created': None  # Could read from config.json if needed
        })

    # Sort completed_years for each version
    for version_data in versions_dict.values():
        version_data['completed_years'].sort()

    # Convert to list and sort by version
    versions_list = list(versions_dict.values())
    versions_list.sort(key=lambda v: v['name'])

    return versions_list


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

    # Build and insert versions data
    versions = build_versions_data(runs)
    versions_json = json.dumps(versions, indent=8)
    html = html.replace('/* VERSIONS_DATA_PLACEHOLDER */', versions_json)

    # Scan and embed artifacts data
    print(f"\n  Scanning artifacts...")
    # For new structure, artifacts are at outputs/artifacts (sibling to version dirs)
    # Master dashboard is at outputs/index.html
    # scan_artifacts expects output_dir.parent to be outputs/, so we pass any subdirectory
    # Use first available run directory, or create a dummy path
    if runs:
        scan_base = output_path.parent / runs[0]['path']
    else:
        # Create a dummy path so output_dir.parent = outputs/
        scan_base = output_path.parent / 'dummy'

    artifacts = scan_artifacts(scan_base)

    # Fix paths for master dashboard: remove '../' prefix since master is at outputs/index.html (same level as artifacts/)
    # scan_artifacts generates paths like '../artifacts/...' for individual dashboards
    # but master dashboard needs 'artifacts/...' (no '../')
    for guide in artifacts['guides']:
        guide['path'] = guide['path'].replace('../', '', 1)
    for pres in artifacts['presentations']:
        pres['path'] = pres['path'].replace('../', '', 1)
    for paper in artifacts['papers']:
        paper['path'] = paper['path'].replace('../', '', 1)
    for fig in artifacts['figures']['schematic']:
        fig['path'] = fig['path'].replace('../', '', 1)
    for fig in artifacts['figures']['real_tracts']:
        fig['path'] = fig['path'].replace('../', '', 1)
    for fig in artifacts['figures']['round_progression']:
        fig['path'] = fig['path'].replace('../', '', 1)

    artifacts_json = json.dumps(artifacts, indent=8)
    html = html.replace('/* ARTIFACTS_DATA_PLACEHOLDER */', artifacts_json)

    pdf_count = len(artifacts['guides']) + len(artifacts['presentations']) + len(artifacts['papers'])
    fig_count = (len(artifacts['figures']['schematic']) +
                 len(artifacts['figures']['real_tracts']) +
                 len(artifacts['figures']['round_progression']))
    print(f"  Found {pdf_count} PDFs and {fig_count} figures")

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
