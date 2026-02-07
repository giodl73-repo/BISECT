#!/usr/bin/env python3
"""
Compute VRA Compliance Metrics

Analyzes majority-minority districts in enacted vs algorithmic plans.

Majority-minority district: >50% minority population
- Black-majority
- Hispanic-majority
- Asian-majority
- Coalition-majority (combined minorities)

Usage:
    python scripts/demographic/compute_vra_compliance.py
"""

import json
import csv
from pathlib import Path
from typing import Dict, List, Tuple

def load_demographics(json_file: Path) -> Dict[str, List[Dict]]:
    """Load district demographics from JSON"""
    with open(json_file, 'r') as f:
        return json.load(f)

def count_majority_minority_districts(state_districts: List[Dict], threshold: float = 0.5) -> Dict:
    """
    Count majority-minority districts by type

    Returns:
        {
            'black': count of Black-majority districts,
            'hispanic': count of Hispanic-majority districts,
            'asian': count of Asian-majority districts,
            'coalition': count of coalition-majority districts (any minority >50%),
            'total': total MM districts (union of above)
        }
    """
    black_majority = []
    hispanic_majority = []
    asian_majority = []
    coalition_majority = []

    for district in state_districts:
        pct_black = district['pct_black']
        pct_hispanic = district['pct_hispanic']
        pct_asian = district['pct_asian']
        pct_minority = 1.0 - district['pct_white']

        if pct_black > threshold:
            black_majority.append(district['district'])
        if pct_hispanic > threshold:
            hispanic_majority.append(district['district'])
        if pct_asian > threshold:
            asian_majority.append(district['district'])
        if pct_minority > threshold:
            coalition_majority.append(district['district'])

    # Total unique MM districts
    all_mm = set(black_majority + hispanic_majority + asian_majority)

    return {
        'black': len(black_majority),
        'hispanic': len(hispanic_majority),
        'asian': len(asian_majority),
        'coalition': len(coalition_majority),
        'total': len(all_mm),
        'black_districts': black_majority,
        'hispanic_districts': hispanic_majority,
        'asian_districts': asian_majority,
    }

def compute_vra_status(enacted_mm: int, algorithmic_mm: int, state_total_districts: int) -> str:
    """
    Determine VRA compliance status

    Returns:
        - "Compliant": Algorithmic maintains or increases MM districts
        - "At Risk": Algorithmic loses 1 MM district
        - "Non-compliant": Algorithmic loses 2+ MM districts
        - "N/A": No MM districts in enacted plan
    """
    if enacted_mm == 0:
        return "N/A"
    elif algorithmic_mm >= enacted_mm:
        return "Compliant"
    elif enacted_mm - algorithmic_mm == 1:
        return "At Risk"
    else:
        return "Non-compliant"

def main():
    demographics_dir = Path('outputs/data/2020/demographics')
    output_dir = Path('research/gerry-edge-weighted-bisection/tables')
    output_dir.mkdir(parents=True, exist_ok=True)

    print("=" * 70)
    print("Compute VRA Compliance Metrics")
    print("=" * 70)
    print()

    # Load data
    enacted_file = demographics_dir / 'district_demographics_2020_enacted.json'
    algorithmic_file = demographics_dir / 'district_demographics_2020_algorithmic.json'

    print(f"Loading enacted demographics: {enacted_file}")
    enacted_data = load_demographics(enacted_file)
    print(f"[OK] Loaded {len(enacted_data)} states")

    print(f"Loading algorithmic demographics: {algorithmic_file}")
    algorithmic_data = load_demographics(algorithmic_file)
    print(f"[OK] Loaded {len(algorithmic_data)} states")

    # Compute metrics for each state
    print("\nComputing VRA compliance...")
    results = {}

    for state in sorted(enacted_data.keys()):
        enacted_districts = enacted_data[state]
        algorithmic_districts = algorithmic_data[state]

        enacted_mm = count_majority_minority_districts(enacted_districts)
        algorithmic_mm = count_majority_minority_districts(algorithmic_districts)

        num_districts = len(enacted_districts)
        mm_loss = enacted_mm['total'] - algorithmic_mm['total']
        vra_status = compute_vra_status(enacted_mm['total'], algorithmic_mm['total'], num_districts)

        results[state] = {
            'num_districts': num_districts,
            'enacted_total': enacted_mm['total'],
            'enacted_black': enacted_mm['black'],
            'enacted_hispanic': enacted_mm['hispanic'],
            'enacted_asian': enacted_mm['asian'],
            'algorithmic_total': algorithmic_mm['total'],
            'algorithmic_black': algorithmic_mm['black'],
            'algorithmic_hispanic': algorithmic_mm['hispanic'],
            'algorithmic_asian': algorithmic_mm['asian'],
            'difference': mm_loss,
            'vra_status': vra_status,
        }

        if mm_loss != 0:
            print(f"  {state:20s}: Enacted {enacted_mm['total']:2d} MM, Algorithmic {algorithmic_mm['total']:2d} MM, Diff {mm_loss:+3d} ({vra_status})")

    # Summary statistics
    total_enacted_mm = sum(r['enacted_total'] for r in results.values())
    total_algorithmic_mm = sum(r['algorithmic_total'] for r in results.values())
    total_loss = total_enacted_mm - total_algorithmic_mm

    states_with_loss = sum(1 for r in results.values() if r['difference'] > 0)
    states_at_risk = sum(1 for r in results.values() if r['vra_status'] == 'At Risk')
    states_non_compliant = sum(1 for r in results.values() if r['vra_status'] == 'Non-compliant')

    print(f"\n\nNational Summary:")
    print(f"  Total MM districts (enacted):      {total_enacted_mm}")
    print(f"  Total MM districts (algorithmic):  {total_algorithmic_mm}")
    print(f"  Net loss:                          {total_loss}")
    print(f"  States with MM loss:               {states_with_loss}/50")
    print(f"  States 'At Risk':                  {states_at_risk}/50")
    print(f"  States 'Non-compliant':            {states_non_compliant}/50")

    # Save JSON
    json_file = output_dir / 'vra_compliance_2020.json'
    with open(json_file, 'w') as f:
        json.dump(results, f, indent=2)
    print(f"\n[OK] Saved: {json_file}")

    # Save CSV
    csv_file = output_dir / 'vra_compliance_2020.csv'
    with open(csv_file, 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow([
            'state', 'num_districts',
            'enacted_total_mm', 'enacted_black', 'enacted_hispanic', 'enacted_asian',
            'algorithmic_total_mm', 'algorithmic_black', 'algorithmic_hispanic', 'algorithmic_asian',
            'difference', 'vra_status'
        ])

        for state, metrics in sorted(results.items()):
            writer.writerow([
                state,
                metrics['num_districts'],
                metrics['enacted_total'],
                metrics['enacted_black'],
                metrics['enacted_hispanic'],
                metrics['enacted_asian'],
                metrics['algorithmic_total'],
                metrics['algorithmic_black'],
                metrics['algorithmic_hispanic'],
                metrics['algorithmic_asian'],
                metrics['difference'],
                metrics['vra_status']
            ])

    print(f"[OK] Saved: {csv_file}")

    print("\n" + "=" * 70)
    print("SUCCESS - VRA compliance metrics computed")
    print("=" * 70)
    print()
    print("Next step: Generate LaTeX comparison tables")
    print("  python scripts/demographic/generate_vra_tables.py")

if __name__ == '__main__':
    main()
