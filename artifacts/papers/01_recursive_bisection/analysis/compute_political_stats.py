#!/usr/bin/env python3
"""
Compute political statistics across congressional districts.

Analyzes partisan lean distribution and competitive vs safe districts.
"""

import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
from pathlib import Path
import sys

def load_political_data(output_dir='outputs/us_2020_v1'):
    """Load political analysis data from all states."""

    output_dir = Path(output_dir)
    states_dir = output_dir / 'states'

    if not states_dir.exists():
        print(f"ERROR: {states_dir} not found")
        return None

    all_data = []

    for state_dir in sorted(states_dir.iterdir()):
        if not state_dir.is_dir():
            continue

        political_file = state_dir / 'political_analysis' / 'district_political_2020.csv'

        if not political_file.exists():
            continue

        # Check file size (skip empty files)
        if political_file.stat().st_size == 0:
            continue

        state_name = state_dir.name.replace('_', ' ').title()

        try:
            df = pd.read_csv(political_file)
            if len(df) == 0:  # Skip empty dataframes
                continue
            df['state'] = state_name
            all_data.append(df)
        except (pd.errors.EmptyDataError, pd.errors.ParserError):
            # Skip files that can't be parsed
            continue

    if not all_data:
        print("ERROR: No political analysis data found")
        print("Run political analysis first:")
        print("  python scripts/political/run_political_analysis.py --census-year 2020 --version v1")
        return None

    combined = pd.concat(all_data, ignore_index=True)
    print(f"\nLoaded political data for {len(all_data)} states")
    print(f"Total districts with political data: {len(combined)}")

    return combined

def compute_political_statistics(df):
    """Compute political statistics."""

    # Count districts by lean
    lean_counts = df['lean'].value_counts()

    stats = {
        'total_districts': len(df),
        'strong_d': lean_counts.get('Strong D', 0),
        'lean_d': lean_counts.get('Lean D', 0),
        'toss_up': lean_counts.get('Toss-up', 0),
        'lean_r': lean_counts.get('Lean R', 0),
        'strong_r': lean_counts.get('Strong R', 0),
        'mean_dem_margin': df['dem_margin'].mean(),
        'median_dem_margin': df['dem_margin'].median(),
        'std_dem_margin': df['dem_margin'].std(),
    }

    # Competitive vs safe
    stats['competitive'] = stats['toss_up']
    stats['safe_d'] = stats['strong_d'] + stats['lean_d']
    stats['safe_r'] = stats['strong_r'] + stats['lean_r']
    stats['competitive_pct'] = stats['competitive'] / stats['total_districts'] * 100
    stats['safe_d_pct'] = stats['safe_d'] / stats['total_districts'] * 100
    stats['safe_r_pct'] = stats['safe_r'] / stats['total_districts'] * 100

    # Biden vs Trump percentages
    stats['mean_biden_pct'] = df['biden_two_party_pct'].mean()
    stats['mean_trump_pct'] = df['trump_two_party_pct'].mean()

    # Print summary
    print("\n" + "="*70)
    print("POLITICAL STATISTICS")
    print("="*70)
    print(f"\nTotal Districts Analyzed: {stats['total_districts']}")
    print(f"\nPartisan Lean Distribution:")
    print(f"  Strong D (margin > 15%):     {stats['strong_d']:>3} ({stats['strong_d']/stats['total_districts']*100:>5.1f}%)")
    print(f"  Lean D (margin 5-15%):       {stats['lean_d']:>3} ({stats['lean_d']/stats['total_districts']*100:>5.1f}%)")
    print(f"  Toss-up (margin -5% to 5%):  {stats['toss_up']:>3} ({stats['competitive_pct']:>5.1f}%)")
    print(f"  Lean R (margin -15% to -5%): {stats['lean_r']:>3} ({stats['lean_r']/stats['total_districts']*100:>5.1f}%)")
    print(f"  Strong R (margin < -15%):    {stats['strong_r']:>3} ({stats['strong_r']/stats['total_districts']*100:>5.1f}%)")

    print(f"\nCompetitive vs Safe:")
    print(f"  Competitive (toss-up):  {stats['competitive']:>3} ({stats['competitive_pct']:>5.1f}%)")
    print(f"  Safe Democrat:          {stats['safe_d']:>3} ({stats['safe_d_pct']:>5.1f}%)")
    print(f"  Safe Republican:        {stats['safe_r']:>3} ({stats['safe_r_pct']:>5.1f}%)")

    print(f"\nMean Democratic Margin: {stats['mean_dem_margin']:.2f}%")
    print(f"Median Democratic Margin: {stats['median_dem_margin']:.2f}%")
    print(f"Std Dev: {stats['std_dem_margin']:.2f}%")

    return stats

def create_visualizations(df, stats, output_dir='paper/figures'):
    """Create political analysis visualizations."""

    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    print(f"\n" + "="*70)
    print("GENERATING VISUALIZATIONS")
    print("="*70)

    # 1. Pie chart: Partisan lean distribution
    fig, ax = plt.subplots(figsize=(10, 8))

    lean_order = ['Strong D', 'Lean D', 'Toss-up', 'Lean R', 'Strong R']
    lean_counts = [stats[k] for k in ['strong_d', 'lean_d', 'toss_up', 'lean_r', 'strong_r']]
    colors = ['#1a4da6', '#6495ed', '#cccccc', '#f08080', '#c41e3a']

    wedges, texts, autotexts = ax.pie(lean_counts, labels=lean_order, colors=colors,
                                       autopct='%1.1f%%', startangle=90,
                                       textprops={'fontsize': 11, 'weight': 'bold'})

    # Make percentage text white for visibility
    for autotext in autotexts:
        autotext.set_color('white')
        autotext.set_fontsize(12)

    ax.set_title('Partisan Lean Distribution\n2020 Presidential Election Results',
                fontsize=14, fontweight='bold', pad=20)

    # Add legend with counts
    legend_labels = [f'{label}: {count} districts'
                    for label, count in zip(lean_order, lean_counts)]
    ax.legend(legend_labels, loc='center left', bbox_to_anchor=(1, 0, 0.5, 1), fontsize=10)

    plt.tight_layout()
    output_file = output_dir / 'political_lean_pie.png'
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"[OK] Created: {output_file}")
    plt.close()

    # 2. Histogram: Democratic margin distribution
    fig, ax = plt.subplots(figsize=(12, 6))

    ax.hist(df['dem_margin'], bins=60, edgecolor='black', alpha=0.7, color='steelblue')
    ax.axvline(0, color='purple', linestyle='--', linewidth=2, label='Even Split')
    ax.axvline(-5, color='red', linestyle=':', linewidth=1.5, alpha=0.7, label='Toss-up Range')
    ax.axvline(5, color='red', linestyle=':', linewidth=1.5, alpha=0.7)
    ax.axvline(-15, color='orange', linestyle=':', linewidth=1.5, alpha=0.7, label='Strong/Lean Boundary')
    ax.axvline(15, color='orange', linestyle=':', linewidth=1.5, alpha=0.7)

    ax.set_xlabel('Democratic Margin (%)', fontsize=12)
    ax.set_ylabel('Number of Districts', fontsize=12)
    ax.set_title('Distribution of Partisan Lean\n(Biden - Trump Two-Party Vote %)',
                fontsize=14, fontweight='bold')
    ax.legend(fontsize=10)
    ax.grid(True, alpha=0.3, axis='y')

    # Add statistics box
    textstr = f'Mean: {stats["mean_dem_margin"]:.2f}%\n'
    textstr += f'Median: {stats["median_dem_margin"]:.2f}%\n'
    textstr += f'Std Dev: {stats["std_dem_margin"]:.2f}%'
    ax.text(0.02, 0.97, textstr, transform=ax.transAxes, fontsize=10,
            verticalalignment='top', bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.8))

    plt.tight_layout()
    output_file = output_dir / 'political_margin_hist.png'
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"[OK] Created: {output_file}")
    plt.close()

    # 3. Bar chart: Competitive vs Safe
    fig, ax = plt.subplots(figsize=(10, 6))

    categories = ['Safe\nDemocrat', 'Competitive\n(Toss-up)', 'Safe\nRepublican']
    counts = [stats['safe_d'], stats['competitive'], stats['safe_r']]
    colors_bar = ['#4169e1', '#999999', '#dc143c']

    bars = ax.bar(categories, counts, color=colors_bar, edgecolor='black', linewidth=1.5)

    ax.set_ylabel('Number of Districts', fontsize=12)
    ax.set_title('Competitive vs Safe Districts\nAlgorithmic Redistricting (2020 Census)',
                fontsize=14, fontweight='bold')
    ax.grid(True, alpha=0.3, axis='y')

    # Add value labels on bars
    for bar, count in zip(bars, counts):
        height = bar.get_height()
        ax.text(bar.get_x() + bar.get_width()/2., height,
               f'{int(count)}\n({count/stats["total_districts"]*100:.1f}%)',
               ha='center', va='bottom', fontsize=11, fontweight='bold')

    plt.tight_layout()
    output_file = output_dir / 'political_competitive_bars.png'
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
    output_file = output_dir / 'political_stats.csv'
    stats_df.to_csv(output_file, index=False)
    print(f"[OK] Saved summary: {output_file}")

    # Save per-district data
    output_file = output_dir / 'political_per_district.csv'
    df.to_csv(output_file, index=False)
    print(f"[OK] Saved per-district: {output_file}")

    # Save state-level summary
    state_summary = df.groupby('state').agg({
        'district': 'count',
        'dem_margin': ['mean', 'median', 'std'],
        'biden_two_party_pct': 'mean',
        'trump_two_party_pct': 'mean'
    }).round(2)
    state_summary.columns = ['_'.join(col).strip() for col in state_summary.columns.values]
    output_file = output_dir / 'political_by_state.csv'
    state_summary.to_csv(output_file)
    print(f"[OK] Saved state summary: {output_file}")

def main():
    """Main function."""
    import argparse

    parser = argparse.ArgumentParser(description='Compute political statistics')
    parser.add_argument('--output-dir', type=str, default='outputs/us_2020_v1',
                       help='Output directory with redistricting results')
    args = parser.parse_args()

    print("="*70)
    print("POLITICAL ANALYSIS")
    print("="*70)

    # Load data
    df = load_political_data(args.output_dir)
    if df is None:
        return 1

    # Compute statistics
    stats = compute_political_statistics(df)

    # Create visualizations
    create_visualizations(df, stats)

    # Save statistics
    save_statistics(stats, df)

    print("\n" + "="*70)
    print("ANALYSIS COMPLETE")
    print("="*70)
    print("\nGenerated files:")
    print("  paper/data/political_stats.csv")
    print("  paper/data/political_per_district.csv")
    print("  paper/data/political_by_state.csv")
    print("  paper/figures/political_lean_pie.png")
    print("  paper/figures/political_margin_hist.png")
    print("  paper/figures/political_competitive_bars.png")

    return 0

if __name__ == '__main__':
    sys.exit(main())
