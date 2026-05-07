#!/usr/bin/env python3
"""
Generate visualizations for parameter sensitivity analysis.

Reads CSV results from parameter_sensitivity.py and creates publication-quality
figures for the paper.
"""

import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
import numpy as np
from pathlib import Path
from scipy import stats
import sys

# Add project root to path
project_root = Path(__file__).parents[2]
sys.path.insert(0, str(project_root))

# Output directory for figures
OUTPUT_DIR = project_root / "research" / "gerry-recursive-bisection" / "figures"
OUTPUT_DIR.mkdir(exist_ok=True, parents=True)

# Set publication-quality style
plt.style.use('seaborn-v0_8-paper')
plt.rcParams['figure.dpi'] = 300
plt.rcParams['savefig.dpi'] = 300
plt.rcParams['font.size'] = 10
plt.rcParams['axes.labelsize'] = 11
plt.rcParams['axes.titlesize'] = 12
plt.rcParams['xtick.labelsize'] = 9
plt.rcParams['ytick.labelsize'] = 9
plt.rcParams['legend.fontsize'] = 9


def load_sweep_data(sweep_type):
    """Load parameter sweep results."""
    data_file = Path(f'outputs/sensitivity/{sweep_type}_sweep.csv')
    if not data_file.exists():
        print(f"[WARNING] Data file not found: {data_file}")
        print(f"         Run parameter_sensitivity.py first")
        return None

    df = pd.read_csv(data_file)
    print(f"[OK] Loaded {len(df)} records from {sweep_type}_sweep.csv")
    return df


def plot_ufactor_sensitivity(df):
    """Plot compactness and population deviation vs. ufactor."""
    if df is None or len(df) == 0:
        print("[SKIP] No ufactor data to plot")
        return

    print("Generating ufactor sensitivity plot...")

    fig, axes = plt.subplots(2, 2, figsize=(12, 10))

    # Filter to multi-district states only
    df_multi = df[df['num_districts'] > 1].copy()

    if len(df_multi) == 0:
        print("[WARNING] No multi-district states in ufactor data")
        plt.close()
        return

    # Plot 1: Mean population deviation vs. ufactor
    ax = axes[0, 0]
    for state in df_multi['state'].unique():
        state_data = df_multi[df_multi['state'] == state]
        ax.plot(state_data['ufactor'], state_data['mean_pop_deviation'],
               marker='o', label=state, alpha=0.7, linewidth=2)

    ax.set_xlabel('ufactor (imbalance tolerance)', fontsize=11)
    ax.set_ylabel('Mean Population Deviation (%)', fontsize=11)
    ax.set_title('Population Balance vs. Imbalance Tolerance', fontsize=12, fontweight='bold')
    ax.legend(loc='best', fontsize=8, ncol=2)
    ax.grid(True, alpha=0.3)

    # Plot 2: Max population deviation vs. ufactor
    ax = axes[0, 1]
    for state in df_multi['state'].unique():
        state_data = df_multi[df_multi['state'] == state]
        ax.plot(state_data['ufactor'], state_data['max_pop_deviation'],
               marker='s', label=state, alpha=0.7, linewidth=2)

    ax.set_xlabel('ufactor (imbalance tolerance)', fontsize=11)
    ax.set_ylabel('Max Population Deviation (%)', fontsize=11)
    ax.set_title('Worst-Case Population Balance', fontsize=12, fontweight='bold')
    ax.legend(loc='best', fontsize=8, ncol=2)
    ax.grid(True, alpha=0.3)

    # Plot 3: Box plot of population deviations by ufactor
    ax = axes[1, 0]
    ufactor_values = sorted(df_multi['ufactor'].unique())
    data_by_ufactor = [df_multi[df_multi['ufactor'] == uf]['mean_pop_deviation'].values
                       for uf in ufactor_values]
    ax.boxplot(data_by_ufactor, labels=ufactor_values)
    ax.set_xlabel('ufactor (imbalance tolerance)', fontsize=11)
    ax.set_ylabel('Mean Population Deviation (%)', fontsize=11)
    ax.set_title('Distribution of Deviations by ufactor', fontsize=12, fontweight='bold')
    ax.grid(True, alpha=0.3, axis='y')

    # Plot 4: Summary table
    ax = axes[1, 1]
    ax.axis('off')

    # Calculate summary statistics
    summary_text = "Summary Statistics\n\n"
    for uf in ufactor_values:
        uf_data = df_multi[df_multi['ufactor'] == uf]['mean_pop_deviation']
        summary_text += f"ufactor = {uf}:\n"
        summary_text += f"  Mean: {uf_data.mean():.3f}%\n"
        summary_text += f"  Std:  {uf_data.std():.3f}%\n"
        summary_text += f"  Max:  {uf_data.max():.3f}%\n\n"

    ax.text(0.1, 0.9, summary_text, transform=ax.transAxes,
           fontsize=10, verticalalignment='top', fontfamily='monospace',
           bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.3))

    plt.suptitle('Parameter Sensitivity: ufactor (Imbalance Tolerance)',
                fontsize=14, fontweight='bold', y=0.995)
    plt.tight_layout()

    output_path = OUTPUT_DIR / 'figure_4x1_ufactor_sensitivity.pdf'
    plt.savefig(output_path, bbox_inches='tight')
    plt.savefig(OUTPUT_DIR / 'figure_4x1_ufactor_sensitivity.png',
                bbox_inches='tight', dpi=150)
    print(f"  Saved: {output_path}")
    plt.close()


def plot_niter_sensitivity(df):
    """Plot metrics vs. niter."""
    if df is None or len(df) == 0:
        print("[SKIP] No niter data to plot")
        return

    print("Generating niter sensitivity plot...")

    fig, axes = plt.subplots(2, 2, figsize=(12, 10))

    # Filter to multi-district states only
    df_multi = df[df['num_districts'] > 1].copy()

    if len(df_multi) == 0:
        print("[WARNING] No multi-district states in niter data")
        plt.close()
        return

    # Plot 1: Mean population deviation vs. niter
    ax = axes[0, 0]
    for state in df_multi['state'].unique():
        state_data = df_multi[df_multi['state'] == state]
        ax.plot(state_data['niter'], state_data['mean_pop_deviation'],
               marker='o', label=state, alpha=0.7, linewidth=2)

    ax.set_xlabel('niter (refinement iterations)', fontsize=11)
    ax.set_ylabel('Mean Population Deviation (%)', fontsize=11)
    ax.set_title('Population Balance vs. Refinement Iterations', fontsize=12, fontweight='bold')
    ax.legend(loc='best', fontsize=8, ncol=2)
    ax.grid(True, alpha=0.3)
    ax.set_xscale('log')

    # Plot 2: Variation coefficient vs. niter
    ax = axes[0, 1]
    niter_values = sorted(df_multi['niter'].unique())
    variation_by_niter = []
    for niter in niter_values:
        niter_data = df_multi[df_multi['niter'] == niter]['mean_pop_deviation']
        coef_var = (niter_data.std() / niter_data.mean()) * 100 if niter_data.mean() > 0 else 0
        variation_by_niter.append(coef_var)

    ax.plot(niter_values, variation_by_niter, marker='s', linewidth=2,
           markersize=8, color='darkblue')
    ax.set_xlabel('niter (refinement iterations)', fontsize=11)
    ax.set_ylabel('Coefficient of Variation (%)', fontsize=11)
    ax.set_title('Consistency Improves with More Iterations', fontsize=12, fontweight='bold')
    ax.grid(True, alpha=0.3)
    ax.set_xscale('log')

    # Plot 3: Box plot by niter
    ax = axes[1, 0]
    data_by_niter = [df_multi[df_multi['niter'] == ni]['mean_pop_deviation'].values
                     for ni in niter_values]
    ax.boxplot(data_by_niter, labels=niter_values)
    ax.set_xlabel('niter (refinement iterations)', fontsize=11)
    ax.set_ylabel('Mean Population Deviation (%)', fontsize=11)
    ax.set_title('Distribution of Deviations by niter', fontsize=12, fontweight='bold')
    ax.grid(True, alpha=0.3, axis='y')

    # Plot 4: Summary table
    ax = axes[1, 1]
    ax.axis('off')

    summary_text = "Summary Statistics\n\n"
    for ni in niter_values:
        ni_data = df_multi[df_multi['niter'] == ni]['mean_pop_deviation']
        summary_text += f"niter = {ni}:\n"
        summary_text += f"  Mean: {ni_data.mean():.3f}%\n"
        summary_text += f"  Std:  {ni_data.std():.3f}%\n"
        summary_text += f"  Max:  {ni_data.max():.3f}%\n\n"

    ax.text(0.1, 0.9, summary_text, transform=ax.transAxes,
           fontsize=10, verticalalignment='top', fontfamily='monospace',
           bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.3))

    plt.suptitle('Parameter Sensitivity: niter (Refinement Iterations)',
                fontsize=14, fontweight='bold', y=0.995)
    plt.tight_layout()

    output_path = OUTPUT_DIR / 'figure_4x2_niter_sensitivity.pdf'
    plt.savefig(output_path, bbox_inches='tight')
    plt.savefig(OUTPUT_DIR / 'figure_4x2_niter_sensitivity.png',
                bbox_inches='tight', dpi=150)
    print(f"  Saved: {output_path}")
    plt.close()


def plot_objtype_comparison(df):
    """Compare cut vs. vol objectives."""
    if df is None or len(df) == 0:
        print("[SKIP] No objtype data to plot")
        return

    print("Generating objtype comparison plot...")

    # Filter to multi-district states only
    df_multi = df[df['num_districts'] > 1].copy()

    if len(df_multi) == 0:
        print("[WARNING] No multi-district states in objtype data")
        return

    fig, axes = plt.subplots(1, 2, figsize=(12, 5))

    # Plot 1: Population deviation comparison
    ax = axes[0]
    states = sorted(df_multi['state'].unique())
    cut_devs = [df_multi[(df_multi['state'] == s) & (df_multi['objtype'] == 'cut')]['mean_pop_deviation'].values[0]
                if len(df_multi[(df_multi['state'] == s) & (df_multi['objtype'] == 'cut')]) > 0 else 0
                for s in states]
    vol_devs = [df_multi[(df_multi['state'] == s) & (df_multi['objtype'] == 'vol')]['mean_pop_deviation'].values[0]
                if len(df_multi[(df_multi['state'] == s) & (df_multi['objtype'] == 'vol')]) > 0 else 0
                for s in states]

    x = np.arange(len(states))
    width = 0.35

    ax.bar(x - width/2, cut_devs, width, label='Edge-Cut', alpha=0.8, color='steelblue')
    ax.bar(x + width/2, vol_devs, width, label='Volume', alpha=0.8, color='darkorange')

    ax.set_xlabel('State', fontsize=11)
    ax.set_ylabel('Mean Population Deviation (%)', fontsize=11)
    ax.set_title('Population Balance: Edge-Cut vs. Volume', fontsize=12, fontweight='bold')
    ax.set_xticks(x)
    ax.set_xticklabels(states, rotation=45, ha='right')
    ax.legend()
    ax.grid(True, alpha=0.3, axis='y')

    # Plot 2: Scatter plot showing correlation
    ax = axes[1]
    ax.scatter(cut_devs, vol_devs, s=100, alpha=0.7, edgecolors='black', linewidths=1)

    # Add diagonal line (y=x)
    max_val = max(max(cut_devs), max(vol_devs))
    ax.plot([0, max_val], [0, max_val], 'k--', alpha=0.5, linewidth=2, label='Perfect Agreement')

    # Calculate correlation
    if len(cut_devs) > 1:
        r, p = stats.pearsonr(cut_devs, vol_devs)
        ax.text(0.05, 0.95, f'r = {r:.3f}\np = {p:.4f}',
               transform=ax.transAxes, fontsize=10,
               verticalalignment='top',
               bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.3))

    ax.set_xlabel('Edge-Cut: Mean Pop. Deviation (%)', fontsize=11)
    ax.set_ylabel('Volume: Mean Pop. Deviation (%)', fontsize=11)
    ax.set_title('Correlation Between Objectives', fontsize=12, fontweight='bold')
    ax.legend()
    ax.grid(True, alpha=0.3)

    plt.suptitle('Parameter Sensitivity: objtype (Objective Function)',
                fontsize=14, fontweight='bold', y=1.00)
    plt.tight_layout()

    output_path = OUTPUT_DIR / 'figure_4x3_objtype_comparison.pdf'
    plt.savefig(output_path, bbox_inches='tight')
    plt.savefig(OUTPUT_DIR / 'figure_4x3_objtype_comparison.png',
                bbox_inches='tight', dpi=150)
    print(f"  Saved: {output_path}")
    plt.close()


def plot_seed_ensemble_distribution(df):
    """Plot distribution of outcomes across random seeds."""
    if df is None or len(df) == 0:
        print("[SKIP] No seed ensemble data to plot")
        return

    print("Generating seed ensemble distribution plot...")

    # Filter to multi-district states only
    df_multi = df[df['num_districts'] > 1].copy()

    if len(df_multi) == 0:
        print("[WARNING] No multi-district states in seed data")
        return

    fig, axes = plt.subplots(2, 2, figsize=(12, 10))

    # Get unique states
    states = sorted(df_multi['state'].unique())
    colors = plt.cm.tab10(np.linspace(0, 1, len(states)))

    # Plot 1: Histogram of population deviations
    ax = axes[0, 0]
    for i, state in enumerate(states):
        state_data = df_multi[df_multi['state'] == state]
        ax.hist(state_data['mean_pop_deviation'], alpha=0.6,
               label=state, bins=20, color=colors[i])

    ax.set_xlabel('Mean Population Deviation (%)', fontsize=11)
    ax.set_ylabel('Frequency', fontsize=11)
    ax.set_title('Distribution Across 100 Random Seeds', fontsize=12, fontweight='bold')
    ax.legend(loc='best', fontsize=8, ncol=2)
    ax.grid(True, alpha=0.3, axis='y')

    # Plot 2: Box plot by state
    ax = axes[0, 1]
    data_by_state = [df_multi[df_multi['state'] == s]['mean_pop_deviation'].values
                     for s in states]
    bp = ax.boxplot(data_by_state, labels=states, patch_artist=True)

    for patch, color in zip(bp['boxes'], colors):
        patch.set_facecolor(color)
        patch.set_alpha(0.7)

    ax.set_xlabel('State', fontsize=11)
    ax.set_ylabel('Mean Population Deviation (%)', fontsize=11)
    ax.set_title('Variation by State', fontsize=12, fontweight='bold')
    ax.grid(True, alpha=0.3, axis='y')
    plt.setp(ax.xaxis.get_majorticklabels(), rotation=45, ha='right')

    # Plot 3: Coefficient of variation by state
    ax = axes[1, 0]
    state_stats = []
    for state in states:
        state_data = df_multi[df_multi['state'] == state]['mean_pop_deviation']
        mean_val = state_data.mean()
        std_val = state_data.std()
        cv = (std_val / mean_val * 100) if mean_val > 0 else 0
        state_stats.append({'state': state, 'mean': mean_val, 'std': std_val, 'cv': cv})

    stats_df = pd.DataFrame(state_stats)
    ax.bar(stats_df['state'], stats_df['cv'], alpha=0.8, color='steelblue')
    ax.set_xlabel('State', fontsize=11)
    ax.set_ylabel('Coefficient of Variation (%)', fontsize=11)
    ax.set_title('Consistency Across Seeds (Lower = More Stable)', fontsize=12, fontweight='bold')
    ax.grid(True, alpha=0.3, axis='y')
    plt.setp(ax.xaxis.get_majorticklabels(), rotation=45, ha='right')

    # Plot 4: Summary statistics table
    ax = axes[1, 1]
    ax.axis('off')

    summary_text = "Summary Statistics\n\n"
    for _, row in stats_df.iterrows():
        summary_text += f"{row['state']}:\n"
        summary_text += f"  Mean: {row['mean']:.3f}%\n"
        summary_text += f"  Std:  {row['std']:.3f}%\n"
        summary_text += f"  CV:   {row['cv']:.2f}%\n\n"

    # Add overall statistics
    overall_mean = df_multi['mean_pop_deviation'].mean()
    overall_std = df_multi['mean_pop_deviation'].std()
    overall_cv = (overall_std / overall_mean * 100) if overall_mean > 0 else 0

    summary_text += f"\nOverall (all states):\n"
    summary_text += f"  Mean: {overall_mean:.3f}%\n"
    summary_text += f"  Std:  {overall_std:.3f}%\n"
    summary_text += f"  CV:   {overall_cv:.2f}%"

    ax.text(0.1, 0.9, summary_text, transform=ax.transAxes,
           fontsize=9, verticalalignment='top', fontfamily='monospace',
           bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.3))

    plt.suptitle('Parameter Sensitivity: Random Seed Ensemble (100 runs per state)',
                fontsize=14, fontweight='bold', y=0.995)
    plt.tight_layout()

    output_path = OUTPUT_DIR / 'figure_4x4_seed_ensemble.pdf'
    plt.savefig(output_path, bbox_inches='tight')
    plt.savefig(OUTPUT_DIR / 'figure_4x4_seed_ensemble.png',
                bbox_inches='tight', dpi=150)
    print(f"  Saved: {output_path}")
    plt.close()


def generate_summary_statistics(df):
    """Generate summary statistics for paper."""
    if df is None or len(df) == 0:
        print("[SKIP] No data for summary statistics")
        return None

    print("Generating summary statistics...")

    # Filter to multi-district states only
    df_multi = df[df['num_districts'] > 1].copy()

    if len(df_multi) == 0:
        print("[WARNING] No multi-district states for summary")
        return None

    # Group by state and calculate statistics
    summary = df_multi.groupby('state').agg({
        'mean_pop_deviation': ['mean', 'std', 'min', 'max', 'count']
    }).round(4)

    summary.columns = ['_'.join(col).strip() for col in summary.columns.values]

    # Calculate variation percentages
    summary['cv_pct'] = (summary['mean_pop_deviation_std'] /
                         summary['mean_pop_deviation_mean'] * 100).round(2)

    # Save to CSV
    output_file = Path('outputs/sensitivity/summary_statistics.csv')
    output_file.parent.mkdir(parents=True, exist_ok=True)
    summary.to_csv(output_file)
    print(f"  Saved: {output_file}")

    # Print key findings
    print("\n" + "="*70)
    print("KEY FINDINGS")
    print("="*70)
    print(f"Mean population deviation variation: {summary['cv_pct'].mean():.2f}%")
    print(f"States analyzed: {len(summary)}")
    print(f"Total runs: {summary['mean_pop_deviation_count'].sum():.0f}")
    print("="*70 + "\n")

    return summary


def main():
    """Generate all visualizations."""
    print("\n" + "="*70)
    print("PARAMETER SENSITIVITY VISUALIZATION")
    print("="*70)
    print(f"Output directory: {OUTPUT_DIR}")
    print("="*70 + "\n")

    # Load all sweep data
    ufactor_df = load_sweep_data('ufactor')
    niter_df = load_sweep_data('niter')
    objtype_df = load_sweep_data('objtype')
    seed_df = load_sweep_data('seed_ensemble')

    # Generate plots
    plot_ufactor_sensitivity(ufactor_df)
    plot_niter_sensitivity(niter_df)
    plot_objtype_comparison(objtype_df)
    plot_seed_ensemble_distribution(seed_df)

    # Generate summary statistics from seed ensemble
    if seed_df is not None:
        summary = generate_summary_statistics(seed_df)

    print("\n" + "="*70)
    print("VISUALIZATION COMPLETE")
    print("="*70)
    print(f"Figures saved to: {OUTPUT_DIR}/")
    print("\nGenerated figures:")
    print("  - figure_4x1_ufactor_sensitivity.pdf")
    print("  - figure_4x2_niter_sensitivity.pdf")
    print("  - figure_4x3_objtype_comparison.pdf")
    print("  - figure_4x4_seed_ensemble.pdf")
    print("\nNext step: Update paper Section 4.5 with these figures")
    print("="*70 + "\n")


if __name__ == '__main__':
    main()
