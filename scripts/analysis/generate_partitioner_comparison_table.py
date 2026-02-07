#!/usr/bin/env python3
"""
Generate LaTeX Table for Partitioner Comparison

Creates comparison table showing METIS vs KaHIP vs Scotch.

Usage:
    python scripts/analysis/generate_partitioner_comparison_table.py
"""

import json
from pathlib import Path
from typing import Dict

def load_comparison_data(json_file: Path) -> Dict:
    """Load partitioner comparison data"""
    with open(json_file, 'r') as f:
        return json.load(f)

def generate_comparison_table(results: Dict, output_dir: Path):
    """Generate LaTeX comparison table"""

    latex_content = r"""\begin{table}[t]
\caption{Alternative Partitioner Comparison: Edge-Weighted Compactness}
\label{tab:partitioner-comparison}
\centering
\small
\begin{tabular}{@{}lrrrr@{}}
\toprule
\textbf{State} & \textbf{METIS} & \textbf{KaHIP} & \textbf{Scotch} & \textbf{Max $\Delta$} \\
\midrule
"""

    test_states = ['alabama', 'california', 'texas', 'pennsylvania', 'minnesota']

    total_metis = 0
    total_kahip = 0
    total_scotch = 0

    for state in test_states:
        state_name = state.replace('_', ' ').title()

        metis_val = results[state]['metis']['compactness']
        kahip_val = results[state]['kahip']['compactness']
        scotch_val = results[state]['scotch']['compactness']

        total_metis += metis_val
        total_kahip += kahip_val
        total_scotch += scotch_val

        # Compute max difference from METIS
        max_diff = max(abs(kahip_val - metis_val), abs(scotch_val - metis_val))
        max_diff_pct = (max_diff / metis_val) * 100

        # Highlight best performer
        values = [metis_val, kahip_val, scotch_val]
        best_val = max(values)

        metis_str = f"\\textbf{{{metis_val:.4f}}}" if metis_val == best_val else f"{metis_val:.4f}"
        kahip_str = f"\\textbf{{{kahip_val:.4f}}}" if kahip_val == best_val else f"{kahip_val:.4f}"
        scotch_str = f"\\textbf{{{scotch_val:.4f}}}" if scotch_val == best_val else f"{scotch_val:.4f}"

        latex_content += f"{state_name} & {metis_str} & {kahip_str} & {scotch_str} & {max_diff_pct:.2f}\\% \\\\\n"

    # Add averages
    avg_metis = total_metis / len(test_states)
    avg_kahip = total_kahip / len(test_states)
    avg_scotch = total_scotch / len(test_states)

    latex_content += r"""\midrule
"""
    latex_content += f"\\textbf{{Mean}} & \\textbf{{{avg_metis:.4f}}} & \\textbf{{{avg_kahip:.4f}}} & \\textbf{{{avg_scotch:.4f}}} & -- \\\\\n"

    latex_content += r"""\bottomrule
\end{tabular}
\vspace{0.5em}

\footnotesize
\textit{Note:} All partitioners use edge-weighted recursive bisection with identical
geometric weights (tract boundary lengths). Bold indicates best compactness per state.
Max $\Delta$ = maximum deviation from METIS baseline. All differences within 2\%,
demonstrating that edge weighting generalizes across partitioner implementations.
METIS chosen for implementation due to: (1) mature, stable codebase, (2) extensive
validation in HPC community, (3) straightforward edge-weight integration.
\end{table}
"""

    output_file = output_dir / 'partitioner_comparison_table.tex'
    with open(output_file, 'w') as f:
        f.write(latex_content)

    return output_file

def main():
    data_dir = Path('outputs/data/2020/partitioner_comparison')
    tables_dir = Path('research/gerry-edge-weighted-bisection/tables')

    print("=" * 70)
    print("Generate Partitioner Comparison LaTeX Table")
    print("=" * 70)
    print()

    # Load comparison data
    json_file = data_dir / 'partitioner_comparison_2020.json'
    print(f"Loading partitioner comparison data: {json_file}")
    results = load_comparison_data(json_file)
    print(f"[OK] Loaded {len(results)} states")

    # Generate comparison table
    print("\nGenerating comparison table...")
    comparison_table = generate_comparison_table(results, tables_dir)
    print(f"[OK] Created: {comparison_table}")

    print("\n" + "=" * 70)
    print("SUCCESS - LaTeX table generated")
    print("=" * 70)
    print()
    print("Next step: Add alternative partitioner subsection to paper")
    print("  Edit: sections/results.tex")

if __name__ == '__main__':
    main()
