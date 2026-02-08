"""
Measure computational complexity and runtime performance of edge-weighted METIS.

Purpose:
--------
Addresses P2.6 reviewer concern about computational feasibility and scalability.
Measures runtimes for baseline vs edge-weighted METIS, analyzes scaling behavior.

Outputs:
--------
- results/computational_complexity/timing_results.csv: Raw timing data
- results/computational_complexity/complexity_analysis.json: Summary statistics
- results/computational_complexity/timing_table.tex: LaTeX table for paper
"""

import sys
import time
import json
import numpy as np
import pandas as pd
from pathlib import Path
from collections import defaultdict

# Add project root to path
project_root = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(project_root / 'src'))

from apportionment.partition.recursive_bisection import recursive_bisection
from apportionment.data.adjacency import load_adjacency_graph

# States with different graph sizes for scaling analysis
STATES = [
    ('MS', 4),   # Mississippi: ~880 tracts, 4 districts (smallest)
    ('LA', 6),   # Louisiana: ~1,140 tracts, 6 districts
    ('AL', 7),   # Alabama: ~1,180 tracts, 7 districts
    ('SC', 7),   # South Carolina: ~1,100 tracts, 7 districts
    ('GA', 14),  # Georgia: ~1,960 tracts, 14 districts (largest)
]

# Edge weight configurations to test
CONFIGURATIONS = [
    {'beta': 1.0, 'tau': 0.0, 'name': 'baseline'},
    {'beta': 5.0, 'tau': 0.40, 'name': 'edge_5x_40pct'},
    {'beta': 5.0, 'tau': 0.50, 'name': 'edge_5x_50pct'},
    {'beta': 10.0, 'tau': 0.50, 'name': 'edge_10x_50pct'},
]

# Number of replications for reliability
N_REPLICATIONS = 5


def measure_runtime(state_code, n_districts, beta, tau, year=2020):
    """
    Measure runtime for a single METIS partitioning operation.

    Returns:
        float: Runtime in seconds
    """
    # Load adjacency graph
    data_dir = project_root / 'outputs' / 'data' / str(year)
    adjacency_file = data_dir / 'adjacency' / f'{state_code.lower()}_adjacency.json'

    if not adjacency_file.exists():
        raise FileNotFoundError(f"Adjacency file not found: {adjacency_file}")

    graph_data = load_adjacency_graph(adjacency_file)

    # Extract required data
    adjacency = graph_data['adjacency']
    demographics = graph_data['demographics']
    tract_ids = graph_data['tract_ids']

    # Measure runtime
    start_time = time.perf_counter()

    assignments = recursive_bisection(
        adjacency=adjacency,
        demographics=demographics,
        n_districts=n_districts,
        edge_weight_beta=beta,
        minority_threshold_tau=tau,
    )

    end_time = time.perf_counter()
    runtime = end_time - start_time

    return runtime


def collect_timing_data():
    """
    Collect timing data for all states and configurations.

    Returns:
        pd.DataFrame: Timing results
    """
    results = []

    print("=" * 60)
    print("MEASURING COMPUTATIONAL COMPLEXITY")
    print("=" * 60)
    print()

    total_runs = len(STATES) * len(CONFIGURATIONS) * N_REPLICATIONS
    current_run = 0

    for state_code, n_districts in STATES:
        print(f"State: {state_code} ({n_districts} districts)")

        # Load graph to get size
        data_dir = project_root / 'outputs' / 'data' / '2020'
        adjacency_file = data_dir / 'adjacency' / f'{state_code.lower()}_adjacency.json'

        with open(adjacency_file, 'r') as f:
            graph_data = json.load(f)
            n_tracts = len(graph_data['tract_ids'])
            n_edges = sum(len(neighbors) for neighbors in graph_data['adjacency'].values()) // 2

        print(f"  Graph size: {n_tracts} tracts, {n_edges} edges")

        for config in CONFIGURATIONS:
            beta = config['beta']
            tau = config['tau']
            config_name = config['name']

            print(f"    Config: {config_name} (beta={beta}, tau={tau})")

            runtimes = []
            for rep in range(N_REPLICATIONS):
                current_run += 1
                print(f"      Replication {rep + 1}/{N_REPLICATIONS} " +
                      f"[{current_run}/{total_runs}]", end='', flush=True)

                try:
                    runtime = measure_runtime(state_code, n_districts, beta, tau)
                    runtimes.append(runtime)
                    print(f" --> {runtime:.3f}s")
                except Exception as e:
                    print(f" --> ERROR: {e}")
                    continue

            if runtimes:
                results.append({
                    'state': state_code,
                    'n_districts': n_districts,
                    'n_tracts': n_tracts,
                    'n_edges': n_edges,
                    'config': config_name,
                    'beta': beta,
                    'tau': tau,
                    'mean_runtime': np.mean(runtimes),
                    'std_runtime': np.std(runtimes),
                    'min_runtime': np.min(runtimes),
                    'max_runtime': np.max(runtimes),
                    'n_replications': len(runtimes),
                })

        print()

    df = pd.DataFrame(results)
    return df


def analyze_complexity(df):
    """
    Analyze computational complexity from timing data.

    Returns:
        dict: Complexity analysis summary
    """
    analysis = {}

    # 1. Baseline vs edge-weighted comparison
    baseline = df[df['config'] == 'baseline']
    edge_weighted = df[df['config'] != 'baseline']

    baseline_mean = baseline['mean_runtime'].mean()
    edge_weighted_mean = edge_weighted['mean_runtime'].mean()
    overhead_factor = edge_weighted_mean / baseline_mean

    analysis['baseline_vs_edge_weighted'] = {
        'baseline_mean_runtime': float(baseline_mean),
        'edge_weighted_mean_runtime': float(edge_weighted_mean),
        'overhead_factor': float(overhead_factor),
        'overhead_percentage': float((overhead_factor - 1.0) * 100),
    }

    # 2. Scaling analysis (runtime vs graph size)
    # Fit log-log linear regression: log(runtime) = a + b*log(n_tracts)
    baseline_by_state = baseline.groupby('state').first().reset_index()

    log_n = np.log(baseline_by_state['n_tracts'].values)
    log_t = np.log(baseline_by_state['mean_runtime'].values)

    coeffs = np.polyfit(log_n, log_t, 1)
    complexity_exponent = coeffs[0]

    analysis['scaling_analysis'] = {
        'complexity_exponent': float(complexity_exponent),
        'interpretation': 'O(n^{:.2f})'.format(complexity_exponent),
        'note': 'Empirical scaling from 880 to 1960 tracts',
    }

    # 3. Per-state summary
    state_summary = []
    for state in STATES:
        state_code = state[0]
        state_data = df[df['state'] == state_code]

        baseline_runtime = state_data[state_data['config'] == 'baseline']['mean_runtime'].iloc[0]
        edge_10x_runtime = state_data[state_data['config'] == 'edge_10x_50pct']['mean_runtime'].iloc[0]

        state_summary.append({
            'state': state_code,
            'n_tracts': int(state_data['n_tracts'].iloc[0]),
            'n_districts': int(state_data['n_districts'].iloc[0]),
            'baseline_runtime_sec': float(baseline_runtime),
            'edge_10x_runtime_sec': float(edge_10x_runtime),
            'overhead_factor': float(edge_10x_runtime / baseline_runtime),
        })

    analysis['state_summary'] = state_summary

    # 4. Configuration comparison
    config_summary = []
    for config_name in ['baseline', 'edge_5x_40pct', 'edge_5x_50pct', 'edge_10x_50pct']:
        config_data = df[df['config'] == config_name]

        config_summary.append({
            'config': config_name,
            'mean_runtime': float(config_data['mean_runtime'].mean()),
            'std_runtime': float(config_data['std_runtime'].mean()),
            'min_runtime': float(config_data['min_runtime'].min()),
            'max_runtime': float(config_data['max_runtime'].max()),
        })

    analysis['config_summary'] = config_summary

    return analysis


def generate_latex_table(df, analysis):
    """Generate LaTeX table for paper."""

    # State-level timing table
    table_lines = []
    table_lines.append(r"\begin{table}[h]")
    table_lines.append(r"\centering")
    table_lines.append(r"\caption{Computational Performance: Baseline vs Edge-Weighted METIS}")
    table_lines.append(r"\label{tab:computational_complexity}")
    table_lines.append(r"\begin{tabular}{lrrrr}")
    table_lines.append(r"\hline")
    table_lines.append(r"State & Tracts & Baseline (s) & Edge-10$\times$ (s) & Overhead \\")
    table_lines.append(r"\hline")

    for state_info in analysis['state_summary']:
        state = state_info['state']
        n_tracts = state_info['n_tracts']
        baseline_t = state_info['baseline_runtime_sec']
        edge_t = state_info['edge_10x_runtime_sec']
        overhead = state_info['overhead_factor']

        table_lines.append(
            f"{state} & {n_tracts:,} & {baseline_t:.2f} & {edge_t:.2f} & {overhead:.2f}$\\times$ \\\\"
        )

    table_lines.append(r"\hline")

    # Add summary row
    baseline_mean = analysis['baseline_vs_edge_weighted']['baseline_mean_runtime']
    edge_mean = analysis['baseline_vs_edge_weighted']['edge_weighted_mean_runtime']
    overhead_mean = analysis['baseline_vs_edge_weighted']['overhead_factor']

    table_lines.append(
        f"\\textbf{{Mean}} & & {baseline_mean:.2f} & {edge_mean:.2f} & " +
        f"{overhead_mean:.2f}$\\times$ \\\\"
    )

    table_lines.append(r"\hline")
    table_lines.append(r"\end{tabular}")

    # Add notes
    table_lines.append(r"\vspace{0.2cm}")
    table_lines.append(r"\begin{minipage}{0.9\textwidth}")
    table_lines.append(r"\footnotesize")
    table_lines.append(r"\textbf{Notes:} ")
    table_lines.append(f"Runtimes are mean of {N_REPLICATIONS} replications. ")
    table_lines.append(r"Baseline uses uniform edge weights ($\beta=1$). ")
    table_lines.append(r"Edge-10$\times$ uses demographic edge weighting ($\beta=10$, $\tau=0.50$). ")

    complexity = analysis['scaling_analysis']['interpretation']
    table_lines.append(f"Empirical complexity: {complexity}. ")

    overhead_pct = analysis['baseline_vs_edge_weighted']['overhead_percentage']
    table_lines.append(f"Edge-weighting adds {overhead_pct:.1f}\\% overhead on average.")
    table_lines.append(r"\end{minipage}")
    table_lines.append(r"\end{table}")

    latex_table = "\n".join(table_lines)
    return latex_table


def main():
    """Main execution."""
    print("Starting computational complexity measurement...")
    print()

    # Create output directory
    output_dir = Path(__file__).parent.parent / 'results' / 'computational_complexity'
    output_dir.mkdir(parents=True, exist_ok=True)

    # Collect timing data
    df = collect_timing_data()

    # Save raw data
    csv_path = output_dir / 'timing_results.csv'
    df.to_csv(csv_path, index=False)
    print(f"\nSaved timing results: {csv_path}")

    # Analyze complexity
    analysis = analyze_complexity(df)

    # Save analysis
    json_path = output_dir / 'complexity_analysis.json'
    with open(json_path, 'w') as f:
        json.dump(analysis, f, indent=2)
    print(f"Saved complexity analysis: {json_path}")

    # Generate LaTeX table
    latex_table = generate_latex_table(df, analysis)
    tex_path = output_dir / 'timing_table.tex'
    with open(tex_path, 'w') as f:
        f.write(latex_table)
    print(f"Saved LaTeX table: {tex_path}")

    # Print summary
    print()
    print("=" * 60)
    print("SUMMARY")
    print("=" * 60)
    print()
    print(f"Baseline mean runtime: {analysis['baseline_vs_edge_weighted']['baseline_mean_runtime']:.2f}s")
    print(f"Edge-weighted mean runtime: {analysis['baseline_vs_edge_weighted']['edge_weighted_mean_runtime']:.2f}s")
    print(f"Overhead: {analysis['baseline_vs_edge_weighted']['overhead_factor']:.2f}x " +
          f"({analysis['baseline_vs_edge_weighted']['overhead_percentage']:.1f}%)")
    print()
    print(f"Empirical complexity: {analysis['scaling_analysis']['interpretation']}")
    print()
    print("Computational complexity measurement complete!")


if __name__ == '__main__':
    main()
