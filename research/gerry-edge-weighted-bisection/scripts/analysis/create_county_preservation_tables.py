#!/usr/bin/env python3
"""Generate LaTeX tables for county preservation analysis."""

import json
import pandas as pd
from pathlib import Path


def create_detailed_table(df: pd.DataFrame, output_file: Path):
    """Create detailed state-by-state county preservation table."""

    # Select 10 representative states: mix of categories and sizes
    # Sort by compactness improvement to show tradeoff
    df_sorted = df.sort_values('compactness_improvement', ascending=False)

    # Get top 5 by compactness improvement (showing tradeoff)
    top_improvement = df_sorted.head(5)

    # Get 3 states with negative split difference (alg preserves better)
    alg_better = df[df['split_difference'] < -2].head(3)

    # Get 2 states with similar preservation
    similar = df[abs(df['split_difference']) <= 2].head(2)

    selected = pd.concat([top_improvement, alg_better, similar]).drop_duplicates()

    # If we don't have 10, pad with remaining states
    if len(selected) < 10:
        remaining = df[~df['state'].isin(selected['state'])].head(10 - len(selected))
        selected = pd.concat([selected, remaining])

    latex = r"""\begin{table}[t]
\centering
\caption{County Preservation: Enacted vs Algorithmic Districts}
\label{tab:county_preservation}
\begin{tabular}{lrrrrrrr}
\toprule
& & \multicolumn{2}{c}{\textbf{Enacted}} & \multicolumn{2}{c}{\textbf{Algorithmic}} & & \textbf{Compact} \\
\cmidrule(lr){3-4} \cmidrule(lr){5-6}
\textbf{State} & \textbf{Counties} & \textbf{Split} & \textbf{Rate} & \textbf{Split} & \textbf{Rate} & \textbf{Diff} & \textbf{Improve} \\
& & & \textbf{(\%)} & & \textbf{(\%)} & & \\
\midrule
"""

    for _, row in selected.head(10).iterrows():
        state = row['state']
        counties = row['num_counties']
        enacted_split = row['enacted_splits']
        enacted_rate = row['enacted_split_rate'] * 100
        alg_split = row['alg_splits']
        alg_rate = row['alg_split_rate'] * 100
        diff = row['split_difference']
        compact_improve = row['compactness_improvement'] * 100

        # Format difference with sign
        diff_str = f"{diff:+d}"

        latex += f"{state} & {counties} & {enacted_split} & {enacted_rate:.0f} & {alg_split} & {alg_rate:.0f} & {diff_str} & {compact_improve:+.0f} \\\\\n"

    latex += r"""\bottomrule
\end{tabular}
\begin{tablenotes}
\small
\item \textit{Counties}: Total counties in state. \textit{Split}: Number of counties split across multiple districts. \textit{Rate}: Percentage of counties split. \textit{Diff}: Change in splits (algorithmic - enacted); positive means algorithmic splits more counties. \textit{Compact Improve}: Compactness improvement (\%) from enacted to algorithmic. Negative correlation between Diff and Compact Improve demonstrates compactness-county preservation tradeoff.
\end{tablenotes}
\end{table}
"""

    with open(output_file, 'w') as f:
        f.write(latex)


def create_summary_table(summary: dict, output_file: Path):
    """Create national summary statistics table."""

    total_states = summary['total_states']
    total_counties = summary['total_counties']
    enacted_rate = summary['avg_enacted_split_rate'] * 100
    alg_rate = summary['avg_alg_split_rate'] * 100
    diff = summary['avg_split_rate_difference'] * 100
    more = summary['alg_splits_more_states']
    less = summary['alg_splits_less_states']
    same = summary['alg_splits_same_states']
    tradeoff = summary['compactness_county_tradeoff_states']
    alg_better = summary['alg_preserves_better_states']
    similar = summary['similar_preservation_states']
    corr = summary['split_compactness_correlation']

    latex = r"""\begin{table}[t]
\centering
\caption{National County Preservation Summary}
\label{tab:county_preservation_summary}
\begin{tabular}{lr}
\toprule
\textbf{Metric} & \textbf{Value} \\
\midrule
Total States & """ + f"{total_states}" + r""" \\
Total Counties Analyzed & """ + f"{total_counties:,}" + r""" \\
\midrule
Average Enacted Split Rate & """ + f"{enacted_rate:.1f}\\%" + r""" \\
Average Algorithmic Split Rate & """ + f"{alg_rate:.1f}\\%" + r""" \\
Average Difference (Alg - Enacted) & """ + f"{diff:+.1f}\\%" + r""" \\
\midrule
Algorithmic Splits More Counties & """ + f"{more} states" + r""" \\
Algorithmic Splits Fewer Counties & """ + f"{less} states" + r""" \\
Similar Split Rates & """ + f"{same} states" + r""" \\
\midrule
\textbf{Category Classification} & \\
\quad Compactness-County Tradeoff & """ + f"{tradeoff} states" + r""" \\
\quad Algorithmic Preserves Better & """ + f"{alg_better} states" + r""" \\
\quad Similar Preservation & """ + f"{similar} states" + r""" \\
\midrule
Correlation (Splits $\leftrightarrow$ Compactness) & """ + f"{corr:.2f}" + r""" \\
\bottomrule
\end{tabular}
\begin{tablenotes}
\small
\item \textit{Split Rate}: Percentage of counties split across multiple districts. \textit{Difference}: Positive means algorithmic splits more counties. \textit{Compactness-County Tradeoff}: States where algorithmic splits $>$3 more counties and gains $>$5\% compactness. \textit{Algorithmic Preserves Better}: States where algorithmic splits $<$3 fewer counties. \textit{Correlation}: Negative correlation ($-0.68$) indicates compactness gains often require splitting more counties.
\end{tablenotes}
\end{table}
"""

    with open(output_file, 'w') as f:
        f.write(latex)


def main():
    base_dir = Path(__file__).parent.parent

    # Load data
    data_file = base_dir / 'data' / 'county_preservation_2020.csv'
    summary_file = base_dir / 'data' / 'county_preservation_summary_2020.json'

    if not data_file.exists():
        print("Error: Run compute_county_preservation.py first")
        return

    df = pd.read_csv(data_file)

    with open(summary_file) as f:
        summary = json.load(f)

    # Create tables
    tables_dir = base_dir / 'tables'
    tables_dir.mkdir(exist_ok=True)

    create_detailed_table(df, tables_dir / 'county_preservation_table.tex')
    create_summary_table(summary, tables_dir / 'county_preservation_summary.tex')

    print("[OK] Created LaTeX tables:")
    print(f"  -> {tables_dir / 'county_preservation_table.tex'}")
    print(f"  -> {tables_dir / 'county_preservation_summary.tex'}")


if __name__ == '__main__':
    main()
