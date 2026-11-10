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
figures_dir = Path('../../../../outputs/artifacts/presentations/edge_weighted_bisection/figures')
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
pipeline_output_dir = Path(f'../../../outputs/us_{args.year}_{args.version}')
if not pipeline_output_dir.exists():
    print(f"[WARNING] Pipeline outputs not found at: {pipeline_output_dir}")
    print("          Round progression figures will not be available.")
    print(f"          Run: python scripts/pipeline/run_complete_redistricting.py --year {args.year} --version {args.version}")
    print()
else:
    # Minnesota round maps (3 rounds -> 8 districts)
    minnesota_rounds = [
        (f'../../../outputs/us_{args.year}_{args.version}/states/minnesota/maps/rounds/round_01.png',
         'minnesota_round_1_2_regions.png'),
        (f'../../../outputs/us_{args.year}_{args.version}/states/minnesota/maps/rounds/round_02.png',
         'minnesota_round_2_4_regions.png'),
        (f'../../../outputs/us_{args.year}_{args.version}/states/minnesota/maps/rounds/round_03.png',
         'minnesota_round_3_8_regions.png'),
    ]

    # Alabama round maps (3 rounds -> 7 districts)
    alabama_rounds = [
        (f'../../../outputs/us_{args.year}_{args.version}/states/alabama/maps/rounds/round_01.png',
         'alabama_round_1_2_regions.png'),
        (f'../../../outputs/us_{args.year}_{args.version}/states/alabama/maps/rounds/round_02.png',
         'alabama_round_2_4_regions.png'),
        (f'../../../outputs/us_{args.year}_{args.version}/states/alabama/maps/rounds/round_03.png',
         'alabama_round_3_7_regions.png'),
    ]

    # Copy all round maps
    copied_count = 0
    for source, dest in minnesota_rounds + alabama_rounds:
        source_path = Path(source)
        dest_path = figures_dir / dest

        if source_path.exists():
            shutil.copy2(source_path, dest_path)
            print(f"  [OK] Copied: {dest}")
            copied_count += 1
        else:
            print(f"  [MISSING] {source}")

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
# Figure 6: Real Census Tracts to Graph Transformation with METIS Cut
# =============================================================================

import geopandas as gpd
import warnings
warnings.filterwarnings('ignore')

# Define example configurations for various states and partition ratios
EXAMPLE_CONFIGS = [
    {
        'name': 'minneapolis_50_50',
        'title': 'Minneapolis, MN (50-50 Split)',
        'state_fips': '27',
        'state_name': 'mn',
        'county_fips': '053',  # Hennepin County
        'nparts': 2,
        'target_weights': [0.5, 0.5],
        'n_tracts': 12,
        'start_idx': 0
    },
    {
        'name': 'los_angeles_67_33',
        'title': 'Los Angeles, CA (67-33 Split)',
        'state_fips': '06',
        'state_name': 'ca',
        'county_fips': '037',  # Los Angeles County
        'nparts': 2,
        'target_weights': [0.67, 0.33],
        'n_tracts': 12,
        'start_idx': 50
    },
    {
        'name': 'houston_60_40',
        'title': 'Houston, TX (60-40 Split)',
        'state_fips': '48',
        'state_name': 'tx',
        'county_fips': '201',  # Harris County
        'nparts': 2,
        'target_weights': [0.6, 0.4],
        'n_tracts': 12,
        'start_idx': 100
    },
    {
        'name': 'miami_75_25',
        'title': 'Miami, FL (75-25 Split)',
        'state_fips': '12',
        'state_name': 'fl',
        'county_fips': '086',  # Miami-Dade County
        'nparts': 2,
        'target_weights': [0.75, 0.25],
        'n_tracts': 12,
        'start_idx': 150
    },
    {
        'name': 'phoenix_80_20',
        'title': 'Phoenix, AZ (80-20 Split)',
        'state_fips': '04',
        'state_name': 'az',
        'county_fips': '013',  # Maricopa County
        'nparts': 2,
        'target_weights': [0.8, 0.2],
        'n_tracts': 12,
        'start_idx': 200
    },
    {
        'name': 'atlanta_55_45',
        'title': 'Atlanta, GA (55-45 Split)',
        'state_fips': '13',
        'state_name': 'ga',
        'county_fips': '121',  # Fulton County
        'nparts': 2,
        'target_weights': [0.55, 0.45],
        'n_tracts': 12,
        'start_idx': 75
    }
]

def generate_real_tracts_example(config, args, figures_dir):
    """Generate a single real tracts to graph example."""
    print(f"Creating: {config['title']}...")

    year_suffix = str(args.year)[-2:]
    state_fips = config['state_fips']
    state_name = config['state_name']

    # File paths
    tracts_file = Path(f'../../../data/geography/tiger_{args.year}_tracts/tl_{args.year}_{state_fips}_tract{year_suffix}/tl_{args.year}_{state_fips}_tract{year_suffix}.shp')
    population_file = Path(f'../../../data/processed/census_{args.year}/{state_name}_tracts_{args.year}_population.csv')

    geoid_field = f'GEOID{year_suffix}'
    county_field = f'COUNTYFP{year_suffix}'

    # Check if files exist
    if not tracts_file.exists():
        print(f"  [SKIP] Census tracts shapefile not found: {tracts_file}")
        return False
    if not population_file.exists():
        print(f"  [SKIP] Population data not found: {population_file}")
        return False

    # Continue with existing generation logic...
    # (I'll move the existing code into this function)
    return True

print("Generating real census tracts examples...")
print(f"Total examples: {len(EXAMPLE_CONFIGS)}")
print()

# Generate the original Minneapolis example first
print("Creating real census tracts to graph transformation with METIS cut...")

# Try to load real census tract data (Minnesota, FIPS 27) using specified year
year_suffix = str(args.year)[-2:]  # '10' from 2010, '20' from 2020
tracts_file = Path(f'../../../data/geography/tiger_{args.year}_tracts/tl_{args.year}_27_tract{year_suffix}/tl_{args.year}_27_tract{year_suffix}.shp')
population_file = Path(f'../../../data/processed/census_{args.year}/mn_tracts_{args.year}_population.csv')
geoid_field = f'GEOID{year_suffix}'  # GEOID10 for 2010, GEOID20 for 2020
county_field = f'COUNTYFP{year_suffix}'  # COUNTYFP10 for 2010, COUNTYFP20 for 2020

if not tracts_file.exists():
    print(f"  [WARNING] Census tracts shapefile not found at: {tracts_file}")
    print(f"            This is optional - skipping real tracts figure")
elif not population_file.exists():
    print(f"  [WARNING] Population data not found at: {population_file}")
    print(f"            This is optional - skipping real tracts figure")
else:
    try:
        import pandas as pd

        # Load tracts geometry
        tracts_gdf = gpd.read_file(tracts_file)

        # Load population data from CSV
        pop_df = pd.read_csv(population_file)

        # Create mapping from GEOID to population
        # CSV has 'GEOID' and 'population' columns
        pop_dict = dict(zip(pop_df['GEOID'].astype(str).str.zfill(11),
                           pop_df['population']))

        # Merge geometry with population (use year-specific GEOID field)
        tracts_gdf[geoid_field] = tracts_gdf[geoid_field].astype(str).str.zfill(11)
        tracts_gdf['population'] = tracts_gdf[geoid_field].map(pop_dict)

        # Filter to Hennepin County (Minneapolis area) and drop nulls
        hennepin_tracts = tracts_gdf[tracts_gdf[county_field] == '053'].copy()
        hennepin_tracts = hennepin_tracts[hennepin_tracts['population'].notna()]

        if len(hennepin_tracts) >= 6:
            # Build adjacency graph to find contiguous cluster
            from collections import deque

            def get_neighbors(tract_idx, gdf):
                """Get indices of adjacent tracts."""
                neighbors = []
                tract_geom = gdf.loc[tract_idx, 'geometry']
                for idx in gdf.index:
                    if idx != tract_idx:
                        if tract_geom.touches(gdf.loc[idx, 'geometry']):
                            neighbors.append(idx)
                return neighbors

            # Start with a tract in the middle (index 50 is arbitrary but likely central)
            start_idx = hennepin_tracts.index[min(50, len(hennepin_tracts) - 1)]

            # BFS to find 12 contiguous tracts
            selected_indices = [start_idx]
            queue = deque([start_idx])
            visited = {start_idx}

            while len(selected_indices) < 12 and queue:
                current = queue.popleft()
                neighbors = get_neighbors(current, hennepin_tracts)

                for neighbor in neighbors:
                    if neighbor not in visited:
                        visited.add(neighbor)
                        selected_indices.append(neighbor)
                        queue.append(neighbor)

                        if len(selected_indices) >= 12:
                            break

            # Extract our contiguous cluster
            sample_tracts = hennepin_tracts.loc[selected_indices].copy().reset_index(drop=True)

            # Build adjacency and calculate real boundary lengths
            n_tracts = len(sample_tracts)
            adjacency = {i: [] for i in range(n_tracts)}  # Initialize all at once
            edge_weights = {}

            for i in range(n_tracts):
                geom_i = sample_tracts.iloc[i].geometry

                for j in range(i + 1, n_tracts):
                    geom_j = sample_tracts.iloc[j].geometry

                    if geom_i.touches(geom_j):
                        # Calculate boundary length in km
                        boundary = geom_i.intersection(geom_j.boundary)
                        if not boundary.is_empty:
                            # Convert to meters then km (assuming projected CRS)
                            length_km = boundary.length / 1000
                            # If in degrees (unprojected), estimate
                            if length_km < 0.1:
                                length_km = boundary.length * 111  # rough deg to km

                            # Add to adjacency list (symmetric)
                            adjacency[i].append(j)
                            adjacency[j].append(i)
                            edge_weights[(i, j)] = length_km
                            edge_weights[(j, i)] = length_km

            # Run METIS to partition into 2 groups using the codebase wrapper
            try:
                # Add parent directory to path to import from src
                import sys
                sys.path.insert(0, str(Path('../../../src').resolve()))
                from apportionment.partition.metis_wrapper import partition_graph

                # Prepare data for METIS
                adjacency_list = [adjacency.get(i, []) for i in range(n_tracts)]
                vweights = np.array([int(sample_tracts.iloc[i]['population']) for i in range(n_tracts)])

                # Prepare edge weights dictionary for METIS
                edge_weights_dict = {}
                for (i, j), weight in edge_weights.items():
                    if i < j:  # Only include each edge once
                        edge_weights_dict[(i, j)] = weight

                # Run METIS with same parameters as run_state_redistricting
                # This ensures contiguous partitions via -contig flag
                membership = partition_graph(
                    adjacency=adjacency_list,
                    vertex_weights=vweights,
                    nparts=2,
                    target_weights=[0.5, 0.5],  # Equal population split
                    recursive=True,  # Use recursive bisection algorithm
                    ufactor=1.005,  # 0.5% population imbalance tolerance
                    edge_weights=edge_weights_dict,
                    debug=False  # Set True to see METIS command details
                )

                print(f"  Used METIS partitioning with contiguity enforcement")

            except Exception as e:
                print(f"  [WARNING] METIS not available ({e}), using simple cut")
                # Fallback: split by position
                centroids = [sample_tracts.iloc[i].geometry.centroid for i in range(n_tracts)]
                xs = [c.x for c in centroids]
                median_x = sorted(xs)[len(xs) // 2]
                membership = [0 if xs[i] < median_x else 1 for i in range(n_tracts)]

            # Identify cut edges
            cut_edges = []
            for i in range(n_tracts):
                for j in adjacency.get(i, []):
                    if i < j and membership[i] != membership[j]:
                        cut_edges.append((i, j))

            print(f"  Selected {n_tracts} contiguous tracts")
            print(f"  Cut edges: {len(cut_edges)}")

            # Create figure with two subplots - give more space to left panel
            fig = plt.figure(figsize=(16, 7))
            gs = fig.add_gridspec(1, 2, width_ratios=[1.4, 1], wspace=0.15)
            ax1 = fig.add_subplot(gs[0])
            ax2 = fig.add_subplot(gs[1])

            # Left: Geographic tracts colored by partition
            ax1.set_title('Census Tracts + METIS Cut\n(Geographic Reality)', fontsize=12, fontweight='bold')

            # Assign labels
            labels = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L'][:n_tracts]
            sample_tracts['label'] = labels
            sample_tracts['partition'] = membership

            # Colors by partition
            partition_colors = {0: 'lightblue', 1: 'lightcoral'}

            # Plot tracts colored by partition
            for idx in range(n_tracts):
                tract = sample_tracts.iloc[idx]
                color = partition_colors[membership[idx]]
                sample_tracts.iloc[[idx]].plot(
                    ax=ax1,
                    facecolor=color,
                    edgecolor='black',
                    linewidth=1.5,
                    alpha=0.7
                )

                # Add label and population at centroid
                centroid = tract.geometry.centroid
                pop_k = tract['population'] / 1000
                ax1.text(centroid.x, centroid.y, f'{labels[idx]}\n{pop_k:.1f}K',
                        ha='center', va='center',
                        fontsize=9, fontweight='bold',
                        bbox=dict(boxstyle='round', facecolor='white',
                                edgecolor='black', linewidth=1))

            # Label internal boundaries (non-cut edges)
            for i in range(n_tracts):
                for j in adjacency.get(i, []):
                    if i < j:  # Only label each edge once
                        is_cut = (i, j) in cut_edges or (j, i) in cut_edges
                        if not is_cut:
                            geom_i = sample_tracts.iloc[i].geometry
                            geom_j = sample_tracts.iloc[j].geometry
                            boundary = geom_i.intersection(geom_j.boundary)
                            if not boundary.is_empty:
                                # Get midpoint of boundary
                                mid_point = boundary.centroid
                                length_km = edge_weights.get((i, j), 0)
                                # Only label if length is significant (> 0.1 km)
                                if length_km > 0.1:
                                    ax1.text(mid_point.x, mid_point.y, f'{length_km:.1f}',
                                           ha='center', va='center', fontsize=7,
                                           bbox=dict(boxstyle='round', facecolor='white',
                                                   edgecolor='black', linewidth=0.5, alpha=0.8))

            # Highlight cut boundaries (thick red) and label them
            total_cut_length = 0
            for i, j in cut_edges:
                geom_i = sample_tracts.iloc[i].geometry
                geom_j = sample_tracts.iloc[j].geometry
                boundary = geom_i.intersection(geom_j.boundary)
                if not boundary.is_empty:
                    gpd.GeoSeries([boundary]).plot(ax=ax1, color='red', linewidth=4, alpha=0.9, zorder=10)
                    # Label cut edges (zorder=11 to appear on top of red line)
                    mid_point = boundary.centroid
                    length_km = edge_weights.get((i, j), edge_weights.get((j, i), 0))
                    total_cut_length += length_km
                    # Only label if length is significant (> 0.1 km)
                    if length_km > 0.1:
                        ax1.text(mid_point.x, mid_point.y, f'{length_km:.1f}',
                               ha='center', va='center', fontsize=7, fontweight='bold',
                               bbox=dict(boxstyle='round', facecolor='yellow',
                                       edgecolor='red', linewidth=1.5, alpha=0.9),
                               zorder=11)

            # Calculate perimeters
            part0_geoms = sample_tracts[sample_tracts['partition'] == 0].geometry
            part1_geoms = sample_tracts[sample_tracts['partition'] == 1].geometry
            combined_geom = sample_tracts.geometry.unary_union

            # Pre-cut perimeter (external boundary of all tracts)
            pre_cut_perimeter = combined_geom.boundary.length / 1000  # Convert to km
            # If in degrees (unprojected), convert to km
            if pre_cut_perimeter < 1:
                pre_cut_perimeter = combined_geom.boundary.length * 111

            # Calculate region populations and percentages
            pop0 = sample_tracts[sample_tracts['partition'] == 0]['population'].sum() / 1000
            pop1 = sample_tracts[sample_tracts['partition'] == 1]['population'].sum() / 1000
            total_pop = pop0 + pop1
            pct0 = (pop0 * 1000 / (total_pop * 1000)) * 100
            pct1 = (pop1 * 1000 / (total_pop * 1000)) * 100

            # Region labels at top - consistent position
            ax1.text(0.25, 0.92, f'Region 1\n{pop0:.1f}K\n{pct0:.1f}%',
                    transform=ax1.transAxes, ha='center', va='top',
                    fontsize=11, fontweight='bold',
                    bbox=dict(boxstyle='round', facecolor='lightblue',
                            edgecolor='blue', linewidth=1.5, alpha=0.9),
                    color='black')

            ax1.text(0.75, 0.92, f'Region 2\n{pop1:.1f}K\n{pct1:.1f}%',
                    transform=ax1.transAxes, ha='center', va='top',
                    fontsize=11, fontweight='bold',
                    bbox=dict(boxstyle='round', facecolor='lightcoral',
                            edgecolor='red', linewidth=1.5, alpha=0.9),
                    color='black')

            ax1.axis('off')

            # Add summary box with total cut length below the map
            ax1.text(0.5, -0.05,
                    f'Total Cut: {total_cut_length:.1f} km',
                    transform=ax1.transAxes, ha='center', va='top',
                    fontsize=9, fontweight='bold',
                    bbox=dict(boxstyle='round', facecolor='yellow',
                            edgecolor='red', linewidth=1.5, alpha=0.9))

            # Right: Abstract graph representation with cut
            ax2.set_title('Graph + METIS Cut\n(What Algorithm Sees)', fontsize=11, fontweight='bold')

            # Create layout for graph based on tract centroids
            centroids = sample_tracts.geometry.centroid
            xs = [c.x for c in centroids]
            ys = [c.y for c in centroids]

            # Normalize to [0, 4] range for nice plotting
            x_min, x_max = min(xs), max(xs)
            y_min, y_max = min(ys), max(ys)
            positions = {}
            for i in range(n_tracts):
                x_norm = 4 * (xs[i] - x_min) / (x_max - x_min) if x_max > x_min else 2
                y_norm = 4 * (ys[i] - y_min) / (y_max - y_min) if y_max > y_min else 2
                positions[i] = (x_norm, y_norm)

            # Draw edges (colored by whether they're cut)
            for i in range(n_tracts):
                for j in adjacency.get(i, []):
                    if i < j:  # Only draw each edge once
                        x1, y1 = positions[i]
                        x2, y2 = positions[j]

                        length_km = edge_weights.get((i, j), 0)

                        # Skip very small edges (corner adjacencies)
                        if length_km <= 0.1:
                            continue

                        is_cut = (i, j) in cut_edges or (j, i) in cut_edges

                        if is_cut:
                            # Cut edges: thick red dashed
                            ax2.plot([x1, x2], [y1, y2], 'r--',
                                   linewidth=4, alpha=0.9, zorder=2)
                            # Add X marker
                            mid_x, mid_y = (x1 + x2) / 2, (y1 + y2) / 2
                            ax2.plot(mid_x, mid_y, 'rX', markersize=12, markeredgewidth=3, zorder=3)
                        else:
                            # Non-cut edges: gray, thickness by weight
                            thickness = min(6, max(1.5, length_km / 3))
                            ax2.plot([x1, x2], [y1, y2], 'k-',
                                   linewidth=thickness, alpha=0.4, zorder=1)

                        # Add weight label (offset from edge so X marks don't obscure)
                        mid_x, mid_y = (x1 + x2) / 2, (y1 + y2) / 2

                        # Calculate offset perpendicular to edge
                        dx, dy = x2 - x1, y2 - y1
                        edge_length = np.sqrt(dx**2 + dy**2)
                        if edge_length > 0:
                            # Perpendicular offset (rotate 90 degrees)
                            offset_x = -dy / edge_length * 0.2
                            offset_y = dx / edge_length * 0.2
                        else:
                            offset_x, offset_y = 0, 0

                        # Position label off to the side of edge
                        label_x = mid_x + offset_x
                        label_y = mid_y + offset_y

                        # Only label if length is significant (> 0.1 km)
                        if length_km > 0.1:
                            # Style labels: yellow for cut edges, white for non-cut
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

            # Draw nodes colored by partition
            for i in range(n_tracts):
                x, y = positions[i]
                color = partition_colors[membership[i]]
                pop_k = sample_tracts.iloc[i]['population'] / 1000

                # Slightly larger circle to accommodate text
                circle = Circle((x, y), 0.3, facecolor=color,
                              edgecolor='black', linewidth=2.5, alpha=0.8, zorder=4)
                ax2.add_patch(circle)

                # Put label and population inside node (stacked vertically)
                # Match map styling: fontsize=9, bold for both
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

            ax2.set_xlim(-0.5, 4.8)  # Expanded to fit repositioned red label
            ax2.set_ylim(-0.7, 5.0)  # Adjusted for repositioned red label
            ax2.set_aspect('equal')
            ax2.axis('off')

            # Add total cut summary box below the graph
            total_cut_weight = sum(edge_weights.get((i, j), 0) for i, j in cut_edges)
            ax2.text(0.5, 0.05,
                    f'Total Cut: {total_cut_weight:.1f} km',
                    transform=ax2.transAxes, ha='center', va='top',
                    fontsize=9, fontweight='bold',
                    bbox=dict(boxstyle='round', facecolor='yellow',
                            edgecolor='red', linewidth=1.5, alpha=0.9))

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
