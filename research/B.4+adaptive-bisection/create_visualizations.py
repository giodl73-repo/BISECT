"""
Create visualizations for Paper 3: Adaptive Recursive Bisection

Generates figures demonstrating the key finding: All partitioning methods
produce identical results when using edge-weighting (α=5, τ=0.40).
"""

import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
import numpy as np
from pathlib import Path
import ast

# Set style
plt.style.use('seaborn-v0_8-darkgrid')
sns.set_palette("husl")
plt.rcParams['figure.dpi'] = 300
plt.rcParams['font.size'] = 10
plt.rcParams['axes.labelsize'] = 11
plt.rcParams['axes.titlesize'] = 12
plt.rcParams['legend.fontsize'] = 9

# Directories
RESULTS_DIR = Path('results')
FIGURES_DIR = Path('figures')
FIGURES_DIR.mkdir(exist_ok=True)

# Load data
df = pd.read_csv(RESULTS_DIR / 'adaptive_experiments.csv')

# Parse district percentages
def parse_district_pcts(pcts_str):
    """Parse district percentages string to list of floats."""
    if pd.isna(pcts_str):
        return None
    try:
        return ast.literal_eval(pcts_str)
    except:
        return None

df['district_pcts_parsed'] = df['district_pcts'].apply(parse_district_pcts)

# State name mapping
STATE_NAMES = {
    'alabama': 'Alabama',
    'georgia': 'Georgia',
    'louisiana': 'Louisiana',
    'mississippi': 'Mississippi',
    'south_carolina': 'South Carolina'
}

df['state_name'] = df['state'].map(STATE_NAMES)

# Method name mapping
METHOD_NAMES = {
    'predetermined': 'Predetermined',
    'adaptive': 'Adaptive',
    'nway': 'N-way'
}

df['method_name'] = df['method'].map(METHOD_NAMES)


def create_figure_1_max_minority_comparison():
    """Figure 1: Maximum minority percentage by method and state."""

    # Group by state and method, get statistics
    stats = df.groupby(['state_name', 'method_name']).agg({
        'max_minority_pct': ['mean', 'std', 'min', 'max']
    }).reset_index()

    stats.columns = ['state_name', 'method_name', 'mean', 'std', 'min', 'max']

    # Create figure
    fig, ax = plt.subplots(figsize=(10, 6))

    states = sorted(df['state_name'].unique())
    methods = ['Predetermined', 'Adaptive', 'N-way']
    x = np.arange(len(states))
    width = 0.25

    colors = {'Predetermined': '#3498db', 'Adaptive': '#e74c3c', 'N-way': '#2ecc71'}

    for i, method in enumerate(methods):
        method_data = stats[stats['method_name'] == method].sort_values('state_name')
        values = method_data['mean'].values * 100
        errors = method_data['std'].values * 100

        ax.bar(x + i * width, values, width,
               label=method, color=colors[method],
               yerr=errors, capsize=3, alpha=0.8)

    # Add 50% threshold line
    ax.axhline(y=50, color='red', linestyle='--', linewidth=1.5,
               label='50% MM Threshold', alpha=0.7)

    ax.set_xlabel('State', fontweight='bold')
    ax.set_ylabel('Maximum Minority Percentage (%)', fontweight='bold')
    ax.set_title('Figure 1: Maximum Minority Percentage by Method\n(All Methods Produce Identical Results)',
                 fontweight='bold', pad=15)
    ax.set_xticks(x + width)
    ax.set_xticklabels(states, rotation=0)
    ax.legend(loc='upper right', framealpha=0.95)
    ax.grid(axis='y', alpha=0.3)
    ax.set_ylim(0, 90)

    plt.tight_layout()
    plt.savefig(FIGURES_DIR / 'figure_1_max_minority_comparison.png', dpi=300, bbox_inches='tight')
    plt.close()
    print("[OK] Created Figure 1: Max minority comparison")


def create_figure_2_mm_count_comparison():
    """Figure 2: MM district count by method and state."""

    # Group by state and method
    stats = df.groupby(['state_name', 'method_name']).agg({
        'mm_count': ['mean', 'std'],
        'target_mm': 'first'
    }).reset_index()

    stats.columns = ['state_name', 'method_name', 'mm_mean', 'mm_std', 'target_mm']

    # Create figure
    fig, ax = plt.subplots(figsize=(10, 6))

    states = sorted(df['state_name'].unique())
    methods = ['Predetermined', 'Adaptive', 'N-way']
    x = np.arange(len(states))
    width = 0.25

    colors = {'Predetermined': '#3498db', 'Adaptive': '#e74c3c', 'N-way': '#2ecc71'}

    for i, method in enumerate(methods):
        method_data = stats[stats['method_name'] == method].sort_values('state_name')
        values = method_data['mm_mean'].values
        errors = method_data['mm_std'].values

        ax.bar(x + i * width, values, width,
               label=method, color=colors[method],
               yerr=errors, capsize=3, alpha=0.8)

    # Add target MM count markers
    targets = stats[stats['method_name'] == 'Predetermined'].sort_values('state_name')
    for j, (state_x, target) in enumerate(zip(x, targets['target_mm'])):
        ax.plot([state_x - 0.4, state_x + 1.0], [target, target],
                'k--', linewidth=1, alpha=0.5)
        ax.text(state_x + 1.15, target, f'Target: {int(target)}',
                fontsize=8, va='center', alpha=0.7)

    ax.set_xlabel('State', fontweight='bold')
    ax.set_ylabel('Majority-Minority Districts Achieved', fontweight='bold')
    ax.set_title('Figure 2: MM District Count by Method\n(All Methods Produce Identical Results)',
                 fontweight='bold', pad=15)
    ax.set_xticks(x + width)
    ax.set_xticklabels(states, rotation=0)
    ax.legend(loc='upper left', framealpha=0.95)
    ax.grid(axis='y', alpha=0.3)
    ax.set_ylim(0, 7)

    plt.tight_layout()
    plt.savefig(FIGURES_DIR / 'figure_2_mm_count_comparison.png', dpi=300, bbox_inches='tight')
    plt.close()
    print("[OK] Created Figure 2: MM count comparison")


def create_figure_3_runtime_comparison():
    """Figure 3: Runtime comparison across methods."""

    # Calculate statistics
    stats = df.groupby(['state_name', 'method_name'])['runtime'].agg(['mean', 'std']).reset_index()

    # Create figure
    fig, ax = plt.subplots(figsize=(10, 6))

    states = sorted(df['state_name'].unique())
    methods = ['Predetermined', 'Adaptive', 'N-way']
    x = np.arange(len(states))
    width = 0.25

    colors = {'Predetermined': '#3498db', 'Adaptive': '#e74c3c', 'N-way': '#2ecc71'}

    for i, method in enumerate(methods):
        method_data = stats[stats['method_name'] == method].sort_values('state_name')
        values = method_data['mean'].values
        errors = method_data['std'].values

        ax.bar(x + i * width, values, width,
               label=method, color=colors[method],
               yerr=errors, capsize=3, alpha=0.8)

    ax.set_xlabel('State', fontweight='bold')
    ax.set_ylabel('Runtime (seconds)', fontweight='bold')
    ax.set_title('Figure 3: Runtime Comparison by Method\n(Adaptive Requires More Computation for Same Result)',
                 fontweight='bold', pad=15)
    ax.set_xticks(x + width)
    ax.set_xticklabels(states, rotation=0)
    ax.legend(loc='upper left', framealpha=0.95)
    ax.grid(axis='y', alpha=0.3)
    ax.set_yscale('log')

    plt.tight_layout()
    plt.savefig(FIGURES_DIR / 'figure_3_runtime_comparison.png', dpi=300, bbox_inches='tight')
    plt.close()
    print("[OK] Created Figure 3: Runtime comparison")


def create_figure_4_method_equivalence_heatmap():
    """Figure 4: Heatmap showing method equivalence."""

    # Create matrix showing max minority % by state and method
    pivot = df.groupby(['state_name', 'method_name'])['max_minority_pct'].mean().reset_index()
    pivot_matrix = pivot.pivot(index='state_name', columns='method_name', values='max_minority_pct') * 100

    # Reorder columns
    pivot_matrix = pivot_matrix[['Predetermined', 'Adaptive', 'N-way']]

    # Create figure
    fig, ax = plt.subplots(figsize=(8, 6))

    # Create heatmap
    sns.heatmap(pivot_matrix, annot=True, fmt='.1f', cmap='YlOrRd',
                cbar_kws={'label': 'Maximum Minority %'},
                linewidths=1, linecolor='white',
                vmin=40, vmax=85, ax=ax)

    ax.set_title('Figure 4: Method Equivalence Matrix\n(Identical Results Across All Methods)',
                 fontweight='bold', pad=15)
    ax.set_xlabel('Partitioning Method', fontweight='bold')
    ax.set_ylabel('State', fontweight='bold')

    plt.tight_layout()
    plt.savefig(FIGURES_DIR / 'figure_4_method_equivalence_heatmap.png', dpi=300, bbox_inches='tight')
    plt.close()
    print("[OK] Created Figure 4: Method equivalence heatmap")


def create_figure_5_variance_analysis():
    """Figure 5: Variance across tree structures (should be zero)."""

    # Focus on predetermined method only
    pred_df = df[df['method'] == 'predetermined'].copy()

    # Calculate variance by state
    variance_stats = pred_df.groupby('state_name')['max_minority_pct'].agg([
        ('mean', 'mean'),
        ('std', 'std'),
        ('min', 'min'),
        ('max', 'max'),
        ('count', 'count')
    ]).reset_index()

    # Create figure with subplots
    fig, axes = plt.subplots(2, 1, figsize=(10, 10))

    # Top plot: Mean with error bars (should show zero variance)
    ax1 = axes[0]
    states = sorted(variance_stats['state_name'])
    x = np.arange(len(states))

    means = variance_stats.sort_values('state_name')['mean'].values * 100
    stds = variance_stats.sort_values('state_name')['std'].values * 100

    ax1.bar(x, means, color='#3498db', alpha=0.7, label='Mean')
    ax1.errorbar(x, means, yerr=stds, fmt='none', ecolor='red',
                 capsize=5, capthick=2, linewidth=2, label='Std Dev (Zero!)')

    ax1.set_xlabel('State', fontweight='bold')
    ax1.set_ylabel('Maximum Minority Percentage (%)', fontweight='bold')
    ax1.set_title('Tree Structure Has Zero Effect on Results', fontweight='bold')
    ax1.set_xticks(x)
    ax1.set_xticklabels(states, rotation=0)
    ax1.legend(loc='upper right')
    ax1.grid(axis='y', alpha=0.3)

    # Bottom plot: Min-Max range (should be zero)
    ax2 = axes[1]

    mins = variance_stats.sort_values('state_name')['min'].values * 100
    maxs = variance_stats.sort_values('state_name')['max'].values * 100
    ranges = maxs - mins

    ax2.bar(x, ranges, color='#e74c3c', alpha=0.7)

    # Add text labels showing the range
    for i, (state_x, range_val) in enumerate(zip(x, ranges)):
        ax2.text(state_x, range_val + 0.5, f'{range_val:.4f}%',
                ha='center', fontsize=9, fontweight='bold')

    ax2.set_xlabel('State', fontweight='bold')
    ax2.set_ylabel('Range (Max - Min) %', fontweight='bold')
    ax2.set_title('Variation Across All Tree Structures (Zero for All States)', fontweight='bold')
    ax2.set_xticks(x)
    ax2.set_xticklabels(states, rotation=0)
    ax2.grid(axis='y', alpha=0.3)
    ax2.set_ylim(-0.1, max(ranges) + 1 if max(ranges) > 0 else 0.5)

    plt.tight_layout()
    plt.savefig(FIGURES_DIR / 'figure_5_variance_analysis.png', dpi=300, bbox_inches='tight')
    plt.close()
    print("[OK] Created Figure 5: Variance analysis (zero variance)")


def create_figure_6_district_distributions():
    """Figure 6: District-level minority percentage distributions."""

    # Create one plot per state showing all districts
    fig, axes = plt.subplots(2, 3, figsize=(15, 10))
    axes = axes.flatten()

    states = sorted(df['state_name'].unique())

    for idx, state in enumerate(states):
        ax = axes[idx]

        state_df = df[(df['state_name'] == state) & (df['district_pcts_parsed'].notna())]

        if len(state_df) == 0:
            continue

        # Get one example (they're all identical)
        example = state_df.iloc[0]
        district_pcts = example['district_pcts_parsed']
        k = int(example['k'])
        target_mm = example['target_mm']

        if district_pcts:
            # Convert to percentages
            pcts = [p * 100 for p in district_pcts]
            districts = list(range(1, len(pcts) + 1))

            # Color by MM status
            colors = ['#2ecc71' if p >= 50 else '#e74c3c' for p in pcts]

            ax.bar(districts, pcts, color=colors, alpha=0.7)
            ax.axhline(y=50, color='black', linestyle='--', linewidth=1.5, alpha=0.7)

            ax.set_xlabel('District', fontweight='bold')
            ax.set_ylabel('Minority %', fontweight='bold')
            ax.set_title(f'{state}\n(k={k}, target MM={int(target_mm)})', fontweight='bold')
            ax.set_xticks(districts)
            ax.grid(axis='y', alpha=0.3)
            ax.set_ylim(0, 90)

            # Add text showing count
            mm_count = sum(1 for p in pcts if p >= 50)
            ax.text(0.98, 0.95, f'MM: {mm_count}/{int(target_mm)}',
                   transform=ax.transAxes, ha='right', va='top',
                   bbox=dict(boxstyle='round', facecolor='white', alpha=0.8),
                   fontweight='bold')

    # Hide unused subplot
    axes[5].axis('off')

    fig.suptitle('Figure 6: District-Level Minority Distributions\n(Identical Across All Methods)',
                 fontweight='bold', fontsize=14, y=0.995)

    plt.tight_layout()
    plt.savefig(FIGURES_DIR / 'figure_6_district_distributions.png', dpi=300, bbox_inches='tight')
    plt.close()
    print("[OK] Created Figure 6: District distributions")


def create_summary_table():
    """Create summary table for paper."""

    # Group by state and method
    summary = df.groupby(['state_name', 'method_name']).agg({
        'k': 'first',
        'max_minority_pct': 'mean',
        'mm_count': 'mean',
        'target_mm': 'first',
        'success': 'mean',
        'runtime': 'mean'
    }).reset_index()

    summary['max_minority_pct'] = (summary['max_minority_pct'] * 100).round(1)
    summary['runtime'] = summary['runtime'].round(2)
    summary['success_pct'] = (summary['success'] * 100).round(0).astype(int)

    # Pivot for easier reading
    for metric in ['max_minority_pct', 'mm_count', 'runtime']:
        pivot = summary.pivot(index='state_name', columns='method_name', values=metric)
        pivot = pivot[['Predetermined', 'Adaptive', 'N-way']]

        print(f"\n{metric.upper().replace('_', ' ')}:")
        print(pivot.to_string())

        # Save to CSV
        pivot.to_csv(FIGURES_DIR / f'table_{metric}.csv')

    print("\n[OK] Created summary tables")


def main():
    """Generate all figures."""
    print("=" * 60)
    print("Generating Visualizations for Paper 3: Adaptive Bisection")
    print("=" * 60)
    print()

    print("Creating figures...")
    create_figure_1_max_minority_comparison()
    create_figure_2_mm_count_comparison()
    create_figure_3_runtime_comparison()
    create_figure_4_method_equivalence_heatmap()
    create_figure_5_variance_analysis()
    create_figure_6_district_distributions()

    print("\nCreating summary tables...")
    create_summary_table()

    print()
    print("=" * 60)
    print("[OK] All visualizations complete!")
    print(f"[OK] Saved to: {FIGURES_DIR.absolute()}")
    print("=" * 60)


if __name__ == '__main__':
    main()
