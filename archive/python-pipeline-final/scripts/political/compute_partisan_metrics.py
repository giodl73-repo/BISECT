#!/usr/bin/env python3
"""
Compute Partisan Outcome Metrics for Algorithmic vs Enacted Districts

Metrics:
- Efficiency Gap: (wasted_votes_D - wasted_votes_R) / total_votes
- Mean-Median Difference: median(dem_shares) - mean(dem_shares)
- Partisan Bias: Expected seat advantage at 50% vote share
- Seats-Votes Curves: Relationship between vote share and seat share

Usage:
    python scripts/political/compute_partisan_metrics.py --year 2020
"""

import argparse
import csv
import json
import numpy as np
from pathlib import Path
from typing import Dict, List, Tuple
from scipy import stats

def load_election_data(election_file: Path) -> Dict[str, List[Dict]]:
    """
    Load election results by state and district

    Returns:
        {
            'alabama': [
                {'district': '1', 'dem_votes': 12345, 'rep_votes': 67890, ...},
                ...
            ],
            ...
        }
    """
    with open(election_file, 'r') as f:
        data = json.load(f)

    # Convert to list format for easier processing
    results = {}
    for state, districts in data.items():
        results[state] = []
        for district_id, votes in districts.items():
            results[state].append({
                'district': district_id,
                **votes
            })

    return results

def compute_efficiency_gap(district_results: List[Dict]) -> float:
    """
    Efficiency Gap: (wasted_votes_D - wasted_votes_R) / total_votes

    Wasted votes:
    - Losing party: all their votes
    - Winning party: votes beyond 50% threshold
    """
    total_wasted_d = 0
    total_wasted_r = 0
    total_votes = 0

    for district in district_results:
        dem_votes = district['dem_votes']
        rep_votes = district['rep_votes']
        two_party_total = dem_votes + rep_votes
        threshold = two_party_total / 2

        if dem_votes > rep_votes:  # Democrats win
            total_wasted_d += (dem_votes - threshold)
            total_wasted_r += rep_votes
        else:  # Republicans win
            total_wasted_d += dem_votes
            total_wasted_r += (rep_votes - threshold)

        total_votes += two_party_total

    if total_votes == 0:
        return 0.0

    efficiency_gap = (total_wasted_d - total_wasted_r) / total_votes
    return efficiency_gap

def compute_mean_median_difference(district_results: List[Dict]) -> float:
    """
    Mean-Median Difference: median(dem_shares) - mean(dem_shares)

    Negative value = pro-Republican bias
    Positive value = pro-Democratic bias
    """
    dem_shares = []
    for district in district_results:
        dem_votes = district['dem_votes']
        rep_votes = district['rep_votes']
        two_party_total = dem_votes + rep_votes

        if two_party_total > 0:
            dem_share = dem_votes / two_party_total
            dem_shares.append(dem_share)

    if not dem_shares:
        return 0.0

    median_share = np.median(dem_shares)
    mean_share = np.mean(dem_shares)

    return median_share - mean_share

def compute_partisan_bias(district_results: List[Dict]) -> Tuple[float, float]:
    """
    Partisan Bias: Expected seat advantage at 50% statewide vote share

    Uses seats-votes curve regression to estimate seat share at 50% vote

    Returns:
        (bias, r_squared) where bias = seats(0.5) - 0.5
    """
    # Compute statewide vote share and seat share
    total_dem_votes = sum(d['dem_votes'] for d in district_results)
    total_rep_votes = sum(d['rep_votes'] for d in district_results)
    total_votes = total_dem_votes + total_rep_votes

    if total_votes == 0:
        return 0.0, 0.0

    statewide_dem_share = total_dem_votes / total_votes

    # Count seats won
    dem_seats = sum(1 for d in district_results if d['dem_votes'] > d['rep_votes'])
    total_seats = len(district_results)
    dem_seat_share = dem_seats / total_seats if total_seats > 0 else 0

    # For single point, cannot fit curve - return simple bias
    if total_seats < 3:
        return dem_seat_share - 0.5, 1.0

    # Fit 3rd-order polynomial: seat_share = f(vote_share)
    # Use district-level data for more points
    vote_shares = []
    seat_outcomes = []  # 1 if Dem wins, 0 if Rep wins

    for district in district_results:
        dem_votes = district['dem_votes']
        rep_votes = district['rep_votes']
        two_party = dem_votes + rep_votes

        if two_party > 0:
            vote_shares.append(dem_votes / two_party)
            seat_outcomes.append(1 if dem_votes > rep_votes else 0)

    if len(vote_shares) < 3:
        return dem_seat_share - 0.5, 1.0

    # Fit polynomial
    try:
        coeffs = np.polyfit(vote_shares, seat_outcomes, deg=3)
        poly = np.poly1d(coeffs)

        # Evaluate at 50% vote share
        predicted_seats_at_50 = poly(0.5)

        # Bias = expected seat share at 50% vote - 0.5
        bias = predicted_seats_at_50 - 0.5

        # Compute R-squared
        predicted = poly(vote_shares)
        ss_res = np.sum((np.array(seat_outcomes) - predicted) ** 2)
        ss_tot = np.sum((np.array(seat_outcomes) - np.mean(seat_outcomes)) ** 2)
        r_squared = 1 - (ss_res / ss_tot) if ss_tot > 0 else 0

        return bias, r_squared

    except Exception:
        # Fallback to simple bias if polynomial fit fails
        return dem_seat_share - 0.5, 0.0

def compute_all_metrics(district_results: List[Dict]) -> Dict[str, float]:
    """Compute all partisan metrics for a state"""
    efficiency_gap = compute_efficiency_gap(district_results)
    mean_median = compute_mean_median_difference(district_results)
    partisan_bias, r_squared = compute_partisan_bias(district_results)

    return {
        'efficiency_gap': efficiency_gap,
        'mean_median_difference': mean_median,
        'partisan_bias': partisan_bias,
        'bias_r_squared': r_squared,
    }

def main():
    parser = argparse.ArgumentParser(
        description="Compute partisan outcome metrics"
    )
    parser.add_argument(
        '--year',
        type=int,
        default=2020,
        help='Census/election year (default: 2020)'
    )
    parser.add_argument(
        '--election-data',
        type=Path,
        default=Path('outputs/data/2020/elections/election_results_2020_by_district.json'),
        help='Election data file'
    )
    parser.add_argument(
        '--output',
        type=Path,
        default=Path('outputs/data/2020/partisan_metrics'),
        help='Output directory'
    )

    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)

    # Determine output suffix from input filename
    input_name = args.election_data.stem  # e.g., "algorithmic_district_votes_2020"
    if 'algorithmic' in input_name:
        output_suffix = 'algorithmic'
    elif 'enacted' in input_name or 'election_results' in input_name:
        output_suffix = 'enacted'
    else:
        output_suffix = 'unknown'

    print("=" * 70)
    print("Compute Partisan Outcome Metrics")
    print("=" * 70)
    print()

    # Load election data
    print(f"Loading election data: {args.election_data}")
    election_data = load_election_data(args.election_data)
    print(f"[OK] Loaded data for {len(election_data)} states")

    # Compute metrics for each state
    print("\nComputing partisan metrics...")
    results = {}

    for state, districts in sorted(election_data.items()):
        if not districts:
            continue

        metrics = compute_all_metrics(districts)
        results[state] = metrics

        # Print summary
        eg = metrics['efficiency_gap']
        mmd = metrics['mean_median_difference']
        bias = metrics['partisan_bias']

        bias_direction = "pro-D" if eg > 0 else "pro-R"
        print(f"  {state:20s}: EG={eg:+.3f} ({bias_direction}), MMD={mmd:+.3f}, Bias={bias:+.3f}")

    # Save results
    json_file = args.output / f'partisan_metrics_{args.year}_{output_suffix}.json'
    with open(json_file, 'w') as f:
        json.dump(results, f, indent=2)
    print(f"\n[OK] Saved: {json_file}")

    # Save as CSV
    csv_file = args.output / f'partisan_metrics_{args.year}_{output_suffix}.csv'
    with open(csv_file, 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow([
            'state', 'efficiency_gap', 'mean_median_difference',
            'partisan_bias', 'bias_r_squared', 'interpretation'
        ])

        for state, metrics in sorted(results.items()):
            eg = metrics['efficiency_gap']
            if abs(eg) < 0.02:
                interpretation = "Balanced"
            elif eg > 0:
                interpretation = f"Pro-D ({abs(eg):.1%})"
            else:
                interpretation = f"Pro-R ({abs(eg):.1%})"

            writer.writerow([
                state,
                f"{metrics['efficiency_gap']:.4f}",
                f"{metrics['mean_median_difference']:.4f}",
                f"{metrics['partisan_bias']:.4f}",
                f"{metrics['bias_r_squared']:.4f}",
                interpretation
            ])

    print(f"[OK] Saved: {csv_file}")

    print("\n" + "=" * 70)
    print(f"SUCCESS - Partisan metrics computed for {output_suffix} districts")
    print("=" * 70)

    if output_suffix == 'enacted':
        print(f"\nNext step: Compute metrics for algorithmic districts")
        print(f"  python scripts/political/compute_partisan_metrics.py \\")
        print(f"    --election-data outputs/data/2020/elections/algorithmic_district_votes_2020.json \\")
        print(f"    --output outputs/data/2020/partisan_metrics/")
    elif output_suffix == 'algorithmic':
        print(f"\nNext step: Generate comparison tables")
        print(f"  Compare enacted vs algorithmic metrics")
        print(f"  Generate LaTeX tables for paper")

if __name__ == '__main__':
    main()
