"""
District map visualization using matplotlib and geopandas.

Creates static maps with color-coded districts, boundaries, and labels.
"""

from pathlib import Path
from typing import Optional, Dict

import geopandas as gpd
import matplotlib.pyplot as plt
import matplotlib.colors as mcolors
import numpy as np
import pandas as pd


def create_districts_gdf(
    blocks_gdf: gpd.GeoDataFrame,
    assignments: Dict[int, int],
    index_to_geoid: Dict[int, str]
) -> gpd.GeoDataFrame:
    """
    Create district boundaries by dissolving blocks.

    Parameters
    ----------
    blocks_gdf : gpd.GeoDataFrame
        Block geometries with population
    assignments : Dict[int, int]
        Mapping from block index to district_id
    index_to_geoid : Dict[int, str]
        Mapping from block index to GEOID

    Returns
    -------
    gpd.GeoDataFrame
        District boundaries with aggregated statistics
    """
    print("Creating district boundaries...")

    # Convert assignments to GEOID-based mapping
    geoid_assignments = {
        index_to_geoid[idx]: district_id
        for idx, district_id in assignments.items()
    }

    # Add district assignments to blocks
    blocks_with_districts = blocks_gdf.copy()
    blocks_with_districts['district_id'] = blocks_with_districts['GEOID'].map(geoid_assignments)

    # Remove blocks without assignments (shouldn't happen)
    missing = blocks_with_districts['district_id'].isna().sum()
    if missing > 0:
        print(f"  Warning: {missing} blocks without district assignment")
        blocks_with_districts = blocks_with_districts.dropna(subset=['district_id'])

    blocks_with_districts['district_id'] = blocks_with_districts['district_id'].astype(int)

    # Dissolve blocks by district
    print(f"  Dissolving {len(blocks_with_districts):,} blocks into districts...")
    districts_gdf = blocks_with_districts.dissolve(
        by='district_id',
        aggfunc={
            'population': 'sum',
            'ALAND': 'sum',
            'AWATER': 'sum'
        }
    ).reset_index()

    # Add computed fields
    districts_gdf['total_area'] = districts_gdf['ALAND'] + districts_gdf['AWATER']
    districts_gdf['land_area_km2'] = districts_gdf['ALAND'] / 1e6

    print(f"  Created {len(districts_gdf)} districts")

    return districts_gdf


def plot_districts(
    districts_gdf: gpd.GeoDataFrame,
    output_path: Optional[str] = None,
    title: Optional[str] = None,
    show_labels: bool = True,
    show_population: bool = False,
    figsize: tuple = (20, 16),
    dpi: int = 300
):
    """
    Create colored district map with boundaries and labels.

    Parameters
    ----------
    districts_gdf : gpd.GeoDataFrame
        District boundaries
    output_path : str, optional
        Path to save map image
    title : str, optional
        Map title
    show_labels : bool, default True
        Whether to show district ID labels
    show_population : bool, default False
        Whether to show population in labels
    figsize : tuple, default (20, 16)
        Figure size in inches
    dpi : int, default 300
        Resolution (dots per inch)

    Returns
    -------
    fig, ax
        Matplotlib figure and axis objects
    """
    print(f"\nGenerating district map...")

    # Create figure
    fig, ax = plt.subplots(figsize=figsize, dpi=dpi)

    # Generate colors for districts
    n_districts = len(districts_gdf)
    colors = _generate_district_colors(n_districts)

    # Assign colors to districts
    districts_gdf = districts_gdf.copy()
    districts_gdf['color'] = [colors[i % len(colors)] for i in range(n_districts)]

    # Plot districts
    districts_gdf.plot(
        ax=ax,
        color=districts_gdf['color'],
        edgecolor='black',
        linewidth=0.8,
        alpha=0.7
    )

    # Add district labels
    if show_labels:
        for idx, row in districts_gdf.iterrows():
            centroid = row.geometry.centroid

            if show_population:
                pop_k = row['population'] / 1000
                label = f"{row['district_id']}\n({pop_k:.0f}K)"
                fontsize = 7
            else:
                label = str(row['district_id'])
                fontsize = 9

            ax.text(
                centroid.x, centroid.y,
                label,
                ha='center', va='center',
                fontsize=fontsize,
                fontweight='bold',
                bbox=dict(boxstyle='round,pad=0.3', facecolor='white', edgecolor='none', alpha=0.7)
            )

    # Set title
    if title:
        ax.set_title(title, fontsize=16, fontweight='bold', pad=20)
    else:
        ax.set_title(
            f"Congressional Districts (n={n_districts})\nRecursive Bifurcation Algorithm",
            fontsize=16,
            fontweight='bold',
            pad=20
        )

    # Remove axis ticks
    ax.set_xticks([])
    ax.set_yticks([])

    # Add scale bar (approximate)
    _add_scale_bar(ax, districts_gdf)

    # Tight layout
    plt.tight_layout()

    # Save if output path provided
    if output_path:
        output_path = Path(output_path)
        output_path.parent.mkdir(parents=True, exist_ok=True)

        print(f"Saving map to {output_path}...")
        plt.savefig(output_path, dpi=dpi, bbox_inches='tight')
        print(f"Map saved ({output_path.stat().st_size / 1e6:.1f} MB)")

    return fig, ax


def _generate_district_colors(n_districts: int) -> list:
    """
    Generate distinct colors for districts.

    Uses a combination of qualitative colormaps to ensure
    visual distinction between adjacent districts.

    Parameters
    ----------
    n_districts : int
        Number of districts

    Returns
    -------
    list
        List of color hex codes
    """
    # Use tab20 colormap (20 distinct colors) and extend if needed
    if n_districts <= 20:
        cmap = plt.cm.tab20
        colors = [mcolors.rgb2hex(cmap(i)) for i in range(n_districts)]
    else:
        # Combine multiple colormaps
        cmap1 = plt.cm.tab20
        cmap2 = plt.cm.tab20b
        cmap3 = plt.cm.tab20c

        colors = []
        for i in range(n_districts):
            if i < 20:
                colors.append(mcolors.rgb2hex(cmap1(i)))
            elif i < 40:
                colors.append(mcolors.rgb2hex(cmap2(i - 20)))
            else:
                colors.append(mcolors.rgb2hex(cmap3(i - 40)))

    return colors


def _add_scale_bar(ax, gdf: gpd.GeoDataFrame):
    """
    Add simple scale bar to map.

    Parameters
    ----------
    ax : matplotlib axis
        Axis to add scale bar to
    gdf : gpd.GeoDataFrame
        GeoDataFrame for determining scale
    """
    # Get bounds
    minx, miny, maxx, maxy = gdf.total_bounds

    # Determine scale bar length (10% of map width)
    width = maxx - minx
    scale_length = width * 0.1

    # Convert to km (assuming CRS is in degrees, approximate)
    # For more accurate scale, would need to project to meters
    scale_km = scale_length * 111  # 1 degree ≈ 111 km at equator

    # Round to nice number
    if scale_km > 100:
        scale_km = round(scale_km / 100) * 100
    elif scale_km > 10:
        scale_km = round(scale_km / 10) * 10
    else:
        scale_km = round(scale_km)

    # Scale bar position (bottom right)
    bar_x = maxx - scale_length * 1.5
    bar_y = miny + (maxy - miny) * 0.05

    # Draw scale bar
    ax.plot([bar_x, bar_x + scale_length], [bar_y, bar_y], 'k-', linewidth=3)
    ax.plot([bar_x, bar_x], [bar_y - scale_length * 0.02, bar_y + scale_length * 0.02], 'k-', linewidth=2)
    ax.plot([bar_x + scale_length, bar_x + scale_length],
            [bar_y - scale_length * 0.02, bar_y + scale_length * 0.02], 'k-', linewidth=2)

    # Add label
    ax.text(
        bar_x + scale_length / 2, bar_y - (maxy - miny) * 0.02,
        f"{scale_km:.0f} km",
        ha='center', va='top',
        fontsize=10,
        bbox=dict(boxstyle='round,pad=0.3', facecolor='white', edgecolor='none', alpha=0.8)
    )


def plot_districts_comparison(
    districts_list: list,
    titles: list,
    output_path: Optional[str] = None,
    figsize: tuple = (24, 12),
    dpi: int = 150
):
    """
    Create side-by-side comparison of multiple district maps.

    Useful for comparing different algorithms or parameters.

    Parameters
    ----------
    districts_list : list of gpd.GeoDataFrame
        List of district GeoDataFrames to compare
    titles : list of str
        Title for each map
    output_path : str, optional
        Path to save comparison image
    figsize : tuple, default (24, 12)
        Figure size in inches
    dpi : int, default 150
        Resolution

    Returns
    -------
    fig, axes
        Matplotlib figure and axes
    """
    n_maps = len(districts_list)
    fig, axes = plt.subplots(1, n_maps, figsize=figsize, dpi=dpi)

    if n_maps == 1:
        axes = [axes]

    for i, (districts_gdf, title) in enumerate(zip(districts_list, titles)):
        ax = axes[i]

        # Generate colors
        n_districts = len(districts_gdf)
        colors = _generate_district_colors(n_districts)
        districts_gdf = districts_gdf.copy()
        districts_gdf['color'] = [colors[j % len(colors)] for j in range(n_districts)]

        # Plot
        districts_gdf.plot(
            ax=ax,
            color=districts_gdf['color'],
            edgecolor='black',
            linewidth=0.5,
            alpha=0.7
        )

        ax.set_title(title, fontsize=14, fontweight='bold')
        ax.set_xticks([])
        ax.set_yticks([])

    plt.tight_layout()

    if output_path:
        plt.savefig(output_path, dpi=dpi, bbox_inches='tight')
        print(f"Comparison map saved to {output_path}")

    return fig, axes


def export_district_stats(districts_gdf: gpd.GeoDataFrame, output_path: str):
    """
    Export district statistics to CSV.

    Parameters
    ----------
    districts_gdf : gpd.GeoDataFrame
        District boundaries with statistics
    output_path : str
        Path to save CSV
    """
    stats_df = districts_gdf.drop(columns=['geometry']).copy()

    stats_df.to_csv(output_path, index=False)
    print(f"District statistics saved to {output_path}")
