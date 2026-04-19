"""
Compare algorithmic districts to enacted 2020 congressional districts.

Creates state-by-state comparison of compactness scores (Polsby-Popper, Reock)
between the algorithmically-generated districts and the actual enacted districts.

Outputs:
- State-level comparison table (CSV)
- National summary statistics
- Improvement percentages
"""

import argparse
import sys
from pathlib import Path

import pandas as pd
import numpy as np

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from config_2020 import STATE_CONFIG_2020


def load_algorithmic_results(input_dir: Path) -> pd.DataFrame:
    """
    Load algorithmic district results.

    Parameters
    ----------
    input_dir : Path
        Directory containing us_district_summary_projected.csv (or us_district_summary.csv)

    Returns
    -------
    pd.DataFrame
        State-level summary with mean PP and Reock per state
    """
    # Try projected file first (properly computed with Albers projection)
    projected_file = input_dir / 'us_district_summary_projected.csv'
    if projected_file.exists():
        summary_file = projected_file
    else:
        summary_file = input_dir / 'us_district_summary.csv'

    df = pd.read_csv(summary_file)

    # Compute state-level means
    state_summary = df.groupby('state_code').agg({
        'district': 'count',
        'polsby_popper': 'mean',
        'reock': 'mean',
    }).reset_index()

    state_summary.rename(columns={
        'state_code': 'state',
        'district': 'num_districts',
        'polsby_popper': 'mean_pp',
        'reock': 'mean_reock',
    }, inplace=True)

    return state_summary


def load_enacted_results(input_dir: Path) -> pd.DataFrame:
    """
    Load enacted district results.

    Parameters
    ----------
    input_dir : Path
        Directory containing enacted_state_summary.csv

    Returns
    -------
    pd.DataFrame
        State-level summary with mean PP and Reock per state
    """
    summary_file = input_dir / 'enacted_state_summary.csv'
    df = pd.read_csv(summary_file)

    return df[[  'state', 'num_districts', 'mean_pp', 'mean_reock']]


def create_comparison(algorithmic_df: pd.DataFrame, enacted_df: pd.DataFrame) -> pd.DataFrame:
    """
    Create state-by-state comparison table.

    Parameters
    ----------
    algorithmic_df : pd.DataFrame
        Algorithmic results
    enacted_df : pd.DataFrame
        Enacted results

    Returns
    -------
    pd.DataFrame
        Comparison with columns:
        - state
        - num_districts
        - algo_pp, enacted_pp, pp_improvement_pct
        - algo_reock, enacted_reock, reock_improvement_pct
    """
    # Merge on state
    comparison = pd.merge(
        algorithmic_df,
        enacted_df,
        on='state',
        suffixes=('_algo', '_enacted')
    )

    # Compute improvement percentages
    comparison['pp_improvement_pct'] = (
        (comparison['mean_pp_algo'] - comparison['mean_pp_enacted']) /
        comparison['mean_pp_enacted'] * 100
    )

    comparison['reock_improvement_pct'] = (
        (comparison['mean_reock_algo'] - comparison['mean_reock_enacted']) /
        comparison['mean_reock_enacted'] * 100
    )

    # Rename columns for clarity
    comparison.rename(columns={
        'num_districts_algo': 'num_districts',
        'mean_pp_algo': 'algo_pp',
        'mean_pp_enacted': 'enacted_pp',
        'mean_reock_algo': 'algo_reock',
        'mean_reock_enacted': 'enacted_reock',
    }, inplace=True)

    # Drop duplicate num_districts column
    comparison = comparison[[
        'state', 'num_districts',
        'algo_pp', 'enacted_pp', 'pp_improvement_pct',
        'algo_reock', 'enacted_reock', 'reock_improvement_pct'
    ]]

    # Sort by PP improvement (descending)
    comparison = comparison.sort_values('pp_improvement_pct', ascending=False)

    return comparison


def print_summary_statistics(comparison_df: pd.DataFrame):
    """Print summary statistics."""
    print("\n" + "=" * 60)
    print("National Summary Statistics")
    print("=" * 60)
    print()

    # Overall means
    algo_pp_mean = comparison_df['algo_pp'].mean()
    enacted_pp_mean = comparison_df['enacted_pp'].mean()
    pp_improvement = ((algo_pp_mean - enacted_pp_mean) / enacted_pp_mean) * 100

    algo_reock_mean = comparison_df['algo_reock'].mean()
    enacted_reock_mean = comparison_df['enacted_reock'].mean()
    reock_improvement = ((algo_reock_mean - enacted_reock_mean) / enacted_reock_mean) * 100

    print("Polsby-Popper Compactness:")
    print(f"  Algorithmic:  {algo_pp_mean:.4f}")
    print(f"  Enacted:      {enacted_pp_mean:.4f}")
    print(f"  Improvement:  {pp_improvement:+.1f}%")
    print()

    print("Reock Compactness:")
    print(f"  Algorithmic:  {algo_reock_mean:.4f}")
    print(f"  Enacted:      {enacted_reock_mean:.4f}")
    print(f"  Improvement:  {reock_improvement:+.1f}%")
    print()

    # States better/worse
    pp_better = (comparison_df['pp_improvement_pct'] > 0).sum()
    pp_worse = (comparison_df['pp_improvement_pct'] < 0).sum()

    reock_better = (comparison_df['reock_improvement_pct'] > 0).sum()
    reock_worse = (comparison_df['reock_improvement_pct'] < 0).sum()

    print("State Comparison:")
    print(f"  PP: {pp_better} states better, {pp_worse} states worse")
    print(f"  Reock: {reock_better} states better, {reock_worse} states worse")
    print()

    # Top 5 improvements
    print("Top 5 States (PP Improvement):")
    for idx, row in comparison_df.head(5).iterrows():
        print(f"  {row['state']}: {row['pp_improvement_pct']:+.1f}% "
              f"(algo={row['algo_pp']:.3f}, enacted={row['enacted_pp']:.3f})")
    print()

    # Bottom 5 (largest declines)
    print("Bottom 5 States (PP Decline):")
    for idx, row in comparison_df.tail(5).iterrows():
        print(f"  {row['state']}: {row['pp_improvement_pct']:+.1f}% "
              f"(algo={row['algo_pp']:.3f}, enacted={row['enacted_pp']:.3f})")

    print("=" * 60)


def main():
    parser = argparse.ArgumentParser(
        description='Compare algorithmic vs enacted congressional districts'
    )
    parser.add_argument(
        '--algorithmic-dir',
        type=Path,
        default=Path('outputs/us_2020_v1'),
        help='Directory with algorithmic results (default: outputs/us_2020_v1)'
    )
    parser.add_argument(
        '--enacted-dir',
        type=Path,
        default=Path('outputs/baseline_comparison'),
        help='Directory with enacted results (default: outputs/baseline_comparison)'
    )
    parser.add_argument(
        '--output-dir',
        type=Path,
        default=Path('outputs/baseline_comparison'),
        help='Output directory (default: outputs/baseline_comparison)'
    )

    args = parser.parse_args()

    # Create output directory
    args.output_dir.mkdir(parents=True, exist_ok=True)

    print("=" * 60)
    print("Comparing Algorithmic vs Enacted Congressional Districts")
    print("=" * 60)
    print()

    # Load results
    print("Loading algorithmic results...")
    algorithmic_df = load_algorithmic_results(args.algorithmic_dir)
    print(f"  Loaded {len(algorithmic_df)} states")

    print("\nLoading enacted results...")
    enacted_df = load_enacted_results(args.enacted_dir)
    print(f"  Loaded {len(enacted_df)} states")

    # Create comparison
    print("\nCreating comparison table...")
    comparison_df = create_comparison(algorithmic_df, enacted_df)

    # Save comparison
    output_file = args.output_dir / 'algorithmic_vs_enacted_comparison.csv'
    comparison_df.to_csv(output_file, index=False)
    print(f"OK Saved comparison table: {output_file}")

    # Print summary statistics
    print_summary_statistics(comparison_df)


if __name__ == '__main__':
    main()
