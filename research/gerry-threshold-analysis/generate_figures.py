"""
Generate visualizations for threshold analysis paper.

Figures:
1. Success rate by state minority % (demonstrates 42% threshold)
2. Edge-weighted vs Multi-constraint comparison
3. Feasibility metric validation scatter plot
4. Clustering vs Success relationship
"""

import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
from pathlib import Path

# Set up matplotlib style
plt.style.use('seaborn-v0_8-darkgrid')
plt.rcParams['figure.figsize'] = (10, 6)
plt.rcParams['font.size'] = 11
plt.rcParams['axes.labelsize'] = 12
plt.rcParams['axes.titlesize'] = 14
plt.rcParams['xtick.labelsize'] = 10
plt.rcParams['ytick.labelsize'] = 10
plt.rcParams['legend.fontsize'] = 10

# State colors
STATE_COLORS = {
    'MS': '#1f77b4',  # Blue
    'GA': '#ff7f0e',  # Orange
    'LA': '#2ca02c',  # Green
    'AL': '#d62728',  # Red
    'SC': '#9467bd',  # Purple
}


def load_data():
    """Load all analysis results."""
    feasibility_df = pd.read_csv('results/feasibility_analysis.csv')
    clustering_df = pd.read_csv('results/geographic_clustering_metrics.csv')

    return feasibility_df, clustering_df


def figure1_threshold_demonstration(df):
    """
    Figure 1: Success Rate by State Minority Percentage

    Demonstrates the 42% threshold with edge-weighted method.
    """
    # Filter to edge-weighted only
    edge_df = df[df['method'] == 'edge_weighted'].sort_values('state_minority_pct')

    fig, ax = plt.subplots(figsize=(10, 6))

    # Bar chart
    bars = ax.bar(
        edge_df['state_code'],
        edge_df['success_rate'] * 100,
        color=[STATE_COLORS[s] for s in edge_df['state_code']],
        edgecolor='black',
        linewidth=1.5,
        alpha=0.8
    )

    # Add state minority % labels on top of bars
    for bar, (_, row) in zip(bars, edge_df.iterrows()):
        height = bar.get_height()
        ax.text(
            bar.get_x() + bar.get_width() / 2,
            height + 3,
            f"{row['state_minority_pct']:.1%}",
            ha='center',
            va='bottom',
            fontsize=10,
            fontweight='bold'
        )

    # Add 50% success threshold line
    ax.axhline(y=50, color='gray', linestyle='--', linewidth=2, alpha=0.5, label='50% Success Threshold')

    # Add 42% minority threshold annotation
    ax.axvline(x=1.5, color='red', linestyle='--', linewidth=2, alpha=0.7)
    ax.text(1.5, 95, '42% Minority Threshold', ha='center', color='red', fontweight='bold', fontsize=11)

    # Styling
    ax.set_xlabel('State', fontweight='bold')
    ax.set_ylabel('Success Rate (%)', fontweight='bold')
    ax.set_title('Edge-Weighted Success Rate by State Minority Percentage\n(The 42% Threshold)', fontweight='bold', pad=20)
    ax.set_ylim(0, 110)
    ax.legend(loc='upper left')
    ax.grid(True, alpha=0.3, axis='y')

    # Save
    output_dir = Path('results')
    fig.savefig(output_dir / 'figure1_threshold_demonstration.png', dpi=300, bbox_inches='tight')
    fig.savefig(output_dir / 'figure1_threshold_demonstration.pdf', bbox_inches='tight')
    plt.close()

    print("Created Figure 1: Threshold demonstration")


def figure2_method_comparison(df):
    """
    Figure 2: Edge-Weighted vs Multi-Constraint Comparison

    Shows success rates for both methods across all states.
    """
    # Pivot data for grouped bar chart
    pivot_df = df.pivot_table(
        index='state_code',
        columns='method',
        values='success_rate'
    ).reindex(['MS', 'GA', 'LA', 'AL', 'SC'])

    fig, ax = plt.subplots(figsize=(12, 6))

    # Bar positions
    x = np.arange(len(pivot_df))
    width = 0.35

    # Bars
    bars1 = ax.bar(x - width/2, pivot_df['edge_weighted'] * 100, width,
                    label='Edge-Weighted', color='steelblue', edgecolor='black', linewidth=1.5)
    bars2 = ax.bar(x + width/2, pivot_df['multi_constraint'] * 100, width,
                    label='Multi-Constraint', color='coral', edgecolor='black', linewidth=1.5)

    # Value labels on bars
    for bars in [bars1, bars2]:
        for bar in bars:
            height = bar.get_height()
            if height > 0:
                ax.text(bar.get_x() + bar.get_width()/2, height + 2,
                       f'{height:.0f}%', ha='center', va='bottom', fontsize=9)

    # Add state minority % as secondary x-axis labels
    state_minorities = df[df['method'] == 'edge_weighted'].set_index('state_code')['state_minority_pct']
    state_minorities = state_minorities.reindex(['MS', 'GA', 'LA', 'AL', 'SC'])

    # X-axis labels with minority %
    labels = [f"{code}\n({pct:.1%})" for code, pct in zip(pivot_df.index, state_minorities)]

    # Styling
    ax.set_xlabel('State (State Minority %)', fontweight='bold')
    ax.set_ylabel('Success Rate (%)', fontweight='bold')
    ax.set_title('VRA Compliance Success Rates: Edge-Weighted vs Multi-Constraint', fontweight='bold', pad=20)
    ax.set_xticks(x)
    ax.set_xticklabels(labels)
    ax.set_ylim(0, 115)
    ax.legend(loc='upper right')
    ax.grid(True, alpha=0.3, axis='y')

    # Save
    output_dir = Path('results')
    fig.savefig(output_dir / 'figure2_method_comparison.png', dpi=300, bbox_inches='tight')
    fig.savefig(output_dir / 'figure2_method_comparison.pdf', bbox_inches='tight')
    plt.close()

    print("Created Figure 2: Method comparison")


def figure3_feasibility_validation(df):
    """
    Figure 3: State Minority % vs Success Rate (Scatter Plot)

    Shows the strong correlation between state-level minority % and success.
    """
    edge_df = df[df['method'] == 'edge_weighted']

    fig, ax = plt.subplots(figsize=(10, 8))

    # Scatter plot with different markers for achieves_target
    for achieves, marker, label in [(True, 'o', 'Achieves Target'), (False, 'X', 'Fails Target')]:
        subset = edge_df[edge_df['achieves_target'] == achieves]
        ax.scatter(
            subset['state_minority_pct'] * 100,
            subset['success_rate'] * 100,
            s=400,
            marker=marker,
            c=[STATE_COLORS[s] for s in subset['state_code']],
            edgecolors='black',
            linewidth=2,
            alpha=0.8,
            label=label
        )

    # State labels
    for _, row in edge_df.iterrows():
        ax.annotate(
            row['state_code'],
            (row['state_minority_pct'] * 100, row['success_rate'] * 100),
            fontsize=11,
            fontweight='bold',
            ha='center',
            va='center'
        )

    # Add 42% threshold line
    ax.axvline(x=42, color='red', linestyle='--', linewidth=2, alpha=0.7, label='42% Threshold')

    # Add trend line
    from scipy.stats import linregress
    x = edge_df['state_minority_pct'] * 100
    y = edge_df['success_rate'] * 100
    slope, intercept, r_value, p_value, std_err = linregress(x, y)

    x_line = np.linspace(x.min(), x.max(), 100)
    y_line = slope * x_line + intercept
    ax.plot(x_line, y_line, 'k--', alpha=0.5, linewidth=2, label=f'Linear Fit (r={r_value:.3f})')

    # Styling
    ax.set_xlabel('State Minority Percentage (%)', fontweight='bold')
    ax.set_ylabel('Success Rate (%)', fontweight='bold')
    ax.set_title('State Minority % Predicts VRA Compliance Success\n(Edge-Weighted Method)', fontweight='bold', pad=20)
    ax.set_xlim(32, 50)
    ax.set_ylim(-10, 110)
    ax.legend(loc='upper left', framealpha=0.9)
    ax.grid(True, alpha=0.3)

    # Save
    output_dir = Path('results')
    fig.savefig(output_dir / 'figure3_minority_pct_correlation.png', dpi=300, bbox_inches='tight')
    fig.savefig(output_dir / 'figure3_minority_pct_correlation.pdf', bbox_inches='tight')
    plt.close()

    print("Created Figure 3: State minority % correlation")


def figure4_clustering_impact(feasibility_df, clustering_df):
    """
    Figure 4: Moran's I vs Success Rate (Clustering Impact)

    Shows how spatial clustering affects VRA compliance success.
    """
    # Merge edge-weighted data with clustering metrics
    edge_df = feasibility_df[feasibility_df['method'] == 'edge_weighted']

    fig, ax = plt.subplots(figsize=(10, 8))

    # Scatter plot with size proportional to state minority %
    for _, row in edge_df.iterrows():
        size = row['state_minority_pct'] * 2000  # Scale for visibility
        ax.scatter(
            row['morans_i'],
            row['success_rate'] * 100,
            s=size,
            color=STATE_COLORS[row['state_code']],
            edgecolors='black',
            linewidth=2,
            alpha=0.7
        )

    # State labels
    for _, row in edge_df.iterrows():
        ax.annotate(
            f"{row['state_code']}\n({row['state_minority_pct']:.1%})",
            (row['morans_i'], row['success_rate'] * 100),
            fontsize=10,
            fontweight='bold',
            ha='center',
            va='center'
        )

    # Styling
    ax.set_xlabel("Moran's I (Spatial Autocorrelation)", fontweight='bold')
    ax.set_ylabel('Success Rate (%)', fontweight='bold')
    ax.set_title("Geographic Clustering Impact on VRA Compliance\n(Bubble size = State Minority %)", fontweight='bold', pad=20)
    ax.set_xlim(0.55, 0.80)
    ax.set_ylim(-10, 110)
    ax.grid(True, alpha=0.3)

    # Add note
    ax.text(
        0.57, 100,
        "Note: Higher Moran's I = more clustered minority population",
        fontsize=9,
        style='italic',
        bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.5)
    )

    # Save
    output_dir = Path('results')
    fig.savefig(output_dir / 'figure4_clustering_impact.png', dpi=300, bbox_inches='tight')
    fig.savefig(output_dir / 'figure4_clustering_impact.pdf', bbox_inches='tight')
    plt.close()

    print("Created Figure 4: Clustering impact")


def main():
    """Generate all figures."""
    print("Generating figures for threshold analysis paper...")

    # Load data
    feasibility_df, clustering_df = load_data()

    # Create figures
    figure1_threshold_demonstration(feasibility_df)
    figure2_method_comparison(feasibility_df)
    figure3_feasibility_validation(feasibility_df)
    figure4_clustering_impact(feasibility_df, clustering_df)

    print("\nAll figures created successfully!")
    print("Saved to: research/gerry-threshold-analysis/results/")


if __name__ == '__main__':
    main()
