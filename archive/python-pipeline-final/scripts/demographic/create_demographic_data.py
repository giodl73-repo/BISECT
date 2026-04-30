#!/usr/bin/env python3
"""
Create Demographic Data for VRA Compliance Analysis

Since we don't have tract-level demographic data yet, this generates
state-level and district-level demographic estimates based on actual
2020 Census data at the state level.

Key assumption: Algorithmic (compact) districts may have different
minority concentration patterns than enacted districts, particularly
in states where minority populations are geographically concentrated.

Usage:
    python scripts/demographic/create_demographic_data.py
"""

import json
import csv
from pathlib import Path
from typing import Dict, List, Tuple
from dataclasses import dataclass
import random

@dataclass
class StateDemographics:
    """State-level demographic data from 2020 Census"""
    name: str
    num_districts: int
    total_pop: int
    pct_white: float
    pct_black: float
    pct_hispanic: float
    pct_asian: float
    pct_other: float

# Source: 2020 Census Redistricting Data
# https://www.census.gov/data/tables/2020/dec/2020-apportionment-data.html
STATE_DEMOGRAPHICS = {
    'alabama': StateDemographics('alabama', 7, 5024279, 0.637, 0.265, 0.053, 0.015, 0.030),
    'alaska': StateDemographics('alaska', 1, 733391, 0.578, 0.032, 0.073, 0.063, 0.254),
    'arizona': StateDemographics('arizona', 9, 7151502, 0.532, 0.044, 0.315, 0.035, 0.074),
    'arkansas': StateDemographics('arkansas', 4, 3011524, 0.688, 0.154, 0.082, 0.017, 0.059),
    'california': StateDemographics('california', 52, 39538223, 0.347, 0.052, 0.397, 0.151, 0.053),
    'colorado': StateDemographics('colorado', 8, 5773714, 0.655, 0.039, 0.218, 0.032, 0.056),
    'connecticut': StateDemographics('connecticut', 5, 3605944, 0.632, 0.103, 0.173, 0.048, 0.044),
    'delaware': StateDemographics('delaware', 1, 990837, 0.586, 0.230, 0.101, 0.041, 0.042),
    'florida': StateDemographics('florida', 28, 21538187, 0.514, 0.154, 0.268, 0.029, 0.035),
    'georgia': StateDemographics('georgia', 14, 10711908, 0.500, 0.322, 0.101, 0.043, 0.034),
    'hawaii': StateDemographics('hawaii', 2, 1455271, 0.216, 0.019, 0.106, 0.367, 0.292),
    'idaho': StateDemographics('idaho', 2, 1839106, 0.787, 0.006, 0.130, 0.015, 0.062),
    'illinois': StateDemographics('illinois', 17, 12812508, 0.583, 0.140, 0.178, 0.058, 0.041),
    'indiana': StateDemographics('indiana', 9, 6785528, 0.756, 0.098, 0.076, 0.025, 0.045),
    'iowa': StateDemographics('iowa', 4, 3190369, 0.828, 0.041, 0.064, 0.027, 0.040),
    'kansas': StateDemographics('kansas', 4, 2937880, 0.722, 0.054, 0.127, 0.030, 0.067),
    'kentucky': StateDemographics('kentucky', 6, 4505836, 0.820, 0.080, 0.041, 0.017, 0.042),
    'louisiana': StateDemographics('louisiana', 6, 4657757, 0.553, 0.317, 0.055, 0.017, 0.058),
    'maine': StateDemographics('maine', 2, 1362359, 0.904, 0.014, 0.020, 0.013, 0.049),
    'maryland': StateDemographics('maryland', 8, 6177224, 0.471, 0.294, 0.109, 0.066, 0.060),
    'massachusetts': StateDemographics('massachusetts', 9, 7029917, 0.672, 0.069, 0.127, 0.072, 0.060),
    'michigan': StateDemographics('michigan', 13, 10077331, 0.737, 0.137, 0.055, 0.032, 0.039),
    'minnesota': StateDemographics('minnesota', 8, 5706494, 0.764, 0.070, 0.058, 0.051, 0.057),
    'mississippi': StateDemographics('mississippi', 4, 2961279, 0.552, 0.374, 0.033, 0.011, 0.030),
    'missouri': StateDemographics('missouri', 8, 6154913, 0.773, 0.113, 0.045, 0.022, 0.047),
    'montana': StateDemographics('montana', 2, 1084225, 0.838, 0.005, 0.042, 0.008, 0.107),
    'nebraska': StateDemographics('nebraska', 3, 1961504, 0.752, 0.051, 0.117, 0.026, 0.054),
    'nevada': StateDemographics('nevada', 4, 3104614, 0.451, 0.095, 0.298, 0.091, 0.065),
    'new_hampshire': StateDemographics('new_hampshire', 2, 1377529, 0.872, 0.015, 0.042, 0.030, 0.041),
    'new_jersey': StateDemographics('new_jersey', 12, 9288994, 0.513, 0.126, 0.216, 0.101, 0.044),
    'new_mexico': StateDemographics('new_mexico', 3, 2117522, 0.365, 0.020, 0.499, 0.016, 0.100),
    'new_york': StateDemographics('new_york', 26, 20201249, 0.523, 0.139, 0.195, 0.094, 0.049),
    'north_carolina': StateDemographics('north_carolina', 14, 10439388, 0.603, 0.207, 0.099, 0.032, 0.059),
    'north_dakota': StateDemographics('north_dakota', 1, 779094, 0.830, 0.013, 0.043, 0.014, 0.100),
    'ohio': StateDemographics('ohio', 15, 11799448, 0.754, 0.124, 0.043, 0.026, 0.053),
    'oklahoma': StateDemographics('oklahoma', 5, 3959353, 0.605, 0.072, 0.113, 0.024, 0.186),
    'oregon': StateDemographics('oregon', 6, 4237256, 0.708, 0.019, 0.137, 0.046, 0.090),
    'pennsylvania': StateDemographics('pennsylvania', 17, 13002700, 0.738, 0.109, 0.080, 0.037, 0.036),
    'rhode_island': StateDemographics('rhode_island', 2, 1097379, 0.690, 0.057, 0.166, 0.035, 0.052),
    'south_carolina': StateDemographics('south_carolina', 7, 5118425, 0.631, 0.257, 0.061, 0.018, 0.033),
    'south_dakota': StateDemographics('south_dakota', 1, 886667, 0.799, 0.020, 0.043, 0.016, 0.122),
    'tennessee': StateDemographics('tennessee', 9, 6910840, 0.718, 0.163, 0.061, 0.020, 0.038),
    'texas': StateDemographics('texas', 38, 29145505, 0.395, 0.118, 0.396, 0.053, 0.038),
    'utah': StateDemographics('utah', 4, 3271616, 0.754, 0.012, 0.147, 0.023, 0.064),
    'vermont': StateDemographics('vermont', 1, 643077, 0.894, 0.013, 0.021, 0.017, 0.055),
    'virginia': StateDemographics('virginia', 11, 8631393, 0.586, 0.189, 0.100, 0.069, 0.056),
    'washington': StateDemographics('washington', 10, 7705281, 0.633, 0.039, 0.136, 0.095, 0.097),
    'west_virginia': StateDemographics('west_virginia', 2, 1793716, 0.901, 0.037, 0.019, 0.008, 0.035),
    'wisconsin': StateDemographics('wisconsin', 8, 5893718, 0.787, 0.063, 0.071, 0.031, 0.048),
    'wyoming': StateDemographics('wyoming', 1, 576851, 0.814, 0.010, 0.105, 0.009, 0.062),
}

def simulate_enacted_district_demographics(state_demo: StateDemographics) -> List[Dict]:
    """
    Simulate enacted district demographics

    Key patterns:
    - Some states pack minorities into majority-minority districts (VRA)
    - Other districts have lower minority percentages
    - Creates districts that meet VRA requirements but may sacrifice compactness
    """
    random.seed(hash(state_demo.name + "_enacted"))

    districts = []

    # Determine how many majority-minority districts to create
    # Based on state minority percentage and historical patterns
    target_mm_districts = 0

    # States with significant Black populations (>20%) typically have majority-Black districts
    if state_demo.pct_black > 0.20:
        target_mm_districts = max(1, int(state_demo.num_districts * state_demo.pct_black * 1.3))

    # States with significant Hispanic populations (>30%) typically have majority-Hispanic districts
    if state_demo.pct_hispanic > 0.30:
        target_mm_districts = max(target_mm_districts, int(state_demo.num_districts * state_demo.pct_hispanic * 1.2))

    for i in range(1, state_demo.num_districts + 1):
        if i <= target_mm_districts:
            # Create majority-minority district (packed)
            if state_demo.pct_black > state_demo.pct_hispanic:
                # Majority-Black district
                pct_black = random.uniform(0.50, 0.70)
                pct_hispanic = state_demo.pct_hispanic * random.uniform(0.5, 1.0)
                pct_asian = state_demo.pct_asian * random.uniform(0.5, 1.0)
            else:
                # Majority-Hispanic district
                pct_hispanic = random.uniform(0.50, 0.70)
                pct_black = state_demo.pct_black * random.uniform(0.5, 1.0)
                pct_asian = state_demo.pct_asian * random.uniform(0.5, 1.0)
        else:
            # Other districts: minorities diluted
            pct_black = state_demo.pct_black * random.uniform(0.3, 0.8)
            pct_hispanic = state_demo.pct_hispanic * random.uniform(0.3, 0.8)
            pct_asian = state_demo.pct_asian * random.uniform(0.5, 1.2)

        pct_other = state_demo.pct_other * random.uniform(0.8, 1.2)
        pct_white = 1.0 - (pct_black + pct_hispanic + pct_asian + pct_other)
        pct_white = max(0.0, pct_white)  # Ensure non-negative

        districts.append({
            'district': str(i),
            'total_pop': state_demo.total_pop // state_demo.num_districts,
            'pct_white': pct_white,
            'pct_black': pct_black,
            'pct_hispanic': pct_hispanic,
            'pct_asian': pct_asian,
            'pct_other': pct_other,
        })

    return districts

def simulate_algorithmic_district_demographics(state_demo: StateDemographics) -> List[Dict]:
    """
    Simulate algorithmic (compact) district demographics

    Key assumption: Compact districts follow geographic patterns, not demographic patterns.
    This often results in:
    - Fewer majority-minority districts (minorities less "packed")
    - More even distribution of minorities across districts
    - Possible VRA compliance issues if minorities are geographically concentrated
    """
    random.seed(hash(state_demo.name + "_algorithmic"))

    districts = []

    for i in range(1, state_demo.num_districts + 1):
        # Algorithmic districts cluster around state average with less extreme variation
        # (compactness = geographic clustering, not demographic targeting)

        pct_black = state_demo.pct_black * random.uniform(0.6, 1.4)
        pct_hispanic = state_demo.pct_hispanic * random.uniform(0.6, 1.4)
        pct_asian = state_demo.pct_asian * random.uniform(0.7, 1.3)
        pct_other = state_demo.pct_other * random.uniform(0.8, 1.2)

        pct_white = 1.0 - (pct_black + pct_hispanic + pct_asian + pct_other)
        pct_white = max(0.0, pct_white)

        districts.append({
            'district': str(i),
            'total_pop': state_demo.total_pop // state_demo.num_districts,
            'pct_white': pct_white,
            'pct_black': pct_black,
            'pct_hispanic': pct_hispanic,
            'pct_asian': pct_asian,
            'pct_other': pct_other,
        })

    return districts

def main():
    output_dir = Path('outputs/data/2020/demographics')
    output_dir.mkdir(parents=True, exist_ok=True)

    print("=" * 70)
    print("Create Demographic Data for VRA Compliance Analysis")
    print("=" * 70)
    print()
    print("[!] NOTE: Simulates district demographics based on state-level Census data")
    print("    Enacted: Models VRA-compliant packing (majority-minority districts)")
    print("    Algorithmic: Models geographic compactness (even distribution)")
    print()

    # Generate enacted district demographics
    print("Generating enacted district demographics...")
    enacted_data = {}
    for state_name, state_demo in STATE_DEMOGRAPHICS.items():
        districts = simulate_enacted_district_demographics(state_demo)
        enacted_data[state_name] = districts

        mm_count = sum(1 for d in districts if
                      d['pct_black'] > 0.5 or d['pct_hispanic'] > 0.5 or d['pct_asian'] > 0.5)
        print(f"  {state_name:20s}: {state_demo.num_districts:2d} districts, {mm_count} majority-minority")

    # Save enacted
    json_file = output_dir / 'district_demographics_2020_enacted.json'
    with open(json_file, 'w') as f:
        json.dump(enacted_data, f, indent=2)
    print(f"\n[OK] Saved: {json_file}")

    # Generate algorithmic district demographics
    print("\nGenerating algorithmic district demographics...")
    algorithmic_data = {}
    for state_name, state_demo in STATE_DEMOGRAPHICS.items():
        districts = simulate_algorithmic_district_demographics(state_demo)
        algorithmic_data[state_name] = districts

        mm_count = sum(1 for d in districts if
                      d['pct_black'] > 0.5 or d['pct_hispanic'] > 0.5 or d['pct_asian'] > 0.5)
        print(f"  {state_name:20s}: {state_demo.num_districts:2d} districts, {mm_count} majority-minority")

    # Save algorithmic
    json_file = output_dir / 'district_demographics_2020_algorithmic.json'
    with open(json_file, 'w') as f:
        json.dump(algorithmic_data, f, indent=2)
    print(f"\n[OK] Saved: {json_file}")

    print("\n" + "=" * 70)
    print("SUCCESS - Demographic data generated")
    print("=" * 70)
    print()
    print("Next step: Compute VRA compliance metrics")
    print("  python scripts/demographic/compute_vra_compliance.py")

if __name__ == '__main__':
    main()
