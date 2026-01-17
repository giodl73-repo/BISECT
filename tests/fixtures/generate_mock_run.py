"""
Generate complete mock redistricting run for dashboard testing.

Creates all files expected by dashboard.html including:
- District assignments and summaries
- Political analysis CSVs and maps
- Demographic analysis CSVs and maps
- Compactness analysis CSVs and maps
- Round progression data and maps
- Metro area maps
- National aggregations

Usage:
    python tests/fixtures/generate_mock_run.py --year 2020 --version test
    python tests/fixtures/generate_mock_run.py --states "VT,AL" --year 2020 --version test
"""

import argparse
import sys
from pathlib import Path
import pandas as pd
import warnings

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))
sys.path.insert(0, str(project_root / 'src'))

# Import mock generators
from tests.mocks.mock_tracts import generate_mock_tracts
from tests.mocks.mock_adjacency import generate_mock_adjacency
from tests.mocks.mock_districts import generate_mock_districts
from tests.mocks.mock_analysis import (
    generate_mock_political_analysis,
    generate_mock_demographic_analysis,
    generate_mock_compactness_analysis
)
from tests.mocks.mock_maps import (
    generate_mock_state_map,
    generate_mock_national_map,
    generate_mock_round_progression_map,
    generate_mock_metro_map
)

# Import config module for generating config.json
from apportionment.config import RunConfig, write_config

# State configurations (district counts for 2020)
STATE_DISTRICTS = {
    'vermont': 1,
    'delaware': 1,
    'wyoming': 1,
    'alabama': 7,
    'california': 52,
    'texas': 38,
    'new_york': 26,
    'florida': 28,
}

# Metro areas by state
STATE_METROS = {
    'alabama': ['birmingham', 'montgomery', 'mobile'],
    'california': ['los_angeles', 'san_francisco', 'san_diego'],
    'texas': ['houston', 'dallas', 'austin'],
    'new_york': ['new_york_city', 'buffalo'],
    'florida': ['miami', 'tampa', 'orlando'],
}


def create_directory_structure(base_path, state):
    """Create full output directory structure for a state."""
    state_path = base_path / 'states' / state

    # Create directories
    (state_path / 'data').mkdir(parents=True, exist_ok=True)
    (state_path / 'maps' / 'districts').mkdir(parents=True, exist_ok=True)
    (state_path / 'maps' / 'rounds').mkdir(parents=True, exist_ok=True)
    (state_path / 'maps' / 'metros').mkdir(parents=True, exist_ok=True)
    (state_path / 'political' / 'maps').mkdir(parents=True, exist_ok=True)
    (state_path / 'demographic' / 'maps').mkdir(parents=True, exist_ok=True)
    (state_path / 'compactness' / 'maps').mkdir(parents=True, exist_ok=True)

    return state_path


def generate_district_summary_csv(state_path, state, num_districts, year):
    """Generate district_summary.csv with all metrics."""
    districts = []

    for d in range(1, num_districts + 1):
        districts.append({
            'state': state.replace('_', ' ').title(),
            'district': d,
            'population': 750000 + (d * 1000),  # Slight variation
            'area_sq_km': 5000 + (d * 100),
            'perimeter_km': 350 + (d * 5),
            'polsby_popper': 0.35 + (d * 0.02),  # 0.35-0.70 range
            'reock': 0.45 + (d * 0.02),
            'num_tracts': 200 + (d * 10),
            'num_cities': 15 + d,
            'largest_city': f'City{d}',
            'largest_city_pop': 50000 + (d * 5000)
        })

    df = pd.DataFrame(districts)
    output_file = state_path / 'data' / 'district_summary.csv'
    df.to_csv(output_file, index=False)
    print(f"  [OK] Generated {output_file.name}")
    return df


def generate_district_cities_csv(state_path, state, num_districts):
    """Generate district_cities.csv with city assignments."""
    cities = []

    for d in range(1, num_districts + 1):
        # 5-10 cities per district
        num_cities = 5 + (d % 5)
        for c in range(1, num_cities + 1):
            cities.append({
                'district': d,
                'city': f'City{d}_{c}',
                'population': 5000 + (c * 1000),
                'lat': 30.0 + (d * 0.5),
                'lon': -85.0 + (d * 0.3)
            })

    df = pd.DataFrame(cities)
    output_file = state_path / 'data' / 'district_cities.csv'
    df.to_csv(output_file, index=False)
    print(f"  [OK] Generated {output_file.name}")
    return df


def generate_rounds_hierarchy_csv(state_path, state, num_districts):
    """Generate rounds_hierarchy.csv showing bisection tree."""
    # Calculate number of rounds needed
    import math
    num_rounds = int(math.ceil(math.log2(num_districts)))

    rounds = []
    for r in range(1, num_rounds + 1):
        num_cuts = 2 ** (r - 1)
        for c in range(num_cuts):
            rounds.append({
                'round': r,
                'cut': c + 1,
                'parent_region': c // 2 + 1 if r > 1 else None,
                'num_districts': num_districts // (2 ** (r - 1)),
                'population': 750000 * (num_districts // (2 ** (r - 1))),
                'edge_cut': 100 + (r * 10)
            })

    df = pd.DataFrame(rounds)
    output_file = state_path / 'data' / 'rounds_hierarchy.csv'
    df.to_csv(output_file, index=False)
    print(f"  [OK] Generated {output_file.name}")
    return df


def generate_district_political_csv(state_path, state, num_districts, year):
    """Generate district_political.csv with vote shares."""
    # Only for 2020
    if year != '2020':
        print(f"  [SKIP] No political data for {year}")
        return None

    districts = []

    # State lean affects districts
    state_leans = {
        'vermont': 0.65,  # D+30
        'alabama': 0.35,  # R+30
        'california': 0.60,
        'texas': 0.45,
        'new_york': 0.62,
        'florida': 0.48,
    }
    state_lean = state_leans.get(state, 0.50)

    for d in range(1, num_districts + 1):
        # Add district variation
        district_lean = state_lean + ((d % 3 - 1) * 0.05)
        district_lean = max(0.3, min(0.7, district_lean))

        dem_share = district_lean
        rep_share = 1.0 - dem_share
        winner = 'D' if dem_share > 0.5 else 'R'
        margin = abs(dem_share - rep_share)

        districts.append({
            'state': state.replace('_', ' ').title(),
            'district': d,
            'dem_votes': int(300000 * dem_share),
            'rep_votes': int(300000 * rep_share),
            'other_votes': 5000,
            'total_votes': 305000,
            'dem_share': round(dem_share, 4),
            'rep_share': round(rep_share, 4),
            'winner': winner,
            'margin': round(margin, 4),
            'competitive': 'Yes' if margin < 0.1 else 'No'
        })

    df = pd.DataFrame(districts)
    output_file = state_path / 'political' / 'district_political.csv'
    df.to_csv(output_file, index=False)
    print(f"  [OK] Generated {output_file.name}")
    return df


def generate_rounds_political_csv(state_path, state, num_districts, year):
    """Generate rounds_political.csv."""
    if year != '2020':
        return None

    import math
    num_rounds = int(math.ceil(math.log2(num_districts)))

    rounds = []
    for r in range(1, num_rounds + 1):
        rounds.append({
            'round': r,
            'dem_share': 0.48 + (r * 0.02),
            'rep_share': 0.52 - (r * 0.02),
            'num_competitive': 1 + r
        })

    df = pd.DataFrame(rounds)
    output_file = state_path / 'political' / 'rounds_political.csv'
    df.to_csv(output_file, index=False)
    print(f"  [OK] Generated {output_file.name}")
    return df


def generate_district_demographics_csv(state_path, state, num_districts):
    """Generate district_demographics.csv with race/ethnicity."""
    districts = []

    for d in range(1, num_districts + 1):
        # Vary demographics by district
        base_white = 0.60 - (d * 0.02)
        districts.append({
            'state': state.replace('_', ' ').title(),
            'district': d,
            'population': 750000,
            'white': round(base_white, 4),
            'black': round(0.15 + (d * 0.01), 4),
            'hispanic': round(0.15, 4),
            'asian': round(0.05 + (d * 0.005), 4),
            'other': round(0.05, 4),
            'diversity_index': round(0.5 + (d * 0.02), 4),
            'majority_race': 'White' if base_white > 0.5 else 'No Majority',
            'majority_minority': 'No' if base_white > 0.5 else 'Yes'
        })

    df = pd.DataFrame(districts)
    output_file = state_path / 'demographic' / 'district_demographics.csv'
    df.to_csv(output_file, index=False)
    print(f"  [OK] Generated {output_file.name}")
    return df


def generate_all_maps(state_path, state, num_districts, year):
    """Generate all map PNG files."""
    print(f"  Generating maps...")

    # Main state map
    generate_mock_state_map(
        state_path / 'maps' / 'all_districts.png',
        state=state,
        map_type='districts'
    )

    # State map with cities
    generate_mock_state_map(
        state_path / 'maps' / 'all_districts_with_cities.png',
        state=state,
        map_type='districts'
    )

    # Individual district maps
    for d in range(1, num_districts + 1):
        district_num = str(d).zfill(2)
        generate_mock_state_map(
            state_path / 'maps' / 'districts' / f'district_{district_num}.png',
            state=state,
            map_type='districts',
            width=600,
            height=600
        )

    # Round progression maps
    import math
    num_rounds = int(math.ceil(math.log2(num_districts)))
    for r in range(1, num_rounds + 1):
        round_num = str(r).zfill(2)
        generate_mock_round_progression_map(
            state_path / 'maps' / 'rounds' / f'round_{round_num}.png',
            num_rounds=r,
            width=800,
            height=600
        )

    # Political maps (2020 only)
    if year == '2020':
        generate_mock_state_map(
            state_path / 'political' / 'maps' / 'partisan_lean.png',
            state=state,
            map_type='political'
        )

    # Demographic maps
    generate_mock_state_map(
        state_path / 'demographic' / 'maps' / 'majority_race.png',
        state=state,
        map_type='demographic'
    )
    generate_mock_state_map(
        state_path / 'demographic' / 'maps' / 'diversity_index.png',
        state=state,
        map_type='demographic'
    )
    generate_mock_state_map(
        state_path / 'demographic' / 'maps' / 'gender_balance.png',
        state=state,
        map_type='demographic'
    )

    # Compactness maps
    generate_mock_state_map(
        state_path / 'compactness' / 'maps' / 'polsby_popper.png',
        state=state,
        map_type='compactness'
    )
    generate_mock_state_map(
        state_path / 'compactness' / 'maps' / 'reock.png',
        state=state,
        map_type='compactness'
    )

    # Metro maps
    metros = STATE_METROS.get(state, [])
    for metro in metros:
        generate_mock_metro_map(
            state_path / 'maps' / 'metros' / f'{metro}.png',
            metro_name=metro.replace('_', ' ').title()
        )

    print(f"  [OK] Generated all maps")


def generate_national_aggregations(base_path, states, year):
    """Generate national-level CSVs and maps."""
    print(f"\nGenerating national aggregations...")

    # Create directories
    (base_path / 'data').mkdir(parents=True, exist_ok=True)
    (base_path / 'maps' / 'political').mkdir(parents=True, exist_ok=True)
    (base_path / 'maps' / 'demographic').mkdir(parents=True, exist_ok=True)
    (base_path / 'maps' / 'compactness').mkdir(parents=True, exist_ok=True)

    # Aggregate all districts
    all_districts = []
    district_id = 1

    for state in states:
        num_districts = STATE_DISTRICTS.get(state, 1)
        for d in range(1, num_districts + 1):
            all_districts.append({
                'id': district_id,
                'state': state.replace('_', ' ').title(),
                'district': d,
                'population': 750000,
                'polsby_popper': 0.40 + (district_id % 10) * 0.03
            })
            district_id += 1

    df = pd.DataFrame(all_districts)
    df.to_csv(base_path / 'data' / 'us_all_districts.csv', index=False)
    print(f"[OK] Generated us_all_districts.csv ({len(df)} districts)")

    # National summary
    df.to_csv(base_path / 'data' / 'us_district_summary.csv', index=False)

    # National rounds (mock)
    rounds = []
    for r in range(1, 10):
        rounds.append({'round': r, 'edge_cut': 1000 + (r * 100)})
    pd.DataFrame(rounds).to_csv(base_path / 'data' / 'us_rounds_hierarchy.csv', index=False)

    # National maps
    generate_mock_national_map(
        base_path / 'maps' / 'us_all_districts.png',
        map_type='districts'
    )

    if year == '2020':
        generate_mock_national_map(
            base_path / 'maps' / 'political' / 'partisan_lean.png',
            map_type='political'
        )

    generate_mock_national_map(
        base_path / 'maps' / 'demographic' / 'majority_demographics.png',
        map_type='demographic'
    )

    generate_mock_national_map(
        base_path / 'maps' / 'compactness' / 'polsby_popper.png',
        map_type='compactness'
    )

    print(f"[OK] Generated national maps")


def generate_mock_run(states, year, version, output_dir=None):
    """Generate complete mock run for specified states."""
    if output_dir is None:
        # Use new directory structure: outputs/dev/{version}_{year}/
        output_dir = project_root / 'outputs' / 'dev' / f'{version}_{year}'
    else:
        output_dir = Path(output_dir)

    print(f"\nGenerating mock run:")
    print(f"  Year: {year}")
    print(f"  Version: {version}")
    print(f"  States: {', '.join(states)}")
    print(f"  Output: {output_dir}")
    print()

    # Create output directory
    output_dir.mkdir(parents=True, exist_ok=True)

    # Generate config.json
    print("Generating config.json...")
    config = RunConfig.create(
        version=version,
        census_year=int(year),
        election_year=int(year),
        partition_mode='edge_weighted',  # Default for mocks
        data_level='tract',
        run_type='test',  # Mark as test/mock run
        scope='state' if len(states) < 50 else 'us',
        states=states
    )
    write_config(config, output_dir)
    print("[OK] Generated config.json")
    print()

    # Suppress warnings from mock generators
    warnings.filterwarnings('ignore')

    # Generate each state
    for state in states:
        print(f"Processing {state.replace('_', ' ').title()}...")

        num_districts = STATE_DISTRICTS.get(state, 1)
        state_path = create_directory_structure(output_dir, state)

        # Generate CSVs
        generate_district_summary_csv(state_path, state, num_districts, year)
        generate_district_cities_csv(state_path, state, num_districts)
        generate_rounds_hierarchy_csv(state_path, state, num_districts)
        generate_district_political_csv(state_path, state, num_districts, year)
        generate_rounds_political_csv(state_path, state, num_districts, year)
        generate_district_demographics_csv(state_path, state, num_districts)

        # Generate maps
        generate_all_maps(state_path, state, num_districts, year)

        print()

    # Generate national aggregations
    generate_national_aggregations(output_dir, states, year)

    # Copy dashboard template
    import shutil
    dashboard_template = project_root / 'web' / 'dashboard.html'
    dashboard_output = output_dir / 'index.html'

    if dashboard_template.exists():
        shutil.copy(dashboard_template, dashboard_output)
        print(f"[OK] Copied dashboard template to index.html")
    else:
        print(f"[WARN] Dashboard template not found at {dashboard_template}")

    print(f"\n[OK] Mock run generated successfully!")
    print(f"Output directory: {output_dir}")
    print(f"\nTo view dashboard:")
    print(f"  1. Open {dashboard_output} in browser")
    print(f"  2. Or run: deploy_web.bat --year {year} --version {version}")


def main():
    parser = argparse.ArgumentParser(
        description='Generate complete mock redistricting run for dashboard testing'
    )
    parser.add_argument(
        '--states',
        type=str,
        default='vermont,alabama',
        help='Comma-separated list of states (default: vermont,alabama)'
    )
    parser.add_argument(
        '--year',
        type=str,
        default='2020',
        choices=['2000', '2010', '2020'],
        help='Census year (default: 2020)'
    )
    parser.add_argument(
        '--version',
        type=str,
        default='test',
        help='Version name (default: test)'
    )
    parser.add_argument(
        '--output-dir',
        type=str,
        help='Custom output directory (default: outputs/dev/{version}_{year})'
    )

    args = parser.parse_args()

    # Parse states
    states = [s.strip().lower().replace(' ', '_') for s in args.states.split(',')]

    # Validate states
    for state in states:
        if state not in STATE_DISTRICTS:
            print(f"[WARNING] Unknown state: {state}, using 1 district")

    # Generate mock run
    generate_mock_run(states, args.year, args.version, args.output_dir)


if __name__ == '__main__':
    main()
