"""
Analyze how well cross-state districts at beta=5 align with real metro areas.

Tests whether the algorithm finds "natural" economic/geographic regions.
"""

import pickle
from pathlib import Path
from collections import defaultdict

import sys
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root / "scripts"))

from config.download_sources import FIPS_TO_STATE


# Major multi-state metro areas
MULTI_STATE_METROS = {
    'New York-Newark': {'states': {'NY', 'NJ', 'CT', 'PA'}, 'core': ['NJ', 'NY']},
    'Washington DC': {'states': {'DC', 'MD', 'VA', 'WV'}, 'core': ['DC', 'MD', 'VA']},
    'Philadelphia': {'states': {'PA', 'NJ', 'DE'}, 'core': ['PA', 'NJ']},
    'Chicago': {'states': {'IL', 'IN', 'WI'}, 'core': ['IL', 'IN']},
    'Kansas City': {'states': {'KS', 'MO'}, 'core': ['KS', 'MO']},
    'Portland OR': {'states': {'OR', 'WA'}, 'core': ['OR', 'WA']},
    'Cincinnati': {'states': {'OH', 'KY', 'IN'}, 'core': ['OH', 'KY']},
    'Memphis': {'states': {'TN', 'AR', 'MS'}, 'core': ['TN', 'AR']},
    'Louisville': {'states': {'KY', 'IN'}, 'core': ['KY', 'IN']},
    'Charlotte': {'states': {'NC', 'SC'}, 'core': ['NC', 'SC']},
    'Minneapolis-St Paul': {'states': {'MN', 'WI'}, 'core': ['MN', 'WI']},
    'Omaha': {'states': {'NE', 'IA'}, 'core': ['NE', 'IA']},
    'Texarkana': {'states': {'TX', 'AR'}, 'core': ['TX', 'AR']},
    'Augusta GA': {'states': {'GA', 'SC'}, 'core': ['GA', 'SC']},
    'Chattanooga': {'states': {'TN', 'GA'}, 'core': ['TN', 'GA']},
    'Fargo-Moorhead': {'states': {'ND', 'MN'}, 'core': ['ND', 'MN']},
    'Sioux City': {'states': {'IA', 'NE', 'SD'}, 'core': ['IA', 'NE']},
    'Huntington-Ashland': {'states': {'WV', 'KY', 'OH'}, 'core': ['WV', 'KY']},
    'Toledo OH': {'states': {'OH', 'MI'}, 'core': ['OH', 'MI']},
    'Davenport-Rock Island': {'states': {'IA', 'IL'}, 'core': ['IA', 'IL']},
}


def load_districts_by_state(beta=5.0):
    """Load district assignments and organize by states."""

    ablation_file = Path(f'outputs/experimental/ablation_beta_{beta}_2020.pkl')
    with open(ablation_file, 'rb') as f:
        data = pickle.load(f)

    geoid_to_district = data['geoid_to_district']

    # Group by district
    districts = defaultdict(list)
    for geoid, district in geoid_to_district.items():
        districts[district].append(geoid)

    # Find states in each district
    district_states = {}
    for district, geoids in districts.items():
        states = set()
        for geoid in geoids:
            state_fips = geoid[:2]
            if state_fips in FIPS_TO_STATE:
                states.add(FIPS_TO_STATE[state_fips])
        district_states[district] = states

    return district_states


def analyze_metro_alignment(beta=5.0):
    """Analyze how well cross-state districts match metro areas."""

    district_states = load_districts_by_state(beta)

    # Find cross-state districts
    cross_state = {d: states for d, states in district_states.items() if len(states) > 1}

    print(f"\n{'='*70}")
    print(f"Metro Area Alignment Analysis (beta={beta})")
    print(f"{'='*70}")
    print(f"\nTotal cross-state districts: {len(cross_state)} ({len(cross_state)/435*100:.1f}%)")

    # Match districts to metros
    metro_matches = defaultdict(list)
    unmatched_districts = []

    for district, states in cross_state.items():
        matched = False
        for metro_name, metro_info in MULTI_STATE_METROS.items():
            # Check if district's states are subset of metro's states
            if states.issubset(metro_info['states']):
                # Check if it includes core states
                if any(s in states for s in metro_info['core']):
                    metro_matches[metro_name].append((district, states))
                    matched = True
                    break

        if not matched:
            unmatched_districts.append((district, states))

    # Report results
    print(f"\n{'='*70}")
    print(f"Metro Area Matches")
    print(f"{'='*70}")

    matched_count = sum(len(districts) for districts in metro_matches.values())
    print(f"\nDistricts matching known metro areas: {matched_count} ({matched_count/len(cross_state)*100:.1f}%)")
    print(f"\nBreakdown by metro area:\n")

    for metro_name in sorted(metro_matches.keys(), key=lambda m: len(metro_matches[m]), reverse=True):
        districts = metro_matches[metro_name]
        metro_info = MULTI_STATE_METROS[metro_name]
        print(f"{metro_name:<25} {len(districts):>2} districts  States: {', '.join(sorted(metro_info['core']))}")

        # Show first few district examples
        for district, states in districts[:3]:
            print(f"  - District {district}: {', '.join(sorted(states))}")
        if len(districts) > 3:
            print(f"  ... and {len(districts)-3} more districts")
        print()

    # Show unmatched patterns
    print(f"\n{'='*70}")
    print(f"Other Cross-State Patterns")
    print(f"{'='*70}")
    print(f"\nDistricts not matching known metros: {len(unmatched_districts)} ({len(unmatched_districts)/len(cross_state)*100:.1f}%)")

    # Group unmatched by state pair
    state_pair_counts = defaultdict(int)
    for district, states in unmatched_districts:
        pair = '-'.join(sorted(states))
        state_pair_counts[pair] += 1

    print(f"\nTop unmatched state combinations:\n")
    for pair, count in sorted(state_pair_counts.items(), key=lambda x: x[1], reverse=True)[:20]:
        print(f"  {pair:<30} {count:>2} districts")

    # Summary statistics
    print(f"\n{'='*70}")
    print(f"Summary")
    print(f"{'='*70}")
    print(f"\nKnown metro areas matched: {len(metro_matches)}/{len(MULTI_STATE_METROS)} ({len(metro_matches)/len(MULTI_STATE_METROS)*100:.1f}%)")
    print(f"Districts in metro areas: {matched_count}/{len(cross_state)} ({matched_count/len(cross_state)*100:.1f}%)")
    print(f"Average districts per metro: {matched_count/len(metro_matches):.1f}")


if __name__ == '__main__':
    analyze_metro_alignment(beta=5.0)
