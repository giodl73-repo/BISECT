"""
County Representation Threshold Ablation Study

Tests different population thresholds for treating counties as separate
entities in congressional apportionment.

Key Question: At what threshold should counties get direct representation
vs being grouped into their state's redistricting pool?

For each threshold, computes:
- Number of counties that qualify for direct representation
- Seat allocation to qualifying counties
- Remaining seat allocation to states (for small counties)
- Impact on state representation

Usage:
    python scripts/threshold_ablation.py --year 2020
"""

import sys
from pathlib import Path
from typing import Dict, List, Tuple
import pandas as pd
from collections import defaultdict

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent / 'src'))

from apportionment.huntington_hill import apportion


# Test thresholds (in millions of people)
THRESHOLDS = [0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 2.5, 3.0]

# 2020 ideal district size
IDEAL_DISTRICT_SIZE_2020 = 761_169  # 331,449,281 / 435


def load_county_data(year: int) -> pd.DataFrame:
    """
    Load county population data.

    Returns DataFrame with columns: fips, state, population
    """
    # Load from prepared county dataset
    county_file = Path(__file__).parent.parent.parent.parent / f'outputs/data/{year}/counties/all_counties_{year}.csv'

    if not county_file.exists():
        print(f"ERROR: County data file not found: {county_file}")
        print("Run: python scripts/prepare_county_data.py --year {year}")
        sys.exit(1)

    df = pd.read_csv(county_file)

    # Ensure required columns exist
    if 'fips' not in df.columns or 'population' not in df.columns or 'state' not in df.columns:
        print(f"ERROR: County data missing required columns (fips, state, population)")
        sys.exit(1)

    return df


def load_state_populations(year: int) -> Dict[str, int]:
    """Load state populations for given census year."""
    # 2020 state populations (from Census Bureau)
    if year == 2020:
        return {
            'CA': 39_538_223,
            'TX': 29_145_505,
            'FL': 21_538_187,
            'NY': 20_201_249,
            'PA': 13_002_700,
            'IL': 12_812_508,
            'OH': 11_799_448,
            'GA': 10_711_908,
            'NC': 10_439_388,
            'MI': 10_077_331,
            'NJ': 9_288_994,
            'VA': 8_631_393,
            'WA': 7_705_281,
            'AZ': 7_151_502,
            'MA': 7_029_917,
            'TN': 6_910_840,
            'IN': 6_785_528,
            'MD': 6_177_224,
            'MO': 6_154_913,
            'WI': 5_893_718,
            'CO': 5_773_714,
            'MN': 5_706_494,
            'SC': 5_118_425,
            'AL': 5_024_279,
            'LA': 4_657_757,
            'KY': 4_505_836,
            'OR': 4_237_256,
            'OK': 3_959_353,
            'CT': 3_605_944,
            'UT': 3_271_616,
            'IA': 3_190_369,
            'NV': 3_104_614,
            'AR': 3_011_524,
            'MS': 2_961_279,
            'KS': 2_937_880,
            'NM': 2_117_522,
            'NE': 1_961_504,
            'ID': 1_839_106,
            'WV': 1_793_716,
            'HI': 1_455_271,
            'NH': 1_377_529,
            'ME': 1_362_359,
            'RI': 1_097_379,
            'MT': 1_084_225,
            'DE': 989_948,
            'SD': 886_667,
            'ND': 779_094,
            'AK': 733_391,
            'VT': 643_077,
            'WY': 576_851,
        }
    else:
        raise ValueError(f"Year {year} not supported yet")


def compute_hybrid_apportionment(
    state_populations: Dict[str, int],
    county_data: pd.DataFrame,
    threshold_population: int
) -> Dict[str, any]:
    """
    Compute hybrid apportionment: large counties + remaining states.

    Args:
        state_populations: Dict of state abbrev -> total population
        county_data: DataFrame with county populations
        threshold_population: Minimum population for direct county representation

    Returns:
        Dict with:
        - qualifying_counties: List of counties above threshold
        - entity_list: Combined list for Huntington-Hill
        - allocation: Seat allocation result
        - state_impacts: How each state's seats changed
    """
    # Identify qualifying counties
    qualifying = county_data[county_data['population'] >= threshold_population].copy()
    qualifying_counties = qualifying.to_dict('records')

    # Calculate remaining state populations (total - large counties)
    remaining_state_pops = state_populations.copy()
    county_seats_by_state = defaultdict(int)

    for county in qualifying_counties:
        state = county['state']
        pop = county['population']
        remaining_state_pops[state] -= pop
        # Estimate seats this county would get (rough approximation)
        county_seats_by_state[state] += pop / IDEAL_DISTRICT_SIZE_2020

    # Build entity list for Huntington-Hill
    entities = []

    # Add large counties
    for _, county in qualifying_counties.iterrows():
        entities.append({
            'name': f"County {county['fips']}, {county['state']}",
            'population': county['population'],
            'type': 'county',
            'state': county['state'],
            'fips': county['fips']
        })

    # Add remaining state populations
    for state, pop in remaining_state_pops.items():
        if pop > 0:  # Only include if population remains
            entities.append({
                'name': f"{state} (remaining)",
                'population': pop,
                'type': 'state',
                'state': state
            })

    # Run Huntington-Hill
    allocation = apportion(entities, total_seats=435, min_seats=1)

    # Analyze state impacts
    state_impacts = {}
    for state in state_populations:
        # Seats allocated to state's remaining population
        state_key = f"{state} (remaining)"
        state_seats = allocation.get(state_key, 0)

        # Seats allocated to state's large counties
        county_seats = sum(
            allocation.get(f"{c['county']} County, {state}", 0)
            for c in qualifying_counties
            if c['state'] == state
        )

        total_seats = state_seats + county_seats

        state_impacts[state] = {
            'total_seats': total_seats,
            'state_pool_seats': state_seats,
            'county_direct_seats': county_seats,
            'num_qualifying_counties': len(qualifying_counties[qualifying_counties['state'] == state])
        }

    return {
        'qualifying_counties': qualifying_counties,
        'entities': entities,
        'allocation': allocation,
        'state_impacts': state_impacts
    }


def run_threshold_analysis(year: int = 2020):
    """
    Run ablation study across different thresholds.
    """
    print(f"County Representation Threshold Ablation Study ({year})")
    print("=" * 80)
    print()

    # Load data
    counties = load_county_data(year)
    states = load_state_populations(year)

    print(f"Loaded {len(counties)} counties (top 20 by population)")
    print(f"Loaded {len(states)} states")
    print(f"Ideal district size: {IDEAL_DISTRICT_SIZE_2020:,} people")
    print()

    # Run analysis for each threshold
    results = []

    for threshold_millions in THRESHOLDS:
        threshold_pop = int(threshold_millions * 1_000_000)
        threshold_districts = threshold_pop / IDEAL_DISTRICT_SIZE_2020

        print(f"Threshold: {threshold_millions}M people (~{threshold_districts:.1f} districts)")
        print("-" * 80)

        result = compute_hybrid_apportionment(states, counties, threshold_pop)

        num_counties = len(result['qualifying_counties'])
        county_seats = sum(
            seats for name, seats in result['allocation'].items()
            if 'County' in name
        )

        print(f"  Qualifying counties: {num_counties}")
        print(f"  Total seats to counties: {county_seats}")
        print(f"  Average seats per qualifying county: {county_seats / num_counties if num_counties > 0 else 0:.1f}")
        print()

        # Show top qualifying counties
        if num_counties > 0:
            print("  Top qualifying counties:")
            top_counties = result['qualifying_counties'].nlargest(5, 'population')
            for _, county in top_counties.iterrows():
                name = f"County {county['fips']}, {county['state']}"
                seats = result['allocation'][name]
                print(f"    {county['fips']} ({county['state']})  {county['population']:>12,} pop  ->  {seats:2d} seats")
            print()

        # Show state impacts
        affected_states = [
            state for state, impact in result['state_impacts'].items()
            if impact['num_qualifying_counties'] > 0
        ]

        if affected_states:
            print(f"  States with qualifying counties: {len(affected_states)}")
            for state in sorted(affected_states)[:5]:
                impact = result['state_impacts'][state]
                print(f"    {state}: {impact['num_qualifying_counties']} counties, "
                      f"{impact['county_direct_seats']} direct seats, "
                      f"{impact['state_pool_seats']} pool seats, "
                      f"{impact['total_seats']} total")
            print()

        results.append({
            'threshold_millions': threshold_millions,
            'threshold_population': threshold_pop,
            'num_qualifying_counties': num_counties,
            'county_seats': county_seats,
            'num_affected_states': len(affected_states),
            'result': result
        })
        print()

    # Summary table
    print()
    print("Summary Table")
    print("=" * 80)
    print(f"{'Threshold':<12} {'Counties':<10} {'Seats':<8} {'% Total':<10} {'States':<10}")
    print("-" * 80)

    for r in results:
        pct = (r['county_seats'] / 435) * 100
        print(f"{r['threshold_millions']:>6.2f}M     "
              f"{r['num_qualifying_counties']:>4d}       "
              f"{r['county_seats']:>4d}    "
              f"{pct:>6.1f}%     "
              f"{r['num_affected_states']:>4d}")

    print()
    print("Key Insights:")
    print("  - Lower threshold = more counties qualify = more direct representation")
    print("  - Higher threshold = fewer counties = closer to traditional state system")
    print("  - 'Sweet spot' depends on policy goals (representation vs simplicity)")
    print()


if __name__ == '__main__':
    import argparse

    parser = argparse.ArgumentParser(
        description='County representation threshold ablation study'
    )
    parser.add_argument('--year', type=int, default=2020,
                       help='Census year (default: 2020)')

    args = parser.parse_args()

    run_threshold_analysis(args.year)
