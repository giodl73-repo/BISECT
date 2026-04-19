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
import pandas as pd

# Import scan_artifacts from generate_dashboard
from generate_dashboard import scan_artifacts
# Import version dashboard generator
from generate_version_dashboard import generate_version_dashboard

# State code to name mapping for matching enacted baseline with algorithmic data
STATE_CODE_TO_NAME = {
    'AL': 'alabama', 'AK': 'alaska', 'AZ': 'arizona', 'AR': 'arkansas',
    'CA': 'california', 'CO': 'colorado', 'CT': 'connecticut', 'DE': 'delaware',
    'FL': 'florida', 'GA': 'georgia', 'HI': 'hawaii', 'ID': 'idaho',
    'IL': 'illinois', 'IN': 'indiana', 'IA': 'iowa', 'KS': 'kansas',
    'KY': 'kentucky', 'LA': 'louisiana', 'ME': 'maine', 'MD': 'maryland',
    'MA': 'massachusetts', 'MI': 'michigan', 'MN': 'minnesota', 'MS': 'mississippi',
    'MO': 'missouri', 'MT': 'montana', 'NE': 'nebraska', 'NV': 'nevada',
    'NH': 'new_hampshire', 'NJ': 'new_jersey', 'NM': 'new_mexico', 'NY': 'new_york',
    'NC': 'north_carolina', 'ND': 'north_dakota', 'OH': 'ohio', 'OK': 'oklahoma',
    'OR': 'oregon', 'PA': 'pennsylvania', 'RI': 'rhode_island', 'SC': 'south_carolina',
    'SD': 'south_dakota', 'TN': 'tennessee', 'TX': 'texas', 'UT': 'utah',
    'VT': 'vermont', 'VA': 'virginia', 'WA': 'washington', 'WV': 'west_virginia',
    'WI': 'wisconsin', 'WY': 'wyoming'
}


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


def build_versions_data(runs, outputs_dir='outputs'):
    """Build VERSIONS data structure by scanning for version.json files."""
    outputs_path = Path(outputs_dir)
    versions_list = []

    # Find all version directories by looking for version.json files
    version_pattern = re.compile(r'^[vV]\d+$', re.IGNORECASE)

    for version_dir in sorted(outputs_path.iterdir()):
        if not version_dir.is_dir():
            continue

        # Skip special directories
        if version_dir.name in ['artifacts', 'experiments', 'dev', 'test', 'test_post']:
            continue

        # Check if this matches version pattern
        if not version_pattern.match(version_dir.name):
            continue

        version_json_path = version_dir / 'version.json'
        if not version_json_path.exists():
            continue

        try:
            # Read version.json
            with open(version_json_path, 'r', encoding='utf-8') as f:
                version_metadata = json.load(f)

            version_name = version_dir.name.lower()  # Normalize to lowercase
            description = version_metadata.get('description', f'Redistricting variant: {version_name}')
            partition_mode = version_metadata.get('algorithm', {}).get('partition_mode', 'edge_weighted')
            data_level = version_metadata.get('algorithm', {}).get('data_level', 'tract')

            # Find completed runs for this version
            version_runs = [r for r in runs if r['version'] == version_name]
            completed_years = sorted(set(r['year'] for r in version_runs))

            # Build runs list
            runs_list = []
            for run in version_runs:
                runs_list.append({
                    'year': run['year'],
                    'mode': run['mode'],
                    'num_states': run['num_states'],
                    'path': run['path'] + '/index.html',
                    'created': None
                })

            versions_list.append({
                'name': version_name,
                'description': description,
                'partition_mode': partition_mode,
                'data_level': data_level,
                'num_runs': len(version_runs),
                'completed_years': completed_years,
                'runs': runs_list
            })

        except Exception as e:
            print(f"Warning: Could not read {version_json_path}: {e}")
            continue

    # Sort by version name
    versions_list.sort(key=lambda v: v['name'])

    return versions_list


def aggregate_compactness_data(run_path):
    """
    Aggregate compactness data from all states in a run.

    Returns dict with overall stats and per-state data, or None if no data found.
    """
    states_dir = Path(run_path) / 'states'
    if not states_dir.exists():
        return None

    all_districts = []
    state_summaries = {}

    for state_dir in sorted(states_dir.iterdir()):
        if not state_dir.is_dir():
            continue

        # Check for district_summary.csv in data/ subdirectory
        summary_file = state_dir / 'data' / 'district_summary.csv'
        if not summary_file.exists():
            continue

        try:
            # Read district data
            df = pd.read_csv(summary_file)
            df['state'] = state_dir.name
            all_districts.append(df)

            # Calculate state-level stats
            state_key = state_dir.name  # Keep lowercase with underscores
            state_summaries[state_key] = {
                'algorithmic': df['polsby_popper'].mean(),
                'num_districts': len(df),
            }
        except Exception as e:
            print(f"    Warning: Could not read {summary_file}: {e}")
            continue

    if not all_districts:
        return None

    # Combine all districts
    combined_df = pd.concat(all_districts, ignore_index=True)

    # Calculate overall stats
    overall_pp = combined_df['polsby_popper'].mean()

    return {
        'algorithmic': overall_pp,
        'states': state_summaries
    }


def load_baseline_data(year):
    """Load enacted baseline compactness for a census year.

    Returns dict with overall stats and per-state data, or None if not available.
    """
    baseline_files = {
        '2000': 'data/enacted_districts/2000/enacted_compactness_2000.csv',
        '2010': 'data/enacted_districts/2010/enacted_compactness_2010.csv',
        '2020': [
            'outputs/baseline_comparison_edge/enacted_district_compactness.csv',
            'outputs/baseline_comparison/enacted_district_compactness.csv'
        ]
    }

    baseline_file = baseline_files.get(year)
    if not baseline_file:
        return None

    # Handle multiple possible locations for 2020
    if isinstance(baseline_file, list):
        for f in baseline_file:
            if Path(f).exists():
                baseline_file = f
                break
        else:
            return None

    if not Path(baseline_file).exists():
        return None

    try:
        df = pd.read_csv(baseline_file)

        # Determine state column name (varies between years)
        state_col = 'state_code' if 'state_code' in df.columns else 'state'

        # Calculate per-state stats
        state_stats = {}
        for state_code in df[state_col].unique():
            state_df = df[df[state_col] == state_code]

            # Convert state code to full name (lowercase with underscores)
            state_key = STATE_CODE_TO_NAME.get(state_code.upper())
            if state_key:
                state_stats[state_key] = {
                    'enacted': state_df['polsby_popper'].mean(),
                    'num_districts': len(state_df)
                }

        return {
            'enacted': df['polsby_popper'].mean(),
            'states': state_stats
        }
    except Exception as e:
        print(f"  Warning: Could not load baseline data from {baseline_file}: {e}")
        return None


def create_comparison_data(runs):
    """
    Create cross-census comparison data structure.

    Structure: {year: {version: {algorithmic, enacted, states, improvement_pct}}}
    """
    comparison = {}

    print("\n  Aggregating compactness data for comparison...")

    for run in runs:
        year = run['year']
        version = run['version']

        # Initialize year structure
        if year not in comparison:
            comparison[year] = {}

        # Skip if we already processed this year/version
        if version in comparison[year]:
            continue

        print(f"    Processing {year} {version}...")

        # Aggregate algorithmic data
        run_path = Path('outputs') / version / year
        algo_data = aggregate_compactness_data(run_path)

        if not algo_data:
            print(f"      Warning: No compactness data found for {year} {version}")
            continue

        # Load baseline data (only need to load once per year)
        if year not in comparison or not any('enacted' in v for v in comparison[year].values()):
            baseline_data = load_baseline_data(year)
        else:
            # Reuse baseline from first version
            first_version = list(comparison[year].values())[0]
            baseline_data = {'enacted': first_version.get('enacted'), 'states': {}}

        # Build version data
        version_data = {
            'algorithmic': algo_data['algorithmic'],
            'enacted': baseline_data['enacted'] if baseline_data else None,
            'states': {}
        }

        # Add state-level data with improvement calculations
        for state, state_algo in algo_data['states'].items():
            state_data = {
                'algorithmic': state_algo['algorithmic'],
                'enacted': None,
                'improvement_pct': None
            }

            # Match with enacted data if available
            if baseline_data and 'states' in baseline_data:
                if state in baseline_data['states']:
                    enacted_pp = baseline_data['states'][state]['enacted']
                    state_data['enacted'] = enacted_pp

                    # Calculate improvement
                    if enacted_pp > 0:
                        improvement = ((state_algo['algorithmic'] / enacted_pp) - 1) * 100
                        state_data['improvement_pct'] = improvement

            version_data['states'][state] = state_data

        # Calculate overall improvement if baseline available
        if baseline_data and baseline_data['enacted']:
            algo_pp = algo_data['algorithmic']
            enacted_pp = baseline_data['enacted']
            improvement = ((algo_pp / enacted_pp) - 1) * 100
            version_data['improvement_pct'] = improvement
            print(f"      Algorithmic PP: {algo_pp:.4f}, Enacted PP: {enacted_pp:.4f}, Improvement: {improvement:+.2f}%")
        else:
            version_data['improvement_pct'] = None
            print(f"      Algorithmic PP: {algo_data['algorithmic']:.4f} (no baseline)")

        comparison[year][version] = version_data

    return comparison


def generate_versions_html(versions):
    """
    Generate static HTML for version cards.

    Args:
        versions: List of version dicts with structure from build_versions_data()

    Returns:
        HTML string containing all version cards
    """
    if not versions:
        return '<p style="color: #999;">No versions found. Run redistricting with new directory structure to create versions.</p>'

    html_parts = []

    for version in versions:
        # Format partition mode and data level
        partition_mode = version['partition_mode'].replace('_', '-').split('-')
        partition_mode = ' '.join(w.capitalize() for w in partition_mode)
        data_level = version['data_level'].capitalize()

        # Start version card
        html_parts.append(f'''
                <div class="version-card" onclick="openVersionDashboard('{version['name']}')">
                    <div class="version-card-clickable">
                        <div style="display: flex; justify-content: space-between; align-items: start; margin-bottom: 1rem;">
                            <div>
                                <h3 style="color: #667eea; margin: 0; font-size: 1.5rem;">{version['name']}</h3>''')

        # Add description if available
        if version.get('description'):
            html_parts.append(f'''
                                <p style="color: #666; margin: 0.5rem 0 0 0; font-size: 0.95rem;">{version['description']}</p>''')

        # Add version metadata
        completed_years = ', '.join(version['completed_years']) if version['completed_years'] else 'None'
        html_parts.append(f'''
                            </div>
                            <div style="text-align: right;">
                                <div style="font-size: 0.9rem; color: #999;">{version['num_runs']} run(s)</div>
                                <div style="font-size: 0.85rem; color: #667eea; margin-top: 0.25rem;">Click to view &rarr;</div>
                            </div>
                        </div>

                        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 1rem; margin-bottom: 1.5rem; padding: 1rem; background: #f8f9fa; border-radius: 8px;">
                            <div>
                                <div style="font-size: 0.85rem; color: #666; margin-bottom: 0.25rem;">Partition Mode</div>
                                <div style="font-weight: 600; color: #333;">{partition_mode}</div>
                            </div>
                            <div>
                                <div style="font-size: 0.85rem; color: #666; margin-bottom: 0.25rem;">Data Level</div>
                                <div style="font-weight: 600; color: #333;">{data_level}</div>
                            </div>
                            <div>
                                <div style="font-size: 0.85rem; color: #666; margin-bottom: 0.25rem;">Completed Years</div>
                                <div style="font-weight: 600; color: #333;">{completed_years}</div>
                            </div>
                        </div>
                    </div>

                    <div class="version-card-runs" style="border-top: 1px solid #e0e0e0; padding-top: 1rem;">
                        <h4 style="color: #495057; font-size: 1rem; margin-bottom: 0.75rem;">Individual Runs:</h4>
                        <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 1rem;">''')

        # Add run cards
        for run in version['runs']:
            mode_class = 'mode-edge-weighted' if 'edge' in run['mode'].lower() else 'mode-unweighted'
            year_class = f"year-{run['year']}"
            created_date = 'Unknown date'
            if run.get('created'):
                # Format date if available (JavaScript would use new Date().toLocaleDateString())
                created_date = run['created']

            html_parts.append(f'''
                        <div class="run-card {mode_class} {year_class}" onclick="openRun('{run['path']}')">
                            <div class="run-card-header">
                                <span class="run-card-year">{run['year']}</span>
                                <span class="run-card-states">{run['num_states']} states</span>
                            </div>
                            <div class="run-card-mode">{run['mode']}</div>
                            <div class="run-card-footer">
                                {created_date}
                            </div>
                        </div>''')

        # Close version card
        html_parts.append('''
                        </div>
                    </div>
                </div>
                ''')

    return ''.join(html_parts)


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
    versions = build_versions_data(runs, outputs_dir=output_path.parent)
    print(f"\n  Built versions data:")
    for v in versions:
        print(f"    {v['name']}: {v['num_runs']} runs, {len(v['runs'])} in list")
    versions_json = json.dumps(versions, indent=8)
    html = html.replace('/* VERSIONS_DATA_PLACEHOLDER */', versions_json)

    # Generate static HTML for versions
    versions_html = generate_versions_html(versions)
    html = html.replace('<!-- VERSIONS_HTML_PLACEHOLDER -->', versions_html)

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

    # Generate comparison data from runs
    comparison_data = create_comparison_data(runs)
    comparison_json = json.dumps(comparison_data, indent=8)
    html = html.replace('/* COMPARISON_DATA_PLACEHOLDER */', comparison_json)

    # Write comparison.json for version dashboards to use
    comparison_path = output_path.parent / 'comparison.json'
    with open(comparison_path, 'w', encoding='utf-8') as f:
        json.dump(comparison_data, f, indent=2)
    print(f"  Generated comparison data: {comparison_path}")

    # Write output
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write(html)

    # Also write runs.json for dynamic loading by individual dashboards
    runs_json_path = output_path.parent / 'runs.json'
    with open(runs_json_path, 'w', encoding='utf-8') as f:
        json.dump(runs, f, indent=2)

    print(f"\nSUCCESS: Master dashboard generated: {output_path}")
    print(f"  Years: {', '.join(sorted(set(r['year'] for r in runs)))}")
    print(f"  Total versions: {len(versions)}")
    print(f"  Runs list: {runs_json_path}")

    return 0


def regenerate_version_dashboards(runs, output_dir='outputs'):
    """Generate version-level dashboards for all versions."""
    print("\nGenerating version dashboards...")

    # Get unique versions from runs
    versions = sorted(set(r['version'] for r in runs))

    if not versions:
        print("  No versions found")
        return

    success_count = 0
    for version in versions:
        print(f"  Generating {version} dashboard...")
        success = generate_version_dashboard(
            version=version,
            output_dir=output_dir,
            template_file='web/version_dashboard.html'
        )
        if success:
            success_count += 1

    print(f"  Generated {success_count}/{len(versions)} version dashboards")


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
    parser.add_argument('--skip-version-dashboards', action='store_true',
                       help='Skip generating version-level dashboards')

    args = parser.parse_args()

    # First, generate master dashboard and runs.json
    result = generate_master_dashboard(args.output, args.template)

    if result != 0:
        sys.exit(result)

    # Get runs for subsequent operations
    runs = find_all_runs()

    # Generate version dashboards (unless --skip-version-dashboards)
    if not args.skip_version_dashboards:
        if runs:
            output_dir = Path(args.output).parent
            regenerate_version_dashboards(runs, output_dir=str(output_dir))
        else:
            print("\nNo runs found for version dashboards")
    else:
        print("\nSkipped version dashboard generation (--skip-version-dashboards flag)")

    # Then regenerate all individual dashboards (unless --skip-dashboards)
    if not args.skip_dashboards:
        if runs:
            regenerate_all_dashboards(runs)
        else:
            print("\nNo runs found to regenerate")
    else:
        print("\nSkipped dashboard regeneration (--skip-dashboards flag)")

    sys.exit(0)
