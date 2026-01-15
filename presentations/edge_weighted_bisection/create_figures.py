#!/usr/bin/env python3
"""
Create placeholder figures for the presentation.

Generates:
1. figures/example_gerrymander.png - Famous gerrymandered district example
2. figures/tract_to_graph.png - Visual showing tract-to-graph transformation
"""

import matplotlib.pyplot as plt
import matplotlib.patches as patches
from matplotlib.patches import FancyBboxPatch, Circle, FancyArrowPatch
import numpy as np
from pathlib import Path

# Create figures directory in outputs
figures_dir = Path('../../outputs/presentations/edge_weighted_bisection/figures')
figures_dir.mkdir(parents=True, exist_ok=True)

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

print("\nAll figures created successfully!")
print(f"Location: {figures_dir.resolve()}")
