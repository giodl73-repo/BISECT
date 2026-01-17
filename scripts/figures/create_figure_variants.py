#!/usr/bin/env python3
"""
Generate figure variants for educational materials.

Supports flexible display options:
- Show map only, graph only, or both
- Show boundary labels: all edges, cut edges only, or no labels
- Show original (before partition) or split (after partition)

Usage:
    python create_figure_variants.py --city minneapolis --panels both --boundary-labels cut --partition after
    python create_figure_variants.py --city minneapolis --panels map --boundary-labels none --partition before
"""

import argparse
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.gridspec import GridSpec
from matplotlib.patches import Circle
import geopandas as gpd
import pandas as pd
from pathlib import Path
from collections import deque
import warnings
warnings.filterwarnings('ignore')


# Example configurations
EXAMPLE_CONFIGS = {
    'minneapolis_50_50': {
        'title': 'Minneapolis, MN',
        'subtitle': '50-50 Split',
        'state_fips': '27',
        'state_name': 'mn',
        'county_fips': '053',  # Hennepin County
        'nparts': 2,
        'target_weights': [0.5, 0.5],
        'n_tracts': 12,
        'start_tract_idx': 0
    },
    'minneapolis_67_33': {
        'title': 'Minneapolis, MN',
        'subtitle': '67-33 Split (2:1)',
        'state_fips': '27',
        'state_name': 'mn',
        'county_fips': '053',  # Hennepin County
        'nparts': 2,
        'target_weights': [2/3, 1/3],
        'n_tracts': 12,
        'start_tract_idx': 0
    },
    'minneapolis': {  # Keep for backward compatibility
        'title': 'Minneapolis, MN',
        'subtitle': '50-50 Split',
        'state_fips': '27',
        'state_name': 'mn',
        'county_fips': '053',  # Hennepin County
        'nparts': 2,
        'target_weights': [0.5, 0.5],
        'n_tracts': 12,
        'start_tract_idx': 0
    },
    'houston': {
        'title': 'Houston, TX',
        'subtitle': '60-40 Split',
        'state_fips': '48',
        'state_name': 'tx',
        'county_fips': '201',  # Harris County
        'nparts': 2,
        'target_weights': [0.6, 0.4],
        'n_tracts': 12,
        'start_tract_idx': 100
    },
    'los_angeles': {
        'title': 'Los Angeles, CA',
        'subtitle': '43-57 Split',
        'state_fips': '06',
        'state_name': 'ca',
        'county_fips': '037',  # Los Angeles County
        'nparts': 2,
        'target_weights': [3/7, 4/7],
        'n_tracts': 12,
        'start_tract_idx': 50
    }
}


def find_contiguous_tracts_bfs(tracts_gdf, start_idx, n_tracts):
    """Use BFS to find n_tracts contiguous tracts."""
    def get_neighbors(tract_idx, gdf):
        """Get indices of adjacent tracts."""
        neighbors = []
        tract_geom = gdf.loc[tract_idx, 'geometry']
        for idx in gdf.index:
            if idx != tract_idx:
                if tract_geom.touches(gdf.loc[idx, 'geometry']):
                    neighbors.append(idx)
        return neighbors

    selected_indices = [start_idx]
    queue = deque([start_idx])
    visited = {start_idx}

    while len(selected_indices) < n_tracts and queue:
        current = queue.popleft()
        neighbors = get_neighbors(current, tracts_gdf)

        for neighbor in neighbors:
            if neighbor not in visited:
                visited.add(neighbor)
                selected_indices.append(neighbor)
                queue.append(neighbor)

                if len(selected_indices) >= n_tracts:
                    break

    return selected_indices


def build_adjacency_graph(sample_tracts):
    """Build adjacency graph with real boundary lengths."""
    n_tracts = len(sample_tracts)
    adjacency = {i: [] for i in range(n_tracts)}
    edge_weights = {}

    for i in range(n_tracts):
        geom_i = sample_tracts.iloc[i].geometry

        for j in range(i + 1, n_tracts):
            geom_j = sample_tracts.iloc[j].geometry

            if geom_i.touches(geom_j):
                boundary = geom_i.intersection(geom_j.boundary)
                if not boundary.is_empty:
                    length_km = boundary.length / 1000

                    # Handle unprojected coordinates
                    if length_km < 0.1:
                        length_km = boundary.length * 111

                    # Filter corner adjacencies
                    if length_km >= 0.1:
                        adjacency[i].append(j)
                        adjacency[j].append(i)
                        edge_weights[(i, j)] = length_km
                        edge_weights[(j, i)] = length_km

    return adjacency, edge_weights


def export_metis_data(sample_tracts, adjacency, edge_weights, membership, config, output_path):
    """Export METIS input/output data to text file for documentation.

    Creates a text file with:
    - Tract populations
    - Adjacency structure
    - Edge weights (boundary lengths)
    - Partition assignments
    - Summary statistics
    """
    n_tracts = len(sample_tracts)

    # Calculate partition totals
    pop_by_partition = {}
    for i in range(n_tracts):
        part = membership[i]
        pop = sample_tracts.iloc[i]['population']
        pop_by_partition[part] = pop_by_partition.get(part, 0) + pop

    # Count edges
    n_edges = sum(len(neighbors) for neighbors in adjacency.values()) // 2

    with open(output_path, 'w') as f:
        f.write(f"# METIS Data Export: {config['title']} - {config['subtitle']}\n")
        f.write(f"# {n_tracts} tracts, {n_edges} edges\n")
        f.write("#\n")

        # Summary
        f.write("# SUMMARY\n")
        f.write(f"# Total population: {sum(sample_tracts['population']):,}\n")
        for part in sorted(pop_by_partition.keys()):
            pct = pop_by_partition[part] / sum(sample_tracts['population']) * 100
            f.write(f"# Partition {part}: {pop_by_partition[part]:,} ({pct:.1f}%)\n")
        f.write("#\n")

        # Tract populations
        f.write("\n# TRACT POPULATIONS\n")
        f.write("# Tract_ID  Population  Partition\n")
        for i in range(n_tracts):
            pop = int(sample_tracts.iloc[i]['population'])
            part = membership[i]
            f.write(f"{i+1:2d}        {pop:6d}      {part}\n")

        # Adjacency (unweighted format 001)
        f.write("\n\n# METIS INPUT FORMAT 001 (UNWEIGHTED)\n")
        f.write(f"# Header: {n_tracts} {n_edges} 001\n")
        f.write(f"{n_tracts} {n_edges} 001\n")
        for i in range(n_tracts):
            pop = int(sample_tracts.iloc[i]['population'])
            neighbors = sorted(adjacency.get(i, []))
            # Convert to 1-indexed for METIS
            neighbors_str = ' '.join(str(n+1) for n in neighbors)
            f.write(f"{pop} {neighbors_str}\n")

        # Adjacency with edge weights (format 011)
        f.write("\n\n# METIS INPUT FORMAT 011 (EDGE-WEIGHTED)\n")
        f.write(f"# Header: {n_tracts} {n_edges} 011\n")
        f.write(f"{n_tracts} {n_edges} 011\n")
        for i in range(n_tracts):
            pop = int(sample_tracts.iloc[i]['population'])
            neighbors = sorted(adjacency.get(i, []))
            # Build alternating neighbor_id edge_weight pairs
            pairs = []
            for n in neighbors:
                weight = int(edge_weights.get((i, n), 0) * 1000)  # Convert km to meters
                pairs.append(f"{n+1} {weight}")
            pairs_str = ' '.join(pairs)
            f.write(f"{pop} {pairs_str}\n")

        # Partition output
        f.write("\n\n# METIS OUTPUT (PARTITION ASSIGNMENT)\n")
        f.write("# One line per tract, value is partition ID (0 or 1)\n")
        for i in range(n_tracts):
            f.write(f"{membership[i]}\n")

        # Edge weights reference
        f.write("\n\n# EDGE WEIGHTS (BOUNDARY LENGTHS IN KILOMETERS)\n")
        f.write("# Tract_I  Tract_J  Length_km\n")
        for (i, j) in sorted(edge_weights.keys()):
            if i < j:  # Only list each edge once
                length_km = edge_weights[(i, j)]
                f.write(f"{i+1:2d}       {j+1:2d}       {length_km:.2f}\n")


def run_metis_partition(sample_tracts, adjacency, edge_weights, config, use_edge_weights=True):
    """Run METIS to partition tracts.

    Args:
        use_edge_weights: If True, use real boundary lengths as edge weights.
                         If False, run unweighted (all edges equal).
    """
    import sys
    sys.path.insert(0, str(Path('src').resolve()))
    from apportionment.partition.metis_wrapper import partition_graph

    n_tracts = len(sample_tracts)
    adjacency_list = [adjacency.get(i, []) for i in range(n_tracts)]
    vweights = np.array([int(sample_tracts.iloc[i]['population']) for i in range(n_tracts)])

    # Only use edge weights if requested
    if use_edge_weights:
        edge_weights_dict = {}
        for (i, j), weight in edge_weights.items():
            if i < j:
                edge_weights_dict[(i, j)] = weight
    else:
        edge_weights_dict = None

    membership = partition_graph(
        adjacency=adjacency_list,
        vertex_weights=vweights,
        nparts=config['nparts'],
        target_weights=config['target_weights'],
        recursive=True,
        ufactor=1.005,
        edge_weights=edge_weights_dict,
        debug=False
    )

    return membership


def create_figure_variant(sample_tracts, adjacency, edge_weights, membership, config,
                          panels='both', boundary_labels='cut', show_partition=True, output_path=None):
    """
    Create figure with flexible display options.

    Args:
        sample_tracts: GeoDataFrame with census tracts
        adjacency: Adjacency dictionary
        edge_weights: Edge weights dictionary
        membership: Partition membership array (ignored if show_partition=False)
        config: Configuration dictionary
        panels: 'map', 'graph', or 'both'
        boundary_labels: 'all' (label all edges), 'cut' (label cut edges only), or 'none' (no labels)
        show_partition: If False, show original without partition
        output_path: Where to save the figure
    """
    n_tracts = len(sample_tracts)
    labels = [str(i + 1) for i in range(n_tracts)]  # 1, 2, 3, ...

    # Identify cut edges (only if showing partition)
    cut_edges = set()
    total_cut_length = 0.0
    if show_partition:
        for i in range(n_tracts):
            for j in adjacency.get(i, []):
                if i < j and membership[i] != membership[j]:
                    cut_edges.add((i, j))
                    total_cut_length += edge_weights.get((i, j), 0)

    # Colors
    if show_partition:
        partition_colors = {0: 'lightblue', 1: 'lightcoral'}
    else:
        partition_colors = {0: 'lightgray', 1: 'lightgray'}  # All same color if no partition

    # Determine figure layout
    if panels == 'both':
        fig = plt.figure(figsize=(16, 7))
        gs = fig.add_gridspec(1, 2, width_ratios=[1.4, 1], wspace=0.15)
        show_map = True
        show_graph = True
        ax_map_idx = 0
        ax_graph_idx = 1
    elif panels == 'map':
        fig = plt.figure(figsize=(10, 8))
        gs = fig.add_gridspec(1, 1)
        show_map = True
        show_graph = False
        ax_map_idx = 0
    elif panels == 'graph':
        fig = plt.figure(figsize=(8, 8))
        gs = fig.add_gridspec(1, 1)
        show_map = False
        show_graph = True
        ax_graph_idx = 0
    else:
        raise ValueError(f"Invalid panels option: {panels}")

    # Title
    if show_partition:
        title = f"{config['title']} - {config['subtitle']}"
    else:
        title = f"{config['title']} - Original"
    fig.suptitle(title, fontsize=15, fontweight='bold', y=0.98)

    # MAP PANEL
    if show_map:
        ax_map = fig.add_subplot(gs[ax_map_idx])

        # Plot tracts
        for idx in range(n_tracts):
            if show_partition:
                color = partition_colors[membership[idx]]
            else:
                color = 'lightgray'

            sample_tracts.iloc[[idx]].plot(
                ax=ax_map,
                facecolor=color,
                edgecolor='black',
                linewidth=1.5,
                alpha=0.7
            )

            # Label tract
            centroid = sample_tracts.iloc[idx].geometry.centroid
            pop_k = sample_tracts.iloc[idx]['population'] / 1000
            ax_map.text(centroid.x, centroid.y, f'{labels[idx]}\n{pop_k:.1f}K',
                    ha='center', va='center', fontsize=10, fontweight='bold',
                    bbox=dict(boxstyle='round', facecolor='white',
                            edgecolor='black', linewidth=1))

        # Show all boundaries (structure always visible)
        # Labels controlled by boundary_labels parameter
        for i in range(n_tracts):
            for j in adjacency.get(i, []):
                if i < j:
                    is_cut = (i, j) in cut_edges

                    geom_i = sample_tracts.iloc[i].geometry
                    geom_j = sample_tracts.iloc[j].geometry
                    boundary = geom_i.intersection(geom_j.boundary)
                    if not boundary.is_empty:
                        length_km = edge_weights.get((i, j), 0)

                        if is_cut and show_partition:
                            # Highlight cut boundaries
                            gpd.GeoSeries([boundary]).plot(ax=ax_map, color='red', linewidth=4, alpha=0.9, zorder=10)
                            # Show label based on boundary_labels setting
                            if boundary_labels != 'none':
                                mid_point = boundary.centroid
                                if length_km > 0.1:
                                    ax_map.text(mid_point.x, mid_point.y, f'{length_km:.1f}',
                                           ha='center', va='center', fontsize=8, fontweight='bold',
                                           bbox=dict(boxstyle='round', facecolor='yellow',
                                                   edgecolor='red', linewidth=1.5, alpha=0.9),
                                           zorder=11)
                        else:
                            # Non-cut boundaries (or no partition shown)
                            # Draw boundary line (always)
                            gpd.GeoSeries([boundary]).plot(ax=ax_map, color='gray', linewidth=1, alpha=0.5, zorder=5)
                            # Show label only if boundary_labels='all'
                            if boundary_labels == 'all':
                                mid_point = boundary.centroid
                                if length_km > 0.1:
                                    ax_map.text(mid_point.x, mid_point.y, f'{length_km:.1f}',
                                           ha='center', va='center', fontsize=8,
                                           bbox=dict(boxstyle='round', facecolor='white',
                                                   edgecolor='black', linewidth=0.5, alpha=0.8))

        # Region labels (only if showing partition)
        if show_partition:
            pop0 = sample_tracts[membership == 0]['population'].sum() / 1000
            pop1 = sample_tracts[membership == 1]['population'].sum() / 1000
            total_pop = pop0 + pop1
            pct0 = (pop0 * 1000 / (total_pop * 1000)) * 100
            pct1 = (pop1 * 1000 / (total_pop * 1000)) * 100

            combined_bounds = sample_tracts.geometry.total_bounds
            map_height = combined_bounds[3] - combined_bounds[1]
            map_width = combined_bounds[2] - combined_bounds[0]

            part0_geoms = sample_tracts[membership == 0].geometry
            part1_geoms = sample_tracts[membership == 1].geometry

            # Calculate Polsby-Popper compactness for each region (needed for labels and average)
            # PP = 4π × area / perimeter²
            pp0 = 0
            pp1 = 0
            if len(part0_geoms) > 0 and len(part1_geoms) > 0:
                region0_union = part0_geoms.unary_union
                region1_union = part1_geoms.unary_union
                centroid0 = region0_union.centroid
                centroid1 = region1_union.centroid

                area0 = region0_union.area
                perimeter0 = region0_union.length
                pp0 = (4 * np.pi * area0) / (perimeter0 ** 2) if perimeter0 > 0 else 0

                area1 = region1_union.area
                perimeter1 = region1_union.length
                pp1 = (4 * np.pi * area1) / (perimeter1 ** 2) if perimeter1 > 0 else 0

                h_separation = abs(centroid1.x - centroid0.x)
                v_separation = abs(centroid1.y - centroid0.y)

                if h_separation > v_separation:
                    label_y = combined_bounds[3] + map_height * 0.10
                    ax_map.text(centroid0.x, label_y, f'Region 1\n{pop0:.1f}K\n{pct0:.1f}%\nPP: {pp0:.3f}',
                            ha='center', va='bottom', fontsize=12, fontweight='bold',
                            bbox=dict(boxstyle='round', facecolor='lightblue',
                                    edgecolor='blue', linewidth=1.5, alpha=0.9))
                    ax_map.text(centroid1.x, label_y, f'Region 2\n{pop1:.1f}K\n{pct1:.1f}%\nPP: {pp1:.3f}',
                            ha='center', va='bottom', fontsize=12, fontweight='bold',
                            bbox=dict(boxstyle='round', facecolor='lightcoral',
                                    edgecolor='red', linewidth=1.5, alpha=0.9))
                else:
                    label_x0 = combined_bounds[2] + map_width * 0.10
                    label_x1 = combined_bounds[0] - map_width * 0.10
                    ax_map.text(label_x0, centroid0.y, f'Region 1\n{pop0:.1f}K\n{pct0:.1f}%\nPP: {pp0:.3f}',
                            ha='center', va='center', fontsize=12, fontweight='bold',
                            bbox=dict(boxstyle='round', facecolor='lightblue',
                                    edgecolor='blue', linewidth=1.5, alpha=0.9))
                    ax_map.text(label_x1, centroid1.y, f'Region 2\n{pop1:.1f}K\n{pct1:.1f}%\nPP: {pp1:.3f}',
                            ha='center', va='center', fontsize=12, fontweight='bold',
                            bbox=dict(boxstyle='round', facecolor='lightcoral',
                                    edgecolor='red', linewidth=1.5, alpha=0.9))

        ax_map.axis('off')
        if panels == 'map':
            ax_map.set_title('Real Census Tracts', fontsize=12, fontweight='bold')

        # Add metrics label to map panel (if showing partition)
        if show_partition:
            # Calculate population-weighted average compactness
            pop0_raw = sample_tracts[membership == 0]['population'].sum()
            pop1_raw = sample_tracts[membership == 1]['population'].sum()
            total_pop_raw = pop0_raw + pop1_raw
            avg_pp = (pp0 * pop0_raw + pp1 * pop1_raw) / total_pop_raw if total_pop_raw > 0 else 0

            # Calculate population-weighted standard deviation
            std_dev_pp = np.sqrt(((pp0 - avg_pp)**2 * pop0_raw + (pp1 - avg_pp)**2 * pop1_raw) / total_pop_raw) if total_pop_raw > 0 else 0

            xlim = ax_map.get_xlim()
            ylim = ax_map.get_ylim()
            label_x = (xlim[0] + xlim[1]) / 2
            label_y = ylim[0] - (ylim[1] - ylim[0]) * 0.08

            # Include total cut only if boundary labels are shown
            if boundary_labels != 'none':
                label_text = f'Total Cut: {total_cut_length:.1f} km\nAve PP: {avg_pp:.3f}\nStd Dev PP: {std_dev_pp:.3f}'
            else:
                label_text = f'Ave PP: {avg_pp:.3f}\nStd Dev PP: {std_dev_pp:.3f}'

            ax_map.text(label_x, label_y, label_text,
                       ha='center', va='top', fontsize=11, fontweight='bold',
                       bbox=dict(boxstyle='round', facecolor='yellow',
                                edgecolor='red', linewidth=1.5, alpha=0.9))

    # GRAPH PANEL
    if show_graph:
        ax_graph = fig.add_subplot(gs[ax_graph_idx])

        # Node positions from centroids
        centroids = sample_tracts.geometry.centroid
        xs = [c.x for c in centroids]
        ys = [c.y for c in centroids]

        x_min, x_max = min(xs), max(xs)
        y_min, y_max = min(ys), max(ys)
        positions = {}
        for i in range(n_tracts):
            x_norm = 4 * (xs[i] - x_min) / (x_max - x_min) if x_max > x_min else 2
            y_norm = 4 * (ys[i] - y_min) / (y_max - y_min) if y_max > y_min else 2
            positions[i] = (x_norm, y_norm)

        # Draw all edges (structure always visible)
        # Labels controlled by boundary_labels parameter
        for i in range(n_tracts):
            for j in adjacency.get(i, []):
                if i < j:
                    x1, y1 = positions[i]
                    x2, y2 = positions[j]
                    length_km = edge_weights.get((i, j), 0)

                    if length_km > 0.1:
                        is_cut = (i, j) in cut_edges

                        # Always draw edge (no skipping)
                        if is_cut and show_partition:
                            ax_graph.plot([x1, x2], [y1, y2], 'r--',
                                   linewidth=4, alpha=0.9, zorder=2)
                            mid_x, mid_y = (x1 + x2) / 2, (y1 + y2) / 2
                            ax_graph.plot(mid_x, mid_y, 'rX', markersize=12,
                                   markeredgewidth=3, zorder=3)
                        else:
                            thickness = min(6, max(1.5, length_km / 3))
                            ax_graph.plot([x1, x2], [y1, y2], 'k-',
                                   linewidth=thickness, alpha=0.4, zorder=1)

                        # Show labels based on boundary_labels parameter
                        show_label = False
                        if boundary_labels == 'all':
                            show_label = True
                        elif boundary_labels == 'cut' and is_cut and show_partition:
                            show_label = True

                        if show_label:
                            mid_x, mid_y = (x1 + x2) / 2, (y1 + y2) / 2
                            dx, dy = x2 - x1, y2 - y1
                            edge_length = np.sqrt(dx**2 + dy**2)
                            if edge_length > 0:
                                offset_x = -dy / edge_length * 0.2
                                offset_y = dx / edge_length * 0.2
                            else:
                                offset_x, offset_y = 0, 0

                            label_x = mid_x + offset_x
                            label_y = mid_y + offset_y

                            if is_cut and show_partition:
                                label_style = dict(boxstyle='round', facecolor='yellow',
                                                 edgecolor='red', linewidth=1.5, alpha=0.9)
                                fontweight = 'bold'
                            else:
                                label_style = dict(boxstyle='round', facecolor='white',
                                                 edgecolor='gray', linewidth=0.5, alpha=0.8)
                                fontweight = 'normal'

                            ax_graph.text(label_x, label_y, f'{length_km:.1f}',
                                   ha='center', va='center', fontsize=10,
                                   fontweight=fontweight, bbox=label_style)

        # Draw nodes
        for i in range(n_tracts):
            x, y = positions[i]
            if show_partition:
                color = partition_colors[membership[i]]
            else:
                color = 'lightgray'
            pop_k = sample_tracts.iloc[i]['population'] / 1000

            circle = Circle((x, y), 0.3, facecolor=color,
                           edgecolor='black', linewidth=2.5, alpha=0.8, zorder=4)
            ax_graph.add_patch(circle)

            ax_graph.text(x, y + 0.06, labels[i], ha='center', va='center',
                    fontsize=10, fontweight='bold', zorder=5)
            ax_graph.text(x, y - 0.09, f'{pop_k:.1f}K', ha='center', va='center',
                    fontsize=10, fontweight='bold', color='black', zorder=5)

        # Region labels (only if showing partition and graph only)
        # Skip when both map and graph are shown (map already has region labels)
        if show_partition and panels == 'graph':
            pop0_total = sample_tracts[membership == 0]['population'].sum() / 1000
            pop1_total = sample_tracts[membership == 1]['population'].sum() / 1000

            part0_positions = [positions[i] for i in range(n_tracts) if membership[i] == 0]
            part1_positions = [positions[i] for i in range(n_tracts) if membership[i] == 1]

            if part0_positions:
                xs0 = [p[0] for p in part0_positions]
                ys0 = [p[1] for p in part0_positions]
                label_x0 = sum(xs0) / len(xs0)
                label_y0 = max(ys0) + 0.5
                ax_graph.text(label_x0, label_y0, f'Region 1\n{pop0_total:.1f}K',
                        ha='center', va='bottom', fontsize=12, fontweight='bold',
                        bbox=dict(boxstyle='round', facecolor='lightblue',
                                edgecolor='blue', linewidth=1.5, alpha=0.9))

            if part1_positions:
                xs1 = [p[0] for p in part1_positions]
                ys1 = [p[1] for p in part1_positions]
                label_x1 = sum(xs1) / len(xs1)
                label_y1 = max(ys1) + 0.5
                ax_graph.text(label_x1, label_y1, f'Region 2\n{pop1_total:.1f}K',
                        ha='center', va='bottom', fontsize=12, fontweight='bold',
                        bbox=dict(boxstyle='round', facecolor='lightcoral',
                                edgecolor='red', linewidth=1.5, alpha=0.9))

        ax_graph.set_xlim(-0.5, 4.8)
        ax_graph.set_ylim(-0.7, 5.0)
        ax_graph.set_aspect('equal')
        ax_graph.axis('off')

        # Add cut label to graph panel (if showing partition and labels enabled)
        if show_partition and boundary_labels != 'none':
            ax_graph.text(2.15, -0.5, f'Total Cut: {total_cut_length:.1f} km',
                         ha='center', va='top', fontsize=11, fontweight='bold',
                         bbox=dict(boxstyle='round', facecolor='yellow',
                                  edgecolor='red', linewidth=1.5, alpha=0.9))

    # Save figure
    if output_path:
        plt.savefig(output_path, dpi=150, bbox_inches='tight',
                   facecolor='white', edgecolor='none')
        print(f"[OK] Created: {output_path}")

    plt.close()


def load_example_data(city, year, use_edge_weights=True):
    """Load census tract data for specified city.

    Args:
        use_edge_weights: If True, use edge-weighted partitioning.
                         If False, use unweighted partitioning.
    """
    if city not in EXAMPLE_CONFIGS:
        raise ValueError(f"Unknown city: {city}. Available: {list(EXAMPLE_CONFIGS.keys())}")

    config = EXAMPLE_CONFIGS[city]
    year_suffix = str(year)[-2:]
    state_fips = config['state_fips']
    state_name = config['state_name']
    county_fips = config['county_fips']

    # File paths (relative to repo root)
    tracts_file = Path(f'data/geography/tiger_{year}_tracts/tl_{year}_{state_fips}_tract{year_suffix}/tl_{year}_{state_fips}_tract{year_suffix}.shp')
    population_file = Path(f'data/processed/census_{year}/{state_name}_tracts_{year}_population.csv')

    geoid_field = f'GEOID{year_suffix}'
    county_field = f'COUNTYFP{year_suffix}'

    # Check files exist
    if not tracts_file.exists():
        print(f"[SKIP] Tracts shapefile not found: {tracts_file}")
        print(f"       Try running with --year 2010 if 2020 data is not available")
        return None, None, None, None, None
    if not population_file.exists():
        print(f"[SKIP] Population data not found: {population_file}")
        return None, None, None, None, None

    # Load data
    tracts_gdf = gpd.read_file(tracts_file)
    pop_df = pd.read_csv(population_file)

    pop_dict = dict(zip(pop_df['GEOID'].astype(str).str.zfill(11),
                       pop_df['population']))

    tracts_gdf[geoid_field] = tracts_gdf[geoid_field].astype(str).str.zfill(11)
    tracts_gdf['population'] = tracts_gdf[geoid_field].map(pop_dict)

    # Filter to county
    county_tracts = tracts_gdf[tracts_gdf[county_field] == county_fips].copy()
    county_tracts = county_tracts[county_tracts['population'].notna()]

    if len(county_tracts) < config['n_tracts']:
        raise ValueError(f"Not enough tracts in county ({len(county_tracts)} < {config['n_tracts']})")

    # Find contiguous tracts
    start_idx = county_tracts.index[min(config['start_tract_idx'], len(county_tracts) - 1)]
    selected_indices = find_contiguous_tracts_bfs(county_tracts, start_idx, config['n_tracts'])

    if len(selected_indices) < config['n_tracts']:
        raise ValueError(f"Could not find {config['n_tracts']} contiguous tracts")

    sample_tracts = county_tracts.loc[selected_indices].copy().reset_index(drop=True)

    # Build adjacency
    adjacency, edge_weights = build_adjacency_graph(sample_tracts)

    # Run METIS
    membership = run_metis_partition(sample_tracts, adjacency, edge_weights, config, use_edge_weights)

    return sample_tracts, adjacency, edge_weights, membership, config


def main():
    parser = argparse.ArgumentParser(
        description='Generate figure variants with flexible display options'
    )
    parser.add_argument('--city', type=str, required=True,
                       choices=list(EXAMPLE_CONFIGS.keys()),
                       help='City to generate example for')
    parser.add_argument('--year', type=int, default=2010,
                       choices=[2000, 2010, 2020],
                       help='Census year (default: 2010)')
    parser.add_argument('--panels', type=str, default='both',
                       choices=['map', 'graph', 'both'],
                       help='Which panels to show (default: both)')
    parser.add_argument('--boundary-labels', type=str, default='cut',
                       choices=['all', 'cut', 'none'],
                       help='Which boundary labels to show: all edges, cut edges only, or none (default: cut)')
    parser.add_argument('--partition', type=str, default='after',
                       choices=['before', 'after'],
                       help='Show before or after partition (default: after)')
    parser.add_argument('--partition-mode', type=str, default='edgeweighted',
                       choices=['edgeweighted', 'unweighted'],
                       help='Partitioning mode: edgeweighted (use boundary lengths) or unweighted (all edges equal) (default: edgeweighted)')
    parser.add_argument('--output', type=str, default=None,
                       help='Output file path (default: auto-generated)')
    args = parser.parse_args()

    # Change to repository root (in case we're running from elsewhere)
    script_dir = Path(__file__).resolve().parent
    repo_root = script_dir.parent.parent
    import os
    os.chdir(repo_root)

    print("=" * 70)
    print("Generating Figure Variant")
    print("=" * 70)
    print(f"City: {args.city}")
    print(f"Year: {args.year}")
    print(f"Panels: {args.panels}")
    print(f"Boundary labels: {args.boundary_labels}")
    print(f"Partition: {args.partition}")
    print(f"Partition mode: {args.partition_mode}")
    print()

    # Get config to extract ratio
    if args.city not in EXAMPLE_CONFIGS:
        print(f"[ERROR] Unknown city: {args.city}")
        return 1

    temp_config = EXAMPLE_CONFIGS[args.city]
    # Format ratio from target_weights (e.g., [0.5, 0.5] -> "50_50", [2/3, 1/3] -> "67_33")
    w0 = int(round(temp_config['target_weights'][0] * 100))
    w1 = int(round(temp_config['target_weights'][1] * 100))
    ratio_str = f"{w0}_{w1}"

    # Extract base city name (strip ratio if present in city key)
    base_city = args.city.split('_')[0] if '_' in args.city else args.city

    # Generate output filename
    if args.output is None:
        output_dir = Path('outputs/artifacts/figures/real_tracts_examples')
        output_dir.mkdir(parents=True, exist_ok=True)
        filename = f"{base_city}_{ratio_str}_{args.panels}_{args.boundary_labels}_{args.partition}_{args.partition_mode}.png"
        output_path = output_dir / filename
    else:
        output_path = Path(args.output)
        output_path.parent.mkdir(parents=True, exist_ok=True)

    # Skip if file already exists
    if output_path.exists():
        print(f"[SKIP] File already exists: {output_path}")
        print()
        print("=" * 70)
        print(f"Skipped (already exists)")
        print("=" * 70)
        print(f"Location: {output_path.resolve()}")
        print()
        return 0

    # Load data
    print("Loading data...")
    use_edge_weights = (args.partition_mode == 'edgeweighted')
    sample_tracts, adjacency, edge_weights, membership, config = load_example_data(args.city, args.year, use_edge_weights)

    # Check if data loading failed
    if sample_tracts is None:
        print("\n[SKIP] Cannot generate figure - data not available")
        return 1

    print(f"[OK] Loaded {len(sample_tracts)} census tracts")

    # Create figure
    print("Creating figure...")
    show_partition = (args.partition == 'after')
    create_figure_variant(
        sample_tracts, adjacency, edge_weights, membership, config,
        panels=args.panels,
        boundary_labels=args.boundary_labels,
        show_partition=show_partition,
        output_path=output_path
    )

    # Export METIS data to text file
    data_output_path = output_path.with_suffix('.txt')
    print(f"Exporting METIS data to: {data_output_path.name}...")
    export_metis_data(sample_tracts, adjacency, edge_weights, membership, config, data_output_path)

    print()
    print("=" * 70)
    print(f"Complete!")
    print("=" * 70)
    print(f"Figure saved to: {output_path.resolve()}")
    print(f"Data saved to:   {data_output_path.resolve()}")
    print()


if __name__ == '__main__':
    main()
