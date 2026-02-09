"""
Constrained County Representation Analysis

Tests county-based apportionment with constraints:
- Counties can only "fork" if remaining state population >= threshold
- Prevents tiny state pools (e.g., Delaware with 420K remaining)

Compares:
1. Unconstrained: Any county above threshold qualifies
2. Constrained: County only qualifies if state remains viable

Usage:
    python scripts/analyze_with_constraints.py --year 2020 --min-state 761169
"""

import sys
from pathlib import Path
from typing import Dict, List, Tuple
import pandas as pd
import numpy as np

# Add src to path
PROJECT_ROOT = Path(__file__).parent.parent.parent.parent
sys.path.insert(0, str(PROJECT_ROOT / 'src'))

from apportionment.huntington_hill import apportion


# Test thresholds
THRESHOLDS = [0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 2.5, 3.0]

# 2020 ideal district size
IDEAL_DISTRICT_SIZE_2020 = 761_169


def load_county_data(year: int) -> pd.DataFrame:
    """Load county population data."""
    county_file = PROJECT_ROOT / f'outputs/data/{year}/counties/all_counties_{year}.csv'
    if not county_file.exists():
        print(f"ERROR: County data not found: {county_file}")
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


def select_qualifying_counties_constrained(
    counties_df: pd.DataFrame,
    state_pops: Dict[str, int],
    county_threshold: int,
    min_state_remaining: int
) -> Tuple[pd.DataFrame, List[str]]:
    """
    Select qualifying counties with constraint.

    A county can only qualify if:
    1. Its population >= county_threshold
    2. After removing it, state remaining >= min_state_remaining

    Args:
        counties_df: All counties
        state_pops: State total populations
        county_threshold: Minimum county population to qualify
        min_state_remaining: Minimum remaining state population

    Returns:
        (qualifying_counties_df, rejected_counties_list)
    """
    # Sort counties by population (largest first) for greedy selection
    candidates = counties_df[counties_df['population'] >= county_threshold].copy()
    candidates = candidates.sort_values('population', ascending=False)

    qualifying = []
    rejected = []

    # Track remaining population per state
    state_remaining = state_pops.copy()

    for _, county in candidates.iterrows():
        state = county['state']
        pop = county['population']

        # Check if removing this county leaves state viable
        would_remain = state_remaining[state] - pop

        if would_remain >= min_state_remaining:
            # Accept county
            qualifying.append(county)
            state_remaining[state] = would_remain
        else:
            # Reject county (would make state too small)
            rejected.append({
                'fips': county['fips'],
                'state': state,
                'population': pop,
                'reason': f'Would leave {would_remain:,} in state (< {min_state_remaining:,})'
            })

    qualifying_df = pd.DataFrame(qualifying) if qualifying else pd.DataFrame()

    return qualifying_df, rejected


def run_constrained_analysis(
    year: int,
    min_state_remaining: int
):
    """
    Run analysis with minimum remaining state constraint.

    Args:
        year: Census year
        min_state_remaining: Minimum population that must remain in state pool
    """
    print(f"Constrained County Representation Analysis ({year})")
    print("=" * 80)
    print(f"Constraint: State remaining >= {min_state_remaining:,} people")
    print(f"           (~{min_state_remaining / IDEAL_DISTRICT_SIZE_2020:.1f} districts)")
    print()

    counties = load_county_data(year)
    state_pops = load_state_populations(year)

    results = []

    for threshold_m in THRESHOLDS:
        threshold_pop = int(threshold_m * 1_000_000)

        print(f"Threshold: {threshold_m}M people")
        print("-" * 80)

        # Unconstrained selection
        unconstrained = counties[counties['population'] >= threshold_pop]

        # Constrained selection
        constrained, rejected = select_qualifying_counties_constrained(
            counties, state_pops, threshold_pop, min_state_remaining
        )

        print(f"  Unconstrained: {len(unconstrained)} counties qualify")
        print(f"  Constrained:   {len(constrained)} counties qualify")
        print(f"  Rejected:      {len(rejected)} counties (would violate constraint)")

        if len(rejected) > 0:
            print()
            print("  Rejected counties (top 5):")
            for rej in rejected[:5]:
                print(f"    {rej['fips']} ({rej['state']}): {rej['population']:>10,} - {rej['reason']}")

        # Build entities and run H-H for constrained case
        entities = []
        entity_pops = {}

        # Add remaining states
        for state, state_pop in state_pops.items():
            if len(constrained) > 0:
                large_county_pop = constrained[constrained['state'] == state]['population'].sum()
            else:
                large_county_pop = 0
            remaining_pop = state_pop - large_county_pop

            if remaining_pop > 0:
                name = f'{state} (remaining)'
                entities.append({'name': name, 'population': remaining_pop})
                entity_pops[name] = remaining_pop

        # Add qualifying counties
        if len(constrained) > 0:
            for _, county in constrained.iterrows():
                name = f"County {county['fips']}, {county['state']}"
                entities.append({'name': name, 'population': county['population']})
                entity_pops[name] = county['population']

        # Run Huntington-Hill
        allocation = apportion(entities, total_seats=435, min_seats=1)

        # Calculate statistics
        pops_per_seat = []
        for entity_name, seats in allocation.items():
            if seats > 0:
                pop = entity_pops.get(entity_name, 0)
                if pop > 0:
                    pops_per_seat.append(pop / seats)

        stats = {
            'threshold': threshold_m,
            'num_qualifying': len(constrained),
            'num_rejected': len(rejected),
            'mean_pop_per_seat': np.mean(pops_per_seat),
            'std_pop_per_seat': np.std(pops_per_seat),
            'min_pop_per_seat': np.min(pops_per_seat),
            'max_pop_per_seat': np.max(pops_per_seat),
            'cv': np.std(pops_per_seat) / np.mean(pops_per_seat)
        }

        results.append(stats)

        print()
        print(f"  Representation equality:")
        print(f"    Mean:   {stats['mean_pop_per_seat']:>10,.0f} pop/seat")
        print(f"    Std:    {stats['std_pop_per_seat']:>10,.0f}")
        print(f"    CV:     {stats['cv']:>10.3f}")
        print(f"    Range:  {stats['min_pop_per_seat']:>10,.0f} - {stats['max_pop_per_seat']:>10,.0f}")
        print()

    # Summary table
    print()
    print("=" * 80)
    print("Summary: Constrained vs Unconstrained")
    print("=" * 80)
    print()
    print(f"{'Threshold':<12} {'Qualified':<12} {'Rejected':<12} {'Mean':<12} {'CV':<10} {'Min':<12} {'Max':<12}")
    print("-" * 80)

    for r in results:
        print(f"{r['threshold']:>6.2f}M     "
              f"{r['num_qualifying']:>4d}         "
              f"{r['num_rejected']:>4d}         "
              f"{r['mean_pop_per_seat']:>10,.0f}  "
              f"{r['cv']:>6.3f}    "
              f"{r['min_pop_per_seat']:>10,.0f}  "
              f"{r['max_pop_per_seat']:>10,.0f}")

    print()
    print(f"Ideal: {IDEAL_DISTRICT_SIZE_2020:,} people per seat")
    print()
    print("Key insights:")
    print(f"  - Constraint prevents states from dropping below {min_state_remaining:,} people")
    print(f"  - Reduces representation inequality (higher min pop/seat)")
    print(f"  - Trade-off: Fewer counties get direct representation")
    print()


if __name__ == '__main__':
    import argparse

    parser = argparse.ArgumentParser(
        description='Analyze county representation with minimum state constraint'
    )
    parser.add_argument('--year', type=int, default=2020,
                       help='Census year')
    parser.add_argument('--min-state', type=int, default=IDEAL_DISTRICT_SIZE_2020,
                       help=f'Minimum remaining state population (default: {IDEAL_DISTRICT_SIZE_2020:,})')

    args = parser.parse_args()

    run_constrained_analysis(args.year, args.min_state)
