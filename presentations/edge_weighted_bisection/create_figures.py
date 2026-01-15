#!/usr/bin/env python3
"""
Create placeholder figures for the presentation.

Generates:
1. figures/example_gerrymander.png - Famous gerrymandered district example
2. figures/tract_to_graph.png - Visual showing tract-to-graph transformation
"""

import argparse
import matplotlib.pyplot as plt
import matplotlib.patches as patches
from matplotlib.patches import FancyBboxPatch, Circle, FancyArrowPatch
import numpy as np
from pathlib import Path

# Parse arguments (for consistency, though not currently used)
parser = argparse.ArgumentParser(description='Generate figures for presentation')
parser.add_argument('--year', type=int, default=2020,
                   help='Census year (default: 2020)')
parser.add_argument('--version', type=str, default='v1',
                   help='Pipeline version (default: v1)')
args = parser.parse_args()

# Create figures directory in outputs
figures_dir = Path('../../outputs/presentations/edge_weighted_bisection/figures')
figures_dir.mkdir(parents=True, exist_ok=True)

print("=" * 70)
print("Generating Figures for Edge-Weighted Bisection Presentation")
print("=" * 70)
print(f"Census year: {args.year}")
print(f"Pipeline version: {args.version}")
print()

# =============================================================================
# Part 1: Copy Round Progression Maps from Pipeline Outputs
# =============================================================================
print("[1/3] Copying round progression maps from pipeline outputs...")
print("-" * 70)

import shutil

# Check if pipeline outputs exist
pipeline_output_dir = Path(f'../../outputs/us_{args.year}_{args.version}')
if not pipeline_output_dir.exists():
    print(f"[WARNING] Pipeline outputs not found at: {pipeline_output_dir}")
    print("          Round progression figures will not be available.")
    print(f"          Run: python scripts/pipeline/run_complete_redistricting.py --year {args.year} --version {args.version}")
    print()
else:
    # Minnesota round maps (3 rounds -> 8 districts)
    minnesota_rounds = [
        (f'../../outputs/us_{args.year}_{args.version}/states/minnesota/maps/rounds/round_01.png',
         'minnesota_round_1_2_regions.png'),
        (f'../../outputs/us_{args.year}_{args.version}/states/minnesota/maps/rounds/round_02.png',
         'minnesota_round_2_4_regions.png'),
        (f'../../outputs/us_{args.year}_{args.version}/states/minnesota/maps/rounds/round_03.png',
         'minnesota_round_3_8_regions.png'),
    ]

    # Alabama round maps (3 rounds -> 7 districts)
    alabama_rounds = [
        (f'../../outputs/us_{args.year}_{args.version}/states/alabama/maps/rounds/round_01.png',
         'alabama_round_1_2_regions.png'),
        (f'../../outputs/us_{args.year}_{args.version}/states/alabama/maps/rounds/round_02.png',
         'alabama_round_2_4_regions.png'),
        (f'../../outputs/us_{args.year}_{args.version}/states/alabama/maps/rounds/round_03.png',
         'alabama_round_3_7_regions.png'),
    ]

    # Copy all round maps
    copied_count = 0
    for source, dest in minnesota_rounds + alabama_rounds:
        source_path = Path(source)
        dest_path = figures_dir / dest

        if source_path.exists():
            shutil.copy2(source_path, dest_path)
            print(f"  ✓ Copied: {dest}")
            copied_count += 1
        else:
            print(f"  ✗ Missing: {source}")

    if copied_count > 0:
        print(f"  Copied {copied_count} round progression maps")
    print()

# =============================================================================
# Part 2: Generate Custom Visualization Figures
# =============================================================================
print("[2/3] Generating custom visualization figures...")
print("-" * 70)

# =============================================================================
# Figure 1: Example Gerrymandered District
# =============================================================================
print("Creating example gerrymander figure...")

fig, ax = plt.subplots(1, 1, figsize=(8, 6))

# Create a simplified version of Illinois 4th District (the "earmuffs")
# This is one of the most famous gerrymandered districts

# Draw the two main communities (connected by a thin corridor)
# North community
north = patches.Rectangle((2, 7), 3, 2, linewidth=2, edgecolor='darkblue',
                          facecolor='lightblue', alpha=0.7)
ax.add_patch(north)

# South community
south = patches.Rectangle((2, 1), 3, 2, linewidth=2, edgecolor='darkblue',
                          facecolor='lightblue', alpha=0.7)
ax.add_patch(south)

# Thin corridor connecting them along highway
corridor_x = [2.5, 2.5, 1.5, 1.5, 0.5, 0.5, 1.5, 1.5, 2.5, 2.5]
corridor_y = [3, 3.5, 3.5, 5, 5, 5.5, 5.5, 6.5, 6.5, 7]
corridor = patches.Polygon(list(zip(corridor_x, corridor_y)),
                          linewidth=2, edgecolor='darkblue',
                          facecolor='lightblue', alpha=0.7)
ax.add_patch(corridor)

# Add labels
ax.text(3.5, 8, 'North\nCommunity', ha='center', va='center',
        fontsize=11, fontweight='bold')
ax.text(3.5, 2, 'South\nCommunity', ha='center', va='center',
        fontsize=11, fontweight='bold')
ax.text(0.7, 5.2, 'Narrow\nCorridor', ha='center', va='center',
        fontsize=9, style='italic', color='darkred')

# Add arrow pointing to corridor
arrow = FancyArrowPatch((0.2, 4.5), (0.5, 5),
                       arrowstyle='->', mutation_scale=20,
                       linewidth=2, color='darkred')
ax.add_patch(arrow)

# Add grid to show how unnatural this is
for i in range(11):
    ax.axhline(i, color='gray', linewidth=0.5, alpha=0.3)
    ax.axvline(i, color='gray', linewidth=0.5, alpha=0.3)

# Labels
ax.set_xlim(0, 10)
ax.set_ylim(0, 10)
ax.set_aspect('equal')
ax.set_title('Example of Gerrymandering:\nConnecting Distant Communities via Narrow Corridor',
            fontsize=13, fontweight='bold', pad=15)
ax.text(5, -0.7, 'Inspired by Illinois 4th District ("The Earmuffs")',
       ha='center', fontsize=10, style='italic', color='gray')
ax.axis('off')

plt.tight_layout()
plt.savefig(figures_dir / 'example_gerrymander.png', dpi=150, bbox_inches='tight',
           facecolor='white', edgecolor='none')
print(f"  Created: {figures_dir / 'example_gerrymander.png'}")
plt.close()

# =============================================================================
# Figure 2: Tract to Graph Transformation
# =============================================================================
print("Creating tract-to-graph figure...")

fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 5))

# Left side: Geographic tracts
ax1.set_title('Census Tracts\n(Geographic Reality)', fontsize=12, fontweight='bold')

# Create 5 simple tract shapes
tracts = [
    # Tract A (top left)
    patches.FancyBboxPatch((0.5, 2.5), 1.5, 1.5, boxstyle="round,pad=0.05",
                           linewidth=2, edgecolor='black', facecolor='lightgreen', alpha=0.6),
    # Tract B (top right)
    patches.FancyBboxPatch((2, 2.5), 1.5, 1.5, boxstyle="round,pad=0.05",
                           linewidth=2, edgecolor='black', facecolor='lightcoral', alpha=0.6),
    # Tract C (middle left)
    patches.FancyBboxPatch((0.5, 1), 1.5, 1.5, boxstyle="round,pad=0.05",
                           linewidth=2, edgecolor='black', facecolor='lightyellow', alpha=0.6),
    # Tract D (middle right)
    patches.FancyBboxPatch((2, 1), 1.5, 1.5, boxstyle="round,pad=0.05",
                           linewidth=2, edgecolor='black', facecolor='lightblue', alpha=0.6),
    # Tract E (bottom)
    patches.FancyBboxPatch((1.25, -0.5), 1.5, 1.5, boxstyle="round,pad=0.05",
                           linewidth=2, edgecolor='black', facecolor='plum', alpha=0.6),
]

for tract in tracts:
    ax1.add_patch(tract)

# Add labels
ax1.text(1.25, 3.25, 'A\n15K', ha='center', va='center', fontsize=10, fontweight='bold')
ax1.text(2.75, 3.25, 'B\n12K', ha='center', va='center', fontsize=10, fontweight='bold')
ax1.text(1.25, 1.75, 'C\n18K', ha='center', va='center', fontsize=10, fontweight='bold')
ax1.text(2.75, 1.75, 'D\n20K', ha='center', va='center', fontsize=10, fontweight='bold')
ax1.text(2, 0.25, 'E\n10K', ha='center', va='center', fontsize=10, fontweight='bold')

ax1.set_xlim(0, 4)
ax1.set_ylim(-1, 4.5)
ax1.set_aspect('equal')
ax1.axis('off')
ax1.text(2, 4.7, 'Shapes follow streets,\nrivers, boundaries',
        ha='center', fontsize=9, style='italic', color='gray')

# Right side: Abstract graph
ax2.set_title('Graph Representation\n(For Algorithm)', fontsize=12, fontweight='bold')

# Node positions
nodes = {
    'A': (1, 3),
    'B': (3, 3),
    'C': (1, 1.5),
    'D': (3, 1.5),
    'E': (2, 0),
}

# Edges (adjacencies)
edges = [
    ('A', 'B'), ('A', 'C'), ('B', 'D'),
    ('C', 'D'), ('C', 'E'), ('D', 'E')
]

# Draw edges
for n1, n2 in edges:
    x1, y1 = nodes[n1]
    x2, y2 = nodes[n2]
    ax2.plot([x1, x2], [y1, y2], 'k-', linewidth=2, alpha=0.5, zorder=1)

# Draw nodes with colors matching tracts
node_colors = {
    'A': 'lightgreen',
    'B': 'lightcoral',
    'C': 'lightyellow',
    'D': 'lightblue',
    'E': 'plum',
}

for label, (x, y) in nodes.items():
    circle = Circle((x, y), 0.35, facecolor=node_colors[label],
                   edgecolor='black', linewidth=2.5, alpha=0.8, zorder=2)
    ax2.add_patch(circle)
    ax2.text(x, y, label, ha='center', va='center',
            fontsize=12, fontweight='bold', zorder=3)

# Add population labels
pops = {'A': '15K', 'B': '12K', 'C': '18K', 'D': '20K', 'E': '10K'}
for label, (x, y) in nodes.items():
    ax2.text(x, y-0.55, pops[label], ha='center', va='top',
            fontsize=9, style='italic', color='darkgray')

ax2.set_xlim(0, 4)
ax2.set_ylim(-1, 4)
ax2.set_aspect('equal')
ax2.axis('off')
ax2.text(2, 4.2, 'Nodes = tracts\nEdges = adjacency',
        ha='center', fontsize=9, style='italic', color='gray')

# Add arrow between the two
fig.text(0.5, 0.5, '→', fontsize=60, ha='center', va='center',
        color='darkblue', weight='bold')
fig.text(0.5, 0.42, 'Transform', fontsize=11, ha='center', va='center',
        style='italic', color='darkblue')

plt.tight_layout()
plt.savefig(figures_dir / 'tract_to_graph.png', dpi=150, bbox_inches='tight',
           facecolor='white', edgecolor='none')
print(f"  Created: {figures_dir / 'tract_to_graph.png'}")
plt.close()

# =============================================================================
# Figure 3: Graph with Cut Visualization
# =============================================================================
print("Creating graph with cut visualization...")

fig, ax = plt.subplots(1, 1, figsize=(10, 7))

# Define a simple 3x3 grid graph (9 tracts)
positions = {
    'A': (0, 2), 'B': (1, 2), 'C': (2, 2),
    'D': (0, 1), 'E': (1, 1), 'F': (2, 1),
    'G': (0, 0), 'H': (1, 0), 'I': (2, 0)
}

# Population for each tract
populations = {
    'A': 40, 'B': 45, 'C': 35,
    'D': 42, 'E': 48, 'F': 40,
    'G': 38, 'H': 47, 'I': 40
}

# Define edges (adjacencies)
edges = [
    ('A', 'B'), ('B', 'C'),  # Top row
    ('D', 'E'), ('E', 'F'),  # Middle row
    ('G', 'H'), ('H', 'I'),  # Bottom row
    ('A', 'D'), ('D', 'G'),  # Left column
    ('B', 'E'), ('E', 'H'),  # Middle column
    ('C', 'F'), ('F', 'I')   # Right column
]

# Cut edges (partition between left 6 tracts and right 3 tracts)
cut_edges = [('B', 'C'), ('E', 'F'), ('H', 'I')]

# Partition assignments
left_partition = ['A', 'B', 'D', 'E', 'G', 'H']  # 187K total
right_partition = ['C', 'F', 'I']  # 115K total

# Draw non-cut edges
for edge in edges:
    if edge not in cut_edges:
        x1, y1 = positions[edge[0]]
        x2, y2 = positions[edge[1]]
        ax.plot([x1, x2], [y1, y2], 'k-', linewidth=2, alpha=0.3, zorder=1)

# Draw cut edges with red dashed line
for edge in cut_edges:
    x1, y1 = positions[edge[0]]
    x2, y2 = positions[edge[1]]
    ax.plot([x1, x2], [y1, y2], 'r--', linewidth=3, alpha=0.8, zorder=2,
           label='Cut Edge' if edge == cut_edges[0] else '')

# Draw nodes (color by partition)
for node, (x, y) in positions.items():
    if node in left_partition:
        color = 'lightblue'
        label = 'Region 0' if node == 'A' else ''
    else:
        color = 'lightcoral'
        label = 'Region 1' if node == 'C' else ''

    circle = Circle((x, y), 0.25, facecolor=color, edgecolor='black',
                   linewidth=2.5, alpha=0.9, zorder=3, label=label)
    ax.add_patch(circle)
    ax.text(x, y, f'{node}\n{populations[node]}K', ha='center', va='center',
           fontsize=10, fontweight='bold', zorder=4)

# Add partition labels
ax.text(0.5, -0.8, 'Region 0\n(187K pop)', ha='center', va='center',
       fontsize=12, fontweight='bold', color='darkblue',
       bbox=dict(boxstyle='round', facecolor='lightblue', alpha=0.5))
ax.text(2, -0.8, 'Region 1\n(115K pop)', ha='center', va='center',
       fontsize=12, fontweight='bold', color='darkred',
       bbox=dict(boxstyle='round', facecolor='lightcoral', alpha=0.5))

ax.set_xlim(-0.5, 2.5)
ax.set_ylim(-1.2, 2.8)
ax.set_aspect('equal')
ax.set_title('Graph Partitioning with Cut Edges\n(3 edges cut to create 2 balanced regions)',
            fontsize=13, fontweight='bold', pad=15)
ax.legend(loc='upper right', fontsize=10)
ax.axis('off')

plt.tight_layout()
plt.savefig(figures_dir / 'graph_with_cut.png', dpi=150, bbox_inches='tight',
           facecolor='white', edgecolor='none')
print(f"  Created: {figures_dir / 'graph_with_cut.png'}")
plt.close()

# =============================================================================
# Figure 4: Edge Weights Visualization
# =============================================================================
print("Creating edge weights visualization...")

fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 6))

# Left: Unweighted graph
ax1.set_title('Unweighted Graph\n(All edges equal)', fontsize=12, fontweight='bold')

# Simple 2x3 graph for clarity
positions_simple = {
    'A': (0, 1), 'B': (1, 1), 'C': (2, 1),
    'D': (0, 0), 'E': (1, 0), 'F': (2, 0)
}

edges_simple = [
    ('A', 'B'), ('B', 'C'),
    ('D', 'E'), ('E', 'F'),
    ('A', 'D'), ('B', 'E'), ('C', 'F')
]

# Draw edges (all same weight)
for edge in edges_simple:
    x1, y1 = positions_simple[edge[0]]
    x2, y2 = positions_simple[edge[1]]
    ax1.plot([x1, x2], [y1, y2], 'gray', linewidth=3, alpha=0.6, zorder=1)
    # Add weight label
    mid_x, mid_y = (x1 + x2) / 2, (y1 + y2) / 2
    ax1.text(mid_x, mid_y, '1', ha='center', va='center',
            fontsize=9, fontweight='bold',
            bbox=dict(boxstyle='circle', facecolor='white', edgecolor='gray'))

# Draw nodes
for node, (x, y) in positions_simple.items():
    circle = Circle((x, y), 0.2, facecolor='lightyellow', edgecolor='black',
                   linewidth=2, alpha=0.8, zorder=2)
    ax1.add_patch(circle)
    ax1.text(x, y, node, ha='center', va='center',
            fontsize=11, fontweight='bold', zorder=3)

ax1.set_xlim(-0.4, 2.4)
ax1.set_ylim(-0.5, 1.5)
ax1.set_aspect('equal')
ax1.axis('off')
ax1.text(1, -0.7, 'Each edge has weight = 1\n(boundary length not considered)',
        ha='center', fontsize=9, style='italic', color='gray')

# Right: Edge-weighted graph
ax2.set_title('Edge-Weighted Graph\n(Boundary lengths matter)', fontsize=12, fontweight='bold')

# Edge weights (boundary lengths in km)
edge_weights = {
    ('A', 'B'): 5.2,
    ('B', 'C'): 12.8,  # Long boundary
    ('D', 'E'): 4.7,
    ('E', 'F'): 11.5,  # Long boundary
    ('A', 'D'): 6.1,
    ('B', 'E'): 3.9,
    ('C', 'F'): 13.2   # Long boundary
}

# Normalize weights for line thickness (1-8)
max_weight = max(edge_weights.values())
min_weight = min(edge_weights.values())

# Draw edges with varying thickness
for edge, weight in edge_weights.items():
    x1, y1 = positions_simple[edge[0]]
    x2, y2 = positions_simple[edge[1]]

    # Line thickness proportional to weight
    normalized = (weight - min_weight) / (max_weight - min_weight)
    thickness = 2 + 6 * normalized
    color = 'darkred' if weight > 10 else 'darkblue'

    ax2.plot([x1, x2], [y1, y2], color, linewidth=thickness, alpha=0.7, zorder=1)

    # Add weight label
    mid_x, mid_y = (x1 + x2) / 2, (y1 + y2) / 2
    ax2.text(mid_x, mid_y, f'{weight:.1f}', ha='center', va='center',
            fontsize=8, fontweight='bold',
            bbox=dict(boxstyle='circle', facecolor='white', edgecolor=color, linewidth=1.5))

# Draw nodes
for node, (x, y) in positions_simple.items():
    circle = Circle((x, y), 0.2, facecolor='lightgreen', edgecolor='black',
                   linewidth=2, alpha=0.8, zorder=2)
    ax2.add_patch(circle)
    ax2.text(x, y, node, ha='center', va='center',
            fontsize=11, fontweight='bold', zorder=3)

ax2.set_xlim(-0.4, 2.4)
ax2.set_ylim(-0.5, 1.5)
ax2.set_aspect('equal')
ax2.axis('off')
ax2.text(1, -0.7, 'Edge weight = boundary length (km)\n(prefer cutting short boundaries)',
        ha='center', fontsize=9, style='italic', color='gray')

plt.tight_layout()
plt.savefig(figures_dir / 'edge_weights_example.png', dpi=150, bbox_inches='tight',
           facecolor='white', edgecolor='none')
print(f"  Created: {figures_dir / 'edge_weights_example.png'}")
plt.close()

# =============================================================================
# Figure 5: Before and After Cut
# =============================================================================
print("Creating before/after cut comparison...")

fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 6))

# Left: Before cut
ax1.set_title('Before Partitioning\n(Connected Graph)', fontsize=12, fontweight='bold')

# Use same positions
for edge in edges_simple:
    weight = edge_weights.get(edge, edge_weights.get((edge[1], edge[0]), 1))
    x1, y1 = positions_simple[edge[0]]
    x2, y2 = positions_simple[edge[1]]

    normalized = (weight - min_weight) / (max_weight - min_weight)
    thickness = 2 + 6 * normalized
    color = 'darkred' if weight > 10 else 'darkblue'

    ax1.plot([x1, x2], [y1, y2], color, linewidth=thickness, alpha=0.7, zorder=1)

# Draw nodes (single color)
for node, (x, y) in positions_simple.items():
    circle = Circle((x, y), 0.2, facecolor='lightgray', edgecolor='black',
                   linewidth=2, alpha=0.8, zorder=2)
    ax1.add_patch(circle)
    ax1.text(x, y, node, ha='center', va='center',
            fontsize=11, fontweight='bold', zorder=3)

ax1.set_xlim(-0.4, 2.4)
ax1.set_ylim(-0.5, 1.5)
ax1.set_aspect('equal')
ax1.axis('off')
ax1.text(1, -0.7, 'One region (6 tracts)\nTotal: 275K population',
        ha='center', fontsize=10, style='italic', color='gray')

# Right: After cut
ax2.set_title('After Partitioning\n(2 Districts)', fontsize=12, fontweight='bold')

# Partition: left 3 vs right 3
left_part = ['A', 'D']
right_part = ['B', 'C', 'E', 'F']
cut_edges_simple = [('A', 'B'), ('D', 'E')]

# Draw non-cut edges
for edge in edges_simple:
    if edge not in cut_edges_simple and (edge[1], edge[0]) not in cut_edges_simple:
        weight = edge_weights.get(edge, edge_weights.get((edge[1], edge[0]), 1))
        x1, y1 = positions_simple[edge[0]]
        x2, y2 = positions_simple[edge[1]]

        normalized = (weight - min_weight) / (max_weight - min_weight)
        thickness = 2 + 6 * normalized
        color = 'darkred' if weight > 10 else 'darkblue'

        ax2.plot([x1, x2], [y1, y2], color, linewidth=thickness, alpha=0.7, zorder=1)

# Draw cut edges
for edge in cut_edges_simple:
    weight = edge_weights.get(edge, edge_weights.get((edge[1], edge[0]), 1))
    x1, y1 = positions_simple[edge[0]]
    x2, y2 = positions_simple[edge[1]]

    # Draw as dashed with X marker
    ax2.plot([x1, x2], [y1, y2], 'red', linewidth=3, linestyle='--',
            alpha=0.8, zorder=2)
    mid_x, mid_y = (x1 + x2) / 2, (y1 + y2) / 2
    ax2.plot(mid_x, mid_y, 'rX', markersize=15, markeredgewidth=3, zorder=3)

# Draw nodes (colored by partition)
for node, (x, y) in positions_simple.items():
    if node in left_part:
        color = 'lightblue'
    else:
        color = 'lightcoral'

    circle = Circle((x, y), 0.2, facecolor=color, edgecolor='black',
                   linewidth=2, alpha=0.8, zorder=2)
    ax2.add_patch(circle)
    ax2.text(x, y, node, ha='center', va='center',
            fontsize=11, fontweight='bold', zorder=3)

# Add partition labels
ax2.text(0, 1.7, 'District 1\n(~137K)', ha='center', fontsize=10,
        fontweight='bold', color='darkblue',
        bbox=dict(boxstyle='round', facecolor='lightblue', alpha=0.6))
ax2.text(1.5, 1.7, 'District 2\n(~138K)', ha='center', fontsize=10,
        fontweight='bold', color='darkred',
        bbox=dict(boxstyle='round', facecolor='lightcoral', alpha=0.6))

ax2.set_xlim(-0.4, 2.4)
ax2.set_ylim(-0.5, 2.0)
ax2.set_aspect('equal')
ax2.axis('off')
ax2.text(1, -0.7, f'Cut weight: {edge_weights[("A", "B")] + edge_weights[("D", "E")]:.1f} km\n(minimized boundary length)',
        ha='center', fontsize=10, style='italic', color='gray')

plt.tight_layout()
plt.savefig(figures_dir / 'before_after_cut.png', dpi=150, bbox_inches='tight',
           facecolor='white', edgecolor='none')
print(f"  Created: {figures_dir / 'before_after_cut.png'}")
plt.close()

# =============================================================================
# Figure 6: Real Census Tracts to Graph Transformation
# =============================================================================
print("Creating real census tracts to graph transformation...")

import geopandas as gpd
import warnings
warnings.filterwarnings('ignore')

# Try to load real census tract data
tracts_file = Path(f'../../data/geography/nhgis_shapefiles_{args.year}/tracts/US_tract_{args.year}.shp')

if not tracts_file.exists():
    print(f"  [WARNING] Census tracts shapefile not found at: {tracts_file}")
    print(f"            Skipping real tracts figure")
else:
    try:
        # Load tracts
        tracts_gdf = gpd.read_file(tracts_file)

        # Pick a small interesting cluster - let's use a few tracts from Hennepin County, MN (Minneapolis area)
        # This gives us urban tracts with interesting shapes
        hennepin_tracts = tracts_gdf[tracts_gdf['GISJOIN'].str.startswith('G2700053')]

        if len(hennepin_tracts) >= 6:
            # Take first 6 tracts for simplicity
            sample_tracts = hennepin_tracts.head(6).copy()

            # Create figure with two subplots
            fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 6))

            # Left: Geographic tracts with boundaries
            ax1.set_title('Census Tracts\n(Geographic Reality)', fontsize=12, fontweight='bold')

            # Assign labels A-F
            labels = ['A', 'B', 'C', 'D', 'E', 'F']
            sample_tracts['label'] = labels[:len(sample_tracts)]

            # Plot tracts with different colors
            colors = ['lightgreen', 'lightcoral', 'lightyellow', 'lightblue', 'plum', 'lightgray']
            for idx, (_, tract) in enumerate(sample_tracts.iterrows()):
                sample_tracts[sample_tracts['label'] == labels[idx]].plot(
                    ax=ax1,
                    facecolor=colors[idx],
                    edgecolor='black',
                    linewidth=2,
                    alpha=0.7
                )

                # Add label at centroid
                centroid = tract.geometry.centroid
                ax1.text(centroid.x, centroid.y, labels[idx],
                        ha='center', va='center',
                        fontsize=14, fontweight='bold',
                        bbox=dict(boxstyle='circle', facecolor='white',
                                edgecolor='black', linewidth=1.5))

            # Highlight some boundaries
            for idx in range(len(sample_tracts) - 1):
                geom1 = sample_tracts.iloc[idx].geometry
                geom2 = sample_tracts.iloc[idx + 1].geometry
                if geom1.touches(geom2):
                    boundary = geom1.intersection(geom2.boundary)
                    if not boundary.is_empty:
                        gpd.GeoSeries([boundary]).plot(ax=ax1, color='red', linewidth=3, alpha=0.8)

            ax1.axis('off')
            ax1.text(0.5, -0.05, 'Real census tracts with\nirregular shapes and boundaries',
                    transform=ax1.transAxes, ha='center', fontsize=9,
                    style='italic', color='gray')

            # Right: Abstract graph representation
            ax2.set_title('Graph Representation\n(For Algorithm)', fontsize=12, fontweight='bold')

            # Create layout for graph based on tract centroids
            centroids = sample_tracts.geometry.centroid
            xs = [c.x for c in centroids]
            ys = [c.y for c in centroids]

            # Normalize to [0, 4] range for nice plotting
            x_min, x_max = min(xs), max(xs)
            y_min, y_max = min(ys), max(ys)
            positions = {}
            for i, label in enumerate(labels[:len(sample_tracts)]):
                x_norm = 4 * (xs[i] - x_min) / (x_max - x_min) if x_max > x_min else 2
                y_norm = 4 * (ys[i] - y_min) / (y_max - y_min) if y_max > y_min else 2
                positions[label] = (x_norm, y_norm)

            # Draw edges based on adjacency
            from shapely.geometry import Point
            for i in range(len(sample_tracts)):
                geom1 = sample_tracts.iloc[i].geometry
                label1 = labels[i]
                for j in range(i + 1, len(sample_tracts)):
                    geom2 = sample_tracts.iloc[j].geometry
                    label2 = labels[j]

                    # Check if tracts are adjacent
                    if geom1.touches(geom2):
                        x1, y1 = positions[label1]
                        x2, y2 = positions[label2]

                        # Calculate boundary length (in meters if projected, else degrees)
                        boundary = geom1.intersection(geom2.boundary)
                        if not boundary.is_empty:
                            length = boundary.length / 1000  # Convert to km if in meters
                            if length < 0.01:  # If in degrees, scale up
                                length = length * 100

                            # Line thickness based on length
                            thickness = min(8, max(1, length / 2))

                            ax2.plot([x1, x2], [y1, y2], 'k-',
                                   linewidth=thickness, alpha=0.5, zorder=1)

                            # Add weight label at midpoint
                            mid_x, mid_y = (x1 + x2) / 2, (y1 + y2) / 2
                            ax2.text(mid_x, mid_y, f'{length:.1f}',
                                   ha='center', va='center', fontsize=7,
                                   bbox=dict(boxstyle='round', facecolor='white',
                                           edgecolor='gray', linewidth=0.5))

            # Draw nodes
            for label, (x, y) in positions.items():
                idx = labels.index(label)
                circle = Circle((x, y), 0.25, facecolor=colors[idx],
                              edgecolor='black', linewidth=2.5, alpha=0.8, zorder=2)
                ax2.add_patch(circle)
                ax2.text(x, y, label, ha='center', va='center',
                        fontsize=12, fontweight='bold', zorder=3)

            ax2.set_xlim(-0.5, 4.5)
            ax2.set_ylim(-0.5, 4.5)
            ax2.set_aspect('equal')
            ax2.axis('off')
            ax2.text(0.5, -0.05, 'Nodes = tract centroids\nEdges = shared boundaries (km)',
                    transform=ax2.transAxes, ha='center', fontsize=9,
                    style='italic', color='gray')

            # Add transformation arrow
            fig.text(0.5, 0.5, '→', fontsize=60, ha='center', va='center',
                    color='darkblue', weight='bold', alpha=0.7)

            plt.tight_layout()
            plt.savefig(figures_dir / 'real_tracts_to_graph.png', dpi=150,
                       bbox_inches='tight', facecolor='white', edgecolor='none')
            print(f"  Created: {figures_dir / 'real_tracts_to_graph.png'}")
            plt.close()
        else:
            print(f"  [WARNING] Not enough sample tracts found, skipping")

    except Exception as e:
        print(f"  [WARNING] Error creating real tracts figure: {e}")
        print(f"            Skipping real tracts figure")

print()
print("=" * 70)
print("Figure Generation Complete!")
print("=" * 70)
print(f"Location: {figures_dir.resolve()}")
print()
