#!/usr/bin/env python3
"""
Generate enhanced master dashboard with cross-census comparison tables.

This dashboard shows:
1. Summary statistics for each census year (2000, 2010, 2020)
2. Cross-census comparison tables
3. State-by-state compactness breakdown
4. Run selector for navigating to individual dashboards
"""

import json
import sys
from pathlib import Path
import re
import pandas as pd
import glob

# State code to name mapping for matching enacted baseline with algorithmic data
STATE_CODE_TO_NAME = {
    'AL': 'ALABAMA', 'AK': 'ALASKA', 'AZ': 'ARIZONA', 'AR': 'ARKANSAS',
    'CA': 'CALIFORNIA', 'CO': 'COLORADO', 'CT': 'CONNECTICUT', 'DE': 'DELAWARE',
    'FL': 'FLORIDA', 'GA': 'GEORGIA', 'HI': 'HAWAII', 'ID': 'IDAHO',
    'IL': 'ILLINOIS', 'IN': 'INDIANA', 'IA': 'IOWA', 'KS': 'KANSAS',
    'KY': 'KENTUCKY', 'LA': 'LOUISIANA', 'ME': 'MAINE', 'MD': 'MARYLAND',
    'MA': 'MASSACHUSETTS', 'MI': 'MICHIGAN', 'MN': 'MINNESOTA', 'MS': 'MISSISSIPPI',
    'MO': 'MISSOURI', 'MT': 'MONTANA', 'NE': 'NEBRASKA', 'NV': 'NEVADA',
    'NH': 'NEW_HAMPSHIRE', 'NJ': 'NEW_JERSEY', 'NM': 'NEW_MEXICO', 'NY': 'NEW_YORK',
    'NC': 'NORTH_CAROLINA', 'ND': 'NORTH_DAKOTA', 'OH': 'OHIO', 'OK': 'OKLAHOMA',
    'OR': 'OREGON', 'PA': 'PENNSYLVANIA', 'RI': 'RHODE_ISLAND', 'SC': 'SOUTH_CAROLINA',
    'SD': 'SOUTH_DAKOTA', 'TN': 'TENNESSEE', 'TX': 'TEXAS', 'UT': 'UTAH',
    'VT': 'VERMONT', 'VA': 'VIRGINIA', 'WA': 'WASHINGTON', 'WV': 'WEST_VIRGINIA',
    'WI': 'WISCONSIN', 'WY': 'WYOMING'
}


def find_all_versions_and_runs(outputs_dir='outputs'):
    """
    Find all versions and their runs in the new directory structure.

    New structure:
        outputs/
        ├── v1/                     (version directory)
        │   ├── version.json        (version config)
        │   ├── 2000/               (run directory)
        │   │   └── config.json     (run config)
        │   ├── 2010/
        │   └── 2020/
        ├── edge_weighted/
        │   ├── version.json
        │   ├── 2000/
        │   ├── 2010/
        │   └── 2020/

    Returns:
        versions: List of version dicts with version config and completed runs
        all_runs: Flat list of all runs (for backward compatibility)
    """
    outputs_path = Path(outputs_dir)
    if not outputs_path.exists():
        return [], []

    versions = []
    all_runs = []

    # Iterate through potential version directories
    for version_dir in outputs_path.iterdir():
        if not version_dir.is_dir():
            continue

        # Skip special directories
        if version_dir.name in ['experiments', 'dev', 'archived', 'baseline']:
            continue

        version_config_path = version_dir / 'version.json'

        # Check if this is a version directory (has version.json)
        if version_config_path.exists():
            try:
                with open(version_config_path, 'r') as f:
                    version_config = json.load(f)

                # Find completed runs (year subdirectories with config.json and states/)
                runs_in_version = []
                for year_dir in version_dir.iterdir():
                    if not year_dir.is_dir():
                        continue

                    # Check if year directory name is a valid census year
                    if not year_dir.name.isdigit() or year_dir.name not in ['2000', '2010', '2020']:
                        continue

                    run_config_path = year_dir / 'config.json'
                    states_dir = year_dir / 'states'

                    if run_config_path.exists() and states_dir.exists():
                        # Load run config
                        with open(run_config_path, 'r') as f:
                            run_config = json.load(f)

                        # Count states
                        num_states = sum(1 for state_dir in states_dir.iterdir()
                                       if state_dir.is_dir() and (state_dir / 'district_summary.csv').exists())

                        if num_states > 0:
                            run_info = {
                                'year': year_dir.name,
                                'version': version_dir.name,
                                'mode': version_config.get('algorithm', {}).get('partition_mode', 'unknown').replace('_', '-').title(),
                                'data_level': version_config.get('algorithm', {}).get('data_level', 'tract'),
                                'num_states': num_states,
                                'path': f"{version_dir.name}/{year_dir.name}",
                                'census_year': run_config.get('metadata', {}).get('census_year'),
                                'election_year': run_config.get('metadata', {}).get('election_year'),
                                'created': run_config.get('metadata', {}).get('created', ''),
                                'sort_key': (year_dir.name, version_dir.name)
                            }
                            runs_in_version.append(run_info)
                            all_runs.append(run_info)

                # Add version with its runs
                if runs_in_version:
                    versions.append({
                        'name': version_dir.name,
                        'config': version_config,
                        'partition_mode': version_config.get('algorithm', {}).get('partition_mode', 'unknown'),
                        'data_level': version_config.get('algorithm', {}).get('data_level', 'tract'),
                        'description': version_config.get('description', ''),
                        'completed_years': version_config.get('completed_years', []),
                        'runs': sorted(runs_in_version, key=lambda r: r['year'], reverse=True),
                        'num_runs': len(runs_in_version)
                    })

            except (json.JSONDecodeError, KeyError) as e:
                print(f"Warning: Could not parse version config for {version_dir.name}: {e}")
                continue

    # Sort versions by name
    versions.sort(key=lambda v: v['name'])

    # Sort all_runs by year (desc), then version
    all_runs.sort(key=lambda r: r['sort_key'], reverse=True)

    return versions, all_runs


def find_all_runs(outputs_dir='outputs'):
    """
    Legacy function for backward compatibility.
    Returns flat list of all runs.
    """
    _, all_runs = find_all_versions_and_runs(outputs_dir)
    return all_runs


def aggregate_compactness_data(run_path):
    """
    Aggregate compactness data from all states in a run.

    Returns dict with overall stats and per-state data.
    """
    states_dir = Path(run_path) / 'states'
    if not states_dir.exists():
        return None

    all_districts = []
    state_summaries = []

    for state_dir in sorted(states_dir.iterdir()):
        if not state_dir.is_dir():
            continue

        summary_file = state_dir / 'district_summary.csv'
        if not summary_file.exists():
            continue

        # Read district data
        df = pd.read_csv(summary_file)
        df['state'] = state_dir.name
        all_districts.append(df)

        # Calculate state-level stats
        state_data = {
            'state': state_dir.name.upper(),
            'state_name': state_dir.name.replace('_', ' ').title(),
            'num_districts': len(df),
            'mean_pp': df['polsby_popper'].mean(),
            'median_pp': df['polsby_popper'].median(),
            'mean_reock': df['reock'].mean() if 'reock' in df.columns else None,
        }
        state_summaries.append(state_data)

    if not all_districts:
        return None

    # Combine all districts
    combined_df = pd.concat(all_districts, ignore_index=True)

    # Calculate overall stats
    overall = {
        'total_districts': len(combined_df),
        'mean_pp': combined_df['polsby_popper'].mean(),
        'median_pp': combined_df['polsby_popper'].median(),
        'std_pp': combined_df['polsby_popper'].std(),
        'min_pp': combined_df['polsby_popper'].min(),
        'max_pp': combined_df['polsby_popper'].max(),
        'mean_reock': combined_df['reock'].mean() if 'reock' in combined_df.columns else None,
    }

    return {
        'overall': overall,
        'states': state_summaries,
        'raw_districts': combined_df
    }


def load_baseline_data(year):
    """Load enacted baseline compactness for a census year.

    Returns dict with overall stats and per-state data.
    """
    if year == '2010':
        baseline_file = 'data/enacted_districts/2010/enacted_compactness_2010.csv'
    elif year == '2020':
        # Check multiple locations
        baseline_files = [
            'outputs/baseline_comparison_edge/enacted_district_compactness.csv',
            'outputs/baseline_comparison/enacted_district_compactness.csv'
        ]
        baseline_file = None
        for f in baseline_files:
            if Path(f).exists():
                baseline_file = f
                break
    elif year == '2000':
        baseline_file = 'data/enacted_districts/2000/enacted_compactness_2000.csv'
    else:
        return None

    if baseline_file and Path(baseline_file).exists():
        df = pd.read_csv(baseline_file)

        # Determine state column name (varies between years)
        state_col = 'state_code' if 'state_code' in df.columns else 'state'

        # Calculate per-state stats
        state_stats = {}
        for state_code in df[state_col].unique():
            state_df = df[df[state_col] == state_code]

            # Convert state code to full name for matching with algorithmic data
            state_key = STATE_CODE_TO_NAME.get(state_code.upper(), state_code.upper())

            state_stats[state_key] = {
                'mean_pp': state_df['polsby_popper'].mean(),
                'median_pp': state_df['polsby_popper'].median(),
                'num_districts': len(state_df)
            }

        return {
            'overall': {
                'mean_pp': df['polsby_popper'].mean(),
                'median_pp': df['polsby_popper'].median(),
                'total_districts': len(df)
            },
            'states': state_stats
        }
    return None


def create_cross_census_comparison(runs):
    """
    Create cross-census comparison data structure.

    Structure: {year: {version: {run, algorithmic, enacted, states, improvement}}}
    """
    comparison = {}

    # Group all edge-weighted runs by year and version
    for run in runs:
        year = run['year']
        version = run['version']

        # Only consider edge-weighted runs
        if run['mode'] != 'Edge-Weighted':
            continue

        # Initialize year structure
        if year not in comparison:
            comparison[year] = {}

        print(f"  Loading {year} {version} data from {run['path']}...")

        algo_data = aggregate_compactness_data(f"outputs/{run['path']}")
        baseline_data = load_baseline_data(year)

        if algo_data:
            version_data = {
                'run': run,
                'algorithmic': algo_data['overall'],
                'enacted': baseline_data['overall'] if baseline_data else None,
                'states': []
            }

            # Add state-level data with improvement calculations
            for state in algo_data['states']:
                state_copy = state.copy()

                # Match with enacted data if available
                if baseline_data and 'states' in baseline_data:
                    state_code = state['state'].upper()
                    if state_code in baseline_data['states']:
                        enacted_state = baseline_data['states'][state_code]
                        state_copy['enacted_mean_pp'] = enacted_state['mean_pp']
                        state_copy['enacted_median_pp'] = enacted_state['median_pp']

                        # Calculate state-level improvement
                        if enacted_state['mean_pp'] > 0:
                            improvement = ((state_copy['mean_pp'] / enacted_state['mean_pp']) - 1) * 100
                            state_copy['improvement_pct'] = improvement

                version_data['states'].append(state_copy)

            # Calculate overall improvement if baseline available
            if baseline_data and baseline_data['overall']:
                algo_pp = algo_data['overall']['mean_pp']
                enacted_pp = baseline_data['overall']['mean_pp']
                improvement = ((algo_pp / enacted_pp) - 1) * 100
                version_data['improvement_pct'] = improvement
            else:
                version_data['improvement_pct'] = None

            comparison[year][version] = version_data

    return comparison


def generate_enhanced_master_dashboard(
    output_file='outputs/index.html',
    template_file='web/master_dashboard.html'
):
    """Generate enhanced master dashboard with cross-census comparison."""

    template_path = Path(template_file)
    output_path = Path(output_file)

    # Check template exists
    if not template_path.exists():
        print(f"ERROR: Template not found: {template_path}")
        return 1

    # Find all versions and runs
    print("\nScanning for versions and runs...")
    versions, runs = find_all_versions_and_runs()

    if not runs:
        print("ERROR: No valid runs found in outputs directory")
        return 1

    # Print found versions and runs
    print(f"\nFound {len(versions)} version(s) with {len(runs)} total run(s):")
    for version in versions:
        print(f"  {version['name']}: {version['partition_mode']} - {version['num_runs']} run(s)")
        for run in version['runs']:
            print(f"    - {run['year']}: {run['num_states']} states")

    # Create cross-census comparison
    print("\nAggregating cross-census data...")
    comparison = create_cross_census_comparison(runs)

    print(f"\nCross-census data loaded:")
    for year in sorted(comparison.keys()):
        versions = comparison[year]
        print(f"  {year}: {len(versions)} version(s) - {', '.join(sorted(versions.keys()))}")

        # Show v1 stats as representative
        if 'v1' in versions:
            data = versions['v1']
            algo_pp = data['algorithmic']['mean_pp']
            if data['enacted']:
                enacted_pp = data['enacted']['mean_pp']
                improvement = data['improvement_pct']
                print(f"        v1: Algo PP={algo_pp:.4f}, Enacted PP={enacted_pp:.4f}, {improvement:+.1f}%")
            else:
                print(f"        v1: Algo PP={algo_pp:.4f}, Enacted baseline not available")

    # Read template
    with open(template_path, 'r', encoding='utf-8') as f:
        html = f.read()

    # Generate dropdown options HTML
    options_html = []
    current_year = None
    for run in runs:
        if run['year'] != current_year:
            if current_year is not None:
                options_html.append('')  # Empty line between years
            current_year = run['year']

        label = f"{run['year']} - {run['version_full']}"
        if run['mode']:
            label += f" ({run['mode']})"
        label += f" - {run['num_states']} states"

        options_html.append(f'                <option value="{run["path"]}">{label}</option>')

    # Insert data into template placeholders
    html = html.replace('<!-- OPTIONS_PLACEHOLDER -->', '\n'.join(options_html))
    html = html.replace('/* RUNS_DATA_PLACEHOLDER */', json.dumps(runs, indent=8))
    html = html.replace('/* COMPARISON_DATA_PLACEHOLDER */', json.dumps(comparison, indent=8))
    html = html.replace('/* VERSIONS_DATA_PLACEHOLDER */', json.dumps(versions, indent=8))

    # Write output
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write(html)

    # Also write runs.json and comparison.json for dynamic loading
    runs_json_path = output_path.parent / 'runs.json'
    with open(runs_json_path, 'w', encoding='utf-8') as f:
        json.dump(runs, f, indent=2)

    comparison_json_path = output_path.parent / 'comparison.json'
    with open(comparison_json_path, 'w', encoding='utf-8') as f:
        json.dump(comparison, f, indent=2)

    print(f"\nSUCCESS: Enhanced master dashboard generated")
    print(f"  Output: {output_path}")
    print(f"  Years: {', '.join(sorted(comparison.keys()))}")
    print(f"  Total runs: {len(runs)}")
    print(f"  Data files: {runs_json_path}, {comparison_json_path}")

    return 0


if __name__ == '__main__':
    import argparse

    parser = argparse.ArgumentParser(
        description='Generate enhanced master dashboard with cross-census comparison tables'
    )
    parser.add_argument('--output', type=str, default='outputs/index.html',
                       help='Output file path (default: outputs/index.html)')
    parser.add_argument('--template', type=str, default='web/master_dashboard.html',
                       help='Dashboard template file (default: web/master_dashboard.html)')

    args = parser.parse_args()

    result = generate_enhanced_master_dashboard(args.output, args.template)
    sys.exit(result)
