#!/usr/bin/env python3
"""
Visualize a redistricting split on a map.

Example usage:
    python scripts/visualize_split.py --state CA --tracts data/raw/ca_tracts_2020.parquet --split-results split_assignments.pkl
"""

import argparse
import pickle
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent.parent / 'src'))

import geopandas as gpd
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import numpy as np


def visualize_split(
    tracts_file: str,
    assignments: dict,
    output_file: str = None,
    title: str = "Redistricting Split",
    dpi: int = 150
):
    """
    Visualize redistricting split on a map.

    Parameters
    ----------
    tracts_file : str
        Path to tracts GeoDataFrame
    assignments : dict
        tract_idx -> partition_id mapping
    output_file : str
        Output image file
    title : str
        Map title
    """
    print(f"Loading tracts from {tracts_file}...")
    tracts_gdf = gpd.read_parquet(tracts_file)

    # Add partition assignments
    tracts_gdf['partition'] = tracts_gdf.index.map(assignments)

    # Count tracts and population per partition
    stats = tracts_gdf.groupby('partition').agg({
        'GEOID': 'count',
        'population': 'sum'
    }).rename(columns={'GEOID': 'n_tracts'})

    print(f"\nPartition Statistics:")
    for part_id, row in stats.iterrows():
        print(f"  Partition {part_id}: {row['n_tracts']:,} tracts, {row['population']:,} population")

    # Create map
    fig, ax = plt.subplots(1, 1, figsize=(12, 10))

    # Define colors
    colors = ['#e74c3c', '#3498db', '#2ecc71', '#f39c12', '#9b59b6']
    n_partitions = tracts_gdf['partition'].nunique()

    # Plot each partition
    for i, part_id in enumerate(sorted(tracts_gdf['partition'].unique())):
        partition_data = tracts_gdf[tracts_gdf['partition'] == part_id]
        color = colors[i % len(colors)]

        partition_data.plot(
            ax=ax,
            color=color,
            edgecolor='white',
            linewidth=0.1,
            alpha=0.7
        )

    # Add thick partition boundaries on top
    partitions_dissolved = tracts_gdf.dissolve(by='partition', as_index=False)
    partitions_dissolved.boundary.plot(
        ax=ax,
        edgecolor='black',
        linewidth=1.5,
        zorder=10
    )

    # Remove axes
    ax.set_axis_off()

    # Add title
    ax.set_title(title, fontsize=16, fontweight='bold', pad=20)

    # Create legend
    legend_elements = []
    for i, part_id in enumerate(sorted(tracts_gdf['partition'].unique())):
        color = colors[i % len(colors)]
        pop = stats.loc[part_id, 'population']
        n_tracts = stats.loc[part_id, 'n_tracts']
        label = f"Partition {part_id}: {pop:,} people ({n_tracts:,} tracts)"
        legend_elements.append(mpatches.Patch(facecolor=color, edgecolor='black', label=label))

    ax.legend(
        handles=legend_elements,
        loc='lower right',
        frameon=True,
        fancybox=True,
        shadow=True,
        fontsize=10
    )

    plt.tight_layout()

    # Save
    if output_file:
        print(f"\nSaving map to {output_file}...")
        plt.savefig(output_file, dpi=dpi, bbox_inches='tight')
        print(f"Map saved!")
    else:
        plt.show()

    plt.close()


def main():
    parser = argparse.ArgumentParser(description='Visualize redistricting split')
    parser.add_argument('--state', type=str, required=True, help='State code')
    parser.add_argument('--tracts', type=str, required=True, help='Tracts parquet file')
    parser.add_argument('--assignments', type=str, required=True, help='Assignments pickle file')
    parser.add_argument('--output', type=str, help='Output image file')
    parser.add_argument('--title', type=str, default='Redistricting Split', help='Map title')
    parser.add_argument('--dpi', type=int, default=150, help='DPI for output maps')

    args = parser.parse_args()

    # Load assignments
    print(f"Loading assignments from {args.assignments}...")
    with open(args.assignments, 'rb') as f:
        assignments = pickle.load(f)

    try:
        visualize_split(args.tracts, assignments, args.output, args.title, args.dpi)
        return 0
    except Exception as e:
        print(f"\nError: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == '__main__':
    sys.exit(main())
