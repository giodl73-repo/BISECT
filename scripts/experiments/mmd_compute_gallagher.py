#!/usr/bin/env python3
"""
Gallagher Index (Least Squares Index) for Electoral Proportionality

Measures disproportionality between vote shares and seat shares.
Formula: sqrt(0.5 * sum((vote_share_i - seat_share_i)^2))

Scale:
  0 = Perfect proportionality
  1-2 = Very high proportionality (most PR systems)
  3-5 = Moderate proportionality
  10+ = High disproportionality (typical single-member FPTP)
  15+ = Very high disproportionality

Usage:
    python scripts/experiments/mmd_compute_gallagher.py --year 2020 --config uniform-5
    python scripts/experiments/mmd_compute_gallagher.py --year 2020 --compare-all
"""

import argparse
import sys
from pathlib import Path
from typing import List, Dict
import pandas as pd
import numpy as np

sys.path.insert(0, str(Path(__file__).parent.parent.parent / 'src'))


# ============================================================================
# Gallagher Index
# ============================================================================

def compute_gallagher_index(vote_shares: pd.Series, seat_shares: pd.Series) -> float:
    """
    Compute Gallagher index (least squares index) for proportionality.

    Args:
        vote_shares: Series of vote shares by party (as decimals, e.g., 0.52)
        seat_shares: Series of seat shares by party

    Returns:
        Gallagher index (0 = perfect proportionality, higher = more disproportional)

    Formula:
        G = sqrt(0.5 * sum((V_i - S_i)^2))
    """

    # Ensure same parties in both series
    assert len(vote_shares) == len(seat_shares), "Vote and seat shares must have same parties"

    # Compute squared deviations
    squared_deviations = (vote_shares - seat_shares) ** 2

    # Gallagher index
    gallagher = np.sqrt(0.5 * squared_deviations.sum())

    return gallagher


def compute_effective_threshold(num_seats: int) -> float:
    """
    Compute effective threshold for winning seats in a district.

    Uses Droop quota: votes / (seats + 1) + epsilon

    Example:
        3-member: 25%
        5-member: 16.7%
        7-member: 12.5%
    """

    return 1.0 / (num_seats + 1)


def interpret_gallagher(index: float) -> str:
    """Provide interpretation of Gallagher index value."""

    if index < 1.0:
        return "Exceptionally high proportionality"
    elif index < 2.0:
        return "Very high proportionality"
    elif index < 3.0:
        return "High proportionality"
    elif index < 5.0:
        return "Moderate proportionality"
    elif index < 8.0:
        return "Low proportionality"
    elif index < 12.0:
        return "Disproportional"
    else:
        return "Highly disproportional"


# ============================================================================
# Load and Compute
# ============================================================================

def load_mmd_results(config_dir: Path) -> pd.DataFrame:
    """Load national summary from electoral simulation."""

    results_path = config_dir / "national_summary.csv"

    if not results_path.exists():
        raise FileNotFoundError(
            f"Electoral results not found: {results_path}\n"
            f"Run mmd_simulate_election.py first"
        )

    return pd.read_csv(results_path)


def compute_proportionality_metrics(national_summary: pd.DataFrame) -> Dict:
    """Compute comprehensive proportionality metrics."""

    # Extract vote and seat shares
    vote_shares = national_summary['vote_share']
    seat_shares = national_summary['seat_share']

    # Gallagher index
    gallagher = compute_gallagher_index(vote_shares, seat_shares)

    # Deviation statistics
    deviations = national_summary['deviation']
    max_positive_deviation = deviations.max()
    max_negative_deviation = deviations.min()
    mean_absolute_deviation = deviations.abs().mean()

    # Identify biggest winners and losers
    biggest_winner = national_summary.loc[deviations.idxmax(), 'party']
    biggest_loser = national_summary.loc[deviations.idxmin(), 'party']

    # Minor party representation
    major_parties = ['Democratic', 'Republican']
    minor_party_seats = national_summary[~national_summary['party'].isin(major_parties)]['seats'].sum()
    total_seats = national_summary['seats'].sum()
    minor_party_share = minor_party_seats / total_seats

    metrics = {
        'gallagher_index': gallagher,
        'interpretation': interpret_gallagher(gallagher),
        'max_positive_deviation': max_positive_deviation,
        'max_negative_deviation': max_negative_deviation,
        'mean_absolute_deviation': mean_absolute_deviation,
        'biggest_winner': biggest_winner,
        'biggest_winner_deviation': max_positive_deviation,
        'biggest_loser': biggest_loser,
        'biggest_loser_deviation': max_negative_deviation,
        'minor_party_seats': int(minor_party_seats),
        'minor_party_share': minor_party_share,
        'total_seats': int(total_seats)
    }

    return metrics


def compare_configurations(base_dir: Path, year: int) -> pd.DataFrame:
    """Compare Gallagher indices across all MMD configurations."""

    print("\n[Comparing All Configurations]")

    # Find all configuration directories
    config_dirs = [d for d in base_dir.iterdir() if d.is_dir()]

    if len(config_dirs) == 0:
        raise ValueError(f"No configuration directories found in {base_dir}")

    print(f"Found {len(config_dirs)} configurations")

    results = []

    for config_dir in config_dirs:
        config_name = config_dir.name

        try:
            # Load results
            national_summary = load_mmd_results(config_dir)

            # Compute metrics
            metrics = compute_proportionality_metrics(national_summary)

            # Add configuration info
            metrics['config_name'] = config_name
            metrics['year'] = year

            # Parse configuration
            if 'uniform' in config_name:
                members = int(config_name.split('-')[1])
                metrics['system_type'] = 'uniform'
                metrics['members'] = members
                metrics['threshold'] = None
            else:
                parts = config_name.split('__')
                metrics['system_type'] = parts[0]
                metrics['threshold'] = parts[1] if len(parts) > 1 else None
                metrics['members'] = None

            results.append(metrics)

        except FileNotFoundError as e:
            print(f"  Skipping {config_name}: {e}")
            continue

    if len(results) == 0:
        raise ValueError("No valid configurations found")

    results_df = pd.DataFrame(results)

    # Sort by Gallagher index
    results_df = results_df.sort_values('gallagher_index')

    return results_df


# ============================================================================
# Baseline Comparisons
# ============================================================================

def load_single_member_baseline(year: int) -> float:
    """Load Gallagher index for single-member algorithmic baseline."""

    baseline_path = Path(f"outputs/v1/{year}/national/proportionality_metrics.csv")

    if baseline_path.exists():
        baseline = pd.read_csv(baseline_path)
        return baseline['gallagher_index'].iloc[0]

    # Compute from seat allocation
    seats_path = Path(f"outputs/v1/{year}/national/seats_by_party.csv")

    if not seats_path.exists():
        print(f"Warning: Single-member baseline not found")
        return None

    seats = pd.read_csv(seats_path)

    # Assume vote shares are available
    if 'vote_share' in seats.columns and 'seat_share' in seats.columns:
        return compute_gallagher_index(seats['vote_share'], seats['seat_share'])

    return None


def load_enacted_baseline(year: int) -> float:
    """Load Gallagher index for enacted (actual) districts."""

    enacted_path = Path(f"data/{year}/redistricting/enacted_proportionality.csv")

    if enacted_path.exists():
        enacted = pd.read_csv(enacted_path)
        return enacted['gallagher_index'].iloc[0]

    print(f"Warning: Enacted baseline not found")
    return None


# ============================================================================
# Main
# ============================================================================

def main():
    parser = argparse.ArgumentParser(description="Compute Gallagher index for MMD configurations")
    parser.add_argument('--year', type=int, default=2020, choices=[2000, 2010, 2020])
    parser.add_argument('--config', type=str,
                        help='Single configuration to analyze (default: analyze all)')
    parser.add_argument('--compare-all', action='store_true',
                        help='Compare all configurations')
    parser.add_argument('--input-dir', type=Path, default=Path('outputs/mmd'))
    parser.add_argument('--output-dir', type=Path, default=Path('outputs/mmd'))

    args = parser.parse_args()

    print("="*70)
    print("GALLAGHER INDEX COMPUTATION")
    print("="*70)
    print(f"Year: {args.year}")

    if args.compare_all or not args.config:
        # Compare all configurations
        comparison_df = compare_configurations(args.input_dir, args.year)

        # Load baselines
        single_member_gallagher = load_single_member_baseline(args.year)
        enacted_gallagher = load_enacted_baseline(args.year)

        print("\n[Proportionality Comparison]")
        print("\nBaselines:")
        if enacted_gallagher:
            print(f"  Enacted (actual districts): {enacted_gallagher:.3f} - {interpret_gallagher(enacted_gallagher)}")
        if single_member_gallagher:
            print(f"  Algorithmic single-member: {single_member_gallagher:.3f} - {interpret_gallagher(single_member_gallagher)}")

        print("\nMMD Configurations:")
        display_cols = ['config_name', 'gallagher_index', 'interpretation',
                        'minor_party_seats', 'minor_party_share']
        print(comparison_df[display_cols].to_string(index=False))

        # Identify best configuration
        best_config = comparison_df.iloc[0]
        print(f"\n[Best Configuration]")
        print(f"  Config: {best_config['config_name']}")
        print(f"  Gallagher Index: {best_config['gallagher_index']:.3f}")
        print(f"  Minor party seats: {best_config['minor_party_seats']}")

        # Save comparison
        output_path = args.output_dir / f"gallagher_comparison_{args.year}.csv"
        comparison_df.to_csv(output_path, index=False)
        print(f"\nSaved: {output_path}")

    else:
        # Single configuration analysis
        config_dir = args.input_dir / args.config

        print(f"Configuration: {args.config}")

        national_summary = load_mmd_results(config_dir)
        metrics = compute_proportionality_metrics(national_summary)

        print(f"\n[Proportionality Metrics]")
        print(f"Gallagher Index: {metrics['gallagher_index']:.3f}")
        print(f"Interpretation: {metrics['interpretation']}")
        print(f"\nDeviations:")
        print(f"  Mean absolute: {metrics['mean_absolute_deviation']:.4f}")
        print(f"  Biggest winner: {metrics['biggest_winner']} (+{metrics['biggest_winner_deviation']:.4f})")
        print(f"  Biggest loser: {metrics['biggest_loser']} ({metrics['biggest_loser_deviation']:.4f})")
        print(f"\nMinor Parties:")
        print(f"  Total seats: {metrics['minor_party_seats']}/{metrics['total_seats']}")
        print(f"  Share: {metrics['minor_party_share']*100:.1f}%")

        # Save metrics
        output_path = config_dir / "proportionality_metrics.csv"
        pd.DataFrame([metrics]).to_csv(output_path, index=False)
        print(f"\nSaved: {output_path}")

    print(f"\n[SUCCESS] Gallagher index computation complete")


if __name__ == '__main__':
    main()
