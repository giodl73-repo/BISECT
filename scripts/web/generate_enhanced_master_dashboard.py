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
    """Load enacted baseline compactness for a census year."""
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
        return {
            'mean_pp': df['polsby_popper'].mean(),
            'median_pp': df['polsby_popper'].median(),
            'total_districts': len(df)
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
                'enacted': baseline_data,
                'states': []
            }

            # Add state-level data with improvement calculations
            for state in algo_data['states']:
                state_copy = state.copy()
                # States don't have enacted data yet, but structure is ready
                version_data['states'].append(state_copy)

            # Calculate improvement if baseline available
            if baseline_data:
                algo_pp = algo_data['overall']['mean_pp']
                enacted_pp = baseline_data['mean_pp']
                improvement = ((algo_pp / enacted_pp) - 1) * 100
                version_data['improvement_pct'] = improvement
            else:
                version_data['improvement_pct'] = None

            comparison[year][version] = version_data

    return comparison


def generate_enhanced_master_dashboard(
    output_file='outputs/index.html',
    template_file='web/enhanced_master_dashboard.html'
):
    """Generate enhanced master dashboard with cross-census comparison."""

    template_path = Path(template_file)
    output_path = Path(output_file)

    # Check template exists
    if not template_path.exists():
        print(f"ERROR: Template not found: {template_path}")
        return 1

    # Find all runs
    print("\nScanning for runs...")
    runs = find_all_runs()

    if not runs:
        print("ERROR: No valid runs found in outputs directory")
        return 1

    # Print found runs
    print(f"\nFound {len(runs)} runs:")
    for run in runs:
        print(f"  {run['year']} {run['version_full']:<12} - {run['mode']:<15} ({run['num_states']} states)")

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
    parser.add_argument('--template', type=str, default='web/enhanced_master_dashboard.html',
                       help='Dashboard template file (default: web/enhanced_master_dashboard.html)')

    args = parser.parse_args()

    result = generate_enhanced_master_dashboard(args.output, args.template)
    sys.exit(result)
