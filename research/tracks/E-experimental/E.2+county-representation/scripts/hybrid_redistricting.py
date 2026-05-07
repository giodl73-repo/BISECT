"""
Hybrid County/State Redistricting with VRA Comparison

Integrates Huntington-Hill apportionment with recursive bisection:
1. Apportion seats to [remaining_states + large_counties] via Huntington-Hill
2. Large counties: Direct representation (no redistricting needed)
3. Remaining states: Run METIS recursive bisection on small counties
4. Compare regular vs VRA-constrained redistricting (5x @ 40%)

Usage:
    python scripts/hybrid_redistricting.py --year 2020 --threshold 2000000 --states CA TX
"""

import sys
from pathlib import Path
from typing import Dict, List, Optional
import pandas as pd
import geopandas as gpd
from collections import defaultdict

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent / 'src'))

from apportionment.huntington_hill import apportion
# from apportionment.partition.recursive_bisection import RecursiveBisection  # TODO: Use when implementing
# from apportionment.data.loader import load_redistricting_data  # TODO: Use when implementing


# VRA Standard Parameters (from Paper 03)
VRA_EDGE_WEIGHT = 5.0          # 5x multiplier for minority-minority edges
VRA_MINORITY_THRESHOLD = 0.40   # Tracts with >40% minority get edges weighted
VRA_TARGET_MM_PCT = 0.60        # Target 60% minority in MM districts


def load_county_populations(year: int) -> pd.DataFrame:
    """
    Load all county populations for given census year.

    TODO: Aggregate from tract-level data
    For now, returns hardcoded top counties
    """
    # Placeholder - in production, aggregate from tract data
    data = [
        {'state': 'CA', 'county': 'Los Angeles', 'fips': '06037', 'population': 10_014_009},
        {'state': 'IL', 'county': 'Cook', 'fips': '17031', 'population': 5_275_541},
        {'state': 'TX', 'county': 'Harris', 'fips': '48201', 'population': 4_731_145},
        {'state': 'AZ', 'county': 'Maricopa', 'fips': '04013', 'population': 4_420_568},
        {'state': 'CA', 'county': 'San Diego', 'fips': '06073', 'population': 3_298_634},
        {'state': 'CA', 'county': 'Orange', 'fips': '06059', 'population': 3_186_989},
        {'state': 'FL', 'county': 'Miami-Dade', 'fips': '12086', 'population': 2_716_940},
        {'state': 'NY', 'county': 'Kings', 'fips': '36047', 'population': 2_736_074},
        {'state': 'NY', 'county': 'Queens', 'fips': '36081', 'population': 2_405_464},
        {'state': 'CA', 'county': 'Riverside', 'fips': '06065', 'population': 2_418_185},
        {'state': 'CA', 'county': 'San Bernardino', 'fips': '06071', 'population': 2_181_654},
        {'state': 'WA', 'county': 'King', 'fips': '53033', 'population': 2_269_675},
        {'state': 'TX', 'county': 'Dallas', 'fips': '48113', 'population': 2_613_539},
        {'state': 'CA', 'county': 'Santa Clara', 'fips': '06085', 'population': 1_936_259},
        {'state': 'FL', 'county': 'Broward', 'fips': '12011', 'population': 1_944_375},
        {'state': 'TX', 'county': 'Tarrant', 'fips': '48439', 'population': 2_110_640},
        {'state': 'TX', 'county': 'Bexar', 'fips': '48029', 'population': 2_009_324},
        {'state': 'CA', 'county': 'Alameda', 'fips': '06001', 'population': 1_682_353},
        {'state': 'NY', 'county': 'New York', 'fips': '36061', 'population': 1_694_251},
        {'state': 'PA', 'county': 'Philadelphia', 'fips': '42101', 'population': 1_603_797},
    ]
    return pd.DataFrame(data)


def load_state_data(year: int) -> Dict[str, Dict]:
    """Load state populations and district targets."""
    # Add scripts to path for config imports
    scripts_path = Path(__file__).parent.parent.parent.parent / 'scripts'
    sys.path.insert(0, str(scripts_path))

    if year == 2020:
        from config_2020 import STATE_CONFIG_2020
        return STATE_CONFIG_2020
    elif year == 2010:
        from config_2010 import STATE_CONFIG_2010
        return STATE_CONFIG_2010
    elif year == 2000:
        from config_2000 import STATE_CONFIG_2000
        return STATE_CONFIG_2000
    else:
        raise ValueError(f"Year {year} not supported")


def compute_edge_weights_vra(
    tracts_gdf: gpd.GeoDataFrame,
    adjacency: dict,
    minority_col: str = 'minority_pct',
    threshold: float = VRA_MINORITY_THRESHOLD,
    weight: float = VRA_EDGE_WEIGHT
) -> Dict:
    """
    Compute VRA-compliant edge weights.

    Edges between tracts with >40% minority get 5x weight.

    Args:
        tracts_gdf: GeoDataFrame with tract geometries and demographics
        adjacency: NetworkX adjacency dict
        minority_col: Column name for minority percentage
        threshold: Minority threshold for edge weighting (0.40)
        weight: Weight multiplier for minority-minority edges (5.0)

    Returns:
        Dict mapping (tract_i, tract_j) to edge weight
    """
    edge_weights = {}

    # Build minority lookup
    minority_pct = {}
    for idx, row in tracts_gdf.iterrows():
        minority_pct[idx] = row.get(minority_col, 0.0)

    # Compute edge weights
    for tract_i, neighbors in adjacency.items():
        minority_i = minority_pct.get(tract_i, 0.0)

        for tract_j in neighbors:
            if tract_i >= tract_j:  # Only process each edge once
                continue

            minority_j = minority_pct.get(tract_j, 0.0)

            # If both tracts exceed minority threshold, apply weight
            if minority_i > threshold and minority_j > threshold:
                edge_weights[(tract_i, tract_j)] = weight
                edge_weights[(tract_j, tract_i)] = weight  # Symmetric
            else:
                edge_weights[(tract_i, tract_j)] = 1.0
                edge_weights[(tract_j, tract_i)] = 1.0

    return edge_weights


def run_hybrid_redistricting(
    year: int,
    threshold_population: int,
    states_to_process: Optional[List[str]] = None,
    output_dir: Optional[Path] = None,
    version: str = 'test'
):
    """
    Run hybrid county/state redistricting with VRA comparison.

    Args:
        year: Census year (2000, 2010, 2020)
        threshold_population: Minimum population for county to be H-H unit
        states_to_process: List of state abbrevs to process (None = all)
        output_dir: Output directory (default: outputs/hybrid_{version}_{year}/)
        version: Version tag for outputs
    """
    print(f"Hybrid County/State Redistricting ({year})")
    print("=" * 80)
    print(f"Threshold: {threshold_population:,} people")
    print(f"VRA Parameters: {VRA_EDGE_WEIGHT}x @ {VRA_MINORITY_THRESHOLD*100:.0f}%")
    print()

    # Setup output directory
    if output_dir is None:
        output_dir = Path(f'outputs/hybrid_{version}_{year}')
    output_dir.mkdir(parents=True, exist_ok=True)

    # Load data
    print("Loading data...")
    counties = load_county_populations(year)
    state_config = load_state_data(year)

    # Filter states if specified
    if states_to_process:
        counties = counties[counties['state'].isin(states_to_process)]
        state_config = {k: v for k, v in state_config.items() if k in states_to_process}

    print(f"  {len(counties)} counties loaded")
    print(f"  {len(state_config)} states to process")
    print()

    # Step 1: Identify qualifying counties
    print("Step 1: Huntington-Hill Apportionment")
    print("-" * 80)

    qualifying_counties = counties[counties['population'] >= threshold_population]
    print(f"Qualifying counties (>={threshold_population:,}): {len(qualifying_counties)}")

    # Build entity list for Huntington-Hill
    entities = []
    state_populations = {}

    # TODO: Load actual state populations from census data
    # For now, use 2020 hardcoded values
    state_populations_2020 = {
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

    for state_abbrev, config in state_config.items():
        state_pop = state_populations_2020.get(state_abbrev, 0)
        state_populations[state_abbrev] = state_pop

        # Subtract large county populations
        large_county_pop = qualifying_counties[
            qualifying_counties['state'] == state_abbrev
        ]['population'].sum()

        remaining_pop = state_pop - large_county_pop

        if remaining_pop > 0:
            entities.append({
                'name': f'{state_abbrev} (remaining)',
                'population': remaining_pop,
                'type': 'state',
                'state': state_abbrev
            })

    # Add large counties
    for _, county in qualifying_counties.iterrows():
        entities.append({
            'name': f"{county['county']} County, {county['state']}",
            'population': county['population'],
            'type': 'county',
            'state': county['state'],
            'fips': county['fips']
        })

    # Calculate total seats for selected states
    if states_to_process:
        total_seats = sum(config['districts'] for config in state_config.values())
        print(f"Processing subset: {total_seats} seats across {len(state_config)} states")
    else:
        total_seats = 435

    # Run Huntington-Hill
    print(f"\nRunning Huntington-Hill on {len(entities)} entities...")
    allocation = apportion(entities, total_seats=total_seats, min_seats=1)

    # Report results
    total_allocated = sum(allocation.values())
    print("\nApportionment Results:")
    print(f"  Total seats allocated: {total_allocated}")

    county_seats = sum(seats for name, seats in allocation.items() if 'County' in name)
    state_seats = sum(seats for name, seats in allocation.items() if 'remaining' in name)

    print(f"  Direct county seats: {county_seats} ({county_seats/total_allocated*100:.1f}%)")
    print(f"  State pool seats: {state_seats} ({state_seats/total_allocated*100:.1f}%)")
    print()

    # Show top qualifying counties
    print("Top qualifying counties:")
    county_alloc = [(name, seats) for name, seats in allocation.items() if 'County' in name]
    for name, seats in sorted(county_alloc, key=lambda x: x[1], reverse=True)[:10]:
        print(f"  {name:45s} {seats:2d} seats")
    print()

    # Step 2: Redistricting for remaining state populations
    print("\nStep 2: Recursive Bisection for Remaining States")
    print("-" * 80)

    results = {}

    for state_abbrev in state_config.keys():
        state_key = f'{state_abbrev} (remaining)'
        target_districts = allocation.get(state_key, 0)

        if target_districts == 0:
            print(f"{state_abbrev}: No remaining districts (all seats to large counties)")
            continue

        print(f"\n{state_abbrev}: {target_districts} districts")

        # TODO: Load tracts excluding large counties
        # For now, just report what would happen
        print(f"  [TODO] Load tracts excluding large counties")
        print(f"  [TODO] Run regular recursive bisection -> {target_districts} districts")
        print(f"  [TODO] Run VRA recursive bisection ({VRA_EDGE_WEIGHT}x @ {VRA_MINORITY_THRESHOLD*100:.0f}%) -> {target_districts} districts")
        print(f"  [TODO] Compare compactness, minority representation")

        results[state_abbrev] = {
            'target_districts': target_districts,
            'large_counties': list(qualifying_counties[qualifying_counties['state'] == state_abbrev]['county']),
            'regular_result': None,  # TODO
            'vra_result': None       # TODO
        }

    print()
    print("=" * 80)
    print("Summary:")
    print(f"  {len(qualifying_counties)} large counties with direct representation")
    print(f"  {county_seats} seats to counties, {state_seats} seats to state pools")
    print(f"  {len([r for r in results.values() if r['target_districts'] > 0])} states need redistricting")
    print()
    print("Next steps:")
    print("  1. Implement tract loading (excluding large counties)")
    print("  2. Run recursive bisection (regular + VRA)")
    print("  3. Analyze and compare results")
    print()

    return {
        'allocation': allocation,
        'qualifying_counties': qualifying_counties,
        'redistricting_results': results
    }


if __name__ == '__main__':
    import argparse

    parser = argparse.ArgumentParser(
        description='Hybrid county/state redistricting with VRA comparison'
    )
    parser.add_argument('--year', type=int, required=True,
                       help='Census year (2000, 2010, 2020)')
    parser.add_argument('--threshold', type=int, required=True,
                       help='Minimum county population for direct representation')
    parser.add_argument('--states', type=str, nargs='+',
                       help='States to process (default: all)')
    parser.add_argument('--version', type=str, default='test',
                       help='Version tag for outputs')
    parser.add_argument('--output', type=str,
                       help='Output directory')

    args = parser.parse_args()

    output_dir = Path(args.output) if args.output else None

    run_hybrid_redistricting(
        year=args.year,
        threshold_population=args.threshold,
        states_to_process=args.states,
        output_dir=output_dir,
        version=args.version
    )
