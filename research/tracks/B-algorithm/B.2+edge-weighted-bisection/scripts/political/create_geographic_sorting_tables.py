#!/usr/bin/env python3
"""Generate LaTeX tables for geographic sorting analysis."""

import json
import pandas as pd
from pathlib import Path


def create_detailed_table(df: pd.DataFrame, output_file: Path):
    """Create detailed state-by-state geographic sorting table."""

    # Select 10 representative states: mix of classifications
    geography_dominated = df[df['classification'] == 'Geography-Dominated'].head(3)
    mixed = df[df['classification'] == 'Mixed'].head(3)
    gerrymander_dominated = df[df['classification'] == 'Gerrymandering-Dominated'].head(4)

    selected = pd.concat([geography_dominated, mixed, gerrymander_dominated])

    latex = r"""\begin{table}[t]
\centering
\caption{Geographic Sorting vs Gerrymandering: Representative States}
\label{tab:geographic_sorting}
\begin{tabular}{lrrrrr}
\toprule
\textbf{State} & \textbf{Enacted} & \textbf{Algorithmic} & \textbf{Premium} & \textbf{Geo Frac} & \textbf{Class} \\
& \textbf{EG (\%)} & \textbf{EG (\%)} & \textbf{(\%)} & \textbf{(\%)} & \\
\midrule
"""

    for _, row in selected.iterrows():
        state = row['state']
        enacted_eg = row['enacted_eg'] * 100
        alg_eg = row['algorithmic_eg'] * 100
        premium = row['gerrymander_premium_eg'] * 100
        geo_frac = row['avg_geographic_fraction'] * 100
        classification = row['classification'].replace('-', ' ')

        latex += f"{state} & {enacted_eg:+.1f} & {alg_eg:+.1f} & {premium:+.1f} & {geo_frac:.0f} & {classification} \\\\\n"

    latex += r"""\bottomrule
\end{tabular}
\begin{tablenotes}
\small
\item \textit{Enacted EG}: Efficiency gap in enacted districts. \textit{Algorithmic EG}: Efficiency gap in compact algorithmic districts (geographic baseline). \textit{Premium}: Gerrymandering premium (enacted - algorithmic). \textit{Geo Frac}: Average geographic fraction across three metrics (efficiency gap, mean-median, partisan bias). \textit{Class}: Geography-Dominated ($>$60\% geographic), Mixed (30-60\%), or Gerrymandering-Dominated ($<$30\% geographic).
\end{tablenotes}
\end{table}
"""

    with open(output_file, 'w') as f:
        f.write(latex)


def create_summary_table(summary: dict, output_file: Path):
    """Create national summary statistics table."""

    total = summary['total_states']
    geo_dom = summary['geography_dominated']
    mixed = summary['mixed']
    gerr_dom = summary['gerrymander_dominated']
    avg_geo = summary['avg_geographic_fraction'] * 100
    worsens = summary['states_where_gerrymandering_worsens_bias']
    improves = summary['states_where_gerrymandering_improves_balance']

    latex = r"""\begin{table}[t]
\centering
\caption{National Geographic Sorting Summary}
\label{tab:geographic_sorting_summary}
\begin{tabular}{lr}
\toprule
\textbf{Metric} & \textbf{Value} \\
\midrule
Total States & """ + f"{total}" + r""" \\
\midrule
Geography-Dominated States & """ + f"{geo_dom} ({geo_dom/total*100:.0f}\\%)" + r""" \\
Mixed States & """ + f"{mixed} ({mixed/total*100:.0f}\\%)" + r""" \\
Gerrymandering-Dominated States & """ + f"{gerr_dom} ({gerr_dom/total*100:.0f}\\%)" + r""" \\
\midrule
Average Geographic Fraction & """ + f"{avg_geo:.0f}\\%" + r""" \\
\midrule
Gerrymandering Worsens Bias & """ + f"{worsens} states" + r""" \\
Gerrymandering Improves Balance & """ + f"{improves} states" + r""" \\
\bottomrule
\end{tabular}
\begin{tablenotes}
\small
\item \textit{Geography-Dominated}: $>$60\% of partisan bias is geographic (unavoidable). \textit{Mixed}: 30-60\% geographic. \textit{Gerrymandering-Dominated}: $<$30\% geographic (mostly intentional manipulation). \textit{Average Geographic Fraction}: Mean percentage of total partisan bias attributable to geographic voter sorting across all states. \textit{Worsens Bias}: States where gerrymandering increases partisan bias beyond geographic baseline. \textit{Improves Balance}: States where enacted plans are more balanced than geographic baseline (rare).
\end{tablenotes}
\end{table}
"""

    with open(output_file, 'w') as f:
        f.write(latex)


def main():
    base_dir = Path(__file__).parent.parent

    # Load data
    data_file = base_dir / 'data' / 'geographic_sorting_2020.csv'
    summary_file = base_dir / 'data' / 'geographic_sorting_summary_2020.json'

    if not data_file.exists():
        print("Error: Run compute_geographic_sorting.py first")
        return

    df = pd.read_csv(data_file)

    with open(summary_file) as f:
        summary = json.load(f)

    # Create tables
    tables_dir = base_dir / 'tables'
    tables_dir.mkdir(exist_ok=True)

    create_detailed_table(df, tables_dir / 'geographic_sorting_table.tex')
    create_summary_table(summary, tables_dir / 'geographic_sorting_summary.tex')

    print("[OK] Created LaTeX tables:")
    print(f"  -> {tables_dir / 'geographic_sorting_table.tex'}")
    print(f"  -> {tables_dir / 'geographic_sorting_summary.tex'}")


if __name__ == '__main__':
    main()
