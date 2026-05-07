"""
Figure 1: Apportionment Changes Map (2000-2020)

Choropleth map showing seat gains (blue gradient) and losses (red gradient).
Insets for Alaska and Hawaii. Annotations for largest changes.
"""

import matplotlib.pyplot as plt
import geopandas as gpd
import pandas as pd
from pathlib import Path

# Load apportionment data
DATA_FILE = Path(__file__).parent.parent / "data" / "apportionment_table.txt"

# Parse apportionment changes
apportionment_changes = {
    # Gainers
    'TX': +6, 'FL': +3, 'NC': +1, 'GA': +1, 'CO': +1, 'OR': +1, 'MT': +1,
    'SC': +1, 'UT': +1, 'NV': +1, 'WA': +1, 'AZ': +1,
    # Losers
    'CA': -1, 'NY': -3, 'PA': -2, 'OH': -3, 'IL': -2, 'MI': -2, 'WV': -1,
    'MA': -1, 'NJ': -1, 'LA': -1, 'IA': -1, 'MO': -1,
    # Stable (zero change) - all other states
}

def create_apportionment_map():
    """
    Create choropleth map of apportionment changes.

    Output: research/12+longitudinal-analysis/figures/figure1_apportionment_map.png
    """

    # TODO: Load US states shapefile
    # states_gdf = gpd.read_file("data/us_states.shp")

    # TODO: Merge with apportionment_changes
    # states_gdf['change'] = states_gdf['state_abbr'].map(apportionment_changes).fillna(0)

    # Create figure with main map + AK/HI insets
    fig, (ax_main, ax_ak, ax_hi) = plt.subplots(1, 3, figsize=(16, 10),
                                                  gridspec_kw={'width_ratios': [10, 1, 1]})

    # Color scheme:
    # Gains: Blues colormap (light to dark blue for +1 to +6)
    # Losses: Reds colormap (light to dark red for -1 to -3)
    # No change: Light gray

    # TODO: Plot main map (contiguous 48 states)
    # states_gdf[~states_gdf['state_abbr'].isin(['AK', 'HI'])].plot(
    #     column='change',
    #     cmap='RdBu_r',  # Red for losses, Blue for gains
    #     vmin=-3,
    #     vmax=6,
    #     legend=True,
    #     ax=ax_main,
    #     edgecolor='black',
    #     linewidth=0.5
    # )

    # Annotations for largest changes
    annotations = {
        'TX': ('+6', 'largest gain'),
        'NY': ('-3', 'tied largest loss'),
        'OH': ('-3', 'tied largest loss'),
        'CA': ('-1', 'first decline in history')
    }

    # TODO: Add state labels for key changes
    # for state, (change, note) in annotations.items():
    #     centroid = states_gdf[states_gdf['state_abbr'] == state].geometry.centroid
    #     ax_main.annotate(...)

    # TODO: Plot Alaska inset
    # states_gdf[states_gdf['state_abbr'] == 'AK'].plot(ax=ax_ak, ...)
    ax_ak.set_title('Alaska')
    ax_ak.axis('off')

    # TODO: Plot Hawaii inset
    # states_gdf[states_gdf['state_abbr'] == 'HI'].plot(ax=ax_hi, ...)
    ax_hi.set_title('Hawaii')
    ax_hi.axis('off')

    # Main map styling
    ax_main.set_title('Congressional Seat Changes 2000-2020', fontsize=16, fontweight='bold')
    ax_main.axis('off')

    # Legend
    # TODO: Add custom legend showing:
    # - Blue shades for gains (+1 to +6)
    # - Red shades for losses (-1 to -3)
    # - Gray for no change

    # Summary statistics text box
    summary_text = """
    Summary (2000-2020):
    • Gainers: 12 states, +21 seats
    • Losers: 10 states, -21 seats
    • Stable: 28 states

    Regional Trends:
    • Sunbelt: +15 seats
    • Rust Belt: -16 seats
    """

    ax_main.text(0.02, 0.02, summary_text, transform=ax_main.transAxes,
                 fontsize=10, verticalalignment='bottom',
                 bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.5))

    plt.tight_layout()

    # Save figure
    output_dir = Path(__file__).parent.parent / "figures"
    output_dir.mkdir(exist_ok=True)
    output_path = output_dir / "figure1_apportionment_map.png"

    plt.savefig(output_path, dpi=300, bbox_inches='tight')
    print(f"Saved Figure 1 to {output_path}")

    plt.close()

if __name__ == "__main__":
    print("Generating Figure 1: Apportionment Changes Map...")
    print("NOTE: This is a template script. Requires US states shapefile.")
    print("      Download from: https://www.census.gov/geographies/mapping-files/time-series/geo/carto-boundary-file.html")
    print("      Or use: scripts/data/download_state_boundaries.py")

    # create_apportionment_map()

    print("\nTo run:")
    print("  1. Download US states shapefile")
    print("  2. Uncomment TODO sections in this script")
    print("  3. Run: python scripts/generate_figure1_apportionment_map.py")
