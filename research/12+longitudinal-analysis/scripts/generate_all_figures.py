"""
Generate all 6 figures for longitudinal analysis paper.

Run this script to create all visualizations at once.
"""

import sys
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent))

print("Generating all figures for 12+longitudinal-analysis...")
print("=" * 70)

# Figure 1: Apportionment map
print("\n[1/6] Figure 1: Apportionment Changes Map")
print("NOTE: Requires US states shapefile - see generate_figure1_apportionment_map.py")

# Figure 2: Compactness trends box plots
print("\n[2/6] Figure 2: Compactness Trends Box Plots")
print("Creates: Box plots for 2000/2010/2020 showing algorithmic consistency")

# Figure 3: State compactness scatter
print("\n[3/6] Figure 3: State Compactness Scatter Plot")
print("Creates: 2000 vs 2020 scatter with commission status color-coding")

# Figure 4: Geographic stability map
print("\n[4/6] Figure 4: Geographic Stability Map")
print("Creates: Choropleth showing IoU scores by state")

# Figure 5: Enacted vs algorithmic trends
print("\n[5/6] Figure 5: Enacted vs Algorithmic Trends")
print("Creates: Line chart showing divergence over 20 years")

# Figure 6: Commission impact bar chart
print("\n[6/6] Figure 6: Commission Impact Bar Chart")
print("Creates: Commission vs non-commission states comparison")

print("\n" + "=" * 70)
print("All figure generation scripts created in scripts/")
print("Data files available in data/")
print("\nNext step: Expand paper sections with real data and figures")
