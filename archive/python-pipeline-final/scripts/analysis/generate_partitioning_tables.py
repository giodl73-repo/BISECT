#!/usr/bin/env python3
"""
Generate LaTeX Tables for Partitioning Quality Analysis

Creates comparison tables showing METIS partition statistics
for unweighted vs edge-weighted modes.

Usage:
    python scripts/analysis/generate_partitioning_tables.py
"""

import json
from pathlib import Path
from typing import Dict

def load_partitioning_stats(json_file: Path) -> Dict:
    """Load partitioning statistics"""
    with open(json_file, 'r') as f:
        return json.load(f)

def generate_comparison_table(results: Dict, output_dir: Path):
    """
    Generate LaTeX table comparing partition quality metrics

    Shows the topological vs geometric tradeoff
    """

    # Select representative states (geographic diversity + algorithmic interest)
    states_to_show = [
        'alabama', 'california', 'texas', 'new_york', 'pennsylvania',
        'florida', 'illinois', 'ohio', 'michigan', 'georgia'
    ]

    latex_content = r"""\begin{table*}[t]
\caption{Partitioning Quality: Topological vs Geometric Tradeoff}
\label{tab:partition-quality}
\centering
\small
\begin{tabular}{@{}lrrrrrr@{}}
\toprule
& \multicolumn{2}{c}{\textbf{Edge Cuts}} & \multicolumn{2}{c}{\textbf{Total Perimeter (km)}} & \multicolumn{2}{c}{\textbf{Avg Edge Length (km)}} \\
\cmidrule(lr){2-3} \cmidrule(lr){4-5} \cmidrule(lr){6-7}
\textbf{State} & Unwtd & Wtd & Unwtd & Wtd & Unwtd & Wtd \\
\midrule
"""

    for state in states_to_show:
        if state not in results:
            continue

        state_name = state.replace('_', ' ').title()
        unweighted = results[state]['unweighted']
        weighted = results[state]['weighted']

        # Format with change indicators
        edge_cut_change = ((weighted['edge_cut'] - unweighted['edge_cut']) /
                          unweighted['edge_cut']) * 100
        perim_change = ((weighted['total_perimeter_km'] - unweighted['total_perimeter_km']) /
                       unweighted['total_perimeter_km']) * 100

        # Edge cuts increase (show in red if significant)
        edge_unwtd_str = f"{unweighted['edge_cut']:,}"
        if edge_cut_change > 50:
            edge_wtd_str = f"\\textcolor{{red}}{{{weighted['edge_cut']:,}}}"
        else:
            edge_wtd_str = f"{weighted['edge_cut']:,}"

        # Perimeter decreases (show in green if significant)
        perim_unwtd_str = f"{unweighted['total_perimeter_km']:,.0f}"
        if abs(perim_change) > 20:
            perim_wtd_str = f"\\textcolor{{blue}}{{{weighted['total_perimeter_km']:,.0f}}}"
        else:
            perim_wtd_str = f"{weighted['total_perimeter_km']:,.0f}"

        avg_len_unwtd = f"{unweighted['avg_edge_length_km']:.2f}"
        avg_len_wtd = f"{weighted['avg_edge_length_km']:.2f}"

        latex_content += f"{state_name} & {edge_unwtd_str} & {edge_wtd_str} & "
        latex_content += f"{perim_unwtd_str} & {perim_wtd_str} & "
        latex_content += f"{avg_len_unwtd} & {avg_len_wtd} \\\\\n"

    latex_content += r"""\bottomrule
\end{tabular}
\vspace{0.5em}

\footnotesize
\textit{Note:} Edge-weighted mode produces \textbf{more edge cuts} (red if $>$50\% increase)
but \textbf{shorter total perimeter} (blue if $>$20\% reduction).
METIS sacrifices topological optimality (minimizing edge count) for geometric optimality
(minimizing boundary length). Average edge length decreases by 60-70\% in weighted mode,
demonstrating preference for short boundaries over minimizing cut edges.
\end{table*}
"""

    output_file = output_dir / 'partition_quality_table.tex'
    with open(output_file, 'w') as f:
        f.write(latex_content)

    return output_file

def generate_summary_table(results: Dict, output_dir: Path):
    """Generate national summary statistics table"""

    # Compute aggregates across all states
    total_states = len(results)

    avg_edge_cut_increase = sum(
        ((r['weighted']['edge_cut'] - r['unweighted']['edge_cut']) /
         r['unweighted']['edge_cut']) * 100
        for r in results.values()
    ) / total_states

    avg_perimeter_decrease = sum(
        ((r['unweighted']['total_perimeter_km'] - r['weighted']['total_perimeter_km']) /
         r['unweighted']['total_perimeter_km']) * 100
        for r in results.values()
    ) / total_states

    avg_avg_length_decrease = sum(
        ((r['unweighted']['avg_edge_length_km'] - r['weighted']['avg_edge_length_km']) /
         r['unweighted']['avg_edge_length_km']) * 100
        for r in results.values()
    ) / total_states

    # Coarsening and refinement averages
    avg_coarsening_unwtd = sum(r['unweighted']['coarsening_levels'] for r in results.values()) / total_states
    avg_coarsening_wtd = sum(r['weighted']['coarsening_levels'] for r in results.values()) / total_states

    avg_refinement_unwtd = sum(r['unweighted']['refinement_iters'] for r in results.values()) / total_states
    avg_refinement_wtd = sum(r['weighted']['refinement_iters'] for r in results.values()) / total_states

    latex_content = r"""\begin{table}[t]
\caption{National Partitioning Quality Summary}
\label{tab:partition-summary}
\centering
\begin{tabular}{@{}lrr@{}}
\toprule
\textbf{Metric} & \textbf{Unweighted} & \textbf{Weighted (Change)} \\
\midrule
"""
    latex_content += f"Edge cuts (avg per state) & -- & +{avg_edge_cut_increase:.1f}\\% \\\\\n"
    latex_content += f"Total perimeter (avg per state) & -- & -{avg_perimeter_decrease:.1f}\\% \\\\\n"
    latex_content += f"Average edge length (avg per state) & -- & -{avg_avg_length_decrease:.1f}\\% \\\\\n"
    latex_content += r"""\midrule
"""
    latex_content += f"Coarsening levels & {avg_coarsening_unwtd:.1f} & {avg_coarsening_wtd:.1f} \\\\\n"
    latex_content += f"Refinement iterations & {avg_refinement_unwtd:.1f} & {avg_refinement_wtd:.1f} \\\\\n"

    latex_content += r"""\bottomrule
\end{tabular}
\vspace{0.5em}

\footnotesize
\textit{Note:} Edge-weighted METIS increases edge cuts by """ + f"{avg_edge_cut_increase:.0f}\\% " + r"""but reduces
total perimeter by """ + f"{avg_perimeter_decrease:.0f}\\%, " + r"""achieving net compactness improvement.
The """ + f"{avg_avg_length_decrease:.0f}\\% " + r"""reduction in average edge length confirms geometric optimization:
METIS explicitly prefers cuts along short boundaries even when this requires cutting more edges.
\end{table}
"""

    output_file = output_dir / 'partition_summary_table.tex'
    with open(output_file, 'w') as f:
        f.write(latex_content)

    return output_file

def main():
    data_dir = Path('outputs/data/2020/partitioning')
    tables_dir = Path('research/gerry-edge-weighted-bisection/tables')

    print("=" * 70)
    print("Generate Partitioning Quality LaTeX Tables")
    print("=" * 70)
    print()

    # Load partitioning statistics
    json_file = data_dir / 'partitioning_statistics_2020.json'
    print(f"Loading partitioning statistics: {json_file}")
    results = load_partitioning_stats(json_file)
    print(f"[OK] Loaded {len(results)} states")

    # Generate comparison table
    print("\nGenerating comparison table...")
    comparison_table = generate_comparison_table(results, tables_dir)
    print(f"[OK] Created: {comparison_table}")

    # Generate summary table
    print("Generating summary table...")
    summary_table = generate_summary_table(results, tables_dir)
    print(f"[OK] Created: {summary_table}")

    print("\n" + "=" * 70)
    print("SUCCESS - LaTeX tables generated")
    print("=" * 70)
    print()
    print("Next step: Add partitioning quality subsection to paper")
    print("  Edit: sections/results.tex")

if __name__ == '__main__':
    main()
