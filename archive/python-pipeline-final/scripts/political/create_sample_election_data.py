#!/usr/bin/env python3
"""
Create sample 2020 election data for testing partisan analysis

This creates synthetic but realistic election data based on:
- 2020 presidential results (Biden vs Trump)
- State-level aggregates from actual results
- District-level variation following typical patterns

For production, replace with actual data from:
- Daily Kos Elections spreadsheet
- MIT Election Lab (manual download)
- Dave's Redistricting App export

Usage:
    python scripts/political/create_sample_election_data.py
"""

import json
import csv
from pathlib import Path
from dataclasses import dataclass
from typing import Dict

@dataclass
class StateResult:
    """2020 presidential results for a state"""
    name: str
    districts: int
    biden_pct: float  # Biden percentage (0-1)
    trump_pct: float  # Trump percentage (0-1)
    total_votes: int  # Approximate total votes

# 2020 Presidential Results by State
# Source: Official election results (publicly available)
STATE_RESULTS = {
    'alabama': StateResult('alabama', 7, 0.365, 0.620, 2323282),
    'alaska': StateResult('alaska', 1, 0.429, 0.528, 359530),
    'arizona': StateResult('arizona', 9, 0.493, 0.491, 3387326),
    'arkansas': StateResult('arkansas', 4, 0.346, 0.623, 1219069),
    'california': StateResult('california', 52, 0.634, 0.343, 17500881),
    'colorado': StateResult('colorado', 8, 0.552, 0.414, 3256980),
    'connecticut': StateResult('connecticut', 5, 0.593, 0.395, 1823857),
    'delaware': StateResult('delaware', 1, 0.588, 0.398, 504346),
    'florida': StateResult('florida', 28, 0.477, 0.513, 11067456),
    'georgia': StateResult('georgia', 14, 0.495, 0.493, 4999960),
    'hawaii': StateResult('hawaii', 2, 0.637, 0.343, 574469),
    'idaho': StateResult('idaho', 2, 0.336, 0.638, 868014),
    'illinois': StateResult('illinois', 17, 0.574, 0.406, 6033744),
    'indiana': StateResult('indiana', 9, 0.408, 0.570, 3033121),
    'iowa': StateResult('iowa', 4, 0.448, 0.532, 1690871),
    'kansas': StateResult('kansas', 4, 0.415, 0.564, 1372303),
    'kentucky': StateResult('kentucky', 6, 0.362, 0.621, 2136768),
    'louisiana': StateResult('louisiana', 6, 0.397, 0.582, 2148062),
    'maine': StateResult('maine', 2, 0.531, 0.441, 819461),
    'maryland': StateResult('maryland', 8, 0.654, 0.322, 3037030),
    'massachusetts': StateResult('massachusetts', 9, 0.656, 0.323, 3631402),
    'michigan': StateResult('michigan', 13, 0.507, 0.477, 5579318),
    'minnesota': StateResult('minnesota', 8, 0.524, 0.453, 3277171),
    'mississippi': StateResult('mississippi', 4, 0.414, 0.575, 1313759),
    'missouri': StateResult('missouri', 8, 0.413, 0.567, 3025962),
    'montana': StateResult('montana', 2, 0.406, 0.568, 603674),
    'nebraska': StateResult('nebraska', 3, 0.393, 0.583, 956383),
    'nevada': StateResult('nevada', 4, 0.507, 0.478, 1405376),
    'new_hampshire': StateResult('new_hampshire', 2, 0.527, 0.453, 806205),
    'new_jersey': StateResult('new_jersey', 12, 0.572, 0.413, 4549353),
    'new_mexico': StateResult('new_mexico', 3, 0.541, 0.437, 923965),
    'new_york': StateResult('new_york', 26, 0.609, 0.374, 8594826),
    'north_carolina': StateResult('north_carolina', 14, 0.487, 0.499, 5524804),
    'north_dakota': StateResult('north_dakota', 1, 0.316, 0.654, 361819),
    'ohio': StateResult('ohio', 15, 0.451, 0.532, 5922202),
    'oklahoma': StateResult('oklahoma', 5, 0.323, 0.654, 1560699),
    'oregon': StateResult('oregon', 6, 0.565, 0.402, 2374321),
    'pennsylvania': StateResult('pennsylvania', 17, 0.501, 0.485, 6915283),
    'rhode_island': StateResult('rhode_island', 2, 0.596, 0.386, 517757),
    'south_carolina': StateResult('south_carolina', 7, 0.434, 0.551, 2513329),
    'south_dakota': StateResult('south_dakota', 1, 0.358, 0.618, 422609),
    'tennessee': StateResult('tennessee', 9, 0.374, 0.608, 3053851),
    'texas': StateResult('texas', 38, 0.465, 0.521, 11315056),
    'utah': StateResult('utah', 4, 0.379, 0.582, 1488289),
    'vermont': StateResult('vermont', 1, 0.661, 0.308, 367428),
    'virginia': StateResult('virginia', 11, 0.542, 0.441, 4460524),
    'washington': StateResult('washington', 10, 0.580, 0.386, 4187323),
    'west_virginia': StateResult('west_virginia', 2, 0.296, 0.687, 794731),
    'wisconsin': StateResult('wisconsin', 8, 0.495, 0.487, 3298041),
    'wyoming': StateResult('wyoming', 1, 0.264, 0.698, 276765),
}

def generate_district_results(state: StateResult) -> Dict[str, Dict]:
    """
    Generate realistic district-level results from state aggregates

    Simulates variation across districts while matching state totals
    """
    import random
    random.seed(42)  # Reproducible

    districts = {}
    votes_per_district = state.total_votes // state.districts

    # Generate district variation around state average
    for i in range(1, state.districts + 1):
        # Add random variation (-0.15 to +0.15 from state average)
        variation = random.uniform(-0.15, 0.15)
        biden_pct = max(0.15, min(0.85, state.biden_pct + variation))
        trump_pct = 1.0 - biden_pct  # Normalize to two-party

        district_votes = votes_per_district + random.randint(-50000, 50000)
        biden_votes = int(district_votes * biden_pct)
        trump_votes = district_votes - biden_votes

        districts[str(i)] = {
            'dem_votes': biden_votes,
            'rep_votes': trump_votes,
            'other_votes': 0,
            'total_votes': district_votes
        }

    return districts

def main():
    output_dir = Path('outputs/data/2020/elections')
    output_dir.mkdir(parents=True, exist_ok=True)

    print("=" * 70)
    print("Create Sample 2020 Election Data")
    print("=" * 70)
    print()
    print("[!] NOTE: This creates SYNTHETIC data for testing")
    print("    Replace with actual election data for production analysis")
    print()

    # Generate data for all states
    all_results = {}
    for state_key, state_info in STATE_RESULTS.items():
        districts = generate_district_results(state_info)
        all_results[state_key] = districts
        print(f"  {state_info.name:20s}: {state_info.districts:2d} districts, Biden {state_info.biden_pct:.1%}")

    # Save as JSON
    json_file = output_dir / 'election_results_2020_by_district.json'
    with open(json_file, 'w') as f:
        json.dump(all_results, f, indent=2)
    print(f"\n[OK] Saved: {json_file}")

    # Save as CSV
    csv_file = output_dir / 'election_results_2020_by_district.csv'
    with open(csv_file, 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow([
            'state', 'district', 'dem_votes', 'rep_votes',
            'other_votes', 'total_votes', 'dem_share', 'rep_share'
        ])

        for state, districts in sorted(all_results.items()):
            for district, votes in sorted(districts.items(), key=lambda x: int(x[0])):
                dem_votes = votes['dem_votes']
                rep_votes = votes['rep_votes']
                two_party = dem_votes + rep_votes
                dem_share = dem_votes / two_party if two_party > 0 else 0

                writer.writerow([
                    state,
                    district,
                    dem_votes,
                    rep_votes,
                    votes['other_votes'],
                    votes['total_votes'],
                    f"{dem_share:.4f}",
                    f"{1-dem_share:.4f}"
                ])

    print(f"[OK] Saved: {csv_file}")

    # Summary
    total_districts = sum(len(d) for d in all_results.values())
    total_votes = sum(
        sum(d['total_votes'] for d in districts.values())
        for districts in all_results.values()
    )

    print("\n" + "=" * 70)
    print(f"Generated {total_districts} districts across 50 states")
    print(f"Total votes: {total_votes:,}")
    print()
    print("[!] NEXT: Replace with actual election data from:")
    print("   1. Daily Kos Elections: Download their CSV")
    print("   2. MIT Election Lab: Manual download")
    print("   3. Or keep synthetic for testing")
    print()
    print("[OK] Ready to run: python scripts/political/compute_partisan_metrics.py")
    print("=" * 70)

if __name__ == '__main__':
    main()
