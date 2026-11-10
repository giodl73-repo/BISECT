#!/usr/bin/env python3
"""
Generate real census tract examples for laymen's guide appendix.

Creates multiple examples showing different states, cities, and partition ratios
to demonstrate how METIS handles various geographic and population distributions.
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
# All ratios use consecutive integers that sum to a prime number
# (otherwise we would have split differently at an earlier level)
EXAMPLES = [
    {
        'name': 'minneapolis_50_50',
        'title': 'Minneapolis, MN',
        'subtitle': '50-50 Split',
        'state_fips': '27',
        'state_name': 'mn',
        'county_fips': '053',  # Hennepin County
        'nparts': 2,
        'target_weights': [0.5, 0.5],  # 1:1
        'n_tracts': 12,
        'start_tract_idx': 0
    },
    {
        'name': 'houston_60_40',
        'title': 'Houston, TX',
        'subtitle': '60-40 Split',
        'state_fips': '48',
        'state_name': 'tx',
        'county_fips': '201',  # Harris County
        'nparts': 2,
        'target_weights': [0.6, 0.4],  # 3:2
        'n_tracts': 12,
        'start_tract_idx': 100
    },
    {
        'name': 'los_angeles_43_57',
        'title': 'Los Angeles, CA',
        'subtitle': '43-57 Split',
        'state_fips': '06',
        'state_name': 'ca',
        'county_fips': '037',  # Los Angeles County
        'nparts': 2,
        'target_weights': [3/7, 4/7],  # 3:4
        'n_tracts': 12,
        'start_tract_idx': 50
    },
    {
        'name': 'atlanta_45_55',
        'title': 'Atlanta, GA',
        'subtitle': '45-55 Split',
        'state_fips': '13',
        'state_name': 'ga',
        'county_fips': '121',  # Fulton County
        'nparts': 2,
        'target_weights': [5/11, 6/11],  # 5:6
        'n_tracts': 12,
        'start_tract_idx': 75
    },
    {
        'name': 'phoenix_46_54',
        'title': 'Phoenix, AZ',
        'subtitle': '46-54 Split',
        'state_fips': '04',
        'state_name': 'az',
        'county_fips': '013',  # Maricopa County
        'nparts': 2,
        'target_weights': [6/13, 7/13],  # 6:7
        'n_tracts': 12,
        'start_tract_idx': 200
    },
    {
        'name': 'miami_47_53',
        'title': 'Miami, FL',
        'subtitle': '47-53 Split',
        'state_fips': '12',
        'state_name': 'fl',
        'county_fips': '086',  # Miami-Dade County
        'nparts': 2,
        'target_weights': [8/17, 9/17],  # 8:9
        'n_tracts': 12,
        'start_tract_idx': 150
    }
]


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


def calculate_compactness(sample_tracts, membership):
    """
    Calculate Polsby-Popper compactness for each region.

    Compactness = (4 * π * area) / (perimeter^2)
    Range: 0 (very irregular) to 1.0 (perfect circle)

    Args:
        sample_tracts: GeoDataFrame with geometry
        membership: Partition membership array

    Returns:
        (compactness0: float, compactness1: float, min_compactness: float)
    """
    import math

    # Dissolve each region
    region0 = sample_tracts[membership == 0].geometry.unary_union
    region1 = sample_tracts[membership == 1].geometry.unary_union

    # Calculate Polsby-Popper compactness for each region
    def polsby_popper(geom):
        area = geom.area
        perimeter = geom.length
        if perimeter == 0:
            return 0.0
        return (4 * math.pi * area) / (perimeter ** 2)

    comp0 = polsby_popper(region0)
    comp1 = polsby_popper(region1)

    return comp0, comp1, min(comp0, comp1)


def validate_partition_ratio(sample_tracts, membership, target_weights, tolerance=0.005, min_compactness=0.25):
    """
    Validate that the partition achieves target ratio within tolerance and has good compactness.

    Args:
        sample_tracts: GeoDataFrame with population data
        membership: Partition membership array
        target_weights: Target weight distribution [w0, w1]
        tolerance: Maximum acceptable deviation from target (default 0.5%)
        min_compactness: Minimum Polsby-Popper compactness required (default 0.25)

    Returns:
        (is_valid: bool, actual_ratios: list, error: float, compactness: tuple)
    """
    total_pop = sample_tracts['population'].sum()

    # Calculate actual populations
    pop0 = sample_tracts[membership == 0]['population'].sum()
    pop1 = sample_tracts[membership == 1]['population'].sum()

    # Calculate actual ratios
    actual_ratio0 = pop0 / total_pop
    actual_ratio1 = pop1 / total_pop

    # Calculate error from target
    error0 = abs(actual_ratio0 - target_weights[0])
    error1 = abs(actual_ratio1 - target_weights[1])
    max_error = max(error0, error1)

    # Calculate compactness
    comp0, comp1, min_comp = calculate_compactness(sample_tracts, membership)

    # Validation requires both ratio accuracy AND compactness (both regions)
    ratio_valid = max_error <= tolerance
    compactness_valid = (comp0 >= min_compactness) and (comp1 >= min_compactness)
    is_valid = ratio_valid and compactness_valid

    return is_valid, [actual_ratio0, actual_ratio1], max_error, (comp0, comp1, min_comp)


def run_metis_partition(sample_tracts, adjacency, edge_weights, config):
    """Run METIS to partition tracts."""
    import sys
    sys.path.insert(0, str(Path('../../../src').resolve()))
    from apportionment.partition.metis_wrapper import partition_graph

    n_tracts = len(sample_tracts)
    adjacency_list = [adjacency.get(i, []) for i in range(n_tracts)]
    vweights = np.array([int(sample_tracts.iloc[i]['population']) for i in range(n_tracts)])

    edge_weights_dict = {}
    for (i, j), weight in edge_weights.items():
        if i < j:
            edge_weights_dict[(i, j)] = weight

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


def create_visualization(sample_tracts, adjacency, edge_weights, membership, config, output_path):
    """Create side-by-side map and graph visualization."""
    n_tracts = len(sample_tracts)
    labels = [chr(65 + i) for i in range(n_tracts)]  # A, B, C, ...

    # Identify cut edges
    cut_edges = set()
    for i in range(n_tracts):
        for j in adjacency.get(i, []):
            if i < j and membership[i] != membership[j]:
                cut_edges.add((i, j))

    # Colors
    partition_colors = {0: 'lightblue', 1: 'lightcoral'}

    # Create figure
    fig = plt.figure(figsize=(16, 7))
    gs = fig.add_gridspec(1, 2, width_ratios=[1.4, 1], wspace=0.15)
    ax1 = fig.add_subplot(gs[0])
    ax2 = fig.add_subplot(gs[1])

    # Title
    fig.suptitle(f"{config['title']} - {config['subtitle']}",
                 fontsize=14, fontweight='bold', y=0.98)

    # LEFT PANEL: Geographic map
    sample_tracts['partition'] = membership

    # Plot tracts
    for idx in range(n_tracts):
        color = partition_colors[membership[idx]]
        sample_tracts.iloc[[idx]].plot(
            ax=ax1,
            facecolor=color,
            edgecolor='black',
            linewidth=1.5,
            alpha=0.7
        )

        # Label tract
        centroid = sample_tracts.iloc[idx].geometry.centroid
        pop_k = sample_tracts.iloc[idx]['population'] / 1000
        ax1.text(centroid.x, centroid.y, f'{labels[idx]}\n{pop_k:.1f}K',
                ha='center', va='center', fontsize=9, fontweight='bold',
                bbox=dict(boxstyle='round', facecolor='white',
                        edgecolor='black', linewidth=1))

    # Label internal boundaries
    for i in range(n_tracts):
        for j in adjacency.get(i, []):
            if i < j:
                is_cut = (i, j) in cut_edges
                if not is_cut:
                    geom_i = sample_tracts.iloc[i].geometry
                    geom_j = sample_tracts.iloc[j].geometry
                    boundary = geom_i.intersection(geom_j.boundary)
                    if not boundary.is_empty:
                        mid_point = boundary.centroid
                        length_km = edge_weights.get((i, j), 0)
                        if length_km > 0.1:
                            ax1.text(mid_point.x, mid_point.y, f'{length_km:.1f}',
                                   ha='center', va='center', fontsize=7,
                                   bbox=dict(boxstyle='round', facecolor='white',
                                           edgecolor='black', linewidth=0.5, alpha=0.8))

    # Highlight cut boundaries
    total_cut_length = 0
    for i, j in cut_edges:
        geom_i = sample_tracts.iloc[i].geometry
        geom_j = sample_tracts.iloc[j].geometry
        boundary = geom_i.intersection(geom_j.boundary)
        if not boundary.is_empty:
            gpd.GeoSeries([boundary]).plot(ax=ax1, color='red', linewidth=4, alpha=0.9, zorder=10)
            mid_point = boundary.centroid
            length_km = edge_weights.get((i, j), edge_weights.get((j, i), 0))
            total_cut_length += length_km
            if length_km > 0.1:
                ax1.text(mid_point.x, mid_point.y, f'{length_km:.1f}',
                       ha='center', va='center', fontsize=7, fontweight='bold',
                       bbox=dict(boxstyle='round', facecolor='yellow',
                               edgecolor='red', linewidth=1.5, alpha=0.9),
                       zorder=11)

    # Calculate region populations and percentages
    pop0 = sample_tracts[sample_tracts['partition'] == 0]['population'].sum() / 1000
    pop1 = sample_tracts[sample_tracts['partition'] == 1]['population'].sum() / 1000
    total_pop = pop0 + pop1
    pct0 = (pop0 * 1000 / (total_pop * 1000)) * 100
    pct1 = (pop1 * 1000 / (total_pop * 1000)) * 100

    # Dissolve regions and position labels outside the combined boundary
    part0_geoms = sample_tracts[sample_tracts['partition'] == 0].geometry
    part1_geoms = sample_tracts[sample_tracts['partition'] == 1].geometry

    # Get combined bounds of entire map (all tracts)
    combined_bounds = sample_tracts.geometry.total_bounds  # (minx, miny, maxx, maxy)
    map_height = combined_bounds[3] - combined_bounds[1]
    map_width = combined_bounds[2] - combined_bounds[0]

    if len(part0_geoms) > 0 and len(part1_geoms) > 0:
        # Dissolve each region
        region0_union = part0_geoms.unary_union
        region1_union = part1_geoms.unary_union
        centroid0 = region0_union.centroid
        centroid1 = region1_union.centroid

        # Determine if regions are more horizontal or vertical split
        # Check if centroids are more separated horizontally or vertically
        h_separation = abs(centroid1.x - centroid0.x)
        v_separation = abs(centroid1.y - centroid0.y)

        if h_separation > v_separation:
            # Horizontal split: place labels above map, aligned with region centroids
            label_y = combined_bounds[3] + map_height * 0.10  # 10% above top

            ax1.text(centroid0.x, label_y, f'Region 1\n{pop0:.1f}K\n{pct0:.1f}%',
                    ha='center', va='bottom', fontsize=11, fontweight='bold',
                    bbox=dict(boxstyle='round', facecolor='lightblue',
                            edgecolor='blue', linewidth=1.5, alpha=0.9),
                    color='black', zorder=12)

            ax1.text(centroid1.x, label_y, f'Region 2\n{pop1:.1f}K\n{pct1:.1f}%',
                    ha='center', va='bottom', fontsize=11, fontweight='bold',
                    bbox=dict(boxstyle='round', facecolor='lightcoral',
                            edgecolor='red', linewidth=1.5, alpha=0.9),
                    color='black', zorder=12)
        else:
            # Vertical split: place labels on sides, aligned with region centroids
            if centroid0.y > centroid1.y:
                # Region 0 is on top, Region 1 on bottom
                label_x0 = combined_bounds[2] + map_width * 0.10  # 10% right of right edge
                label_x1 = combined_bounds[0] - map_width * 0.10  # 10% left of left edge
            else:
                # Region 1 is on top, Region 0 on bottom
                label_x0 = combined_bounds[0] - map_width * 0.10
                label_x1 = combined_bounds[2] + map_width * 0.10

            ax1.text(label_x0, centroid0.y, f'Region 1\n{pop0:.1f}K\n{pct0:.1f}%',
                    ha='center', va='center', fontsize=11, fontweight='bold',
                    bbox=dict(boxstyle='round', facecolor='lightblue',
                            edgecolor='blue', linewidth=1.5, alpha=0.9),
                    color='black', zorder=12)

            ax1.text(label_x1, centroid1.y, f'Region 2\n{pop1:.1f}K\n{pct1:.1f}%',
                    ha='center', va='center', fontsize=11, fontweight='bold',
                    bbox=dict(boxstyle='round', facecolor='lightcoral',
                            edgecolor='red', linewidth=1.5, alpha=0.9),
                    color='black', zorder=12)

    ax1.axis('off')

    # Add ratio label in top-left corner
    ratio_text = f"{int(config['target_weights'][0]*100)}-{int(config['target_weights'][1]*100)} Split"
    ax1.text(0.02, 0.98, ratio_text,
            transform=ax1.transAxes, ha='left', va='top',
            fontsize=11, fontweight='bold',
            bbox=dict(boxstyle='round', facecolor='white',
                    edgecolor='black', linewidth=2, alpha=0.95))

    # Total cut summary box
    ax1.text(0.5, -0.05,
            f'Total Cut: {total_cut_length:.1f} km',
            transform=ax1.transAxes, ha='center', va='top',
            fontsize=9, fontweight='bold',
            bbox=dict(boxstyle='round', facecolor='yellow',
                    edgecolor='red', linewidth=1.5, alpha=0.9))

    # RIGHT PANEL: Abstract graph
    ax2.set_title('Graph + METIS Cut\n(What Algorithm Sees)', fontsize=11, fontweight='bold')

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

    # Draw edges
    for i in range(n_tracts):
        for j in adjacency.get(i, []):
            if i < j:
                x1, y1 = positions[i]
                x2, y2 = positions[j]
                length_km = edge_weights.get((i, j), 0)

                if length_km > 0.1:
                    is_cut = (i, j) in cut_edges

                    if is_cut:
                        ax2.plot([x1, x2], [y1, y2], 'r--',
                               linewidth=4, alpha=0.9, zorder=2)
                        mid_x, mid_y = (x1 + x2) / 2, (y1 + y2) / 2
                        ax2.plot(mid_x, mid_y, 'rX', markersize=12,
                               markeredgewidth=3, zorder=3)
                    else:
                        thickness = min(6, max(1.5, length_km / 3))
                        ax2.plot([x1, x2], [y1, y2], 'k-',
                               linewidth=thickness, alpha=0.4, zorder=1)

                    # Edge labels
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

                    if is_cut:
                        label_style = dict(boxstyle='round', facecolor='yellow',
                                         edgecolor='red', linewidth=1.5, alpha=0.9)
                        fontweight = 'bold'
                    else:
                        label_style = dict(boxstyle='round', facecolor='white',
                                         edgecolor='gray', linewidth=0.5, alpha=0.8)
                        fontweight = 'normal'

                    ax2.text(label_x, label_y, f'{length_km:.1f}',
                           ha='center', va='center', fontsize=9,
                           fontweight=fontweight, bbox=label_style)

    # Draw nodes
    for i in range(n_tracts):
        x, y = positions[i]
        color = partition_colors[membership[i]]
        pop_k = sample_tracts.iloc[i]['population'] / 1000

        circle = Circle((x, y), 0.3, facecolor=color,
                       edgecolor='black', linewidth=2.5, alpha=0.8, zorder=4)
        ax2.add_patch(circle)

        ax2.text(x, y + 0.06, labels[i], ha='center', va='center',
                fontsize=9, fontweight='bold', zorder=5)
        ax2.text(x, y - 0.09, f'{pop_k:.1f}K', ha='center', va='center',
                fontsize=9, fontweight='bold', color='black', zorder=5)

    # Region labels at top - position relative to partition extent in graph space
    pop0_total = sample_tracts[sample_tracts['partition'] == 0]['population'].sum() / 1000
    pop1_total = sample_tracts[sample_tracts['partition'] == 1]['population'].sum() / 1000
    total_pop_graph = pop0_total + pop1_total
    pct0_graph = (pop0_total * 1000 / (total_pop_graph * 1000)) * 100
    pct1_graph = (pop1_total * 1000 / (total_pop_graph * 1000)) * 100

    # Calculate position extent for each partition
    part0_positions = [positions[i] for i in range(n_tracts) if membership[i] == 0]
    part1_positions = [positions[i] for i in range(n_tracts) if membership[i] == 1]

    # Position labels above each partition's extent
    if part0_positions:
        xs0 = [p[0] for p in part0_positions]
        ys0 = [p[1] for p in part0_positions]
        label_x0 = sum(xs0) / len(xs0)  # Average x position (centroid)
        label_y0 = max(ys0) + 0.5  # Above highest node
        ax2.text(label_x0, label_y0, f'Region 1\n{pop0_total:.1f}K\n{pct0_graph:.1f}%',
                ha='center', va='bottom',
                fontsize=11, fontweight='bold',
                bbox=dict(boxstyle='round', facecolor='lightblue',
                        edgecolor='blue', linewidth=1.5, alpha=0.9),
                color='black', zorder=12)

    if part1_positions:
        xs1 = [p[0] for p in part1_positions]
        ys1 = [p[1] for p in part1_positions]
        label_x1 = sum(xs1) / len(xs1)  # Average x position (centroid)
        label_y1 = max(ys1) + 0.5  # Above highest node
        ax2.text(label_x1, label_y1, f'Region 2\n{pop1_total:.1f}K\n{pct1_graph:.1f}%',
                ha='center', va='bottom',
                fontsize=11, fontweight='bold',
                bbox=dict(boxstyle='round', facecolor='lightcoral',
                        edgecolor='red', linewidth=1.5, alpha=0.9),
                color='black', zorder=12)

    ax2.set_xlim(-0.5, 4.8)
    ax2.set_ylim(-0.7, 5.0)
    ax2.set_aspect('equal')
    ax2.axis('off')

    # Total cut summary box on graph
    total_cut_weight = sum(edge_weights.get((i, j), 0) for i, j in cut_edges)
    ax2.text(0.5, 0.05,
            f'Total Cut: {total_cut_weight:.1f} km',
            transform=ax2.transAxes, ha='center', va='top',
            fontsize=9, fontweight='bold',
            bbox=dict(boxstyle='round', facecolor='yellow',
                    edgecolor='red', linewidth=1.5, alpha=0.9))

    plt.savefig(output_path, dpi=150, bbox_inches='tight',
               facecolor='white', edgecolor='none')
    plt.close()


def generate_example(config, args, output_dir):
    """Generate a single example visualization with retry logic for better ratio matching."""
    print(f"[{config['name']}] {config['title']} - {config['subtitle']}")

    # Check if output already exists
    output_path = output_dir / f"{config['name']}.png"
    if output_path.exists():
        print(f"  [SKIP] File already exists: {output_path.name}")
        return True

    year_suffix = str(args.year)[-2:]
    state_fips = config['state_fips']
    state_name = config['state_name']
    county_fips = config['county_fips']

    # File paths (go up 3 levels from artifacts/presentations/edge_weighted_bisection/ to project root)
    tracts_file = Path(f'../../../data/geography/tiger_{args.year}_tracts/tl_{args.year}_{state_fips}_tract{year_suffix}/tl_{args.year}_{state_fips}_tract{year_suffix}.shp')
    population_file = Path(f'../../../data/processed/census_{args.year}/{state_name}_tracts_{args.year}_population.csv')

    geoid_field = f'GEOID{year_suffix}'
    county_field = f'COUNTYFP{year_suffix}'

    # Check files exist
    if not tracts_file.exists():
        print(f"  [SKIP] Tracts shapefile not found: {tracts_file}")
        return False
    if not population_file.exists():
        print(f"  [SKIP] Population data not found: {population_file}")
        return False

    try:
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
            print(f"  [SKIP] Not enough tracts in county ({len(county_tracts)} < {config['n_tracts']})")
            return False

        # Try up to 26 times to find a good example (original + 25 retries)
        max_attempts = 26
        best_result = None
        best_error = float('inf')

        # Try different starting points
        start_indices_to_try = [config['start_tract_idx'] + (i * 15) for i in range(max_attempts)]

        for attempt, start_tract_offset in enumerate(start_indices_to_try):
            if attempt > 0:
                print(f"    Retry {attempt}/{max_attempts-1} (adjusting start position)...")

            # Find contiguous tracts
            start_idx = county_tracts.index[min(start_tract_offset, len(county_tracts) - 1)]
            selected_indices = find_contiguous_tracts_bfs(county_tracts, start_idx, config['n_tracts'])

            if len(selected_indices) < config['n_tracts']:
                if attempt == 0:
                    print(f"  [SKIP] Could not find {config['n_tracts']} contiguous tracts (found {len(selected_indices)})")
                continue

            sample_tracts = county_tracts.loc[selected_indices].copy().reset_index(drop=True)

            # Build adjacency
            adjacency, edge_weights = build_adjacency_graph(sample_tracts)

            # Run METIS
            membership = run_metis_partition(sample_tracts, adjacency, edge_weights, config)

            # Validate ratio and compactness
            is_valid, actual_ratios, error, compactness = validate_partition_ratio(
                sample_tracts, membership, config['target_weights'], tolerance=0.005, min_compactness=0.25
            )

            # Unpack compactness
            comp0, comp1, min_comp = compactness

            # Convert ratios to percentages for display
            pct0 = actual_ratios[0] * 100
            pct1 = actual_ratios[1] * 100
            target_pct0 = config['target_weights'][0] * 100
            target_pct1 = config['target_weights'][1] * 100

            if attempt > 0 or not is_valid:
                print(f"    Achieved: {pct0:.1f}-{pct1:.1f} (target: {target_pct0:.0f}-{target_pct1:.0f}), "
                      f"error: {error*100:.2f}%, compact: {comp0:.3f}/{comp1:.3f}")

            # Keep track of best result (prioritize compactness, then error)
            if error < best_error:
                best_error = error
                best_result = {
                    'sample_tracts': sample_tracts.copy(),
                    'adjacency': adjacency,
                    'edge_weights': edge_weights,
                    'membership': membership,
                    'actual_ratios': actual_ratios,
                    'error': error,
                    'compactness': compactness
                }

            # If ratio and compactness are good enough, stop trying
            if is_valid:
                print(f"  [VALID] Ratio: {pct0:.1f}-{pct1:.1f} (within 0.5%), Compactness: {comp0:.3f}/{comp1:.3f} (both >=0.25)")
                break

        # Use best result found
        if best_result is None:
            print(f"  [SKIP] Could not find valid partition after {max_attempts} attempts")
            return False

        # Check if best result meets all criteria
        comp0, comp1, min_comp = best_result['compactness']
        ratio_ok = best_error <= 0.005
        compact_ok = (comp0 >= 0.25) and (comp1 >= 0.25)

        if not (ratio_ok and compact_ok):
            pct0 = best_result['actual_ratios'][0] * 100
            pct1 = best_result['actual_ratios'][1] * 100

            issues = []
            if not ratio_ok:
                issues.append(f"ratio error: {best_error*100:.2f}%")
            if not compact_ok:
                issues.append(f"compactness: {comp0:.3f}/{comp1:.3f}")

            print(f"  [WARN] Using best attempt: {pct0:.1f}-{pct1:.1f}, {', '.join(issues)}")

        # Create visualization with best result
        create_visualization(
            best_result['sample_tracts'],
            best_result['adjacency'],
            best_result['edge_weights'],
            best_result['membership'],
            config,
            output_path
        )

        print(f"  [OK] Created: {output_path.name}")
        return True

    except Exception as e:
        print(f"  [ERROR] {e}")
        import traceback
        traceback.print_exc()
        return False


def main():
    parser = argparse.ArgumentParser(description='Generate appendix examples')
    parser.add_argument('--year', type=int, default=2010,
                       help='Census year (default: 2010)')
    parser.add_argument('--version', type=str, default='v1',
                       help='Pipeline version (default: v1)')
    args = parser.parse_args()

    # Output directory
    output_dir = Path(f'../../../outputs/artifacts/presentations/edge_weighted_bisection/appendix_examples')
    output_dir.mkdir(parents=True, exist_ok=True)

    print("=" * 70)
    print("Generating Appendix Examples for Laymen's Guide")
    print("=" * 70)
    print(f"Census year: {args.year}")
    print(f"Total examples: {len(EXAMPLES)}")
    print(f"Output directory: {output_dir}")
    print()

    # Generate all examples
    successful = 0
    for i, config in enumerate(EXAMPLES, 1):
        print(f"[{i}/{len(EXAMPLES)}] ", end='')
        if generate_example(config, args, output_dir):
            successful += 1
        print()

    print("=" * 70)
    print(f"Complete: {successful}/{len(EXAMPLES)} examples generated")
    print("=" * 70)
    print(f"Location: {output_dir.resolve()}")
    print()


if __name__ == '__main__':
    main()
