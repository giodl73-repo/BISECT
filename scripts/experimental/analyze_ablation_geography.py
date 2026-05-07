"""
Analyze geographic patterns of cross-state districts in ablation study.

Shows which state pairs share districts and how many for a given beta value.
"""

import argparse
import pickle
from pathlib import Path
from collections import defaultdict

import sys
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root / "scripts"))

from config.download_sources import STATE_FIPS, FIPS_TO_STATE


def analyze_geography(beta, year=2020):
    """Analyze which state pairs share districts for given beta."""

    # Load ablation results
    ablation_file = Path(f'outputs/experimental/ablation_beta_{beta}_2020.pkl')
    with open(ablation_file, 'rb') as f:
        data = pickle.load(f)

    geoid_to_district = data['geoid_to_district']

    # Group GEOIDs by district
    districts = defaultdict(list)
    for geoid, district in geoid_to_district.items():
        districts[district].append(geoid)

    # For each district, find which states it contains
    district_states = {}
    for district, geoids in districts.items():
        states = set()
        for geoid in geoids:
            state_fips = geoid[:2]  # First 2 digits are state FIPS
            if state_fips in FIPS_TO_STATE:
                states.add(FIPS_TO_STATE[state_fips])
        district_states[district] = states

    # Find cross-state districts
    cross_state_districts = {d: states for d, states in district_states.items() if len(states) > 1}

    # Count state pairs
    state_pairs = defaultdict(int)
    for district, states in cross_state_districts.items():
        states_list = sorted(list(states))
        for i in range(len(states_list)):
            for j in range(i + 1, len(states_list)):
                pair = f"{states_list[i]}-{states_list[j]}"
                state_pairs[pair] += 1

    # Sort by frequency
    sorted_pairs = sorted(state_pairs.items(), key=lambda x: x[1], reverse=True)

    print(f"\n{'='*70}")
    print(f"Geographic Analysis: Cross-State Districts (beta={beta})")
    print(f"{'='*70}")
    print(f"\nTotal cross-state districts: {len(cross_state_districts)} ({len(cross_state_districts)/435*100:.1f}%)")
    print(f"\nTop 30 State Pairs by Shared Districts:")
    print(f"\n{'State Pair':<20} {'Districts':<10} {'%':<6}")
    print('-' * 40)

    for i, (pair, count) in enumerate(sorted_pairs[:30], 1):
        pct = count / len(cross_state_districts) * 100
        print(f"{i:2d}. {pair:<17} {count:>3} districts  {pct:5.1f}%")

    if len(sorted_pairs) > 30:
        print(f"\n... and {len(sorted_pairs) - 30} more state pairs")

    print(f"\nTotal unique state-pair combinations: {len(sorted_pairs)}")

    # Identify multi-state districts (>2 states)
    multi_state = [(d, states) for d, states in cross_state_districts.items() if len(states) > 2]
    if multi_state:
        print(f"\nDistricts spanning 3+ states: {len(multi_state)}")
        for district, states in multi_state[:10]:
            print(f"  District {district}: {', '.join(sorted(states))}")


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--beta', type=float, default=100.0, help='Beta value to analyze')
    parser.add_argument('--year', type=int, default=2020, help='Census year')
    args = parser.parse_args()

    analyze_geography(args.beta, args.year)
