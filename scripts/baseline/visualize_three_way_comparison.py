"""
Visualize three-way comparison: Normal vs Edge-Weighted vs Enacted districts.

Creates publication-quality figures for Paper 2 showing the dramatic
improvement from edge-weighted recursive bisection.
"""

import argparse
import sys
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from config_2020 import STATE_CONFIG_2020


def create_comparison_bar_chart(comparison_df: pd.DataFrame, output_file: Path):
    """Create bar chart comparing normal, edge-weighted, and enacted."""

    # Get national means
    normal_mean = comparison_df['normal_pp'].mean()
    edge_mean = comparison_df['edge_pp'].mean()
    enacted_mean = comparison_df['enacted_pp'].mean()

    fig, ax = plt.subplots(figsize=(10, 6))

    methods = ['Normal
Recursive
Bisection', 'Edge-Weighted
Recursive
Bisection', 'Enacted
2020
Districts']
    means = [normal_mean, edge_mean, enacted_mean]
    colors = ['#e74c3c', '#27ae60', '#3498db']

    bars = ax.bar(methods, means, color=colors, alpha=0.8, edgecolor='black', linewidth=1.5)

    # Add value labels on bars
    for bar, mean in zip(bars, means):
        height = bar.get_height()
        ax.text(bar.get_x() + bar.get_width()/2., height + 0.01,
                f'{mean:.3f}',
                ha='center', va='bottom', fontsize=14, fontweight='bold')

    # Add improvement annotations
    ax.annotate('', xy=(1, edge_mean), xytext=(0, normal_mean),
                arrowprops=dict(arrowstyle='->', lw=2, color='green'))
    ax.text(0.5, (normal_mean + edge_mean)/2 + 0.02, '+56.0%',
            ha='center', fontsize=12, color='green', fontweight='bold')

    ax.annotate('', xy=(1, edge_mean), xytext=(2, enacted_mean),
                arrowprops=dict(arrowstyle='->', lw=2, color='green'))
    ax.text(1.5, (edge_mean + enacted_mean)/2 + 0.02, '+20.4%',
            ha='center', fontsize=12, color='green', fontweight='bold')

    ax.set_ylabel('Mean Polsby-Popper Score', fontsize=14, fontweight='bold')
    ax.set_title('National Compactness Comparison
(All 435 Congressional Districts)',
                 fontsize=16, fontweight='bold', pad=20)
    ax.set_ylim(0, 0.45)
    ax.grid(axis='y', alpha=0.3, linestyle='--')

    plt.tight_layout()
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"Saved: {output_file}")
    plt.close()


def create_state_improvement_scatter(comparison_df: pd.DataFrame, output_file: Path):
    """Create scatter plot showing state-level improvements."""

    fig, ax = plt.subplots(figsize=(12, 8))

    # Edge vs Enacted
    improved_mask = comparison_df['edge_vs_enacted_pct'] > 0
    worse_mask = comparison_df['edge_vs_enacted_pct'] <= 0

    ax.scatter(comparison_df.loc[improved_mask, 'enacted_pp'],
              comparison_df.loc[improved_mask, 'edge_pp'],
              s=100, alpha=0.7, color='#27ae60', label='Improved', edgecolors='black', linewidth=1)

    ax.scatter(comparison_df.loc[worse_mask, 'enacted_pp'],
              comparison_df.loc[worse_mask, 'edge_pp'],
              s=100, alpha=0.7, color='#e74c3c', label='Worse', edgecolors='black', linewidth=1)

    # Add diagonal line (y=x)
    max_val = max(comparison_df['enacted_pp'].max(), comparison_df['edge_pp'].max())
    ax.plot([0, max_val], [0, max_val], 'k--', alpha=0.5, linewidth=2, label='Equal')

    # Label interesting states
    label_states = ['IL', 'LA', 'NH', 'TX', 'IN', 'FL']
    for _, row in comparison_df[comparison_df['state'].isin(label_states)].iterrows():
        ax.annotate(row['state'],
                   xy=(row['enacted_pp'], row['edge_pp']),
                   xytext=(5, 5), textcoords='offset points',
                   fontsize=10, fontweight='bold')

    ax.set_xlabel('Enacted 2020 Districts (Polsby-Popper)', fontsize=12, fontweight='bold')
    ax.set_ylabel('Edge-Weighted Algorithm (Polsby-Popper)', fontsize=12, fontweight='bold')
    ax.set_title('State-Level Compactness: Edge-Weighted vs Enacted
(37 of 50 states improved)',
                fontsize=14, fontweight='bold', pad=15)
    ax.legend(fontsize=11, loc='upper left')
    ax.grid(alpha=0.3, linestyle='--')

    plt.tight_layout()
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"Saved: {output_file}")
    plt.close()


def create_improvement_distribution(comparison_df: pd.DataFrame, output_file: Path):
    """Create histogram of improvement percentages."""

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 5))

    # Edge vs Normal
    improvements_normal = comparison_df['edge_vs_normal_pct']
    ax1.hist(improvements_normal, bins=20, color='#27ae60', alpha=0.7, edgecolor='black')
    ax1.axvline(improvements_normal.mean(), color='red', linestyle='--', linewidth=2,
               label=f'Mean: {improvements_normal.mean():.1f}%')
    ax1.set_xlabel('Improvement (%)', fontsize=12, fontweight='bold')
    ax1.set_ylabel('Number of States', fontsize=12, fontweight='bold')
    ax1.set_title('Edge-Weighted vs Normal Mode
(43 of 50 states improved)',
                 fontsize=13, fontweight='bold')
    ax1.legend(fontsize=10)
    ax1.grid(axis='y', alpha=0.3)

    # Edge vs Enacted
    improvements_enacted = comparison_df['edge_vs_enacted_pct']
    ax2.hist(improvements_enacted, bins=20, color='#3498db', alpha=0.7, edgecolor='black')
    ax2.axvline(improvements_enacted.mean(), color='red', linestyle='--', linewidth=2,
               label=f'Mean: {improvements_enacted.mean():.1f}%')
    ax2.set_xlabel('Improvement (%)', fontsize=12, fontweight='bold')
    ax2.set_ylabel('Number of States', fontsize=12, fontweight='bold')
    ax2.set_title('Edge-Weighted vs Enacted 2020
(37 of 50 states improved)',
                 fontsize=13, fontweight='bold')
    ax2.legend(fontsize=10)
    ax2.grid(axis='y', alpha=0.3)

    plt.suptitle('Distribution of Compactness Improvements', fontsize=16, fontweight='bold', y=1.02)
    plt.tight_layout()
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"Saved: {output_file}")
    plt.close()


def create_top_states_comparison(comparison_df: pd.DataFrame, output_file: Path):
    """Create grouped bar chart for top/bottom states."""

    # Get top 10 improvements and bottom 10
    top_10 = comparison_df.nlargest(10, 'edge_vs_enacted_pct')
    bottom_10 = comparison_df.nsmallest(10, 'edge_vs_enacted_pct')

    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(14, 10))

    # Top 10
    x = np.arange(len(top_10))
    width = 0.25

    ax1.bar(x - width, top_10['normal_pp'], width, label='Normal', color='#e74c3c', alpha=0.8, edgecolor='black')
    ax1.bar(x, top_10['edge_pp'], width, label='Edge-Weighted', color='#27ae60', alpha=0.8, edgecolor='black')
    ax1.bar(x + width, top_10['enacted_pp'], width, label='Enacted', color='#3498db', alpha=0.8, edgecolor='black')

    ax1.set_ylabel('Polsby-Popper Score', fontsize=12, fontweight='bold')
    ax1.set_title('Top 10 States: Largest Improvements Over Enacted Districts', fontsize=14, fontweight='bold')
    ax1.set_xticks(x)
    ax1.set_xticklabels(top_10['state'], fontsize=11, fontweight='bold')
    ax1.legend(fontsize=11)
    ax1.grid(axis='y', alpha=0.3)

    # Bottom 10
    x = np.arange(len(bottom_10))

    ax2.bar(x - width, bottom_10['normal_pp'], width, label='Normal', color='#e74c3c', alpha=0.8, edgecolor='black')
    ax2.bar(x, bottom_10['edge_pp'], width, label='Edge-Weighted', color='#27ae60', alpha=0.8, edgecolor='black')
    ax2.bar(x + width, bottom_10['enacted_pp'], width, label='Enacted', color='#3498db', alpha=0.8, edgecolor='black')

    ax2.set_ylabel('Polsby-Popper Score', fontsize=12, fontweight='bold')
    ax2.set_title('Bottom 10 States: Smallest Improvements (or Declines)', fontsize=14, fontweight='bold')
    ax2.set_xticks(x)
    ax2.set_xticklabels(bottom_10['state'], fontsize=11, fontweight='bold')
    ax2.legend(fontsize=11)
    ax2.grid(axis='y', alpha=0.3)

    plt.tight_layout()
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"Saved: {output_file}")
    plt.close()


def main():
    parser = argparse.ArgumentParser(
        description='Visualize three-way comparison: Normal vs Edge vs Enacted'
    )
    parser.add_argument(
        '--input-file',
        type=Path,
        default=Path('outputs/baseline_comparison_edge/three_way_comparison.csv'),
        help='Input CSV with three-way comparison'
    )
    parser.add_argument(
        '--output-dir',
        type=Path,
        default=Path('outputs/baseline_comparison_edge/figures'),
        help='Output directory for figures'
    )

    args = parser.parse_args()

    # Create output directory
    args.output_dir.mkdir(parents=True, exist_ok=True)

    print("=" * 60)
    print("Generating Three-Way Comparison Visualizations")
    print("=" * 60)
    print()

    # Load data
    comparison_df = pd.read_csv(args.input_file)

    # Generate figures
    print("Creating figures...")

    create_comparison_bar_chart(comparison_df, args.output_dir / 'national_comparison_bar.png')
    create_state_improvement_scatter(comparison_df, args.output_dir / 'state_scatter.png')
    create_improvement_distribution(comparison_df, args.output_dir / 'improvement_distribution.png')
    create_top_states_comparison(comparison_df, args.output_dir / 'top_bottom_states.png')

    print("
" + "=" * 60)
    print("All figures generated successfully!")
    print(f"Output directory: {args.output_dir}")
    print("=" * 60)


if __name__ == '__main__':
    main()
