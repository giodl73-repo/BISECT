#!/usr/bin/env python3
"""
Generate static dashboard HTML with district data baked in.

This script reads the dashboard template and injects district city data
for all states, creating a fully static HTML file.

Usage:
    python scripts/web/generate_dashboard.py --year 2020 --version v1
"""

import argparse
import json
import os
import sys
from pathlib import Path
import pandas as pd


def load_district_data(state_dir):
    """Load district cities data for a state."""
    cities_file = state_dir / 'district_cities.csv'
    if not cities_file.exists():
        return None

    try:
        df = pd.read_csv(cities_file)

        # Build district data array
        districts = []
        for _, row in df.iterrows():
            district_num = str(row['district'])
            city_name = row['largest_city']

            # Handle NaN or numeric values
            if pd.isna(city_name) or not isinstance(city_name, str):
                city_name = f"District {district_num}"

            # Create filename-safe version of city name
            city_filename = city_name.lower()
            city_filename = city_filename.replace(' ', '_')
            city_filename = city_filename.replace('.', '')
            city_filename = city_filename.replace(',', '')
            city_filename = city_filename.replace("'", '')

            districts.append({
                'num': district_num,
                'city': city_name,
                'filename': city_filename
            })

        return districts
    except Exception as e:
        print(f"  Warning: Error loading district data for {state_dir.name}: {e}")
        return None


def find_all_runs(outputs_dir='outputs'):
    """Find all us_{year}_{version} directories for run selector."""
    import re

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
            year_val = match.group(1)
            version_val = match.group(2)
            noedge_suffix = match.group(3) or ''

            # Check if this run has an index.html
            if not (item / 'index.html').exists():
                continue

            # Determine mode
            mode = 'Unweighted' if noedge_suffix else 'Edge-Weighted'

            runs.append({
                'year': year_val,
                'version': version_val,
                'version_full': version_val + noedge_suffix,
                'mode': mode,
                'path': item.name,
                'sort_key': (year_val, version_val, noedge_suffix)
            })

    # Sort by year (desc), then version, then mode
    runs.sort(key=lambda r: r['sort_key'], reverse=True)
    return runs


def generate_dashboard(year, version, partition_mode='normal', output_dir=None, template_file=None):
    """Generate static dashboard HTML.

    Args:
        year: Census year
        version: Version string (e.g., 'v1', 'v2')
        partition_mode: 'unweighted' or 'edge-weighted'
        output_dir: Output directory (if None, auto-generated)
        template_file: Dashboard template file
    """

    # Default paths
    if template_file is None:
        template_file = Path('web/dashboard.html')

    if output_dir is None:
        # Auto-generate output directory based on partition mode
        # Match main pipeline convention: edge-weighted is default (no suffix), unweighted adds _noedge
        version_suffix = f"{version}_noedge" if partition_mode == 'unweighted' else version
        output_dir = Path(f'outputs/us_{year}_{version_suffix}')
    else:
        output_dir = Path(output_dir)

    # Check paths
    if not template_file.exists():
        print(f"ERROR: Template file not found: {template_file}")
        return 1

    if not output_dir.exists():
        print(f"ERROR: Output directory not found: {output_dir}")
        return 1

    mode_display = "edge-weighted" if partition_mode == 'edge-weighted' else "unweighted"
    print(f"Generating dashboard for {year} Census, version {version} ({mode_display} mode)...")
    print(f"  Template: {template_file}")
    print(f"  Output: {output_dir}")

    # Read template
    with open(template_file, 'r', encoding='utf-8') as f:
        template_html = f.read()

    # Build DISTRICT_DATA object
    district_data = {}
    states_dir = output_dir / 'states'

    if not states_dir.exists():
        print(f"  Warning: States directory not found: {states_dir}")
        print("  Dashboard will be generated without district data")
    else:
        print(f"\n  Loading district data from states...")
        loaded = 0
        skipped = 0

        for state_dir in sorted(states_dir.iterdir()):
            if not state_dir.is_dir():
                continue

            state_name = state_dir.name
            districts = load_district_data(state_dir)

            if districts:
                district_data[state_name] = districts
                loaded += 1
            else:
                skipped += 1

        print(f"  Loaded district data for {loaded} states")
        if skipped > 0:
            print(f"  Skipped {skipped} states (no district_cities.csv)")

    # Generate JavaScript variable
    district_data_js = f"const DISTRICT_DATA = {json.dumps(district_data, indent=8)};"

    # Inject run selector into header (before papers dropdown)
    import re

    current_path = output_dir.name  # us_2020_v1, etc.

    # Load runs from runs.json (or find them if it doesn't exist)
    runs_json_path = Path('outputs/runs.json')
    if runs_json_path.exists():
        with open(runs_json_path, 'r') as f:
            all_runs = json.load(f)
    else:
        all_runs = find_all_runs()

    # Generate options HTML
    options_html = []
    for run in all_runs:
        label = f"{run['year']} - {run['version_full']} ({run['mode']})"
        selected = ' selected' if run['path'] == current_path else ''
        options_html.append(f'                    <option value="{run["path"]}"{selected}>{label}</option>')

    # Run selector HTML (pre-populated with all runs)
    run_selector_html = f'''<select id="runSelector" class="header-select" onchange="switchRun(this.value)" style="margin-right: 1rem;">
{chr(10).join(options_html)}
                </select>
                '''

    # Inject before papers dropdown
    papers_dropdown_pattern = r'(<div class="dropdown" style="position: relative; display: inline-block;">[\s\S]*?📄 Research Papers)'
    dashboard_html = re.sub(
        papers_dropdown_pattern,
        run_selector_html + r'\1',
        template_html,
        count=1
    )

    # Add switchRun function to JavaScript (simple, no async loading needed)
    switch_run_js = f'''
        function switchRun(runPath) {{
            if (runPath) {{
                window.location.href = '../' + runPath + '/index.html';
            }}
        }}
    '''

    # Inject before closing </script> tag
    dashboard_html = dashboard_html.replace('</script>', switch_run_js + '\n    </script>')

    # Inject district data into template
    # Find the line with the comment about DISTRICT_DATA
    injection_comment = "// Note: DISTRICT_DATA will be injected here during deployment"

    if injection_comment in dashboard_html:
        # Replace the comment with the actual data
        dashboard_html = dashboard_html.replace(
            injection_comment,
            f"// District data generated by generate_dashboard.py\n        {district_data_js}"
        )
    else:
        print("  Warning: Injection comment not found in template, appending to script")
        # Fallback: inject before closing </script> tag
        dashboard_html = template_html.replace(
            "    </script>",
            f"\n        // District data generated by generate_dashboard.py\n        {district_data_js}\n    </script>"
        )

    # Write output
    output_file = output_dir / 'index.html'
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write(dashboard_html)

    print(f"\nDashboard generated: {output_file}")
    print(f"  File size: {len(dashboard_html):,} bytes")
    print(f"  District data: {len(district_data)} states")

    return 0


def main():
    parser = argparse.ArgumentParser(
        description='Generate static dashboard HTML with district data'
    )
    parser.add_argument('--year', type=str, required=True,
                       choices=['2000', '2010', '2020', '2030', '2040'],
                       help='Census year')
    parser.add_argument('--version', type=str, required=True,
                       help='Version (e.g., v1, v2)')
    parser.add_argument('--partition-mode', type=str,
                       choices=['unweighted', 'edge-weighted'],
                       default='edge-weighted',
                       help='Partition mode: unweighted or edge-weighted (default: edge-weighted)')
    parser.add_argument('--output-dir', type=str,
                       help='Output directory (default: outputs/us_YEAR_VERSION or us_YEAR_VERSION_noedge)')
    parser.add_argument('--template', type=str,
                       help='Dashboard template file (default: web/dashboard.html)')

    args = parser.parse_args()

    return generate_dashboard(
        args.year,
        args.version,
        args.partition_mode,
        args.output_dir,
        args.template
    )


if __name__ == '__main__':
    sys.exit(main())
