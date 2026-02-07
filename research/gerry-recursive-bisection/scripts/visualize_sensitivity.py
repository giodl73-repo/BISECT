#!/usr/bin/env python3
"""
Visualize Parameter Sensitivity Results for P1.1

Analyzes parameter sweep results and generates:
- Tables showing variation in key metrics
- Box plots showing distributions
- Statistical tests for robustness

Usage:
    python visualize_sensitivity.py --input outputs/sensitivity/ --output research/gerry-recursive-bisection/figures/
"""

import sys
import os
from pathlib import Path
import argparse
import json
import pickle
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import seaborn as sns
from scipy import stats
from tqdm import tqdm

# Add project root to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent / 'src'))
sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent))

from scripts.utils import get_state_config, get_tract_file
import geopandas as gpd


def load_assignments(run_dir):
    """Load district assignments from run directory."""
    final_file = Path(run_dir) / 'data' / 'final_assignments.pkl'
    if not final_file.exists():
        return None

    with open(final_file, 'rb') as f:
        return pickle.load(f)


def compute_metrics(assignments, tracts_gdf, vertex_weights, num_districts):
    """
    Compute key metrics for a redistricting run.

    Returns dict with:
    - compactness_mean: Mean Polsby-Popper score
    - compactness_std: Std deviation of PP scores
    - pop_max_deviation: Maximum population deviation
    - pop_mean_deviation: Mean absolute population deviation
    """
    # Compactness
    tracts = tracts_gdf.copy()
    tracts['district'] = [assignments[i] for i in range(len(tracts))]

    pp_scores = []
    for district_id in sorted(tracts['district'].unique()):
        district_tracts = tracts[tracts['district'] == district_id]
        district_geom = district_tracts.geometry.union_all()

        area = district_geom.area
        perimeter = district_geom.length

        if perimeter > 0:
            pp = (4 * np.pi * area) / (perimeter ** 2)
        else:
            pp = 0.0

        pp_scores.append(pp)

    # Population balance
    populations = []
    for district_id in range(1, num_districts + 1):
        pop = sum(vertex_weights[i] for i, d in assignments.items() if d == district_id)
        populations.append(pop)

    total_pop = sum(populations)
    ideal_pop = total_pop / num_districts
    deviations = [abs(p - ideal_pop) / ideal_pop * 100 for p in populations]

    return {
        'compactness_mean': np.mean(pp_scores),
        'compactness_std': np.std(pp_scores),
        'compactness_min': np.min(pp_scores),
        'compactness_max': np.max(pp_scores),
        'pop_max_deviation': max(deviations),
        'pop_mean_deviation': np.mean(deviations),
        'pop_std_deviation': np.std(deviations)
    }


def analyze_parameter_sensitivity(input_dir, year, state_config):
    """
    Analyze parameter sensitivity results.

    Returns DataFrame with metrics for each run.
    """
    # Load metadata
    metadata_file = Path(input_dir) / 'sensitivity_metadata.json'
    if not metadata_file.exists():
        raise FileNotFoundError(f"Metadata file not found: {metadata_file}")

    with open(metadata_file, 'r') as f:
        metadata = json.load(f)

    results = metadata['results']
    states = metadata['states']

    print(f"\n{'='*80}")
    print(f"ANALYZING PARAMETER SENSITIVITY")
    print(f"{'='*80}")
    print(f"States: {', '.join(states)}")
    print(f"Total runs: {len(results)}")
    print(f"Successful runs: {metadata['successful_runs']}")
    print(f"{'='*80}\n")

    # Analyze each run
    analysis_results = []

    for result in tqdm(results, desc="Computing metrics"):
        if 'error' in result:
            continue

        state_code = result['state']
        run_dir = result['output_dir']

        # Load assignments
        assignments = load_assignments(run_dir)
        if assignments is None:
            continue

        # Load tract data (use v1 version for input data)
        tracts_file = get_tract_file(state_code, year, 'v1')
        if not Path(tracts_file).exists():
            # Try without version
            print(f"  WARNING: Tract file not found: {tracts_file}")
            continue

        tracts_gdf = gpd.read_parquet(tracts_file)
        # Handle different population column names
        if 'POP20' in tracts_gdf.columns:
            vertex_weights = tracts_gdf['POP20'].values
        elif 'population' in tracts_gdf.columns:
            vertex_weights = tracts_gdf['population'].values
        else:
            raise ValueError(f"Population column not found in {tracts_file}")

        num_districts = state_config[state_code]['districts']

        # Compute metrics
        metrics = compute_metrics(assignments, tracts_gdf, vertex_weights, num_districts)

        # Combine with run parameters
        analysis_results.append({
            'state': state_code,
            'ufactor': result['ufactor'],
            'niter': result['niter'],
            'objtype': result['objtype'],
            'seed': result['seed'],
            **metrics
        })

    df = pd.DataFrame(analysis_results)

    return df, metadata


def generate_summary_tables(df, output_dir):
    """Generate LaTeX tables for paper."""
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    print("\nGenerating summary tables...")

    # Table 1: Variation by parameter (across all states)
    tables = []

    # Group by parameter type
    for param_name in ['ufactor', 'niter', 'objtype']:
        if len(df[param_name].unique()) > 1:
            grouped = df.groupby(param_name).agg({
                'compactness_mean': ['mean', 'std'],
                'pop_max_deviation': ['mean', 'std']
            }).round(4)

            tables.append(f"\n\\subsection{{{param_name.upper()} Variation}}")
            tables.append(grouped.to_latex())

    # Table 2: Per-state variation summary
    state_summary = df.groupby('state').agg({
        'compactness_mean': ['mean', 'std', 'min', 'max'],
        'pop_max_deviation': ['mean', 'std', 'min', 'max']
    }).round(4)

    tables.append("\n\\subsection{Per-State Summary}")
    tables.append(state_summary.to_latex())

    # Save to file
    tables_file = output_dir / 'sensitivity_tables.tex'
    with open(tables_file, 'w') as f:
        f.write('\n'.join(tables))

    print(f"  Saved: {tables_file}")

    return tables_file


def generate_visualizations(df, output_dir):
    """Generate figures for paper."""
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    print("\nGenerating visualizations...")

    # Figure 1: Compactness variation by parameter
    fig, axes = plt.subplots(1, 3, figsize=(18, 5))

    # ufactor
    if len(df['ufactor'].unique()) > 1:
        sns.boxplot(data=df, x='ufactor', y='compactness_mean', ax=axes[0])
        axes[0].set_title('Compactness by ufactor')
        axes[0].set_ylabel('Mean Polsby-Popper')
        axes[0].set_xlabel('ufactor')

    # niter
    if len(df['niter'].unique()) > 1:
        sns.boxplot(data=df, x='niter', y='compactness_mean', ax=axes[1])
        axes[1].set_title('Compactness by niter')
        axes[1].set_ylabel('Mean Polsby-Popper')
        axes[1].set_xlabel('niter')

    # objtype
    if len(df['objtype'].unique()) > 1:
        sns.boxplot(data=df, x='objtype', y='compactness_mean', ax=axes[2])
        axes[2].set_title('Compactness by objtype')
        axes[2].set_ylabel('Mean Polsby-Popper')
        axes[2].set_xlabel('objtype')

    plt.tight_layout()
    fig_file = output_dir / 'compactness_by_parameter.png'
    plt.savefig(fig_file, dpi=300, bbox_inches='tight')
    plt.close()
    print(f"  Saved: {fig_file}")

    # Figure 2: Population deviation by parameter
    fig, axes = plt.subplots(1, 3, figsize=(18, 5))

    # ufactor
    if len(df['ufactor'].unique()) > 1:
        sns.boxplot(data=df, x='ufactor', y='pop_max_deviation', ax=axes[0])
        axes[0].set_title('Population Deviation by ufactor')
        axes[0].set_ylabel('Max Deviation (%)')
        axes[0].set_xlabel('ufactor')

    # niter
    if len(df['niter'].unique()) > 1:
        sns.boxplot(data=df, x='niter', y='pop_max_deviation', ax=axes[1])
        axes[1].set_title('Population Deviation by niter')
        axes[1].set_ylabel('Max Deviation (%)')
        axes[1].set_xlabel('niter')

    # objtype
    if len(df['objtype'].unique()) > 1:
        sns.boxplot(data=df, x='objtype', y='pop_max_deviation', ax=axes[2])
        axes[2].set_title('Population Deviation by objtype')
        axes[2].set_ylabel('Max Deviation (%)')
        axes[2].set_xlabel('objtype')

    plt.tight_layout()
    fig_file = output_dir / 'population_by_parameter.png'
    plt.savefig(fig_file, dpi=300, bbox_inches='tight')
    plt.close()
    print(f"  Saved: {fig_file}")

    # Figure 3: Random seed ensemble (if seed sweep was performed)
    if len(df['seed'].unique()) > 10:
        fig, axes = plt.subplots(1, 2, figsize=(14, 5))

        # Compactness distribution across seeds
        for state in df['state'].unique():
            state_data = df[df['state'] == state]
            axes[0].plot(state_data['seed'], state_data['compactness_mean'],
                        'o-', alpha=0.6, label=state, markersize=2)

        axes[0].set_title('Compactness Stability Across Random Seeds')
        axes[0].set_xlabel('Random Seed')
        axes[0].set_ylabel('Mean Polsby-Popper')
        axes[0].legend()
        axes[0].grid(True, alpha=0.3)

        # Population deviation across seeds
        for state in df['state'].unique():
            state_data = df[df['state'] == state]
            axes[1].plot(state_data['seed'], state_data['pop_max_deviation'],
                        'o-', alpha=0.6, label=state, markersize=2)

        axes[1].set_title('Population Balance Across Random Seeds')
        axes[1].set_xlabel('Random Seed')
        axes[1].set_ylabel('Max Deviation (%)')
        axes[1].legend()
        axes[1].grid(True, alpha=0.3)

        plt.tight_layout()
        fig_file = output_dir / 'seed_ensemble.png'
        plt.savefig(fig_file, dpi=300, bbox_inches='tight')
        plt.close()
        print(f"  Saved: {fig_file}")


def compute_statistical_tests(df):
    """
    Compute statistical tests to quantify parameter robustness.

    Returns:
    - Coefficient of variation for each parameter
    - ANOVA F-statistic and p-value
    - Effect sizes
    """
    print("\nStatistical Analysis:")
    print("="*80)

    results = {}

    # For each parameter, compute variation
    for param in ['ufactor', 'niter', 'objtype']:
        if len(df[param].unique()) > 1:
            groups = [df[df[param] == val]['compactness_mean'].values
                     for val in df[param].unique()]

            # ANOVA
            f_stat, p_value = stats.f_oneway(*groups)

            # Coefficient of variation per group
            cv_values = [np.std(g) / np.mean(g) * 100 for g in groups]
            mean_cv = np.mean(cv_values)

            results[param] = {
                'f_statistic': float(f_stat) if not np.isnan(f_stat) else None,
                'p_value': float(p_value) if not np.isnan(p_value) else None,
                'mean_cv': float(mean_cv) if not np.isnan(mean_cv) else None,
                'significant': bool(p_value < 0.05) if not np.isnan(p_value) else False
            }

            print(f"\n{param.upper()}:")
            print(f"  F-statistic: {f_stat:.4f}")
            print(f"  p-value: {p_value:.6f}")
            print(f"  Mean CV: {mean_cv:.2f}%")
            print(f"  Significant: {'Yes' if p_value < 0.05 else 'No'}")

    # Random seed ensemble statistics
    if len(df['seed'].unique()) > 10:
        print(f"\nRANDOM SEED ENSEMBLE:")

        for state in df['state'].unique():
            state_data = df[df['state'] == state]

            compactness_cv = (state_data['compactness_mean'].std() /
                            state_data['compactness_mean'].mean() * 100)

            pop_cv = (state_data['pop_max_deviation'].std() /
                     state_data['pop_max_deviation'].mean() * 100)

            print(f"\n  {state}:")
            print(f"    Compactness CV: {compactness_cv:.2f}%")
            print(f"    Population CV: {pop_cv:.2f}%")
            print(f"    Runs: {len(state_data)}")

    print("="*80)

    return results


def main():
    parser = argparse.ArgumentParser(description='Visualize parameter sensitivity results')
    parser.add_argument('--input', type=str, default='outputs/sensitivity',
                       help='Input directory with sensitivity results')
    parser.add_argument('--output', type=str,
                       default='research/gerry-recursive-bisection/figures',
                       help='Output directory for figures and tables')
    parser.add_argument('--year', type=str, default='2020', choices=['2020', '2010', '2000'],
                       help='Census year (default: 2020)')

    args = parser.parse_args()

    # Load state configuration
    try:
        state_config = get_state_config(args.year)
    except (ValueError, ImportError) as e:
        print(f"ERROR: Could not load config for year {args.year}: {e}")
        sys.exit(1)

    # Analyze results
    df, metadata = analyze_parameter_sensitivity(args.input, args.year, state_config)

    # Save full results
    output_dir = Path(args.output)
    output_dir.mkdir(parents=True, exist_ok=True)

    csv_file = output_dir / 'sensitivity_analysis.csv'
    df.to_csv(csv_file, index=False)
    print(f"\nSaved analysis results: {csv_file}")

    # Generate tables and figures
    generate_summary_tables(df, output_dir)
    generate_visualizations(df, output_dir)

    # Statistical tests
    stats_results = compute_statistical_tests(df)

    # Save statistical results
    stats_file = output_dir / 'sensitivity_statistics.json'
    with open(stats_file, 'w') as f:
        json.dump(stats_results, f, indent=2)
    print(f"\nSaved statistical results: {stats_file}")

    print(f"\n{'='*80}")
    print(f"ANALYSIS COMPLETE")
    print(f"{'='*80}")
    print(f"Results saved to: {output_dir}")
    print(f"{'='*80}\n")


if __name__ == '__main__':
    main()
