"""
South Carolina Deep Dive Investigation

Why can't we achieve 3 MM districts with standard parameters?

Questions:
1. What's the minority tract distribution?
2. Is geographic clustering sufficient (Moran's I)?
3. Can higher weight factors help (200x, 500x, 1000x)?
4. Can lower thresholds help (30%, 35%)?
5. How does SC compare to successful states?
"""

import sys
from pathlib import Path
import pandas as pd
import numpy as np
from collections import defaultdict

# Add paths
project_root = Path(__file__).parent.parent.parent.parent
sys.path.insert(0, str(project_root / 'src'))
sys.path.insert(0, str(project_root))

import geopandas as gpd
import matplotlib.pyplot as plt
from scipy.sparse import lil_matrix
from shapely.ops import unary_union

from apportionment.data.adjacency import build_adjacency_graph
from apportionment.partition.metis_executable import partition_graph_with_executable
from apportionment.partition.vra_utils import load_tract_demographics, analyze_mm_districts, create_vra_vertex_weights
from scripts.utils import get_tract_file

# State config
SC_CONFIG = {
    'name': 'south_carolina',
    'districts': 7,
    'target_mm': 3,
    'minority_pct': 0.351
}

def calculate_morans_i(tracts_gdf, adjacency_matrix, variable_name='pct_minority'):
    """
    Calculate Moran's I spatial autocorrelation statistic.

    Moran's I ranges from -1 (perfect dispersion) to +1 (perfect clustering).
    Values near 0 indicate random spatial distribution.
    Values > 0.3 typically indicate significant clustering.
    """
    from scipy.sparse import csr_matrix

    # Get values
    values = tracts_gdf[variable_name].values

    # Normalize adjacency matrix to weights
    adj_csr = adjacency_matrix.tocsr().astype(float)  # Convert to float for division
    row_sums = np.array(adj_csr.sum(axis=1)).flatten()
    row_sums[row_sums == 0] = 1  # Avoid division by zero

    # Create weight matrix (row-normalized)
    weights = adj_csr.copy()
    for i in range(weights.shape[0]):
        if row_sums[i] > 0:
            weights.data[weights.indptr[i]:weights.indptr[i+1]] /= row_sums[i]

    # Calculate Moran's I
    n = len(values)
    mean_val = values.mean()

    # Numerator: sum of (w_ij * (x_i - mean) * (x_j - mean))
    numerator = 0.0
    for i in range(n):
        deviation_i = values[i] - mean_val
        for j_idx in range(weights.indptr[i], weights.indptr[i+1]):
            j = weights.indices[j_idx]
            w_ij = weights.data[j_idx]
            deviation_j = values[j] - mean_val
            numerator += w_ij * deviation_i * deviation_j

    # Denominator: sum of (x_i - mean)^2
    denominator = np.sum((values - mean_val) ** 2)

    # Moran's I = (n / sum_weights) * (numerator / denominator)
    sum_weights = weights.sum()
    morans_i = (n / sum_weights) * (numerator / denominator)

    return morans_i


def analyze_demographic_distribution(state_code='SC', year='2020', version='V1'):
    """
    Analyze South Carolina's demographic distribution.
    """
    config = SC_CONFIG
    state_name = config['name']

    print("=" * 80)
    print("SOUTH CAROLINA DEMOGRAPHIC ANALYSIS")
    print("=" * 80)

    # Load data
    tracts_file = str(get_tract_file(state_code, year, version))
    tracts_gdf = gpd.read_parquet(tracts_file)

    demographics = load_tract_demographics(state_name, year)
    vertex_weights_vra, tracts_with_demo = create_vra_vertex_weights(tracts_gdf, demographics)

    # Build adjacency
    adjacency_list, _, _, _, _ = build_adjacency_graph(tracts_with_demo)

    # Create adjacency matrix
    n_tracts = len(adjacency_list)
    adj_matrix = lil_matrix((n_tracts, n_tracts), dtype=int)
    for i, neighbors in enumerate(adjacency_list):
        for j in neighbors:
            adj_matrix[i, j] = 1
    adj_matrix = adj_matrix.tocsr()

    # Basic statistics
    minority_pcts = tracts_with_demo['pct_minority'].values

    print(f"\nBASIC STATISTICS:")
    print(f"  Total tracts: {n_tracts}")
    print(f"  Target districts: {config['districts']}")
    print(f"  Target MM districts: {config['target_mm']}")
    print(f"  State minority %: {config['minority_pct']*100:.1f}%")
    print(f"  Avg tract minority %: {minority_pcts.mean()*100:.1f}%")
    print(f"  Median tract minority %: {np.median(minority_pcts)*100:.1f}%")
    print(f"  Std dev: {minority_pcts.std()*100:.1f}%")

    # Distribution by threshold
    print(f"\nTRACT DISTRIBUTION BY MINORITY THRESHOLD:")
    for threshold in [0.30, 0.35, 0.40, 0.45, 0.50, 0.55, 0.60]:
        count = (minority_pcts >= threshold).sum()
        pct = count / n_tracts * 100
        print(f"  >={threshold*100:.0f}%: {count} tracts ({pct:.1f}%)")

    # High-minority tract clustering
    print(f"\nHIGH-MINORITY TRACT COUNTS (>=50%):")
    high_minority_tracts = (minority_pcts >= 0.50).sum()
    print(f"  Count: {high_minority_tracts}")
    print(f"  Percent: {high_minority_tracts/n_tracts*100:.1f}%")
    print(f"  Expected per district: {high_minority_tracts/config['districts']:.1f}")
    print(f"  Needed for 3 MM districts: ~{config['districts']*3//config['districts']*0.5*n_tracts/config['districts']:.0f}")

    # Calculate Moran's I
    print(f"\nSPATIAL AUTOCORRELATION (Moran's I):")
    morans_i = calculate_morans_i(tracts_with_demo, adj_matrix, 'pct_minority')
    print(f"  Moran's I: {morans_i:.3f}")

    if morans_i > 0.5:
        print(f"  -> STRONG clustering (minorities are geographically concentrated)")
    elif morans_i > 0.3:
        print(f"  -> MODERATE clustering")
    elif morans_i > 0.1:
        print(f"  -> WEAK clustering")
    else:
        print(f"  -> MINIMAL clustering (minorities are dispersed)")

    # Compare to other states
    print(f"\nCOMPARISON TO OTHER VRA STATES:")
    comparison_states = {
        'AL': {'minority_pct': 0.369, 'target_mm': 2, 'ratio': 2/7},
        'GA': {'minority_pct': 0.424, 'target_mm': 5, 'ratio': 5/14},
        'LA': {'minority_pct': 0.416, 'target_mm': 2, 'ratio': 2/6},
        'MS': {'minority_pct': 0.461, 'target_mm': 2, 'ratio': 2/4},
        'SC': {'minority_pct': 0.351, 'target_mm': 3, 'ratio': 3/7},
    }

    for state, data in comparison_states.items():
        print(f"  {state}: {data['minority_pct']*100:.1f}% minority, "
              f"{data['target_mm']} MM target, "
              f"{data['ratio']*100:.1f}% of districts")

    print(f"\n  SC has LOWEST minority % but HIGHEST MM ratio (42.9% of districts)")
    print(f"  Compare to AL: 36.9% minority -> 28.6% MM districts (achievable)")
    print(f"  Compare to GA: 42.4% minority -> 35.7% MM districts (achievable)")
    print(f"  SC: 35.1% minority -> 42.9% MM districts (challenging!)")

    return tracts_with_demo, adjacency_list, adj_matrix, morans_i


def test_aggressive_parameters(tracts_with_demo, adjacency_list, adj_matrix,
                                state_code='SC', config=SC_CONFIG):
    """
    Test aggressive parameters: higher weights, lower thresholds.
    """
    print("\n" + "=" * 80)
    print("TESTING AGGRESSIVE PARAMETERS")
    print("=" * 80)

    # Test configurations
    weight_factors = [100, 200, 500, 1000]
    thresholds = [0.30, 0.35, 0.40, 0.45, 0.50]

    results = []

    print(f"\nTesting {len(weight_factors)} × {len(thresholds)} = "
          f"{len(weight_factors) * len(thresholds)} configurations...\n")

    for weight_factor in weight_factors:
        for threshold in thresholds:
            # Create edge weights
            is_minority = tracts_with_demo['pct_minority'] >= threshold

            edge_weights = {}
            minority_edge_count = 0

            adj_coo = adj_matrix.tocoo()
            for i, j in zip(adj_coo.row, adj_coo.col):
                if i < j:
                    if is_minority.iloc[i] and is_minority.iloc[j]:
                        edge_weights[(i, j)] = weight_factor
                        minority_edge_count += 1
                    else:
                        edge_weights[(i, j)] = 1.0

            # Run METIS
            vertex_weights = tracts_with_demo['total_pop'].values

            partition = partition_graph_with_executable(
                adjacency_list,
                vertex_weights,
                nparts=config['districts'],
                ufactor=1.005,
                edge_weights=edge_weights,
                niter=100,
                debug=False
            )

            # Analyze
            analysis = analyze_mm_districts(tracts_with_demo, partition, mm_threshold=0.50)
            mm_count = analysis['mm_count']
            max_minority = max(d['pct_minority'] for d in analysis['districts'])

            # Calculate edge cut
            edge_cut = 0
            for i, neighbors in enumerate(adjacency_list):
                for j in neighbors:
                    if i < j and partition[i] != partition[j]:
                        edge_cut += 1

            success = mm_count >= config['target_mm']

            result = {
                'weight_factor': weight_factor,
                'threshold': threshold,
                'minority_tracts': is_minority.sum(),
                'minority_edges': minority_edge_count,
                'mm_count': mm_count,
                'max_minority_pct': max_minority,
                'edge_cut': edge_cut,
                'success': success
            }
            results.append(result)

            status = "[SUCCESS]" if success else "[FAIL]"
            print(f"{status} {weight_factor:4d}x @ {threshold*100:2.0f}%: "
                  f"{mm_count}/{config['target_mm']} MM, "
                  f"max {max_minority:.1%}, "
                  f"edge cut {edge_cut}")

    results_df = pd.DataFrame(results)

    # Summary
    print("\n" + "=" * 80)
    print("RESULTS SUMMARY")
    print("=" * 80)

    successful = results_df[results_df['success'] == True]

    if len(successful) > 0:
        print(f"\n[OK] SUCCESS! Found {len(successful)} configurations that achieve 3 MM districts")
        print(f"\nBest configurations (by edge cut):")
        best = successful.nsmallest(5, 'edge_cut')
        print(best[['weight_factor', 'threshold', 'mm_count', 'max_minority_pct', 'edge_cut']])
    else:
        print(f"\n[FAIL] FAILED: None of the {len(results_df)} configurations achieved 3 MM districts")
        print(f"\nClosest attempts (2 MM districts):")
        closest = results_df[results_df['mm_count'] == 2].nsmallest(5, 'edge_cut')
        if len(closest) > 0:
            print(closest[['weight_factor', 'threshold', 'mm_count', 'max_minority_pct', 'edge_cut']])
        else:
            print("  No configurations achieved even 2 MM districts")

    return results_df


def visualize_minority_distribution(tracts_with_demo, state_code='SC'):
    """
    Visualize geographic distribution of minority tracts.
    """
    print("\n" + "=" * 80)
    print("GENERATING VISUALIZATIONS")
    print("=" * 80)

    fig, axes = plt.subplots(2, 2, figsize=(16, 14))

    # Plot 1: Minority percentage heatmap
    ax = axes[0, 0]
    tracts_with_demo.plot(column='pct_minority',
                          cmap='RdYlGn',
                          legend=True,
                          ax=ax,
                          edgecolor='black',
                          linewidth=0.3,
                          vmin=0, vmax=1,
                          legend_kwds={'label': 'Minority %', 'shrink': 0.8})
    ax.set_title('South Carolina: Minority Tract Distribution', fontsize=14, fontweight='bold')
    ax.axis('off')

    # Plot 2: Binary thresholds
    ax = axes[0, 1]

    # Create categories
    conditions = [
        tracts_with_demo['pct_minority'] >= 0.60,
        tracts_with_demo['pct_minority'] >= 0.50,
        tracts_with_demo['pct_minority'] >= 0.40,
        tracts_with_demo['pct_minority'] >= 0.30,
    ]
    categories = ['>=60%', '50-60%', '40-50%', '30-40%']
    colors = ['darkgreen', 'green', 'yellow', 'orange']

    tracts_with_demo['category'] = 'Below 30%'
    for i, cond in enumerate(conditions):
        tracts_with_demo.loc[cond, 'category'] = categories[i]

    # Create colormap
    from matplotlib.colors import ListedColormap
    unique_cats = ['>=60%', '50-60%', '40-50%', '30-40%', 'Below 30%']
    color_map = {'>=60%': 'darkgreen', '50-60%': 'green', '40-50%': 'yellow',
                 '30-40%': 'orange', 'Below 30%': 'lightgray'}

    for cat in unique_cats:
        subset = tracts_with_demo[tracts_with_demo['category'] == cat]
        if len(subset) > 0:
            subset.plot(ax=ax, color=color_map[cat], edgecolor='black', linewidth=0.3,
                       label=cat)

    ax.set_title('Minority Tract Categories', fontsize=14, fontweight='bold')
    ax.legend(loc='best', fontsize=10)
    ax.axis('off')

    # Plot 3: Histogram of minority percentages
    ax = axes[1, 0]
    ax.hist(tracts_with_demo['pct_minority'], bins=50, edgecolor='black',
            alpha=0.7, color='steelblue')
    ax.axvline(x=0.50, color='red', linestyle='--', linewidth=2, label='50% threshold (MM)')
    ax.axvline(x=0.351, color='orange', linestyle='--', linewidth=2,
               label='35.1% (state avg)')
    ax.set_xlabel('Minority %', fontsize=12)
    ax.set_ylabel('Number of Tracts', fontsize=12)
    ax.set_title('Distribution of Tract-Level Minority %', fontsize=14, fontweight='bold')
    ax.legend()
    ax.grid(alpha=0.3)

    # Plot 4: Cumulative distribution
    ax = axes[1, 1]
    sorted_pcts = np.sort(tracts_with_demo['pct_minority'].values)[::-1]
    cumulative = np.arange(1, len(sorted_pcts) + 1)

    ax.plot(cumulative, sorted_pcts * 100, linewidth=2, color='steelblue')
    ax.axhline(y=50, color='red', linestyle='--', linewidth=2, label='50% MM threshold')
    ax.axhline(y=35.1, color='orange', linestyle='--', linewidth=2, label='State average')

    # Mark key points
    tracts_above_50 = (tracts_with_demo['pct_minority'] >= 0.50).sum()
    ax.scatter([tracts_above_50], [50], s=200, color='red', zorder=10,
               edgecolors='black', linewidths=2)
    ax.text(tracts_above_50, 52, f'{tracts_above_50} tracts\nabove 50%',
            ha='center', fontsize=10, fontweight='bold')

    ax.set_xlabel('Number of Tracts (sorted by minority %)', fontsize=12)
    ax.set_ylabel('Minority % (cumulative)', fontsize=12)
    ax.set_title('Cumulative Distribution: How Many Tracts Above Each Threshold?',
                 fontsize=14, fontweight='bold')
    ax.legend()
    ax.grid(alpha=0.3)

    plt.tight_layout()

    results_dir = Path(__file__).parent.parent / 'results'
    plt.savefig(results_dir / 'south_carolina_investigation.png',
                dpi=300, bbox_inches='tight')
    print(f"\nSaved: {results_dir / 'south_carolina_investigation.png'}")


if __name__ == '__main__':
    print("=" * 80)
    print("SOUTH CAROLINA VRA COMPLIANCE INVESTIGATION")
    print("=" * 80)
    print("Question: Why can't we achieve 3 MM districts?")
    print("=" * 80)

    # Step 1: Demographic analysis
    tracts_with_demo, adjacency_list, adj_matrix, morans_i = analyze_demographic_distribution()

    # Step 2: Test aggressive parameters
    results_df = test_aggressive_parameters(tracts_with_demo, adjacency_list, adj_matrix)

    # Save results
    results_dir = Path(__file__).parent.parent / 'results'
    results_dir.mkdir(exist_ok=True)
    results_df.to_csv(results_dir / 'south_carolina_aggressive_parameters.csv', index=False)
    print(f"\nSaved: {results_dir / 'south_carolina_aggressive_parameters.csv'}")

    # Step 3: Visualize
    visualize_minority_distribution(tracts_with_demo)

    print("\n" + "=" * 80)
    print("INVESTIGATION COMPLETE")
    print("=" * 80)
