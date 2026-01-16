#!/usr/bin/env python3
"""Quick visualization of California tracts."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent / 'src'))

import geopandas as gpd
import matplotlib.pyplot as plt
import numpy as np

print("Loading California tracts...")
tracts = gpd.read_parquet('data/raw/ca_tracts_2020.parquet')

print(f"Loaded {len(tracts):,} tracts")
print(f"Total population: {tracts['population'].sum():,}")

# Create figure
fig, axes = plt.subplots(1, 2, figsize=(18, 10))

# Map 1: All tracts with boundaries
ax1 = axes[0]
tracts.plot(ax=ax1, facecolor='lightblue', edgecolor='white', linewidth=0.2, alpha=0.7)
ax1.set_title('California Census Tracts
(9,129 tracts)', fontsize=14, fontweight='bold')
ax1.set_axis_off()

# Map 2: Population density
ax2 = axes[1]
tracts['density'] = tracts['population'] / (tracts['ALAND'] / 1e6)  # people per km²
tracts.plot(
    column='density',
    ax=ax2,
    cmap='YlOrRd',
    legend=True,
    legend_kwds={'label': 'Population Density (people/km²)', 'orientation': 'horizontal'},
    edgecolor='white',
    linewidth=0.1,
    vmin=0,
    vmax=5000
)
ax2.set_title('California Population Density by Tract', fontsize=14, fontweight='bold')
ax2.set_axis_off()

plt.tight_layout()

output_file = Path('outputs/maps/ca_tracts_overview.png')
output_file.parent.mkdir(parents=True, exist_ok=True)

print(f"
Saving map to: {output_file}")
plt.savefig(output_file, dpi=300, bbox_inches='tight')
print("Map saved!")

# Also create a simple mock split for visualization
fig2, ax = plt.subplots(1, 1, figsize=(14, 12))

# Create a simple mock split - divide by latitude
median_lat = tracts.geometry.centroid.y.median()
tracts['mock_region'] = (tracts.geometry.centroid.y > median_lat).astype(int)

colors = ['#e74c3c', '#3498db']
for region_id in [0, 1]:
    region_tracts = tracts[tracts['mock_region'] == region_id]
    region_tracts.plot(
        ax=ax,
        color=colors[region_id],
        edgecolor='white',
        linewidth=0.1,
        alpha=0.7
    )

ax.set_title('California Mock Split (North/South by latitude)', fontsize=16, fontweight='bold')
ax.set_axis_off()

# Add legend
from matplotlib.patches import Patch
region_stats = tracts.groupby('mock_region')['population'].sum()
legend_elements = [
    Patch(facecolor=colors[0], label=f'South: {region_stats[0]:,} people'),
    Patch(facecolor=colors[1], label=f'North: {region_stats[1]:,} people')
]
ax.legend(handles=legend_elements, loc='lower right', fontsize=11, frameon=True, shadow=True)

plt.tight_layout()

mock_file = Path('outputs/maps/ca_tracts_mock_split.png')
print(f"
Saving mock split to: {mock_file}")
plt.savefig(mock_file, dpi=300, bbox_inches='tight')
print("Mock split saved!")

print(f"
Files saved:")
print(f"  Overview: {output_file}")
print(f"  Mock split: {mock_file}")
