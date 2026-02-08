"""
Create Alabama district maps comparing edge-weighted vs multi-constraint results
"""

import geopandas as gpd
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
from pathlib import Path
import sys
import os

# Add project root to path
project_root = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(project_root / 'src'))
sys.path.insert(0, str(project_root))

from apportionment.partition.metis_executable import partition_graph_with_executable
from apportionment.data.adjacency import build_adjacency_graph

# Publication style
plt.rcParams.update({
    'font.size': 11,
    'font.family': 'serif',
    'axes.labelsize': 12,
    'axes.titlesize': 13,
    'xtick.labelsize': 10,
    'ytick.labelsize': 10,
    'legend.fontsize': 10,
    'figure.dpi': 300,
    'savefig.dpi': 300,
    'savefig.bbox': 'tight',
})

def load_alabama_data():
    """Load Alabama census tract data with demographics"""
    # Load tract geometries
    tracts_path = Path("outputs/data/2020/units/alabama_2020_tracts.geojson")
    tracts = gpd.read_file(tracts_path)

    # Load demographics
    demo_path = Path("outputs/data/2020/demographics/alabama_demographics.csv")
    demo = pd.read_csv(demo_path)

    # Merge
    tracts = tracts.merge(demo, left_on='GEOID', right_on='geoid', how='left')

    # Calculate minority percentage
    tracts['minority_pct'] = tracts['minority_vap'] / tracts['total_vap']

    return tracts

def run_multi_constraint_best(tracts):
    """Run best multi-constraint configuration (ubvec=5.0)"""
    print("\n[1/2] Running multi-constraint (ubvec=5.0)...")

    # Build adjacency
    adj_list = build_adjacency_graph(tracts)

    # 2D weights
    vertex_weights = [[int(row['total_pop']), int(row['minority_vap'])]
                      for _, row in tracts.iterrows()]

    # Target weights: 2 MM districts at 60%, rest distributed
    total_pop = tracts['total_pop'].sum()
    total_minority = tracts['minority_vap'].sum()
    target_pop_per_district = total_pop / 7

    # 2 MM districts get 60% of their share of minority population
    mm_minority_per_district = 0.60 * (total_minority / 7)
    # Remaining 5 districts split the rest
    non_mm_minority_per_district = (total_minority - 2 * mm_minority_per_district) / 5

    tpwgts = []
    for i in range(7):
        if i < 2:  # First 2 are MM targets
            tpwgts.extend([target_pop_per_district / total_pop,
                          mm_minority_per_district / total_minority])
        else:
            tpwgts.extend([target_pop_per_district / total_pop,
                          non_mm_minority_per_district / total_minority])

    # Run METIS
    partition = partition_graph_with_executable(
        adj_list,
        vertex_weights,
        nparts=7,
        tpwgts=tpwgts,
        ubvec=[1.005, 5.0],
        niter=100,
        seed=42
    )

    return partition

def run_edge_weighted_best(tracts):
    """Run best edge-weighted configuration (5x @ 40%)"""
    print("\n[2/2] Running edge-weighted (5x @ 40%)...")

    # Build adjacency with edge weights
    adj_list = build_adjacency_graph(tracts, mode='edge_weighted',
                                     minority_threshold=0.40,
                                     weight_factor=5)

    # 1D weights (population only)
    vertex_weights = [int(row['total_pop']) for _, row in tracts.iterrows()]

    # Run METIS
    partition = partition_graph_with_executable(
        adj_list,
        vertex_weights,
        nparts=7,
        ufactor=1.005,
        niter=100,
        seed=42
    )

    return partition

def calculate_district_stats(tracts, partition):
    """Calculate statistics for each district"""
    tracts_copy = tracts.copy()
    tracts_copy['district'] = partition

    stats = []
    for district in sorted(tracts_copy['district'].unique()):
        dist_tracts = tracts_copy[tracts_copy['district'] == district]
        total_pop = dist_tracts['total_pop'].sum()
        total_vap = dist_tracts['total_vap'].sum()
        minority_vap = dist_tracts['minority_vap'].sum()
        minority_pct = minority_vap / total_vap if total_vap > 0 else 0

        stats.append({
            'district': district,
            'total_pop': total_pop,
            'minority_vap': minority_vap,
            'minority_pct': minority_pct,
            'is_mm': minority_pct >= 0.50
        })

    return pd.DataFrame(stats)

def create_comparison_map(tracts, partition_multi, partition_edge, output_path):
    """Create side-by-side comparison map"""
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(16, 6))

    # Calculate stats
    stats_multi = calculate_district_stats(tracts, partition_multi)
    stats_edge = calculate_district_stats(tracts, partition_edge)

    # Prepare data
    tracts_multi = tracts.copy()
    tracts_multi['district'] = partition_multi
    tracts_multi = tracts_multi.merge(stats_multi[['district', 'minority_pct', 'is_mm']],
                                      on='district', suffixes=('', '_dist'))

    tracts_edge = tracts.copy()
    tracts_edge['district'] = partition_edge
    tracts_edge = tracts_edge.merge(stats_edge[['district', 'minority_pct', 'is_mm']],
                                    on='district', suffixes=('', '_dist'))

    # Color scheme: MM districts in red, non-MM in blue gradient
    def get_colors(stats_df):
        colors = []
        for _, row in stats_df.iterrows():
            if row['is_mm']:
                colors.append('#DC143C')  # Crimson for MM
            else:
                # Blue gradient based on minority percentage
                intensity = row['minority_pct'] / 0.50
                colors.append(plt.cm.Blues(0.3 + 0.4 * intensity))
        return colors

    colors_multi = get_colors(stats_multi)
    colors_edge = get_colors(stats_edge)

    # Create color mapping
    district_colors_multi = dict(zip(stats_multi['district'], colors_multi))
    district_colors_edge = dict(zip(stats_edge['district'], colors_edge))

    # Plot multi-constraint
    tracts_multi.plot(ax=ax1, color=tracts_multi['district'].map(district_colors_multi),
                     edgecolor='black', linewidth=0.3)
    ax1.set_title('(A) Multi-Constraint: 1/2 MM Districts', fontweight='bold', fontsize=14)
    ax1.axis('off')

    # Add district labels with minority %
    for _, row in stats_multi.iterrows():
        centroid = tracts_multi[tracts_multi['district'] == row['district']].geometry.unary_union.centroid
        label = f"D{row['district']}\n{row['minority_pct']*100:.1f}%"
        color = 'white' if row['is_mm'] else 'black'
        ax1.text(centroid.x, centroid.y, label, ha='center', va='center',
                fontsize=9, fontweight='bold', color=color)

    # Plot edge-weighted
    tracts_edge.plot(ax=ax2, color=tracts_edge['district'].map(district_colors_edge),
                    edgecolor='black', linewidth=0.3)
    ax2.set_title('(B) Edge-Weighted: 2/2 MM Districts', fontweight='bold', fontsize=14)
    ax2.axis('off')

    # Add district labels with minority %
    for _, row in stats_edge.iterrows():
        centroid = tracts_edge[tracts_edge['district'] == row['district']].geometry.unary_union.centroid
        label = f"D{row['district']}\n{row['minority_pct']*100:.1f}%"
        color = 'white' if row['is_mm'] else 'black'
        ax2.text(centroid.x, centroid.y, label, ha='center', va='center',
                fontsize=9, fontweight='bold', color=color)

    # Add legend
    mm_patch = mpatches.Patch(color='#DC143C', label='Majority-Minority (≥50%)')
    non_mm_patch = mpatches.Patch(color=plt.cm.Blues(0.5), label='Non-MM (<50%)')
    fig.legend(handles=[mm_patch, non_mm_patch], loc='lower center',
              ncol=2, fontsize=11, frameon=True)

    # Overall title
    fig.suptitle('Alabama Congressional Districts: Multi-Constraint vs Edge-Weighted',
                fontsize=16, fontweight='bold', y=0.98)

    plt.tight_layout(rect=[0, 0.05, 1, 0.96])
    plt.savefig(output_path)
    plt.savefig(output_path.with_suffix('.pdf'))
    print(f"\n[OK] Map saved: {output_path}")
    plt.close()

    return stats_multi, stats_edge

def main():
    print("\n" + "="*70)
    print("Creating Alabama District Maps - Edge-Weighted vs Multi-Constraint")
    print("="*70)

    # Load data
    print("\nLoading Alabama data...")
    tracts = load_alabama_data()
    print(f"  Loaded {len(tracts)} tracts")
    print(f"  Total population: {tracts['total_pop'].sum():,}")
    print(f"  Total minority VAP: {tracts['minority_vap'].sum():,} ({tracts['minority_vap'].sum()/tracts['total_vap'].sum()*100:.1f}%)")

    # Run partitioning
    partition_multi = run_multi_constraint_best(tracts)
    partition_edge = run_edge_weighted_best(tracts)

    # Create comparison map
    print("\nGenerating comparison map...")
    output_path = Path("research/gerry-multi-vs-edge/results/figure5_alabama_comparison.png")
    stats_multi, stats_edge = create_comparison_map(tracts, partition_multi, partition_edge, output_path)

    # Print statistics
    print("\n" + "="*70)
    print("RESULTS SUMMARY")
    print("="*70)

    print("\nMulti-Constraint (ubvec=5.0):")
    print(stats_multi.to_string(index=False))
    mm_count_multi = stats_multi['is_mm'].sum()
    print(f"\n  MM Districts: {mm_count_multi}/7 (Target: 2)")
    print(f"  Max Minority %: {stats_multi['minority_pct'].max()*100:.1f}%")

    print("\nEdge-Weighted (5x @ 40%):")
    print(stats_edge.to_string(index=False))
    mm_count_edge = stats_edge['is_mm'].sum()
    print(f"\n  MM Districts: {mm_count_edge}/7 (Target: 2)")
    print(f"  Max Minority %: {stats_edge['minority_pct'].max()*100:.1f}%")

    print("\n" + "="*70)
    print("DONE!")
    print("="*70)

if __name__ == '__main__':
    main()
