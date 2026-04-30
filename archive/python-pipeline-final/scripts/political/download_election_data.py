#!/usr/bin/env python3
"""
Download 2020 Presidential Election Data by Congressional District

Data source: MIT Election Data Lab
Alternative: Daily Kos Elections, Dave's Redistricting App

Usage:
    python scripts/political/download_election_data.py --year 2020 --output outputs/data/2020/elections/
"""

import argparse
import csv
import json
from pathlib import Path
from typing import Dict, List
import urllib.request

# MIT Election Lab 2020 presidential results by congressional district
MIT_ELECTION_LAB_URL = "https://dataverse.harvard.edu/api/access/datafile/4299753"

# Daily Kos Elections - 2020 presidential results
DAILY_KOS_2020_URL = "https://www.dailykos.com/stories/2021/2/5/2014924/"

def download_mit_data(output_dir: Path) -> Path:
    """
    Download MIT Election Lab data

    Returns path to downloaded CSV file
    """
    output_file = output_dir / "mit_election_lab_2020.csv"

    if output_file.exists():
        print(f"[OK] Election data already exists: {output_file}")
        return output_file

    print(f"Downloading election data from MIT Election Lab...")
    print(f"URL: {MIT_ELECTION_LAB_URL}")

    try:
        urllib.request.urlretrieve(MIT_ELECTION_LAB_URL, output_file)
        print(f"[OK] Downloaded to: {output_file}")
        return output_file
    except Exception as e:
        print(f"[FAIL] Download failed: {e}")
        print(f"\nAlternative: Download manually from:")
        print(f"  1. MIT Election Lab: https://electionlab.mit.edu/data")
        print(f"  2. Daily Kos Elections: {DAILY_KOS_2020_URL}")
        print(f"  3. Dave's Redistricting: https://davesredistricting.org/")
        raise

def parse_election_data(csv_file: Path) -> Dict[str, Dict]:
    """
    Parse election CSV into state -> district -> results mapping

    Expected CSV format:
        state, district, candidate, party, votes

    Returns:
        {
            'alabama': {
                '1': {'dem_votes': 12345, 'rep_votes': 67890, 'total': 80235},
                '2': {...},
                ...
            },
            ...
        }
    """
    results = {}

    with open(csv_file, 'r') as f:
        reader = csv.DictReader(f)

        for row in reader:
            state = row['state'].lower().replace(' ', '_')
            district = row['district']
            party = row['party'].lower()
            votes = int(row['votes'])

            if state not in results:
                results[state] = {}
            if district not in results[state]:
                results[state][district] = {
                    'dem_votes': 0,
                    'rep_votes': 0,
                    'other_votes': 0,
                    'total_votes': 0
                }

            if party in ['democrat', 'democratic']:
                results[state][district]['dem_votes'] += votes
            elif party in ['republican']:
                results[state][district]['rep_votes'] += votes
            else:
                results[state][district]['other_votes'] += votes

            results[state][district]['total_votes'] += votes

    return results

def save_election_data(results: Dict, output_dir: Path):
    """Save parsed election data as JSON and CSV"""

    # Save as JSON (full structure)
    json_file = output_dir / "election_results_2020_by_district.json"
    with open(json_file, 'w') as f:
        json.dump(results, f, indent=2)
    print(f"[OK] Saved JSON: {json_file}")

    # Save as CSV (flattened for analysis)
    csv_file = output_dir / "election_results_2020_by_district.csv"
    with open(csv_file, 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow([
            'state', 'district', 'dem_votes', 'rep_votes',
            'other_votes', 'total_votes', 'dem_share', 'rep_share'
        ])

        for state, districts in sorted(results.items()):
            for district, votes in sorted(districts.items()):
                dem_votes = votes['dem_votes']
                rep_votes = votes['rep_votes']
                other_votes = votes['other_votes']
                total_votes = votes['total_votes']

                two_party_total = dem_votes + rep_votes
                dem_share = dem_votes / two_party_total if two_party_total > 0 else 0
                rep_share = rep_votes / two_party_total if two_party_total > 0 else 0

                writer.writerow([
                    state, district, dem_votes, rep_votes,
                    other_votes, total_votes,
                    f"{dem_share:.4f}", f"{rep_share:.4f}"
                ])

    print(f"[OK] Saved CSV: {csv_file}")

def main():
    parser = argparse.ArgumentParser(
        description="Download 2020 presidential election data by congressional district"
    )
    parser.add_argument(
        '--year',
        type=int,
        default=2020,
        help='Election year (default: 2020)'
    )
    parser.add_argument(
        '--output',
        type=Path,
        default=Path('outputs/data/2020/elections'),
        help='Output directory (default: outputs/data/2020/elections)'
    )

    args = parser.parse_args()

    # Create output directory
    args.output.mkdir(parents=True, exist_ok=True)

    print("=" * 70)
    print("Download 2020 Presidential Election Data")
    print("=" * 70)
    print()

    # Download data
    csv_file = download_mit_data(args.output)

    # Parse data
    print("\nParsing election data...")
    results = parse_election_data(csv_file)

    # Summary statistics
    total_states = len(results)
    total_districts = sum(len(districts) for districts in results.values())
    print(f"[OK] Parsed {total_districts} districts across {total_states} states")

    # Save processed data
    print("\nSaving processed data...")
    save_election_data(results, args.output)

    print("\n" + "=" * 70)
    print("SUCCESS - Election data ready for partisan analysis")
    print("=" * 70)
    print(f"\nNext step: Run partisan metrics computation")
    print(f"  python scripts/political/compute_partisan_metrics.py")

if __name__ == '__main__':
    main()
