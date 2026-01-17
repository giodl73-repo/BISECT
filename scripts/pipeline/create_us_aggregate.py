#!/usr/bin/env python3
"""
Create US-level aggregate files from all 50 states.

Creates:
1. us_all_districts.csv - All 435 congressional districts with their largest city
2. us_district_summary.csv - Merged summary from all states with statistics
3. US_2020_Redistricting_Report.md - Comprehensive markdown report
"""

import warnings
warnings.filterwarnings('ignore', category=FutureWarning, module='pandas')
warnings.filterwarnings('ignore')

import pandas as pd
from pathlib import Path
import json
import argparse
import os
from tqdm import tqdm

# State configuration
STATE_CONFIG = {
    'CA': {'name': 'California', 'districts': 52},
    'TX': {'name': 'Texas', 'districts': 38},
    'FL': {'name': 'Florida', 'districts': 28},
    'NY': {'name': 'New York', 'districts': 26},
    'PA': {'name': 'Pennsylvania', 'districts': 17},
    'IL': {'name': 'Illinois', 'districts': 17},
    'OH': {'name': 'Ohio', 'districts': 15},
    'GA': {'name': 'Georgia', 'districts': 14},
    'NC': {'name': 'North Carolina', 'districts': 14},
    'MI': {'name': 'Michigan', 'districts': 13},
    'NJ': {'name': 'New Jersey', 'districts': 12},
    'VA': {'name': 'Virginia', 'districts': 11},
    'WA': {'name': 'Washington', 'districts': 10},
    'AZ': {'name': 'Arizona', 'districts': 9},
    'MA': {'name': 'Massachusetts', 'districts': 9},
    'TN': {'name': 'Tennessee', 'districts': 9},
    'IN': {'name': 'Indiana', 'districts': 9},
    'MD': {'name': 'Maryland', 'districts': 8},
    'MO': {'name': 'Missouri', 'districts': 8},
    'WI': {'name': 'Wisconsin', 'districts': 8},
    'CO': {'name': 'Colorado', 'districts': 8},
    'MN': {'name': 'Minnesota', 'districts': 8},
    'SC': {'name': 'South Carolina', 'districts': 7},
    'AL': {'name': 'Alabama', 'districts': 7},
    'LA': {'name': 'Louisiana', 'districts': 6},
    'KY': {'name': 'Kentucky', 'districts': 6},
    'OR': {'name': 'Oregon', 'districts': 6},
    'OK': {'name': 'Oklahoma', 'districts': 5},
    'CT': {'name': 'Connecticut', 'districts': 5},
    'UT': {'name': 'Utah', 'districts': 4},
    'IA': {'name': 'Iowa', 'districts': 4},
    'NV': {'name': 'Nevada', 'districts': 4},
    'AR': {'name': 'Arkansas', 'districts': 4},
    'MS': {'name': 'Mississippi', 'districts': 4},
    'KS': {'name': 'Kansas', 'districts': 4},
    'NM': {'name': 'New Mexico', 'districts': 3},
    'NE': {'name': 'Nebraska', 'districts': 3},
    'ID': {'name': 'Idaho', 'districts': 2},
    'WV': {'name': 'West Virginia', 'districts': 2},
    'HI': {'name': 'Hawaii', 'districts': 2},
    'NH': {'name': 'New Hampshire', 'districts': 2},
    'ME': {'name': 'Maine', 'districts': 2},
    'RI': {'name': 'Rhode Island', 'districts': 2},
    'MT': {'name': 'Montana', 'districts': 2},
    'DE': {'name': 'Delaware', 'districts': 1},
    'SD': {'name': 'South Dakota', 'districts': 1},
    'ND': {'name': 'North Dakota', 'districts': 1},
    'AK': {'name': 'Alaska', 'districts': 1},
    'VT': {'name': 'Vermont', 'districts': 1},
    'WY': {'name': 'Wyoming', 'districts': 1},
}


def collect_all_districts(us_dir=None):
    """Collect all districts with their largest cities and population from all states."""

    if us_dir is None:
        us_dir = Path('outputs/us_2020_v2')
    else:
        us_dir = Path(us_dir)
    all_districts = []

    # Get position from parent (or 0 if standalone)
    position = int(os.environ.get('TQDM_POSITION', '0'))

    # Check if we should send status messages to parent
    send_status = position > 0

    def report_progress(msg):
        if send_status:
            print(f"STATUS:{position}:{msg}", flush=True)

    sorted_states = sorted(STATE_CONFIG.items(), key=lambda x: x[1]['districts'], reverse=True)
    total_states = len(sorted_states)

    for idx, (state_code, config) in enumerate(sorted_states, 1):
        state_name = config['name']
        num_districts = config['districts']

        # Report progress to parent
        report_progress(f"Create US aggregate files - Collecting ({idx}/{total_states})")

        state_dir = us_dir / 'states' / state_name.lower().replace(' ', '_')
        cities_file = state_dir / 'data' / 'district_cities.csv'
        summary_file = state_dir / 'data' / 'district_summary.csv'

        if cities_file.exists():
            df = pd.read_csv(cities_file)
            df['state'] = state_name
            df['state_code'] = state_code

            # Merge in population data from summary
            if summary_file.exists():
                summary = pd.read_csv(summary_file)
                # Handle different column names
                merge_cols = ['district', 'population']
                if 'deviation_percent' in summary.columns:
                    merge_cols.append('deviation_percent')
                elif 'deviation_percent' in summary.columns:
                    merge_cols.append('deviation_percent')
                df = df.merge(summary[merge_cols], on='district', how='left')

            all_districts.append(df)
        elif num_districts == 1:
            # Single-district state - at-large - get population
            import geopandas as gpd
            tracts_file = f'data/raw/{state_code.lower()}_tracts_2020.parquet'
            total_pop = 0
            if Path(tracts_file).exists():
                tracts = gpd.read_parquet(tracts_file)
                total_pop = int(tracts['population'].sum())

            all_districts.append(pd.DataFrame([{
                'state': state_name,
                'state_code': state_code,
                'district': 1,
                'largest_city': 'At-Large District',
                'city_population': 0,
                'city_lon': None,
                'city_lat': None,
                'population': total_pop,
                'deviation_percent': 0.0
            }]))

    # Concatenate all (filter out empty DataFrames to avoid FutureWarning)
    all_districts = [df for df in all_districts if not df.empty]
    us_districts = pd.concat(all_districts, ignore_index=True)

    # Calculate ideal population for each state and deviation from state average
    us_districts['state_ideal_pop'] = us_districts.groupby('state')['population'].transform('mean')
    us_districts['deviation_from_state_avg'] = us_districts['population'] - us_districts['state_ideal_pop']
    us_districts['deviation_from_state_avg_pct'] = (us_districts['deviation_from_state_avg'] / us_districts['state_ideal_pop'] * 100).round(3)

    # Reorder columns
    cols = ['state', 'state_code', 'district', 'population', 'state_ideal_pop',
            'deviation_from_state_avg', 'deviation_from_state_avg_pct',
            'largest_city', 'city_population', 'city_lon', 'city_lat']
    us_districts = us_districts[[c for c in cols if c in us_districts.columns]]

    # Sort by state and district
    us_districts = us_districts.sort_values(['state', 'district'])

    return us_districts


def collect_district_summaries(us_dir=None):
    """Collect and merge all district summary files (simple union)."""

    if us_dir is None:
        us_dir = Path('outputs/us_2020_v2')
    else:
        us_dir = Path(us_dir)
    all_summaries = []

    for state_code, config in sorted(STATE_CONFIG.items(), key=lambda x: x[1]['districts'], reverse=True):
        state_name = config['name']
        num_districts = config['districts']

        state_dir = us_dir / 'states' / state_name.lower().replace(' ', '_')
        summary_file = state_dir / 'data' / 'district_summary.csv'

        if summary_file.exists():
            df = pd.read_csv(summary_file)
            # Add state columns at the beginning
            df.insert(0, 'state_code', state_code)
            df.insert(0, 'state', state_name)
            all_summaries.append(df)
        elif num_districts == 1:
            # Single-district state - load tract data for population
            import geopandas as gpd
            tracts_file = f'data/raw/{state_code.lower()}_tracts_2020.parquet'
            num_tracts = 0
            total_pop = 0
            if Path(tracts_file).exists():
                tracts = gpd.read_parquet(tracts_file)
                total_pop = int(tracts['population'].sum())
                num_tracts = len(tracts)

            # Match the format of multi-district states
            all_summaries.append(pd.DataFrame([{
                'state': state_name,
                'state_code': state_code,
                'district': 1,
                'population': total_pop,
                'num_tracts': num_tracts,
                'ideal_population': total_pop,
                'deviation_from_ideal': 0,
                'deviation_percent': 0.0,
                'largest_city': 'At-Large District',
                'city_population': 0
            }]))

    if not all_summaries:
        return pd.DataFrame()

    # Simple concatenation - preserves all columns
    us_summary = pd.concat(all_summaries, ignore_index=True)

    # Sort by state and district
    us_summary = us_summary.sort_values(['state', 'district'])

    return us_summary


def generate_markdown_report(us_districts, us_summary):
    """Generate comprehensive markdown report."""

    report = []
    report.append("# US Congressional Redistricting - 2020 Census")
    report.append("")
    report.append("## Overview")
    report.append("")
    report.append("This report summarizes the algorithmic redistricting of all 435 US congressional districts")
    report.append("using Census 2020 population data and tract-level geography.")
    report.append("")
    report.append("### Methodology")
    report.append("")
    report.append("- **Algorithm**: Recursive bisection using METIS graph partitioning")
    report.append("- **Geography**: Census tracts (smallest contiguous unit)")
    report.append("- **Objective**: Equal population districts (< 1% deviation)")
    report.append("- **Constraints**: Contiguity enforced via adjacency graphs")
    report.append("- **Data Source**: US Census Bureau 2020 Redistricting Data")
    report.append("")
    report.append("## National Summary")
    report.append("")

    if not us_summary.empty:
        total_pop = us_summary['population'].sum()
        total_districts = len(us_summary)
        ideal_pop = total_pop / total_districts
        max_dev = us_summary['deviation_percent'].abs().max()
        mean_dev = us_summary['deviation_percent'].abs().mean()
        std_dev = us_summary['deviation_percent'].std()

        report.append(f"- **Total US Population**: {total_pop:,}")
        report.append(f"- **Total Congressional Districts**: {total_districts}")
        report.append(f"- **Ideal District Population**: {ideal_pop:,.0f}")
        report.append(f"- **Maximum Deviation**: {max_dev:.3f}%")
        report.append(f"- **Mean Absolute Deviation**: {mean_dev:.3f}%")
        report.append(f"- **Standard Deviation**: {std_dev:.3f}%")
        report.append("")

        # Population stats
        report.append("### District Population Statistics")
        report.append("")
        report.append(f"- **Minimum**: {us_summary['population'].min():,}")
        report.append(f"- **Maximum**: {us_summary['population'].max():,}")
        report.append(f"- **Median**: {us_summary['population'].median():,.0f}")
        report.append(f"- **Mean**: {us_summary['population'].mean():,.0f}")
        report.append("")

    # State-by-state summary
    report.append("## State-by-State Summary")
    report.append("")
    report.append("| State | Districts | Population | Ideal per District | Max Deviation |")
    report.append("|-------|-----------|------------|-------------------|---------------|")

    for state_code, config in sorted(STATE_CONFIG.items(), key=lambda x: x[1]['districts'], reverse=True):
        state_name = config['name']
        num_districts = config['districts']

        state_data = us_summary[us_summary['state'] == state_name]
        if not state_data.empty:
            total_pop = state_data['population'].sum()
            ideal = total_pop / num_districts
            max_dev = state_data['deviation_percent'].abs().max()
            report.append(f"| {state_name} | {num_districts} | {total_pop:,} | {ideal:,.0f} | {max_dev:.2f}% |")

    report.append("")
    report.append("## Largest Cities by District")
    report.append("")
    report.append("Cities with over 1 million population:")
    report.append("")

    large_cities = us_districts[us_districts['city_population'] > 1_000_000].sort_values('city_population', ascending=False)
    if not large_cities.empty:
        report.append("| City | State | District | Population |")
        report.append("|------|-------|----------|-----------|")
        for _, row in large_cities.iterrows():
            report.append(f"| {row['largest_city']} | {row['state']} | {row['district']} | {row['city_population']:,} |")
    else:
        report.append("*No individual districts with cities over 1M (New York's 8.8M is spread across multiple districts)*")

    report.append("")
    report.append("## Data Files")
    report.append("")
    report.append("- `us_all_districts.csv` - All 435 districts with largest city")
    report.append("- `us_district_summary.csv` - Detailed statistics for all districts")
    report.append("- Individual state folders with maps, intermediate rounds, and district-level maps")
    report.append("")
    report.append("## Generation Date")
    report.append("")
    from datetime import datetime
    report.append(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    report.append("")
    report.append("---")
    report.append("")
    report.append("*Generated using algorithmic redistricting with METIS graph partitioning*")

    return "\n".join(report)


def main(output_dir=None, print_only=False, debug=False, force=False, year='2020', version='v1', dpi=150, skip_maps=False):
    """Generate all US-level aggregate files."""

    # Only print header if running standalone (not called from parent)
    is_standalone = not os.environ.get('TQDM_POSITION')

    # Get position from environment
    position = int(os.environ.get('TQDM_POSITION', '0'))

    def report_progress(msg):
        if position > 0:
            print(f"STATUS:{position}:{msg}", flush=True)

    if is_standalone:
        print("\n" + "=" * 70)
        print("US Congressional Redistricting - Creating Aggregate Files")
        print("=" * 70)

    if output_dir is None:
        us_dir = Path(f'outputs/us_{year}_{version}')
    else:
        us_dir = Path(output_dir)

    if is_standalone:
        print(f"Output directory: {us_dir}")
        print("=" * 70)

    # Create data directory if it doesn't exist
    data_dir = us_dir / 'data'
    data_dir.mkdir(parents=True, exist_ok=True)

    # Check if outputs already exist
    districts_file = data_dir / 'us_all_districts.csv'
    summary_file = data_dir / 'us_district_summary.csv'
    report_file = us_dir / 'US_2020_Redistricting_Report.md'

    if not force and districts_file.exists() and summary_file.exists() and report_file.exists():
        if is_standalone:
            print("\nUS aggregate files already exist - skipping")
            print(f"  {districts_file.name}")
            print(f"  {summary_file.name}")
            print(f"  {report_file.name}")
            print("\nUse --force to regenerate")
        return

    # In print-only mode, skip actual work
    if print_only:
        import time
        # Show progress bar simulation
        position = int(os.environ.get('TQDM_POSITION', '0'))
        pbar = tqdm(total=50,
                   desc="  Collecting districts" if position > 0 else "Collecting districts",
                   unit="state",
                   position=position,
                   ncols=100,
                   leave=False)

        if debug:
            for i in range(50):
                time.sleep(0.02)
                pbar.update(1)
        else:
            pbar.update(50)

        pbar.close()
        return

    # 1. Collect all districts with cities
    report_progress("Create US aggregate files - Collecting districts...")
    us_districts = collect_all_districts(us_dir)

    report_progress("Create US aggregate files - Saving districts CSV...")
    us_districts.to_csv(districts_file, index=False)

    # 2. Collect district summaries
    report_progress("Create US aggregate files - Collecting summaries...")
    us_summary = collect_district_summaries(us_dir)

    if not us_summary.empty:
        us_summary.to_csv(summary_file, index=False)

    # 3. Generate markdown report
    report_text = generate_markdown_report(us_districts, us_summary)

    with open(report_file, 'w') as f:
        f.write(report_text)

    # 4. Create national political map (skipped when called from pipeline with --skip-maps)
    if not skip_maps:
        import subprocess
        political_map_file = us_dir / f'us_national_political_{year}.png'

        if force or not political_map_file.exists():
            report_progress("Create national political map")

            if is_standalone:
                print("\nCreating national political map...")

            cmd = [
                'python',
                'scripts/pipeline/create_us_national_political_map.py',
                '--year', year,
                '--version', version,
                '--output-dir', str(us_dir),
                '--dpi', str(dpi)
            ]

            if force:
                cmd.append('--force')

            result = subprocess.run(cmd, capture_output=True, text=True)

            if result.returncode != 0:
                if is_standalone:
                    print(f"  Warning: Political map creation failed")
                    if result.stderr:
                        print(f"  {result.stderr}")
        else:
            if is_standalone:
                print(f"\nNational political map exists: {political_map_file}")

        # 5. Create national demographic map
        demographic_map_file = us_dir / f'us_national_demographic_{year}.png'

        if force or not demographic_map_file.exists():
            report_progress("Create national demographic map")

            if is_standalone:
                print("\nCreating national demographic map...")

            cmd = [
                'python',
                'scripts/pipeline/create_us_national_demographic_map.py',
                '--year', year,
                '--version', version,
                '--output-dir', str(us_dir),
                '--dpi', str(dpi)
            ]

            if force:
                cmd.append('--force')

            result = subprocess.run(cmd, capture_output=True, text=True)

            if result.returncode != 0:
                if is_standalone:
                    print(f"  Warning: Demographic map creation failed")
                    if result.stderr:
                        print(f"  {result.stderr}")
        else:
            if is_standalone:
                print(f"\nNational demographic map exists: {demographic_map_file}")

    # Print summary only at the end and only if standalone
    if is_standalone:
        print("\n" + "=" * 70)
        print("SUCCESS! US aggregate files created")
        print("=" * 70)
        print(f"\nFiles in: {us_dir}")
        print("  - us_all_districts.csv")
        print("  - us_district_summary.csv")
        print("  - US_2020_Redistricting_Report.md")
        if political_map_file.exists():
            print(f"  - us_national_political_{year}.png")
        if demographic_map_file.exists():
            print(f"  - us_national_demographic_{year}.png")
        print("=" * 70)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Create US aggregate files')
    parser.add_argument('--output-dir', type=str, help='Output directory (default: outputs/us_YEAR_VERSION)')
    parser.add_argument('--year', type=str, default='2020',
                        choices=['2000', '2010', '2020'],
                        help='Census year (default: 2020)')
    parser.add_argument('--version', type=str, default='v1',
                        help='Version (e.g., v1, v2) (default: v1)')
    parser.add_argument('--dpi', type=int, default=150,
                        choices=[72, 100, 150, 200, 300],
                        help='DPI for national maps (default: 150)')
    parser.add_argument('--print-only', action='store_true',
                        help='Print what would be done without executing')
    parser.add_argument('--debug', action='store_true',
                        help='Enable debug mode with progress delays')
    parser.add_argument('--force', action='store_true',
                        help='Force regeneration even if outputs exist')
    parser.add_argument('--skip-maps', action='store_true',
                        help='Skip national map creation (maps will be created as separate pipeline steps)')
    args = parser.parse_args()

    main(args.output_dir, args.print_only, args.debug, args.force, args.year, args.version, args.dpi, args.skip_maps)
