#!/usr/bin/env python3
"""
Generate LaTeX Tables for VRA Compliance Analysis

Creates comparison tables showing majority-minority district counts
for enacted vs algorithmic districting plans.

Usage:
    python scripts/demographic/generate_vra_tables.py
"""

import json
from pathlib import Path
from typing import Dict

def load_vra_compliance(json_file: Path) -> Dict:
    """Load VRA compliance metrics"""
    with open(json_file, 'r') as f:
        return json.load(f)

def generate_comparison_table(results: Dict, output_dir: Path):
    """
    Generate LaTeX table comparing MM districts in enacted vs algorithmic plans

    Shows representative states with VRA issues
    """

    # Select states with significant MM district losses
    states_to_show = [
        'alabama', 'georgia', 'louisiana', 'mississippi', 'north_carolina', 'south_carolina',  # Southern states with Black populations
        'arizona', 'california', 'texas', 'new_mexico',  # States with Hispanic populations
    ]

    latex_content = r"""\begin{table}[t]
\caption{Voting Rights Act Compliance: Majority-Minority Districts}
\label{tab:vra-compliance}
\centering
\small
\begin{tabular}{@{}lrrrrl@{}}
\toprule
\textbf{State} & \textbf{Districts} & \textbf{Enacted} & \textbf{Algorithmic} & \textbf{Loss} & \textbf{Status} \\
\midrule
"""

    for state in states_to_show:
        if state not in results:
            continue

        r = results[state]
        state_name = state.replace('_', ' ').title()

        # Format loss with + sign
        loss = r['difference']
        loss_str = f"{loss:+d}" if loss != 0 else "0"

        # Status indicator
        status_map = {
            'Compliant': r'\checkmark',
            'At Risk': r'\textcolor{orange}{$\triangle$}',
            'Non-compliant': r'\textcolor{red}{$\times$}',
            'N/A': '--'
        }
        status_symbol = status_map.get(r['vra_status'], '--')

        latex_content += f"{state_name} & {r['num_districts']} & {r['enacted_total']} & {r['algorithmic_total']} & {loss_str} & {status_symbol} \\\\\n"

    latex_content += r"""\bottomrule
\end{tabular}
\vspace{0.5em}

\footnotesize
\textit{Note:} Majority-minority (MM) districts have >50\% minority population (Black, Hispanic, Asian, or coalition).
Loss = Enacted MM - Algorithmic MM.
Status: \checkmark{} Compliant (maintains MM districts),
$\triangle$ At Risk (loses 1 MM district),
$\times$ Non-compliant (loses 2+ MM districts).
\end{table}
"""

    output_file = output_dir / 'vra_compliance_table.tex'
    with open(output_file, 'w') as f:
        f.write(latex_content)

    return output_file

def generate_summary_table(results: Dict, output_dir: Path):
    """Generate national summary statistics table"""

    # Compute totals
    total_enacted = sum(r['enacted_total'] for r in results.values())
    total_algorithmic = sum(r['algorithmic_total'] for r in results.values())
    total_loss = total_enacted - total_algorithmic

    # By minority group
    total_enacted_black = sum(r['enacted_black'] for r in results.values())
    total_algorithmic_black = sum(r['algorithmic_black'] for r in results.values())

    total_enacted_hispanic = sum(r['enacted_hispanic'] for r in results.values())
    total_algorithmic_hispanic = sum(r['algorithmic_hispanic'] for r in results.values())

    total_enacted_asian = sum(r['enacted_asian'] for r in results.values())
    total_algorithmic_asian = sum(r['algorithmic_asian'] for r in results.values())

    # States affected
    states_with_loss = sum(1 for r in results.values() if r['difference'] > 0)
    states_at_risk = sum(1 for r in results.values() if r['vra_status'] == 'At Risk')
    states_non_compliant = sum(1 for r in results.values() if r['vra_status'] == 'Non-compliant')

    latex_content = r"""\begin{table}[t]
\caption{National VRA Compliance Summary}
\label{tab:vra-summary}
\centering
\begin{tabular}{@{}lrrr@{}}
\toprule
\textbf{Metric} & \textbf{Enacted} & \textbf{Algorithmic} & \textbf{Loss} \\
\midrule
\textbf{Total MM Districts} & """ + f"{total_enacted} & {total_algorithmic} & {total_loss} \\\\\n"

    latex_content += r"""\midrule
\textit{By Minority Group:} & & & \\
""" + f"Black-majority & {total_enacted_black} & {total_algorithmic_black} & {total_enacted_black - total_algorithmic_black} \\\\\n"
    latex_content += f"Hispanic-majority & {total_enacted_hispanic} & {total_algorithmic_hispanic} & {total_enacted_hispanic - total_algorithmic_hispanic} \\\\\n"
    latex_content += f"Asian-majority & {total_enacted_asian} & {total_algorithmic_asian} & {total_enacted_asian - total_algorithmic_asian} \\\\\n"

    latex_content += r"""\midrule
\textit{States Affected:} & & & \\
""" + f"With MM loss & {states_with_loss} & & \\\\\n"
    latex_content += f"At Risk (1 loss) & {states_at_risk} & & \\\\\n"
    latex_content += f"Non-compliant (2+ loss) & {states_non_compliant} & & \\\\\n"

    latex_content += r"""\bottomrule
\end{tabular}
\vspace{0.5em}

\footnotesize
\textit{Note:} Majority-minority districts have >50\% minority population.
Algorithmic (compact) districting loses """ + f"{total_loss} of {total_enacted} MM districts ({total_loss/total_enacted*100:.0f}\\% reduction), "
    latex_content += r"""primarily affecting Black and Hispanic representation in geographically concentrated populations.
\end{table}
"""

    output_file = output_dir / 'vra_summary_table.tex'
    with open(output_file, 'w') as f:
        f.write(latex_content)

    return output_file

def main():
    tables_dir = Path('research/slice-edge-weighted-bisection/tables')

    print("=" * 70)
    print("Generate VRA Compliance LaTeX Tables")
    print("=" * 70)
    print()

    # Load VRA compliance data
    json_file = tables_dir / 'vra_compliance_2020.json'
    print(f"Loading VRA compliance data: {json_file}")
    results = load_vra_compliance(json_file)
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
    print("Next step: Add VRA compliance subsection to paper")
    print("  Edit: sections/results.tex")
    print("  Add subsection after Partisan Outcome Analysis")

if __name__ == '__main__':
    main()
