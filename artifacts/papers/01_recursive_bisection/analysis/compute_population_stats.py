#!/usr/bin/env python3
"""
Compute population deviation statistics across all 435 congressional districts.

Analyzes how closely districts match the ideal population (total_pop / 435)
and generates summary statistics and visualizations.
"""

import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
from pathlib import Path
import sys

def compute_population_statistics(output_dir='outputs/us_2020_v1'):
    """Compute population deviation statistics."""

    output_dir = Path(output_dir)
    us_summary = output_dir / 'us_district_summary.csv'

    if not us_summary.exists():
        print(f"ERROR: {us_summary} not found")
        print("Run the full pipeline first to generate district data")
        return None

    # Load district data
    print(f"Loading district data from {us_summary}")
    df = pd.read_csv(us_summary)

    # Calculate ideal population
    total_pop = df['population'].sum()
    ideal_pop = total_pop / 435

    print(f"\nTotal US Population (2020 Census): {total_pop:,}")
    print(f"Number of Districts: {len(df)}")
    print(f"Ideal Population per District: {ideal_pop:,.0f}")

    # Calculate deviations
    df['deviation'] = df['population'] - ideal_pop
    df['deviation_pct'] = (df['deviation'] / ideal_pop) * 100
    df['abs_deviation'] = df['deviation'].abs()
    df['abs_deviation_pct'] = df['deviation_pct'].abs()

    # Summary statistics
    stats = {
        'total_population': total_pop,
        'num_districts': len(df),
        'ideal_population': ideal_pop,
        'mean_population': df['population'].mean(),
        'median_population': df['population'].median(),
        'std_population': df['population'].std(),
        'min_population': df['population'].min(),
        'max_population': df['population'].max(),
        'range_population': df['population'].max() - df['population'].min(),
        'mean_abs_deviation': df['abs_deviation'].mean(),
        'median_abs_deviation': df['abs_deviation'].median(),
        'std_deviation': df['deviation'].std(),
        'max_positive_deviation': df['deviation'].max(),
        'max_negative_deviation': df['deviation'].min(),
        'mean_abs_deviation_pct': df['abs_deviation_pct'].mean(),
        'median_abs_deviation_pct': df['abs_deviation_pct'].median(),
        'max_abs_deviation_pct': df['abs_deviation_pct'].max(),
        'districts_within_1pct': (df['abs_deviation_pct'] <= 1.0).sum(),
        'districts_within_0.5pct': (df['abs_deviation_pct'] <= 0.5).sum(),
        'pct_within_1pct': (df['abs_deviation_pct'] <= 1.0).sum() / len(df) * 100,
        'pct_within_0.5pct': (df['abs_deviation_pct'] <= 0.5).sum() / len(df) * 100,
    }

    # Print summary
    print("\n" + "="*70)
    print("POPULATION DEVIATION STATISTICS")
    print("="*70)
    print(f"Mean Absolute Deviation: {stats['mean_abs_deviation']:,.0f} people ({stats['mean_abs_deviation_pct']:.3f}%)")
    print(f"Median Absolute Deviation: {stats['median_abs_deviation']:,.0f} people ({stats['median_abs_deviation_pct']:.3f}%)")
    print(f"Standard Deviation: {stats['std_deviation']:,.0f} people")
    print(f"Maximum Positive Deviation: +{stats['max_positive_deviation']:,.0f} people")
    print(f"Maximum Negative Deviation: {stats['max_negative_deviation']:,.0f} people")
    print(f"Maximum Absolute Deviation: {stats['max_abs_deviation_pct']:.3f}%")
    print(f"\nDistricts within ±1.0% of ideal: {stats['districts_within_1pct']}/435 ({stats['pct_within_1pct']:.1f}%)")
    print(f"Districts within ±0.5% of ideal: {stats['districts_within_0.5pct']}/435 ({stats['pct_within_0.5pct']:.1f}%)")

    # State-by-state summary
    print("\n" + "="*70)
    print("STATE-BY-STATE SUMMARY (Top 10 by max deviation)")
    print("="*70)

    state_stats = df.groupby('state').agg({
        'population': ['count', 'mean', 'std'],
        'abs_deviation_pct': ['mean', 'max']
    }).round(3)
    state_stats.columns = ['num_districts', 'mean_pop', 'std_pop', 'mean_dev_pct', 'max_dev_pct']
    state_stats = state_stats.sort_values('max_dev_pct', ascending=False)
    print(state_stats.head(10))

    return df, stats

def create_visualizations(df, stats, output_dir='paper/figures'):
    """Create visualizations of population statistics."""

    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    print(f"\n" + "="*70)
    print("GENERATING VISUALIZATIONS")
    print("="*70)

    # 1. Histogram of population deviations (percentage)
    fig, ax = plt.subplots(figsize=(10, 6))

    ax.hist(df['deviation_pct'], bins=50, edgecolor='black', alpha=0.7, color='steelblue')
    ax.axvline(0, color='red', linestyle='--', linewidth=2, label='Ideal Population')
    ax.axvline(-1, color='orange', linestyle=':', linewidth=1.5, label='±1% Constitutional Limit')
    ax.axvline(1, color='orange', linestyle=':', linewidth=1.5)

    ax.set_xlabel('Population Deviation (%)', fontsize=12)
    ax.set_ylabel('Number of Districts', fontsize=12)
    ax.set_title('Population Deviation from Ideal\n435 Congressional Districts (2020 Census)',
                 fontsize=14, fontweight='bold')
    ax.legend(fontsize=10)
    ax.grid(True, alpha=0.3)

    # Add statistics text box
    textstr = f'Mean: {stats["mean_abs_deviation_pct"]:.3f}%\n'
    textstr += f'Median: {stats["median_abs_deviation_pct"]:.3f}%\n'
    textstr += f'Max: {stats["max_abs_deviation_pct"]:.3f}%\n'
    textstr += f'Within ±1%: {stats["pct_within_1pct"]:.1f}%'
    ax.text(0.98, 0.97, textstr, transform=ax.transAxes, fontsize=10,
            verticalalignment='top', horizontalalignment='right',
            bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.8))

    plt.tight_layout()
    output_file = output_dir / 'population_deviation_hist.png'
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"[OK] Created: {output_file}")
    plt.close()

    # 2. Box plot of deviations by state (top 20 states by number of districts)
    fig, ax = plt.subplots(figsize=(14, 6))

    # Get top 20 states by district count
    top_states = df.groupby('state')['population'].count().sort_values(ascending=False).head(20).index
    df_top = df[df['state'].isin(top_states)]

    # Create box plot
    df_top.boxplot(column='deviation_pct', by='state', ax=ax, grid=False)
    ax.axhline(0, color='red', linestyle='--', linewidth=1.5, alpha=0.7)
    ax.axhline(-1, color='orange', linestyle=':', linewidth=1, alpha=0.7)
    ax.axhline(1, color='orange', linestyle=':', linewidth=1, alpha=0.7)

    ax.set_xlabel('State', fontsize=12)
    ax.set_ylabel('Population Deviation (%)', fontsize=12)
    ax.set_title('Population Deviation by State (Top 20 by District Count)', fontsize=14, fontweight='bold')
    plt.suptitle('')  # Remove default title
    ax.grid(True, alpha=0.3, axis='y')
    plt.xticks(rotation=45, ha='right')

    plt.tight_layout()
    output_file = output_dir / 'population_deviation_by_state.png'
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"[OK] Created: {output_file}")
    plt.close()

    # 3. Scatter plot: District population vs district number
    fig, ax = plt.subplots(figsize=(12, 6))

    df_sorted = df.sort_values('population')
    df_sorted['district_rank'] = range(1, len(df_sorted) + 1)

    ax.scatter(df_sorted['district_rank'], df_sorted['population'],
              alpha=0.6, s=30, color='steelblue')
    ax.axhline(stats['ideal_population'], color='red', linestyle='--',
              linewidth=2, label=f'Ideal: {stats["ideal_population"]:,.0f}')

    # Add ±1% lines
    ax.axhline(stats['ideal_population'] * 1.01, color='orange', linestyle=':',
              linewidth=1.5, label='±1% Limit', alpha=0.7)
    ax.axhline(stats['ideal_population'] * 0.99, color='orange', linestyle=':',
              linewidth=1.5, alpha=0.7)

    ax.set_xlabel('District Rank (by Population)', fontsize=12)
    ax.set_ylabel('District Population', fontsize=12)
    ax.set_title('Congressional District Populations (2020 Census)\n435 Districts Sorted by Population',
                fontsize=14, fontweight='bold')
    ax.legend(fontsize=10)
    ax.grid(True, alpha=0.3)

    # Format y-axis as thousands
    ax.yaxis.set_major_formatter(plt.FuncFormatter(lambda x, p: f'{x/1000:.0f}K'))

    plt.tight_layout()
    output_file = output_dir / 'population_scatter.png'
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"[OK] Created: {output_file}")
    plt.close()

def save_statistics(stats, df, output_dir='paper/data'):
    """Save statistics to CSV files."""

    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    print(f"\n" + "="*70)
    print("SAVING STATISTICS")
    print("="*70)

    # Save summary statistics
    stats_df = pd.DataFrame([stats])
    output_file = output_dir / 'population_stats.csv'
    stats_df.to_csv(output_file, index=False)
    print(f"[OK] Saved summary: {output_file}")

    # Save per-district data
    district_cols = ['state', 'district', 'population', 'deviation',
                    'deviation_pct', 'abs_deviation', 'abs_deviation_pct']
    output_file = output_dir / 'population_per_district.csv'
    df[district_cols].to_csv(output_file, index=False)
    print(f"[OK] Saved per-district: {output_file}")

    # Save state-level summary
    state_summary = df.groupby('state').agg({
        'population': ['count', 'sum', 'mean', 'std', 'min', 'max'],
        'abs_deviation_pct': ['mean', 'median', 'max']
    }).round(3)
    state_summary.columns = ['_'.join(col).strip() for col in state_summary.columns.values]
    output_file = output_dir / 'population_by_state.csv'
    state_summary.to_csv(output_file)
    print(f"[OK] Saved state summary: {output_file}")

def main():
    """Main function."""
    import argparse

    parser = argparse.ArgumentParser(description='Compute population statistics')
    parser.add_argument('--output-dir', type=str, default='outputs/us_2020_v1',
                       help='Output directory with redistricting results')
    args = parser.parse_args()

    print("="*70)
    print("POPULATION DEVIATION ANALYSIS")
    print("="*70)

    # Compute statistics
    result = compute_population_statistics(args.output_dir)
    if result is None:
        return 1

    df, stats = result

    # Create visualizations
    create_visualizations(df, stats)

    # Save statistics
    save_statistics(stats, df)

    print("\n" + "="*70)
    print("ANALYSIS COMPLETE")
    print("="*70)
    print("\nGenerated files:")
    print("  paper/data/population_stats.csv")
    print("  paper/data/population_per_district.csv")
    print("  paper/data/population_by_state.csv")
    print("  paper/figures/population_deviation_hist.png")
    print("  paper/figures/population_deviation_by_state.png")
    print("  paper/figures/population_scatter.png")

    return 0

if __name__ == '__main__':
    sys.exit(main())
