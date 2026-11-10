#!/usr/bin/env python3
"""
Compute compactness statistics across all 435 congressional districts.

Analyzes Polsby-Popper and Reock compactness scores.
"""

import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
from pathlib import Path
import sys

def compute_compactness_statistics(output_dir='outputs/us_2020_v1'):
    """Compute compactness statistics."""

    output_dir = Path(output_dir)
    states_dir = output_dir / 'states'

    if not states_dir.exists():
        print(f"ERROR: {states_dir} not found")
        return None

    # Aggregate compactness data from all state directories
    print(f"Aggregating compactness data from state directories...")
    all_data = []

    for state_dir in sorted(states_dir.iterdir()):
        if not state_dir.is_dir():
            continue

        state_name = state_dir.name.replace('_', ' ').title()
        district_summary = state_dir / 'district_summary.csv'

        if not district_summary.exists():
            print(f"  [X] {state_name}: district_summary.csv not found")
            continue

        try:
            df_state = pd.read_csv(district_summary)

            # Check if compactness columns exist (handle _x, _y suffixes from duplicate merges)
            pp_col = None
            reock_col = None

            for col in ['polsby_popper', 'polsby_popper_x', 'polsby_popper_y']:
                if col in df_state.columns:
                    pp_col = col
                    break

            for col in ['reock', 'reock_x', 'reock_y']:
                if col in df_state.columns:
                    reock_col = col
                    break

            if pp_col is None or reock_col is None:
                print(f"  [X] {state_name}: no compactness data")
                continue

            # Rename columns to standard names
            if pp_col != 'polsby_popper':
                df_state['polsby_popper'] = df_state[pp_col]
            if reock_col != 'reock':
                df_state['reock'] = df_state[reock_col]

            df_state['state'] = state_name
            all_data.append(df_state)
            print(f"  [OK] {state_name}: {len(df_state)} districts")

        except Exception as e:
            print(f"  [X] {state_name}: {e}")
            continue

    if not all_data:
        print("\nERROR: No compactness data found in any state")
        return None

    # Combine all state data
    df = pd.concat(all_data, ignore_index=True)
    print(f"\nTotal districts loaded: {len(df)}")

    # Remove invalid scores (NaN or extreme values)
    df_clean = df[(df['polsby_popper'] > 0) & (df['polsby_popper'] <= 1.0) &
                  (df['reock'] > 0) & (df['reock'] <= 1.0)].copy()

    print(f"Districts with valid compactness scores: {len(df_clean)}")

    # Summary statistics for Polsby-Popper
    pp_stats = {
        'pp_mean': df_clean['polsby_popper'].mean(),
        'pp_median': df_clean['polsby_popper'].median(),
        'pp_std': df_clean['polsby_popper'].std(),
        'pp_min': df_clean['polsby_popper'].min(),
        'pp_max': df_clean['polsby_popper'].max(),
        'pp_q25': df_clean['polsby_popper'].quantile(0.25),
        'pp_q75': df_clean['polsby_popper'].quantile(0.75),
    }

    # Summary statistics for Reock
    reock_stats = {
        'reock_mean': df_clean['reock'].mean(),
        'reock_median': df_clean['reock'].median(),
        'reock_std': df_clean['reock'].std(),
        'reock_min': df_clean['reock'].min(),
        'reock_max': df_clean['reock'].max(),
        'reock_q25': df_clean['reock'].quantile(0.25),
        'reock_q75': df_clean['reock'].quantile(0.75),
    }

    stats = {**pp_stats, **reock_stats, 'num_districts': len(df_clean)}

    # Print summary
    print("\n" + "="*70)
    print("COMPACTNESS STATISTICS")
    print("="*70)
    print("\nPolsby-Popper Score (4*pi * Area / Perimeter^2):")
    print(f"  Mean: {pp_stats['pp_mean']:.4f}")
    print(f"  Median: {pp_stats['pp_median']:.4f}")
    print(f"  Std Dev: {pp_stats['pp_std']:.4f}")
    print(f"  Range: [{pp_stats['pp_min']:.4f}, {pp_stats['pp_max']:.4f}]")
    print(f"  25th-75th percentile: [{pp_stats['pp_q25']:.4f}, {pp_stats['pp_q75']:.4f}]")

    print("\nReock Score (Area / Area of Minimum Bounding Circle):")
    print(f"  Mean: {reock_stats['reock_mean']:.4f}")
    print(f"  Median: {reock_stats['reock_median']:.4f}")
    print(f"  Std Dev: {reock_stats['reock_std']:.4f}")
    print(f"  Range: [{reock_stats['reock_min']:.4f}, {reock_stats['reock_max']:.4f}]")
    print(f"  25th-75th percentile: [{reock_stats['reock_q25']:.4f}, {reock_stats['reock_q75']:.4f}]")

    # Categorize compactness
    df_clean['pp_category'] = pd.cut(df_clean['polsby_popper'],
                                      bins=[0, 0.1, 0.2, 0.3, 0.4, 1.0],
                                      labels=['Very Low', 'Low', 'Medium', 'High', 'Very High'])

    print("\nPolsby-Popper Distribution:")
    print(df_clean['pp_category'].value_counts().sort_index())

    # State-by-state summary
    print("\n" + "="*70)
    print("STATE-BY-STATE SUMMARY (Top 10 by mean Polsby-Popper)")
    print("="*70)

    state_stats = df_clean.groupby('state').agg({
        'polsby_popper': ['count', 'mean', 'std'],
        'reock': ['mean', 'std']
    }).round(4)
    state_stats.columns = ['num_districts', 'pp_mean', 'pp_std', 'reock_mean', 'reock_std']
    state_stats = state_stats.sort_values('pp_mean', ascending=False)
    print(state_stats.head(10))

    return df_clean, stats

def create_visualizations(df, stats, output_dir='paper/figures'):
    """Create visualizations of compactness statistics."""

    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    print(f"\n" + "="*70)
    print("GENERATING VISUALIZATIONS")
    print("="*70)

    # 1. Dual histogram: Polsby-Popper and Reock
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 5))

    # Polsby-Popper histogram
    ax1.hist(df['polsby_popper'], bins=40, edgecolor='black', alpha=0.7, color='steelblue')
    ax1.axvline(stats['pp_mean'], color='red', linestyle='--', linewidth=2, label=f'Mean: {stats["pp_mean"]:.3f}')
    ax1.axvline(stats['pp_median'], color='orange', linestyle=':', linewidth=2, label=f'Median: {stats["pp_median"]:.3f}')

    ax1.set_xlabel('Polsby-Popper Score', fontsize=12)
    ax1.set_ylabel('Number of Districts', fontsize=12)
    ax1.set_title('Polsby-Popper Compactness\n(4π × Area / Perimeter²)', fontsize=13, fontweight='bold')
    ax1.legend(fontsize=10)
    ax1.grid(True, alpha=0.3)

    # Reock histogram
    ax2.hist(df['reock'], bins=40, edgecolor='black', alpha=0.7, color='forestgreen')
    ax2.axvline(stats['reock_mean'], color='red', linestyle='--', linewidth=2, label=f'Mean: {stats["reock_mean"]:.3f}')
    ax2.axvline(stats['reock_median'], color='orange', linestyle=':', linewidth=2, label=f'Median: {stats["reock_median"]:.3f}')

    ax2.set_xlabel('Reock Score', fontsize=12)
    ax2.set_ylabel('Number of Districts', fontsize=12)
    ax2.set_title('Reock Compactness\n(Area / Min Bounding Circle)', fontsize=13, fontweight='bold')
    ax2.legend(fontsize=10)
    ax2.grid(True, alpha=0.3)

    plt.tight_layout()
    output_file = output_dir / 'compactness_distribution.png'
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"[OK] Created: {output_file}")
    plt.close()

    # 2. Scatter plot: Polsby-Popper vs Reock
    fig, ax = plt.subplots(figsize=(10, 8))

    scatter = ax.scatter(df['polsby_popper'], df['reock'],
                        alpha=0.5, s=40, c=df['population'], cmap='viridis')

    ax.set_xlabel('Polsby-Popper Score', fontsize=12)
    ax.set_ylabel('Reock Score', fontsize=12)
    ax.set_title('Compactness Measures Comparison\n435 Congressional Districts',
                fontsize=14, fontweight='bold')
    ax.grid(True, alpha=0.3)

    # Add colorbar
    cbar = plt.colorbar(scatter, ax=ax)
    cbar.set_label('District Population', fontsize=11)

    # Add reference lines
    ax.axhline(stats['reock_mean'], color='red', linestyle='--', alpha=0.5, linewidth=1)
    ax.axvline(stats['pp_mean'], color='red', linestyle='--', alpha=0.5, linewidth=1)

    plt.tight_layout()
    output_file = output_dir / 'compactness_scatter.png'
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"[OK] Created: {output_file}")
    plt.close()

    # 3. Box plot by state (top 15 states)
    fig, ax = plt.subplots(figsize=(14, 6))

    top_states = df.groupby('state')['polsby_popper'].count().sort_values(ascending=False).head(15).index
    df_top = df[df['state'].isin(top_states)]

    df_top.boxplot(column='polsby_popper', by='state', ax=ax, grid=False)
    ax.axhline(stats['pp_mean'], color='red', linestyle='--', linewidth=1.5, alpha=0.7, label='National Mean')

    ax.set_xlabel('State', fontsize=12)
    ax.set_ylabel('Polsby-Popper Score', fontsize=12)
    ax.set_title('Polsby-Popper Compactness by State (Top 15 by District Count)', fontsize=14, fontweight='bold')
    plt.suptitle('')  # Remove default title
    ax.legend(fontsize=10)
    ax.grid(True, alpha=0.3, axis='y')
    plt.xticks(rotation=45, ha='right')

    plt.tight_layout()
    output_file = output_dir / 'compactness_by_state.png'
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
    output_file = output_dir / 'compactness_stats.csv'
    stats_df.to_csv(output_file, index=False)
    print(f"[OK] Saved summary: {output_file}")

    # Save per-district data
    district_cols = ['state', 'district', 'population', 'polsby_popper', 'reock']
    output_file = output_dir / 'compactness_per_district.csv'
    df[district_cols].to_csv(output_file, index=False)
    print(f"[OK] Saved per-district: {output_file}")

    # Save state-level summary
    state_summary = df.groupby('state').agg({
        'polsby_popper': ['count', 'mean', 'median', 'std', 'min', 'max'],
        'reock': ['mean', 'median', 'std', 'min', 'max']
    }).round(4)
    state_summary.columns = ['_'.join(col).strip() for col in state_summary.columns.values]
    output_file = output_dir / 'compactness_by_state.csv'
    state_summary.to_csv(output_file)
    print(f"[OK] Saved state summary: {output_file}")

def main():
    """Main function."""
    import argparse

    parser = argparse.ArgumentParser(description='Compute compactness statistics')
    parser.add_argument('--output-dir', type=str, default='outputs/us_2020_v1',
                       help='Output directory with redistricting results')
    args = parser.parse_args()

    print("="*70)
    print("COMPACTNESS ANALYSIS")
    print("="*70)

    # Compute statistics
    result = compute_compactness_statistics(args.output_dir)
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
    print("  paper/data/compactness_stats.csv")
    print("  paper/data/compactness_per_district.csv")
    print("  paper/data/compactness_by_state.csv")
    print("  paper/figures/compactness_distribution.png")
    print("  paper/figures/compactness_scatter.png")
    print("  paper/figures/compactness_by_state.png")

    return 0

if __name__ == '__main__':
    sys.exit(main())
