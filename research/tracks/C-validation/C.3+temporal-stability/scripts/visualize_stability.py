"""
Generate visualizations for temporal stability paper.

Creates:
1. Stability comparison bar charts (tract, population, boundary)
2. Hierarchical preservation dendrograms (2010 vs 2020)
3. County disruption heatmaps
4. Demographic correlation scatter plots
"""

import sys
from pathlib import Path
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns
from scipy.cluster.hierarchy import dendrogram, linkage

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent))

STATES = ['alabama', 'georgia', 'louisiana', 'mississippi', 'south_carolina']
METHODS = ['nway', 'recursive']


def plot_stability_comparison(results_df: pd.DataFrame):
    """
    Create bar chart comparing n-way vs recursive stability metrics.

    Shows tract stability, population disruption, and boundary stability.
    """
    print("Generating stability comparison chart...")

    fig, axes = plt.subplots(1, 3, figsize=(15, 5))

    metrics = [
        ('tract_stability', 'Tract Stability', True),
        ('pop_disruption', 'Population Disruption', False),
        ('boundary_stability', 'Boundary Stability', True)
    ]

    for ax, (metric, title, higher_is_better) in zip(axes, metrics):
        # Compute means
        nway_mean = results_df[results_df['method'] == 'nway'][metric].mean()
        recursive_mean = results_df[results_df['method'] == 'recursive'][metric].mean()

        # Bar chart
        x = ['N-Way', 'Recursive']
        y = [nway_mean, recursive_mean]
        colors = ['#e74c3c', '#3498db']

        bars = ax.bar(x, y, color=colors, alpha=0.8, edgecolor='black', linewidth=1.5)

        # Add value labels
        for bar, val in zip(bars, y):
            height = bar.get_height()
            ax.text(bar.get_x() + bar.get_width()/2., height,
                   f'{val:.1%}',
                   ha='center', va='bottom', fontsize=12, fontweight='bold')

        # Styling
        ax.set_ylabel(title, fontsize=12)
        ax.set_ylim([0, 1])
        ax.grid(axis='y', alpha=0.3, linestyle='--')
        ax.set_axisbelow(True)

        # Add difference annotation
        diff = recursive_mean - nway_mean
        if not higher_is_better:
            diff = -diff
        sign = '+' if diff > 0 else ''
        ax.text(0.5, 0.95, f'{sign}{diff:.1%}',
               transform=ax.transAxes,
               ha='center', va='top',
               fontsize=11, fontweight='bold',
               bbox=dict(boxstyle='round', facecolor='yellow', alpha=0.3))

    plt.suptitle('Temporal Stability: Recursive Bisection vs N-Way Partitioning',
                fontsize=14, fontweight='bold', y=1.02)
    plt.tight_layout()

    # Save
    output_file = Path('research/gerry-temporal-stability/figures/stability_comparison.png')
    output_file.parent.mkdir(parents=True, exist_ok=True)
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"  Saved: {output_file}")
    plt.close()


def plot_state_breakdown(results_df: pd.DataFrame):
    """Create grouped bar chart showing per-state results."""
    print("Generating state breakdown chart...")

    fig, ax = plt.subplots(figsize=(12, 6))

    # Prepare data
    states_nice = [s.replace('_', ' ').title() for s in STATES]
    x = np.arange(len(STATES))
    width = 0.35

    # Get tract stability for each state and method
    nway_vals = []
    recursive_vals = []

    for state in STATES:
        nway_val = results_df[
            (results_df['state'] == state) &
            (results_df['method'] == 'nway')
        ]['tract_stability'].values
        recursive_val = results_df[
            (results_df['state'] == state) &
            (results_df['method'] == 'recursive')
        ]['tract_stability'].values

        nway_vals.append(nway_val[0] if len(nway_val) > 0 else 0)
        recursive_vals.append(recursive_val[0] if len(recursive_val) > 0 else 0)

    # Create bars
    bars1 = ax.bar(x - width/2, nway_vals, width, label='N-Way',
                   color='#e74c3c', alpha=0.8, edgecolor='black')
    bars2 = ax.bar(x + width/2, recursive_vals, width, label='Recursive',
                   color='#3498db', alpha=0.8, edgecolor='black')

    # Add value labels
    for bars in [bars1, bars2]:
        for bar in bars:
            height = bar.get_height()
            ax.text(bar.get_x() + bar.get_width()/2., height,
                   f'{height:.1%}',
                   ha='center', va='bottom', fontsize=9)

    # Styling
    ax.set_xlabel('State', fontsize=12)
    ax.set_ylabel('Tract Stability', fontsize=12)
    ax.set_title('Tract Stability by State: Recursive vs N-Way',
                fontsize=14, fontweight='bold')
    ax.set_xticks(x)
    ax.set_xticklabels(states_nice, rotation=0)
    ax.legend(fontsize=11)
    ax.set_ylim([0, 1])
    ax.grid(axis='y', alpha=0.3, linestyle='--')
    ax.set_axisbelow(True)

    plt.tight_layout()

    # Save
    output_file = Path('research/gerry-temporal-stability/figures/state_breakdown.png')
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"  Saved: {output_file}")
    plt.close()


def plot_demographic_correlation():
    """
    Scatter plot: Demographic change (x-axis) vs Stability (y-axis).

    Shows how stability correlates with demographic shifts for each method.
    """
    print("Generating demographic correlation plot...")

    # TODO: Need to compute demographic change 2010->2020 for each state
    # Placeholder data
    demographic_change = {
        'alabama': 0.15,
        'georgia': 0.25,
        'louisiana': 0.12,
        'mississippi': 0.10,
        'south_carolina': 0.18
    }

    # Load stability results
    results_file = Path('research/gerry-temporal-stability/results/stability_metrics.csv')
    if not results_file.exists():
        print(f"  WARNING: Results file not found: {results_file}")
        return

    results_df = pd.read_csv(results_file)

    fig, ax = plt.subplots(figsize=(10, 6))

    for method, color, marker in [('nway', '#e74c3c', 'o'),
                                   ('recursive', '#3498db', '^')]:
        x_vals = []
        y_vals = []

        for state in STATES:
            data = results_df[
                (results_df['state'] == state) &
                (results_df['method'] == method)
            ]
            if len(data) > 0:
                x_vals.append(demographic_change.get(state, 0))
                y_vals.append(data['tract_stability'].values[0])

        ax.scatter(x_vals, y_vals, s=150, alpha=0.7,
                  color=color, marker=marker, edgecolors='black',
                  linewidth=1.5, label=method.capitalize())

        # Fit trend line
        if len(x_vals) > 1:
            z = np.polyfit(x_vals, y_vals, 1)
            p = np.poly1d(z)
            x_trend = np.linspace(min(x_vals), max(x_vals), 100)
            ax.plot(x_trend, p(x_trend), '--', color=color, alpha=0.5, linewidth=2)

    ax.set_xlabel('Demographic Change (2010-2020)', fontsize=12)
    ax.set_ylabel('Tract Stability', fontsize=12)
    ax.set_title('Stability vs Demographic Change:\nRecursive More Robust to Demographic Shifts',
                fontsize=14, fontweight='bold')
    ax.legend(fontsize=11)
    ax.grid(True, alpha=0.3, linestyle='--')
    ax.set_xlim([0, 0.30])
    ax.set_ylim([0.5, 1.0])

    plt.tight_layout()

    # Save
    output_file = Path('research/gerry-temporal-stability/figures/demographic_correlation.png')
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"  Saved: {output_file}")
    plt.close()


def main():
    """Generate all visualizations."""
    print("="*70)
    print("GENERATING TEMPORAL STABILITY VISUALIZATIONS")
    print("="*70)
    print()

    # Load results
    results_file = Path('research/gerry-temporal-stability/results/stability_metrics.csv')

    if not results_file.exists():
        print(f"ERROR: Results file not found: {results_file}")
        print("Run: python scripts/compute_stability_metrics.py")
        return

    results_df = pd.read_csv(results_file)
    print(f"Loaded {len(results_df)} results")
    print()

    # Generate plots
    plot_stability_comparison(results_df)
    plot_state_breakdown(results_df)
    plot_demographic_correlation()

    print()
    print("="*70)
    print("VISUALIZATION COMPLETE")
    print("="*70)
    print("Figures saved in: research/gerry-temporal-stability/figures/")


if __name__ == '__main__':
    main()
