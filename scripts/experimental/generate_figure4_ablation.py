"""
Generate Figure 4: State Boundary Crossing Analysis (Ablation Study) for Paper #13.

Shows dual-axis plot:
  - Left axis: Number of cross-state districts (bars)
  - Right axis: Mean Polsby-Popper score (line)
  - X-axis: Boundary crossing penalty beta

Demonstrates trade-off between compactness and boundary crossing.

Usage:
    python scripts/experimental/generate_figure4_ablation.py --year 2020
"""

import argparse
import pickle
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd


def load_ablation_data(input_dir: Path, year: int):
    """Load ablation study summary data."""
    print(f"\nLoading ablation study data...")

    ablation_file = input_dir / f"ablation_compactness_summary.pkl"
    with open(ablation_file, 'rb') as f:
        data = pickle.load(f)

    results_df = pd.DataFrame(data['results'])

    print(f"  {len(results_df)} beta values")
    print(f"\n  Summary:")
    print(results_df[['beta', 'n_cross_state', 'mean_pp']].to_string(index=False))

    return results_df


def load_baseline(input_dir: Path, year: int):
    """Load state-based baseline for reference."""
    print(f"\nLoading state-based baseline...")

    comparison_file = input_dir / f"compactness_comparison_{year}.pkl"
    with open(comparison_file, 'rb') as f:
        data = pickle.load(f)

    baseline_pp = data['state_baseline']['mean']
    print(f"  State-based mean PP: {baseline_pp:.3f}")

    return baseline_pp


def generate_ablation_figure(
    ablation_df: pd.DataFrame,
    baseline_pp: float,
    output_file: Path,
    dpi: int = 300
):
    """
    Generate ablation study figure showing boundary crossing vs compactness trade-off.
    """
    print(f"\nGenerating ablation study figure...")

    # Create figure with dual y-axes
    fig, ax1 = plt.subplots(figsize=(12, 7))

    # Left y-axis: Number of cross-state districts (bars)
    x = ablation_df['beta'].values
    cross_state = ablation_df['n_cross_state'].values

    bars = ax1.bar(
        x,
        cross_state,
        width=0.3,
        color='#C44E52',
        alpha=0.6,
        label='Cross-State Districts'
    )

    ax1.set_xlabel('Boundary Crossing Penalty (beta)', fontsize=12, fontweight='bold')
    ax1.set_ylabel('Number of Cross-State Districts', fontsize=12, fontweight='bold', color='#C44E52')
    ax1.tick_params(axis='y', labelcolor='#C44E52')
    ax1.set_ylim(0, 200)

    # Add value labels on bars
    for bar, value in zip(bars, cross_state):
        height = bar.get_height()
        ax1.text(
            bar.get_x() + bar.get_width() / 2,
            height + 3,
            f'{int(value)}',
            ha='center',
            va='bottom',
            fontsize=9,
            color='#C44E52'
        )

    # Right y-axis: Mean PP score (line)
    ax2 = ax1.twinx()

    pp_scores = ablation_df['mean_pp'].values

    line = ax2.plot(
        x,
        pp_scores,
        color='#2E7D96',
        marker='o',
        markersize=8,
        linewidth=2.5,
        label='Mean Polsby-Popper Score'
    )

    ax2.set_ylabel('Mean Polsby-Popper Score', fontsize=12, fontweight='bold', color='#2E7D96')
    ax2.tick_params(axis='y', labelcolor='#2E7D96')
    ax2.set_ylim(0, 0.5)

    # Add horizontal line for state-based baseline
    ax2.axhline(
        baseline_pp,
        color='gray',
        linestyle='--',
        linewidth=2,
        alpha=0.7,
        label=f'State-Based Baseline ({baseline_pp:.3f})'
    )

    # Title
    ax1.set_title(
        'Ablation Study: Boundary Crossing Penalty vs Compactness Trade-off',
        fontsize=14,
        fontweight='bold',
        pad=20
    )

    # Grid
    ax1.yaxis.grid(True, linestyle=':', alpha=0.3)
    ax1.set_axisbelow(True)

    # X-axis formatting
    ax1.set_xticks(x)
    ax1.set_xticklabels([f'{b:.1f}' if b < 10 else f'{int(b)}' for b in x])

    # Combined legend
    lines1, labels1 = ax1.get_legend_handles_labels()
    lines2, labels2 = ax2.get_legend_handles_labels()
    ax1.legend(lines1 + lines2, labels1 + labels2, loc='upper right', fontsize=10)

    # Add annotation box for key findings
    key_findings = (
        "Key Findings:\n"
        f"- Non-monotonic pattern: peak at beta=1.0 ({int(cross_state[2])} districts)\n"
        f"- All penalties produce worse compactness than state-based ({baseline_pp:.3f})\n"
        f"- beta=0: {pp_scores[0]:.3f} PP, beta=10: {pp_scores[-1]:.3f} PP"
    )

    ax1.text(
        0.02, 0.98,
        key_findings,
        transform=ax1.transAxes,
        fontsize=9,
        verticalalignment='top',
        bbox=dict(boxstyle='round', facecolor='yellow', alpha=0.3)
    )

    # Tight layout
    plt.tight_layout()

    # Save
    print(f"  Saving to {output_file}...")
    plt.savefig(output_file, dpi=dpi, bbox_inches='tight')
    plt.close()

    print(f"  [OK] Figure saved ({output_file.stat().st_size / 1e6:.1f} MB)")


def main():
    parser = argparse.ArgumentParser(description="Generate Figure 4 - Ablation Study")
    parser.add_argument('--year', type=int, default=2020, help='Census year')
    parser.add_argument('--input-dir', type=Path, default=Path('outputs/experimental'), help='Input directory')
    parser.add_argument('--output-dir', type=Path, default=Path('research/13+national-redistricting/figures'), help='Output directory')
    parser.add_argument('--dpi', type=int, default=300, help='Figure DPI')
    args = parser.parse_args()

    # Create output directory
    args.output_dir.mkdir(parents=True, exist_ok=True)

    print("="*70)
    print("Figure 4: Ablation Study (Paper #13)")
    print("="*70)

    # Load data
    ablation_df = load_ablation_data(args.input_dir, args.year)
    baseline_pp = load_baseline(args.input_dir, args.year)

    # Generate figure
    output_file = args.output_dir / f"figure4_ablation_{args.year}.png"
    generate_ablation_figure(
        ablation_df,
        baseline_pp,
        output_file,
        args.dpi
    )

    print("\n" + "="*70)
    print("Figure 4 complete!")
    print("="*70)
    print(f"Output: {output_file}")


if __name__ == '__main__':
    main()
