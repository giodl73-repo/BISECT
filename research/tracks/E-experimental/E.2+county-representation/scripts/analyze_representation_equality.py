"""
Representation Equality Analysis for County-Based Apportionment

Analyzes population-per-seat statistics at different thresholds to evaluate
representation inequality.

Key metrics:
- Mean population per seat (should be ~761,169 for 2020)
- Min/max population per seat (outliers)
- Standard deviation (measure of inequality)
- Coefficient of variation (normalized inequality)

Usage:
    python scripts/analyze_representation_equality.py --year 2020
"""

import sys
from pathlib import Path
from typing import Dict, List
import pandas as pd
import numpy as np

# Add src to path
PROJECT_ROOT = Path(__file__).parent.parent.parent.parent
sys.path.insert(0, str(PROJECT_ROOT / 'src'))

from apportionment.huntington_hill import apportion


# Test thresholds (in millions of people)
THRESHOLDS = [0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 2.5, 3.0]

# 2020 ideal district size
IDEAL_DISTRICT_SIZE_2020 = 761_169  # 331,449,281 / 435


def load_county_data(year: int) -> pd.DataFrame:
    """Load county population data."""
    county_file = PROJECT_ROOT / f'outputs/data/{year}/counties/all_counties_{year}.csv'
    if not county_file.exists():
        print(f"ERROR: County data not found: {county_file}")
        print(f"Run: python scripts/prepare_county_data.py --year {year}")
        sys.exit(1)
    return pd.read_csv(county_file)


def load_state_populations(year: int) -> Dict[str, int]:
    """Load state populations."""
    if year == 2020:
        return {
            'CA': 39_538_223, 'TX': 29_145_505, 'FL': 21_538_187, 'NY': 20_201_249,
            'PA': 13_002_700, 'IL': 12_812_508, 'OH': 11_799_448, 'GA': 10_711_908,
            'NC': 10_439_388, 'MI': 10_077_331, 'NJ': 9_288_994, 'VA': 8_631_393,
            'WA': 7_705_281, 'AZ': 7_151_502, 'MA': 7_029_917, 'TN': 6_910_840,
            'IN': 6_785_528, 'MD': 6_177_224, 'MO': 6_154_913, 'WI': 5_893_718,
            'CO': 5_773_714, 'MN': 5_706_494, 'SC': 5_118_425, 'AL': 5_024_279,
            'LA': 4_657_757, 'KY': 4_505_836, 'OR': 4_237_256, 'OK': 3_959_353,
            'CT': 3_605_944, 'UT': 3_271_616, 'IA': 3_190_369, 'NV': 3_104_614,
            'AR': 3_011_524, 'MS': 2_961_279, 'KS': 2_937_880, 'NM': 2_117_522,
            'NE': 1_961_504, 'ID': 1_839_106, 'WV': 1_793_716, 'HI': 1_455_271,
            'NH': 1_377_529, 'ME': 1_362_359, 'RI': 1_097_379, 'MT': 1_084_225,
            'DE': 989_948, 'SD': 886_667, 'ND': 779_094, 'AK': 733_391,
            'VT': 643_077, 'WY': 576_851,
        }
    else:
        raise ValueError(f"Year {year} not supported")


def analyze_representation_equality(
    counties_df: pd.DataFrame,
    allocation: Dict[str, int],
    entity_populations: Dict[str, int],
    threshold: float
) -> Dict:
    """
    Analyze representation equality for a given allocation.

    Args:
        counties_df: DataFrame with all counties
        allocation: Dict mapping entity name to seat count
        entity_populations: Dict mapping entity name to population
        threshold: Threshold used (for reporting)

    Returns:
        Dict with statistics
    """
    # Extract population per seat for each entity
    pops_per_seat = []

    for entity_name, seats in allocation.items():
        if seats == 0:
            continue

        # Get population from entity_populations dict
        pop = entity_populations.get(entity_name)
        if pop is None or pop == 0:
            continue

        entity_type = 'county' if 'County' in entity_name else 'state_pool'

        pops_per_seat.append({
            'entity': entity_name,
            'type': entity_type,
            'population': pop,
            'seats': seats,
            'pop_per_seat': pop / seats
        })

    if len(pops_per_seat) == 0:
        return None

    df = pd.DataFrame(pops_per_seat)

    # Calculate statistics
    stats = {
        'threshold': threshold,
        'num_entities': len(df),
        'mean_pop_per_seat': df['pop_per_seat'].mean(),
        'median_pop_per_seat': df['pop_per_seat'].median(),
        'std_pop_per_seat': df['pop_per_seat'].std(),
        'min_pop_per_seat': df['pop_per_seat'].min(),
        'max_pop_per_seat': df['pop_per_seat'].max(),
        'ideal_pop_per_seat': IDEAL_DISTRICT_SIZE_2020,
    }

    # Coefficient of variation (normalized inequality measure)
    stats['cv'] = stats['std_pop_per_seat'] / stats['mean_pop_per_seat']

    # Deviation from ideal
    stats['mean_deviation_pct'] = abs(stats['mean_pop_per_seat'] - IDEAL_DISTRICT_SIZE_2020) / IDEAL_DISTRICT_SIZE_2020 * 100

    # Find outliers (entities with extreme pop per seat)
    df['deviation_from_ideal'] = abs(df['pop_per_seat'] - IDEAL_DISTRICT_SIZE_2020)
    outliers = df.nlargest(5, 'deviation_from_ideal')
    stats['worst_outliers'] = outliers[['entity', 'population', 'seats', 'pop_per_seat']].to_dict('records')

    return stats


def run_analysis(year: int = 2020):
    """Run representation equality analysis across thresholds."""
    print(f"Representation Equality Analysis ({year})")
    print("=" * 80)
    print()

    # Load data
    counties = load_county_data(year)
    state_pops = load_state_populations(year)

    print(f"Loaded {len(counties)} counties")
    print(f"Ideal district size: {IDEAL_DISTRICT_SIZE_2020:,} people")
    print()

    # Analyze each threshold
    results = []

    for threshold_m in THRESHOLDS:
        threshold_pop = int(threshold_m * 1_000_000)
        print(f"Threshold: {threshold_m}M people")
        print("-" * 80)

        # Identify qualifying counties
        qualifying = counties[counties['population'] >= threshold_pop].copy()
        num_qualifying = len(qualifying)

        # Build entity list for Huntington-Hill
        entities = []
        entity_populations = {}  # Track populations for analysis

        # Add remaining states (after removing large counties)
        for state, state_pop in state_pops.items():
            large_county_pop = qualifying[qualifying['state'] == state]['population'].sum()
            remaining_pop = state_pop - large_county_pop

            if remaining_pop > 0:
                name = f'{state} (remaining)'
                entities.append({
                    'name': name,
                    'population': remaining_pop
                })
                entity_populations[name] = remaining_pop

        # Add qualifying counties
        for _, county in qualifying.iterrows():
            name = f"County {county['fips']}, {county['state']}"
            entities.append({
                'name': name,
                'population': county['population']
            })
            entity_populations[name] = county['population']

        # Run Huntington-Hill
        allocation = apportion(entities, total_seats=435, min_seats=1)

        # Analyze representation equality
        stats = analyze_representation_equality(counties, allocation, entity_populations, threshold_m)

        if stats:
            results.append(stats)

            print(f"  Qualifying counties: {num_qualifying}")
            print(f"  Mean pop/seat: {stats['mean_pop_per_seat']:>10,.0f} (ideal: {IDEAL_DISTRICT_SIZE_2020:,})")
            print(f"  Median:        {stats['median_pop_per_seat']:>10,.0f}")
            print(f"  Std dev:       {stats['std_pop_per_seat']:>10,.0f}")
            print(f"  Min:           {stats['min_pop_per_seat']:>10,.0f}")
            print(f"  Max:           {stats['max_pop_per_seat']:>10,.0f}")
            print(f"  CV:            {stats['cv']:>10.3f}")
            print()

            print("  Worst outliers (deviation from ideal):")
            for outlier in stats['worst_outliers'][:3]:
                entity = outlier['entity'].split(',')[0]  # Shorten name
                pop_per_seat = outlier['pop_per_seat']
                deviation = abs(pop_per_seat - IDEAL_DISTRICT_SIZE_2020)
                print(f"    {entity:30s} {pop_per_seat:>10,.0f} pop/seat (±{deviation:>8,.0f})")
            print()

    # Summary comparison table
    print()
    print("=" * 80)
    print("Summary: Representation Equality Across Thresholds")
    print("=" * 80)
    print()
    print(f"{'Threshold':<12} {'Counties':<10} {'Mean Pop/Seat':<15} {'Std Dev':<12} {'CV':<8} {'Min':<12} {'Max':<12}")
    print("-" * 80)

    for r in results:
        print(f"{r['threshold']:>6.2f}M     "
              f"{r['num_entities']:>4d}       "
              f"{r['mean_pop_per_seat']:>10,.0f}     "
              f"{r['std_pop_per_seat']:>10,.0f}  "
              f"{r['cv']:>6.3f}  "
              f"{r['min_pop_per_seat']:>10,.0f}  "
              f"{r['max_pop_per_seat']:>10,.0f}")

    print()
    print(f"Ideal (current system): {IDEAL_DISTRICT_SIZE_2020:>10,} people per seat")
    print()

    # Key insights
    print("Key Insights:")
    print("  - Min pop/seat shows smallest counties (overrepresented)")
    print("  - Max pop/seat shows largest counties (underrepresented)")
    print("  - Higher CV = more inequality in representation")
    print("  - Current system assumes min 1 seat per entity")
    print("  - With weighted voting (fractional seats), CV would be 0 (perfect equality)")
    print()


if __name__ == '__main__':
    import argparse

    parser = argparse.ArgumentParser(
        description='Analyze representation equality across thresholds'
    )
    parser.add_argument('--year', type=int, default=2020,
                       help='Census year (default: 2020)')

    args = parser.parse_args()

    run_analysis(args.year)
