#!/usr/bin/env python3
"""
Compare Partisan Metrics: Enacted vs Algorithmic Districts

Generates:
- Side-by-side comparison tables (LaTeX)
- Summary statistics (improvement metrics)
- Visualization data

Usage:
    python scripts/political/compare_partisan_metrics.py
"""

import json
import csv
import numpy as np
from pathlib import Path
from typing import Dict, List, Tuple

def load_metrics(json_file: Path) -> Dict[str, Dict]:
    """Load partisan metrics from JSON file"""
    with open(json_file, 'r') as f:
        return json.load(f)

def compute_improvement_stats(enacted: Dict, algorithmic: Dict) -> Dict:
    """
    Compute improvement statistics

    Returns:
        {
            'efficiency_gap_reduction': mean abs(EG) reduction,
            'states_improved': count of states with lower |EG|,
            'median_improvement': median |EG| reduction,
            ...
        }
    """
    eg_reductions = []
    mmd_reductions = []
    bias_reductions = []

    states_improved_eg = 0
    states_improved_mmd = 0
    states_improved_bias = 0

    for state in enacted.keys():
        if state not in algorithmic:
            continue

        # Efficiency gap reduction (lower absolute value = better)
        enacted_eg = abs(enacted[state]['efficiency_gap'])
        algo_eg = abs(algorithmic[state]['efficiency_gap'])
        eg_reduction = enacted_eg - algo_eg
        eg_reductions.append(eg_reduction)
        if eg_reduction > 0:
            states_improved_eg += 1

        # Mean-median difference reduction
        enacted_mmd = abs(enacted[state]['mean_median_difference'])
        algo_mmd = abs(algorithmic[state]['mean_median_difference'])
        mmd_reduction = enacted_mmd - algo_mmd
        mmd_reductions.append(mmd_reduction)
        if mmd_reduction > 0:
            states_improved_mmd += 1

        # Partisan bias reduction
        enacted_bias = abs(enacted[state]['partisan_bias'])
        algo_bias = abs(algorithmic[state]['partisan_bias'])
        bias_reduction = enacted_bias - algo_bias
        bias_reductions.append(bias_reduction)
        if bias_reduction > 0:
            states_improved_bias += 1

    return {
        'efficiency_gap': {
            'mean_reduction': np.mean(eg_reductions),
            'median_reduction': np.median(eg_reductions),
            'states_improved': states_improved_eg,
            'total_states': len(eg_reductions),
            'pct_improved': states_improved_eg / len(eg_reductions) * 100,
        },
        'mean_median_difference': {
            'mean_reduction': np.mean(mmd_reductions),
            'median_reduction': np.median(mmd_reductions),
            'states_improved': states_improved_mmd,
            'total_states': len(mmd_reductions),
            'pct_improved': states_improved_mmd / len(mmd_reductions) * 100,
        },
        'partisan_bias': {
            'mean_reduction': np.mean(bias_reductions),
            'median_reduction': np.median(bias_reductions),
            'states_improved': states_improved_bias,
            'total_states': len(bias_reductions),
            'pct_improved': states_improved_bias / len(bias_reductions) * 100,
        }
    }

def generate_latex_table(enacted: Dict, algorithmic: Dict, output_dir: Path):
    """Generate LaTeX comparison table"""

    # Select representative states (competitive swing states + examples)
    selected_states = [
        'arizona', 'georgia', 'michigan', 'north_carolina', 'pennsylvania',
        'wisconsin', 'florida', 'texas', 'california', 'new_york'
    ]

    latex_content = r"""\begin{table}[t]
\caption{Partisan Metrics Comparison: Enacted vs Algorithmic Districts (2020)}
\label{tab:partisan-comparison}
\centering
\small
\begin{tabular}{@{}lrrrrrr@{}}
\toprule
& \multicolumn{2}{c}{\textbf{Efficiency Gap}} & \multicolumn{2}{c}{\textbf{Mean-Median Diff}} & \multicolumn{2}{c}{\textbf{Partisan Bias}} \\
\cmidrule(lr){2-3} \cmidrule(lr){4-5} \cmidrule(lr){6-7}
\textbf{State} & Enacted & Algo & Enacted & Algo & Enacted & Algo \\
\midrule
"""

    for state in selected_states:
        if state not in enacted or state not in algorithmic:
            continue

        state_name = state.replace('_', ' ').title()

        enacted_eg = enacted[state]['efficiency_gap']
        algo_eg = algorithmic[state]['efficiency_gap']

        enacted_mmd = enacted[state]['mean_median_difference']
        algo_mmd = algorithmic[state]['mean_median_difference']

        enacted_bias = enacted[state]['partisan_bias']
        algo_bias = algorithmic[state]['partisan_bias']

        # Mark improvement (lower absolute value) with bold
        eg_improved = abs(algo_eg) < abs(enacted_eg)
        mmd_improved = abs(algo_mmd) < abs(enacted_mmd)
        bias_improved = abs(algo_bias) < abs(enacted_bias)

        eg_enacted_str = f"{enacted_eg:+.3f}"
        eg_algo_str = f"\\textbf{{{algo_eg:+.3f}}}" if eg_improved else f"{algo_eg:+.3f}"

        mmd_enacted_str = f"{enacted_mmd:+.3f}"
        mmd_algo_str = f"\\textbf{{{algo_mmd:+.3f}}}" if mmd_improved else f"{algo_mmd:+.3f}"

        bias_enacted_str = f"{enacted_bias:+.3f}"
        bias_algo_str = f"\\textbf{{{algo_bias:+.3f}}}" if bias_improved else f"{algo_bias:+.3f}"

        latex_content += f"{state_name} & {eg_enacted_str} & {eg_algo_str} & "
        latex_content += f"{mmd_enacted_str} & {mmd_algo_str} & "
        latex_content += f"{bias_enacted_str} & {bias_algo_str} \\\\\n"

    latex_content += r"""\bottomrule
\end{tabular}
\vspace{0.5em}

\footnotesize
\textit{Note:} Bold values indicate improvement (lower absolute value).
Efficiency gap: $(W_D - W_R) / V_{total}$ where $W$ = wasted votes.
Mean-median difference: $\text{median}(D_\%) - \text{mean}(D_\%)$.
Partisan bias: Expected seat advantage at 50\% vote share.
Negative values favor Republicans, positive favor Democrats.
\end{table}
"""

    output_file = output_dir / 'partisan_comparison_table.tex'
    with open(output_file, 'w') as f:
        f.write(latex_content)

    return output_file

def generate_summary_table(stats: Dict, output_dir: Path):
    """Generate LaTeX summary statistics table"""

    latex_content = r"""\begin{table}[t]
\caption{Partisan Fairness Improvement: Algorithmic vs Enacted Districts}
\label{tab:partisan-improvement}
\centering
\begin{tabular}{@{}lrrr@{}}
\toprule
\textbf{Metric} & \textbf{Mean $\Delta$} & \textbf{States Improved} & \textbf{\% Improved} \\
\midrule
"""

    eg = stats['efficiency_gap']
    mmd = stats['mean_median_difference']
    bias = stats['partisan_bias']

    latex_content += f"Efficiency Gap & {eg['mean_reduction']:+.3f} & "
    latex_content += f"{eg['states_improved']}/{eg['total_states']} & "
    latex_content += f"{eg['pct_improved']:.1f}\\% \\\\\n"

    latex_content += f"Mean-Median Diff & {mmd['mean_reduction']:+.3f} & "
    latex_content += f"{mmd['states_improved']}/{mmd['total_states']} & "
    latex_content += f"{mmd['pct_improved']:.1f}\\% \\\\\n"

    latex_content += f"Partisan Bias & {bias['mean_reduction']:+.3f} & "
    latex_content += f"{bias['states_improved']}/{bias['total_states']} & "
    latex_content += f"{bias['pct_improved']:.1f}\\% \\\\\n"

    latex_content += r"""\bottomrule
\end{tabular}
\vspace{0.5em}

\footnotesize
\textit{Note:} Mean $\Delta$ = average reduction in absolute value (positive = improvement).
``States Improved'' = count where algorithmic districts have lower bias than enacted.
\end{table}
"""

    output_file = output_dir / 'partisan_improvement_summary.tex'
    with open(output_file, 'w') as f:
        f.write(latex_content)

    return output_file

def main():
    # Paths
    metrics_dir = Path('outputs/data/2020/partisan_metrics')
    output_dir = Path('research/slice-edge-weighted-bisection/tables')
    output_dir.mkdir(parents=True, exist_ok=True)

    enacted_file = metrics_dir / 'partisan_metrics_2020_enacted.json'
    algorithmic_file = metrics_dir / 'partisan_metrics_2020_algorithmic.json'

    print("=" * 70)
    print("Compare Partisan Metrics: Enacted vs Algorithmic")
    print("=" * 70)
    print()

    # Load data
    print(f"Loading enacted metrics: {enacted_file}")
    enacted = load_metrics(enacted_file)
    print(f"[OK] Loaded {len(enacted)} states")

    print(f"Loading algorithmic metrics: {algorithmic_file}")
    algorithmic = load_metrics(algorithmic_file)
    print(f"[OK] Loaded {len(algorithmic)} states")

    # Compute improvement statistics
    print("\nComputing improvement statistics...")
    stats = compute_improvement_stats(enacted, algorithmic)

    print("\nImprovement Summary:")
    print(f"  Efficiency Gap:")
    print(f"    Mean reduction:    {stats['efficiency_gap']['mean_reduction']:+.4f}")
    print(f"    Median reduction:  {stats['efficiency_gap']['median_reduction']:+.4f}")
    print(f"    States improved:   {stats['efficiency_gap']['states_improved']}/{stats['efficiency_gap']['total_states']} ({stats['efficiency_gap']['pct_improved']:.1f}%)")

    print(f"\n  Mean-Median Difference:")
    print(f"    Mean reduction:    {stats['mean_median_difference']['mean_reduction']:+.4f}")
    print(f"    Median reduction:  {stats['mean_median_difference']['median_reduction']:+.4f}")
    print(f"    States improved:   {stats['mean_median_difference']['states_improved']}/{stats['mean_median_difference']['total_states']} ({stats['mean_median_difference']['pct_improved']:.1f}%)")

    print(f"\n  Partisan Bias:")
    print(f"    Mean reduction:    {stats['partisan_bias']['mean_reduction']:+.4f}")
    print(f"    Median reduction:  {stats['partisan_bias']['median_reduction']:+.4f}")
    print(f"    States improved:   {stats['partisan_bias']['states_improved']}/{stats['partisan_bias']['total_states']} ({stats['partisan_bias']['pct_improved']:.1f}%)")

    # Generate LaTeX tables
    print("\n\nGenerating LaTeX tables...")
    comparison_table = generate_latex_table(enacted, algorithmic, output_dir)
    print(f"[OK] Comparison table: {comparison_table}")

    summary_table = generate_summary_table(stats, output_dir)
    print(f"[OK] Summary table: {summary_table}")

    # Save raw comparison as CSV
    csv_file = output_dir / 'partisan_metrics_comparison.csv'
    with open(csv_file, 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow([
            'state',
            'enacted_eg', 'algo_eg', 'eg_delta',
            'enacted_mmd', 'algo_mmd', 'mmd_delta',
            'enacted_bias', 'algo_bias', 'bias_delta'
        ])

        for state in sorted(enacted.keys()):
            if state not in algorithmic:
                continue

            enacted_eg = enacted[state]['efficiency_gap']
            algo_eg = algorithmic[state]['efficiency_gap']
            eg_delta = abs(enacted_eg) - abs(algo_eg)

            enacted_mmd = enacted[state]['mean_median_difference']
            algo_mmd = algorithmic[state]['mean_median_difference']
            mmd_delta = abs(enacted_mmd) - abs(algo_mmd)

            enacted_bias = enacted[state]['partisan_bias']
            algo_bias = algorithmic[state]['partisan_bias']
            bias_delta = abs(enacted_bias) - abs(algo_bias)

            writer.writerow([
                state,
                f"{enacted_eg:.4f}", f"{algo_eg:.4f}", f"{eg_delta:.4f}",
                f"{enacted_mmd:.4f}", f"{algo_mmd:.4f}", f"{mmd_delta:.4f}",
                f"{enacted_bias:.4f}", f"{algo_bias:.4f}", f"{bias_delta:.4f}"
            ])

    print(f"[OK] CSV comparison: {csv_file}")

    print("\n" + "=" * 70)
    print("SUCCESS - Comparison tables generated")
    print("=" * 70)
    print()
    print("Next step: Add results to paper")
    print("  1. Copy LaTeX tables to results section")
    print("  2. Write analysis of findings")
    print("  3. Discuss implications for redistricting")

if __name__ == '__main__':
    main()
