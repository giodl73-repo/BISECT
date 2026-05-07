#!/usr/bin/env python3
"""
Generate all figures for synthesis metapaper (Science submission).

Figures:
1. Research Program Architecture - Directed graph of 10-paper portfolio
2. National Redistricting Results - Map of all 435 districts (2020)
3. VRA Compliance Analysis - 3-panel chart (bar, scatter, Pareto)
4. Temporal Stability - 3-panel comparison (2010 → 2020)

Usage:
    python generate_figures.py --year 2020 --version v1
"""

import argparse
import sys
from pathlib import Path
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
from matplotlib.gridspec import GridSpec
import numpy as np

# Add src to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent / "src"))

def generate_portfolio_architecture():
    """Generate Figure 1: Research Program Architecture."""
    print("\nGenerating Figure 1: Research Program Architecture...")

    fig = plt.figure(figsize=(14, 10))
    ax = fig.add_subplot(111)
    ax.axis('off')

    # Define paper positions (x, y) in normalized coordinates
    papers = {
        'P1': {'pos': (2, 9), 'title': 'Recursive\nBisection', 'finding': 'Baseline Method', 'color': '#e1f5ff'},
        'P2': {'pos': (4, 9), 'title': 'Edge-Weighted\nBisection', 'finding': '56% Compactness\nImprovement', 'color': '#e1f5ff'},
        'P3': {'pos': (2, 7), 'title': 'VRA\nCompliance', 'finding': '+69 MM Districts\n(+101%)', 'color': '#e8f5e9'},
        'P4': {'pos': (4, 7), 'title': 'Threshold\nAnalysis', 'finding': '42% Demographic\nThreshold', 'color': '#e8f5e9'},
        'P5': {'pos': (6, 7), 'title': 'Cross-Census\nValidation', 'finding': '80% Tract\nRetention', 'color': '#e8f5e9'},
        'P6': {'pos': (2, 5), 'title': 'Multi-Constraint\nConflicts', 'finding': 'Pareto Frontiers', 'color': '#fff3e0'},
        'P7': {'pos': (4, 5), 'title': 'Adaptive\nParameters', 'finding': 'Robustness\nValidated', 'color': '#fff3e0'},
        'P8': {'pos': (2, 3), 'title': 'N-Way vs\nRecursive', 'finding': 'Method\nEquivalence', 'color': '#f3e5f5'},
        'P9': {'pos': (4, 3), 'title': 'Temporal\nStability', 'finding': '14pt Hierarchical\nAdvantage', 'color': '#f3e5f5'},
        'P10': {'pos': (6, 3), 'title': 'Compactness\nTradeoffs', 'finding': 'Policy Guidance', 'color': '#f3e5f5'},
        'P11': {'pos': (4, 1), 'title': 'SYNTHESIS\nMETAPAPER', 'finding': 'Science/Nature\nParadigm Shift', 'color': '#ffebee'}
    }

    # Draw papers as boxes
    box_width = 1.2
    box_height = 1.0

    for paper_id, paper in papers.items():
        x, y = paper['pos']
        rect = mpatches.FancyBboxPatch(
            (x - box_width/2, y - box_height/2),
            box_width, box_height,
            boxstyle="round,pad=0.05",
            facecolor=paper['color'],
            edgecolor='black' if paper_id != 'P11' else 'darkred',
            linewidth=2 if paper_id != 'P11' else 3
        )
        ax.add_patch(rect)

        # Paper number
        ax.text(x, y + 0.25, paper_id, ha='center', va='center',
                fontsize=10, fontweight='bold')
        # Title
        ax.text(x, y, paper['title'], ha='center', va='center',
                fontsize=8, fontweight='bold')
        # Finding
        ax.text(x, y - 0.25, paper['finding'], ha='center', va='center',
                fontsize=7, style='italic')

    # Draw arrows (dependencies)
    arrows = [
        ('P1', 'P2'), ('P1', 'P3'), ('P1', 'P8'),
        ('P2', 'P3'), ('P2', 'P4'), ('P2', 'P5'), ('P2', 'P6'), ('P2', 'P7'), ('P2', 'P10'),
        ('P3', 'P4'), ('P3', 'P6'), ('P3', 'P10'),
        ('P4', 'P6'), ('P5', 'P9'), ('P6', 'P10'), ('P8', 'P9'),
        # All to synthesis
        ('P1', 'P11'), ('P2', 'P11'), ('P3', 'P11'), ('P4', 'P11'), ('P5', 'P11'),
        ('P6', 'P11'), ('P7', 'P11'), ('P8', 'P11'), ('P9', 'P11'), ('P10', 'P11')
    ]

    for start, end in arrows:
        x1, y1 = papers[start]['pos']
        x2, y2 = papers[end]['pos']

        # Adjust arrow start/end to box edges
        dy = y2 - y1
        dx = x2 - x1
        if abs(dy) > abs(dx):  # Vertical arrow
            y1 = y1 - box_height/2 if dy < 0 else y1 + box_height/2
            y2 = y2 + box_height/2 if dy < 0 else y2 - box_height/2
        else:  # Horizontal arrow
            x1 = x1 + box_width/2 if dx > 0 else x1 - box_width/2
            x2 = x2 - box_width/2 if dx > 0 else x2 + box_width/2

        ax.annotate('', xy=(x2, y2), xytext=(x1, y1),
                   arrowprops=dict(arrowstyle='->', lw=1.5, color='gray', alpha=0.7))

    # Legend
    legend_elements = [
        mpatches.Patch(facecolor='#e1f5ff', edgecolor='black', label='Foundation'),
        mpatches.Patch(facecolor='#e8f5e9', edgecolor='black', label='Empirical'),
        mpatches.Patch(facecolor='#fff3e0', edgecolor='black', label='Technical'),
        mpatches.Patch(facecolor='#f3e5f5', edgecolor='black', label='Comparison'),
        mpatches.Patch(facecolor='#ffebee', edgecolor='darkred', label='Synthesis')
    ]
    ax.legend(handles=legend_elements, loc='upper right', fontsize=10)

    # Title
    ax.text(4, 10.5, 'Research Program Architecture\n10-Paper Portfolio + Synthesis',
            ha='center', va='center', fontsize=14, fontweight='bold')

    ax.set_xlim(0, 8)
    ax.set_ylim(0, 11)

    output_path = Path(__file__).parent / 'figure1_portfolio_architecture.png'
    plt.savefig(output_path, dpi=300, bbox_inches='tight')
    print(f"[OK] Saved to {output_path}")
    plt.close()


def generate_national_map(year, version):
    """Generate Figure 2: National Redistricting Results."""
    print(f"\nGenerating Figure 2: National Redistricting Results ({year})...")

    # Check if we have the national map from pipeline
    pipeline_output = Path(f"outputs/v{version}/{year}/nation/maps/all_districts_national.png")

    if pipeline_output.exists():
        print(f"[OK] Using pipeline output: {pipeline_output}")
        import shutil
        output_path = Path(__file__).parent / 'figure2_national_map.png'
        shutil.copy(pipeline_output, output_path)
        print(f"[OK] Copied to {output_path}")
    else:
        print(f"[WARNING] Pipeline output not found: {pipeline_output}")
        print("Run: /run-redistricting --year {year} --version {version}")

        # Create placeholder
        fig, ax = plt.subplots(figsize=(16, 10))
        ax.text(0.5, 0.5, f'Figure 2: National Redistricting Results\n\n'
                           f'Pipeline output required:\n{pipeline_output}\n\n'
                           f'Run: /run-redistricting --year {year} --version {version}',
                ha='center', va='center', fontsize=12)
        ax.axis('off')
        output_path = Path(__file__).parent / 'figure2_national_map_placeholder.png'
        plt.savefig(output_path, dpi=300, bbox_inches='tight')
        print(f"[OK] Created placeholder: {output_path}")
        plt.close()


def generate_vra_analysis():
    """Generate Figure 3: VRA Compliance Analysis (3 panels)."""
    print("\nGenerating Figure 3: VRA Compliance Analysis...")

    fig = plt.figure(figsize=(16, 5))
    gs = GridSpec(1, 3, figure=fig, wspace=0.3)

    # Panel A: Enacted vs Algorithmic MM Districts by State (bar chart)
    ax1 = fig.add_subplot(gs[0])

    # Example data (replace with actual data from pipeline)
    states = ['AL', 'GA', 'NC', 'SC', 'LA', 'MS', 'TX', 'FL']
    enacted = [1, 5, 3, 1, 2, 2, 8, 5]
    algorithmic = [2, 6, 4, 2, 3, 2, 9, 5]

    x = np.arange(len(states))
    width = 0.35

    ax1.bar(x - width/2, enacted, width, label='Enacted', color='#e57373')
    ax1.bar(x + width/2, algorithmic, width, label='Algorithmic', color='#81c784')

    ax1.set_xlabel('State', fontweight='bold')
    ax1.set_ylabel('Majority-Minority Districts', fontweight='bold')
    ax1.set_title('Panel A: Enacted vs Algorithmic MM Districts', fontweight='bold')
    ax1.set_xticks(x)
    ax1.set_xticklabels(states)
    ax1.legend()
    ax1.grid(axis='y', alpha=0.3)

    # Panel B: 42% Threshold Scatter Plot
    ax2 = fig.add_subplot(gs[1])

    # Example data (state minority % vs MM success rate)
    minority_pct = [25, 30, 35, 40, 42, 45, 50, 55, 60]
    mm_success = [0.1, 0.2, 0.3, 0.5, 0.75, 0.85, 0.9, 0.95, 0.98]

    ax2.scatter(minority_pct, mm_success, s=100, alpha=0.6, color='#64b5f6')
    ax2.axvline(x=42, color='red', linestyle='--', linewidth=2, label='42% Threshold')
    ax2.axhline(y=0.75, color='orange', linestyle=':', linewidth=1.5, alpha=0.7)

    ax2.set_xlabel('State Minority Population (%)', fontweight='bold')
    ax2.set_ylabel('MM District Success Rate', fontweight='bold')
    ax2.set_title('Panel B: 42% Feasibility Threshold', fontweight='bold')
    ax2.legend()
    ax2.grid(alpha=0.3)

    # Panel C: Pareto Frontier (Alabama example)
    ax3 = fig.add_subplot(gs[2])

    # Example Pareto frontier data
    compactness = [0.50, 0.55, 0.60, 0.65, 0.68, 0.70, 0.72, 0.75]
    mm_districts = [2.0, 1.9, 1.8, 1.6, 1.4, 1.2, 1.0, 0.8]

    ax3.plot(compactness, mm_districts, 'o-', linewidth=2, markersize=8, color='#ab47bc')
    ax3.axhline(y=2, color='green', linestyle='--', linewidth=2, label='Target: 2 MM Districts')
    ax3.axhline(y=1, color='red', linestyle='--', linewidth=2, alpha=0.5, label='Enacted: 1 MM District')

    ax3.set_xlabel('Compactness Score', fontweight='bold')
    ax3.set_ylabel('MM Districts', fontweight='bold')
    ax3.set_title('Panel C: VRA-Compactness Pareto Frontier (Alabama)', fontweight='bold')
    ax3.legend()
    ax3.grid(alpha=0.3)

    output_path = Path(__file__).parent / 'figure3_vra_analysis.png'
    plt.savefig(output_path, dpi=300, bbox_inches='tight')
    print(f"[OK] Saved to {output_path}")
    plt.close()


def generate_temporal_stability():
    """Generate Figure 4: Temporal Stability (3 panels)."""
    print("\nGenerating Figure 4: Temporal Stability...")

    fig = plt.figure(figsize=(16, 5))
    gs = GridSpec(1, 3, figure=fig, wspace=0.3)

    # Panel A: Tract Retention Rates
    ax1 = fig.add_subplot(gs[0])

    methods = ['Recursive\nBisection', 'N-Way\nPartitioning']
    retention_2010_2020 = [80, 66]
    retention_2000_2010 = [78, 64]

    x = np.arange(len(methods))
    width = 0.35

    ax1.bar(x - width/2, retention_2010_2020, width, label='2010→2020', color='#4fc3f7')
    ax1.bar(x + width/2, retention_2000_2010, width, label='2000→2010', color='#ba68c8')

    ax1.set_ylabel('Tract Retention Rate (%)', fontweight='bold')
    ax1.set_title('Panel A: Tract Retention Rates', fontweight='bold')
    ax1.set_xticks(x)
    ax1.set_xticklabels(methods)
    ax1.legend()
    ax1.grid(axis='y', alpha=0.3)
    ax1.set_ylim(0, 100)

    # Add 14pt advantage annotation
    ax1.annotate('14pt\nadvantage', xy=(0, 80), xytext=(0.5, 85),
                arrowprops=dict(arrowstyle='->', color='red', lw=2),
                fontsize=10, fontweight='bold', color='red', ha='center')

    # Panel B: Geographic vs Temporal Variance
    ax2 = fig.add_subplot(gs[1])

    variance_types = ['Geographic\nVariance', 'Temporal\nVariance']
    variance_values = [0.32, 0.10]
    colors = ['#ff8a65', '#aed581']

    ax2.bar(variance_types, variance_values, color=colors, edgecolor='black', linewidth=1.5)
    ax2.set_ylabel('Variance', fontweight='bold')
    ax2.set_title('Panel B: Geographic vs Temporal Variance', fontweight='bold')
    ax2.grid(axis='y', alpha=0.3)

    # Add 3.2x annotation
    ax2.annotate('3.2× larger', xy=(0, 0.32), xytext=(0.5, 0.35),
                arrowprops=dict(arrowstyle='->', color='red', lw=2),
                fontsize=10, fontweight='bold', color='red', ha='center')

    # Panel C: District Boundary Overlay (placeholder - requires actual map data)
    ax3 = fig.add_subplot(gs[2])

    ax3.text(0.5, 0.5, 'Panel C: District Boundary Overlay\n\n'
                       '2010 (blue) / 2020 (red) / Overlap (purple)\n\n'
                       'Requires state map data from pipeline',
             ha='center', va='center', fontsize=10)
    ax3.set_title('Panel C: Boundary Stability Example', fontweight='bold')
    ax3.axis('off')

    output_path = Path(__file__).parent / 'figure4_temporal_stability.png'
    plt.savefig(output_path, dpi=300, bbox_inches='tight')
    print(f"[OK] Saved to {output_path}")
    plt.close()


def main():
    parser = argparse.ArgumentParser(description='Generate figures for synthesis metapaper')
    parser.add_argument('--year', type=int, default=2020, choices=[2000, 2010, 2020],
                       help='Census year (default: 2020)')
    parser.add_argument('--version', type=str, default='v1',
                       help='Pipeline version (default: v1)')
    parser.add_argument('--figures', type=str, default='all',
                       choices=['all', '1', '2', '3', '4'],
                       help='Which figures to generate (default: all)')
    args = parser.parse_args()

    print("=" * 70)
    print("Synthesis Metapaper - Figure Generation")
    print("=" * 70)
    print(f"Year: {args.year}")
    print(f"Version: {args.version}")
    print(f"Figures: {args.figures}")

    # Create output directory
    output_dir = Path(__file__).parent
    output_dir.mkdir(parents=True, exist_ok=True)

    # Generate requested figures
    if args.figures in ['all', '1']:
        generate_portfolio_architecture()

    if args.figures in ['all', '2']:
        generate_national_map(args.year, args.version)

    if args.figures in ['all', '3']:
        generate_vra_analysis()

    if args.figures in ['all', '4']:
        generate_temporal_stability()

    print("\n" + "=" * 70)
    print("Figure generation complete!")
    print("=" * 70)
    print(f"\nOutput directory: {output_dir}")
    print("\nNext steps:")
    print("1. Review figures for quality and accuracy")
    print("2. Update VRA/temporal data with actual pipeline outputs")
    print("3. Include in LaTeX: \\includegraphics{figures/figure1_portfolio_architecture.png}")
    print("4. Compile presentation: /compile-latex")


if __name__ == '__main__':
    main()
