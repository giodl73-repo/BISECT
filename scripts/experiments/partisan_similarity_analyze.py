#!/usr/bin/env python3
"""
Partisan Similarity Analysis (Paper 17)

Analyzes results from partisan similarity experiments and generates figures/tables
for the paper.

Usage:
    python scripts/experiments/partisan_similarity_analyze.py --year 2020
    python scripts/experiments/partisan_similarity_analyze.py --year 2020 --compare-enacted
"""

import argparse
from pathlib import Path
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
import numpy as np

sns.set_style('whitegrid')


def load_all_results(year: int, base_dir: Path = None) -> pd.DataFrame:
    """Load results from all partisan similarity configurations."""

    if base_dir is None:
        base_dir = Path(f"outputs/partisan_similarity/{year}")

    if not base_dir.exists():
        raise FileNotFoundError(f"Results directory not found: {base_dir}")

    # Find all configuration directories
    config_dirs = [d for d in base_dir.iterdir() if d.is_dir() and d.name.startswith('alpha')]

    print(f"Found {len(config_dirs)} configurations")

    # Load district statistics from each config
    all_results = []

    for config_dir in sorted(config_dirs):
        stats_file = config_dir / "district_statistics.csv"

        if not stats_file.exists():
            print(f"  WARNING: Missing {stats_file}")
            continue

        # Parse config name (e.g., "alpha10_tau15")
        parts = config_dir.name.split('_')
        alpha = float(parts[0].replace('alpha', ''))
        tau = float(parts[1].replace('tau', ''))

        # Load stats
        stats = pd.read_csv(stats_file)
        stats['alpha'] = alpha
        stats['tau'] = tau
        stats['config'] = config_dir.name

        all_results.append(stats)

        print(f"  Loaded: α={alpha}, τ={tau} ({len(stats)} districts)")

    if not all_results:
        raise ValueError("No results found")

    combined = pd.concat(all_results, ignore_index=True)
    print(f"\nTotal: {len(combined):,} district records across {len(config_dirs)} configs")

    return combined


def compute_aggregate_metrics(df: pd.DataFrame) -> pd.DataFrame:
    """Compute aggregate metrics per configuration."""

    # Group by config
    grouped = df.groupby(['alpha', 'tau', 'config'])

    metrics = []

    for (alpha, tau, config), group in grouped:
        # Safe seat counts
        safe_10 = (group['partisan_lean'].abs() > 10).sum()
        safe_15 = (group['partisan_lean'].abs() > 15).sum()
        safe_20 = (group['partisan_lean'].abs() > 20).sum()

        total_districts = len(group)

        metrics.append({
            'alpha': alpha,
            'tau': tau,
            'config': config,
            'num_districts': total_districts,
            'mean_abs_lean': group['partisan_lean'].abs().mean(),
            'std_lean': group['partisan_lean'].std(),
            'safe_10_count': safe_10,
            'safe_10_pct': safe_10 / total_districts * 100,
            'safe_15_count': safe_15,
            'safe_15_pct': safe_15 / total_districts * 100,
            'safe_20_count': safe_20,
            'safe_20_pct': safe_20 / total_districts * 100,
            'mean_polsby_popper': group['polsby_popper'].mean(),
            'median_polsby_popper': group['polsby_popper'].median(),
        })

    return pd.DataFrame(metrics)


def plot_compactness_homogeneity_tradeoff(agg: pd.DataFrame, output_dir: Path):
    """Plot Pareto frontier: compactness vs partisan homogeneity."""

    # Filter to tau=15 (middle threshold) for cleaner plot
    data = agg[agg['tau'] == 15].sort_values('alpha')

    fig, ax = plt.subplots(figsize=(10, 6))

    # Scatter plot
    ax.scatter(data['mean_abs_lean'], data['mean_polsby_popper'],
               s=200, alpha=0.7, c=data['alpha'], cmap='viridis')

    # Connect points
    ax.plot(data['mean_abs_lean'], data['mean_polsby_popper'],
            '--', alpha=0.5, color='gray')

    # Annotate alpha values
    for _, row in data.iterrows():
        ax.annotate(f"α={row['alpha']:.0f}",
                    (row['mean_abs_lean'], row['mean_polsby_popper']),
                    textcoords="offset points", xytext=(0, 10),
                    ha='center', fontsize=10)

    ax.set_xlabel('Mean Partisan Lean (abs, pp)', fontsize=12)
    ax.set_ylabel('Mean Polsby-Popper Compactness', fontsize=12)
    ax.set_title('Compactness-Homogeneity Trade-Off (τ=15pp)', fontsize=14)
    ax.grid(alpha=0.3)

    plt.tight_layout()
    plt.savefig(output_dir / 'tradeoff_curve.png', dpi=300)
    print(f"  Saved: {output_dir / 'tradeoff_curve.png'}")
    plt.close()


def plot_safe_seats_by_alpha(agg: pd.DataFrame, output_dir: Path):
    """Plot safe seat counts vs alpha."""

    # Filter to tau=15
    data = agg[agg['tau'] == 15].sort_values('alpha')

    fig, ax = plt.subplots(figsize=(10, 6))

    # Plot lines for each threshold
    ax.plot(data['alpha'], data['safe_10_pct'], 'o-', label='>10pp', linewidth=2)
    ax.plot(data['alpha'], data['safe_15_pct'], 's-', label='>15pp', linewidth=2)
    ax.plot(data['alpha'], data['safe_20_pct'], '^-', label='>20pp', linewidth=2)

    ax.set_xlabel('Edge Weight Scaling Factor (α)', fontsize=12)
    ax.set_ylabel('Safe Seats (%)', fontsize=12)
    ax.set_title('Safe Seat Creation by Partisan Similarity Weighting (τ=15pp)', fontsize=14)
    ax.set_xscale('log')
    ax.legend(fontsize=11)
    ax.grid(alpha=0.3)

    plt.tight_layout()
    plt.savefig(output_dir / 'safe_seats_by_alpha.png', dpi=300)
    print(f"  Saved: {output_dir / 'safe_seats_by_alpha.png'}")
    plt.close()


def generate_summary_table(agg: pd.DataFrame, output_dir: Path):
    """Generate LaTeX table summarizing all configurations."""

    # Select key configs for table (fixed tau=15, vary alpha)
    table_data = agg[agg['tau'] == 15][['alpha', 'mean_abs_lean', 'safe_10_pct',
                                         'safe_20_pct', 'mean_polsby_popper']].copy()

    table_data = table_data.sort_values('alpha')

    # Format columns
    table_data['alpha'] = table_data['alpha'].astype(int)
    table_data['mean_abs_lean'] = table_data['mean_abs_lean'].round(1)
    table_data['safe_10_pct'] = table_data['safe_10_pct'].round(1)
    table_data['safe_20_pct'] = table_data['safe_20_pct'].round(1)
    table_data['mean_polsby_popper'] = table_data['mean_polsby_popper'].round(3)

    # Column names for LaTeX
    table_data.columns = ['α', 'Mean |Lean| (pp)', 'Safe >10pp (%)',
                          'Super-Safe >20pp (%)', 'Polsby-Popper']

    # Generate LaTeX
    latex = table_data.to_latex(index=False, float_format='%.3f')

    # Save
    output_file = output_dir / 'summary_table.tex'
    with open(output_file, 'w') as f:
        f.write(latex)

    print(f"  Saved: {output_file}")

    # Also save CSV
    csv_file = output_dir / 'summary_table.csv'
    table_data.to_csv(csv_file, index=False)
    print(f"  Saved: {csv_file}")


def compare_to_baseline(agg: pd.DataFrame):
    """Compare strong weighting to baseline (alpha=1)."""

    baseline = agg[(agg['alpha'] == 1) & (agg['tau'] == 15)].iloc[0]
    strong = agg[(agg['alpha'] == 50) & (agg['tau'] == 15)].iloc[0]

    print(f"\nComparison: Baseline (α=1) vs Strong (α=50), τ=15pp")
    print(f"{'='*70}")
    print(f"                      Baseline      Strong      Change")
    print(f"{'-'*70}")
    print(f"Mean |Lean| (pp)      {baseline['mean_abs_lean']:6.1f}      {strong['mean_abs_lean']:6.1f}      +{strong['mean_abs_lean']-baseline['mean_abs_lean']:5.1f}")
    print(f"Safe >10pp (%)        {baseline['safe_10_pct']:6.1f}      {strong['safe_10_pct']:6.1f}      +{strong['safe_10_pct']-baseline['safe_10_pct']:5.1f}")
    print(f"Super-Safe >20pp (%)  {baseline['safe_20_pct']:6.1f}      {strong['safe_20_pct']:6.1f}      +{strong['safe_20_pct']-baseline['safe_20_pct']:5.1f}")
    print(f"Polsby-Popper         {baseline['mean_polsby_popper']:6.3f}      {strong['mean_polsby_popper']:6.3f}      {strong['mean_polsby_popper']-baseline['mean_polsby_popper']:+6.3f}")


def main():
    parser = argparse.ArgumentParser(description="Analyze partisan similarity results")
    parser.add_argument('--year', type=int, default=2020, help="Census year")
    parser.add_argument('--compare-enacted', action='store_true',
                        help="Compare to enacted districts (if available)")

    args = parser.parse_args()

    # Load all results
    print(f"Loading results for {args.year}...")
    results = load_all_results(args.year)

    # Compute aggregate metrics
    print(f"\nComputing aggregate metrics...")
    agg = compute_aggregate_metrics(results)
    print(f"Computed metrics for {len(agg)} configurations")

    # Create output directory
    output_dir = Path(f"research/17+partisan-similarity-districts/analysis")
    output_dir.mkdir(parents=True, exist_ok=True)
    print(f"\nOutput directory: {output_dir}/")

    # Generate figures
    print("\nGenerating figures...")
    plot_compactness_homogeneity_tradeoff(agg, output_dir)
    plot_safe_seats_by_alpha(agg, output_dir)

    # Generate tables
    print("\nGenerating tables...")
    generate_summary_table(agg, output_dir)

    # Print comparison
    compare_to_baseline(agg)

    # Save full aggregate metrics
    agg.to_csv(output_dir / 'aggregate_metrics.csv', index=False)
    print(f"\n  Saved: {output_dir / 'aggregate_metrics.csv'}")

    print(f"\n{'='*70}")
    print("Analysis complete!")
    print(f"{'='*70}\n")


if __name__ == '__main__':
    main()
