"""
Generate Figure 2: Regional Zooms (4 panels) for Paper #13.

Shows detailed views of 4 regions with cross-state districts:
  Panel A: Northeast (NY-NJ-CT-PA)
  Panel B: Midwest (IL-IN-OH-MI)
  Panel C: South (VA-NC-TN-KY)
  Panel D: West (CA-NV-OR-WA)

Each panel shows district boundaries with cross-state districts highlighted in red.

Usage:
    python scripts/experimental/generate_figure2_regional_zooms.py --year 2020
"""

import argparse
import pickle
from pathlib import Path
from typing import Dict, List, Tuple

import geopandas as gpd
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
from matplotlib.patches import Patch
from tqdm import tqdm

# Add src and scripts to path
import sys
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root / "src"))
sys.path.insert(0, str(project_root / "scripts"))

from config.download_sources import STATE_FIPS, STATE_NAMES, FIPS_TO_STATE


def load_national_districts(input_dir: Path, year: int) -> Dict:
    """Load national district assignments."""
    print(f"\nLoading national districts...")

    districts_file = input_dir / f"national_districts_{year}.pkl"
    with open(districts_file, 'rb') as f:
        data = pickle.load(f)

    return data


def load_cross_state_analysis(input_dir: Path, year: int) -> set:
    """Load cross-state district IDs."""
    print(f"Loading cross-state analysis...")

    analysis_file = input_dir / f"cross_state_analysis_{year}.pkl"
    with open(analysis_file, 'rb') as f:
        data = pickle.load(f)

    cross_state_districts = set([d['district'] for d in data['cross_state_districts']])
    print(f"  {len(cross_state_districts)} cross-state districts")

    return cross_state_districts


def load_tract_geometries_for_states(
    data_dir: Path,
    year: int,
    state_abbrs: List[str],
    geoid_to_index: Dict[str, int]
) -> gpd.GeoDataFrame:
    """Load tract geometries for specific states."""
    print(f"\nLoading tract geometries for {', '.join(state_abbrs)}...")

    all_tracts = []

    for state_abbr in tqdm(state_abbrs, desc="Loading tracts"):
        fips = STATE_FIPS[state_abbr]
        state_name = STATE_NAMES[state_abbr]

        # Try GPKG first, then shapefile
        tract_file_gpkg = data_dir / f"{year}/tiger/tracts/{state_name}/tracts_{state_name}_{year}.gpkg"
        tract_file_shp = data_dir / f"{year}/tiger/tracts/tl_{year}_{fips}_tract/tl_{year}_{fips}_tract.shp"

        if tract_file_gpkg.exists():
            tract_file = tract_file_gpkg
        elif tract_file_shp.exists():
            tract_file = tract_file_shp
        else:
            print(f"  [WARNING] No tract file found for {state_abbr}")
            continue

        gdf = gpd.read_file(tract_file)

        # Only keep tracts in our graph
        gdf = gdf[gdf['GEOID'].isin(geoid_to_index.keys())]

        # Add state info
        gdf['STATE'] = state_abbr

        all_tracts.append(gdf[['GEOID', 'geometry', 'STATE']])

    # Concatenate
    regional_gdf = gpd.GeoDataFrame(
        pd.concat(all_tracts, ignore_index=True),
        crs=all_tracts[0].crs
    )

    print(f"  Loaded {len(regional_gdf):,} tract geometries")

    return regional_gdf


def compute_regional_districts(
    tracts_gdf: gpd.GeoDataFrame,
    geoid_to_district: Dict[str, int],
    cross_state_districts: set
) -> gpd.GeoDataFrame:
    """Compute district geometries for region."""
    # Add district assignments
    tracts_gdf = tracts_gdf.copy()
    tracts_gdf['district'] = tracts_gdf['GEOID'].map(geoid_to_district)
    tracts_gdf = tracts_gdf[tracts_gdf['district'].notna()]

    # Dissolve by district
    districts_gdf = tracts_gdf.dissolve(by='district', as_index=False)

    # Add cross-state flag
    districts_gdf['cross_state'] = districts_gdf['district'].isin(cross_state_districts)

    return districts_gdf


def load_state_boundaries(
    data_dir: Path,
    year: int,
    state_abbrs: List[str]
) -> gpd.GeoDataFrame:
    """Load state boundary geometries."""
    state_file = data_dir / f"{year}/tiger/states/tl_{year}_us_state.shp"

    if not state_file.exists():
        print(f"  [WARNING] State boundaries not found at {state_file}")
        return None

    states_gdf = gpd.read_file(state_file)

    # Filter to our states
    fips_list = [STATE_FIPS[s] for s in state_abbrs]
    states_gdf = states_gdf[states_gdf['STATEFP'].isin(fips_list)]

    return states_gdf[['STATEFP', 'NAME', 'geometry']]


def plot_regional_panel(
    ax,
    districts_gdf: gpd.GeoDataFrame,
    states_gdf: gpd.GeoDataFrame,
    title: str
):
    """Plot a single regional panel."""
    # Project to Albers
    districts_proj = districts_gdf.to_crs('ESRI:102003')
    states_proj = states_gdf.to_crs('ESRI:102003') if states_gdf is not None else None

    # Plot state boundaries first (if available)
    if states_proj is not None:
        states_proj.plot(
            ax=ax,
            color='none',
            edgecolor='black',
            linewidth=1.5,
            alpha=0.5
        )

    # Separate by cross-state status
    within_state = districts_proj[~districts_proj['cross_state']]
    cross_state = districts_proj[districts_proj['cross_state']]

    # Plot within-state districts (blue)
    if len(within_state) > 0:
        within_state.plot(
            ax=ax,
            color='#2E7D96',
            edgecolor='white',
            linewidth=0.5,
            alpha=0.8
        )

    # Plot cross-state districts (red)
    if len(cross_state) > 0:
        cross_state.plot(
            ax=ax,
            color='#C44E52',
            edgecolor='white',
            linewidth=0.5,
            alpha=0.8
        )

    # Set limits
    ax.set_xlim(districts_proj.total_bounds[0], districts_proj.total_bounds[2])
    ax.set_ylim(districts_proj.total_bounds[1], districts_proj.total_bounds[3])
    ax.axis('off')

    # Add title
    ax.set_title(title, fontsize=12, fontweight='bold', pad=10)

    # Add statistics text
    n_total = len(districts_proj)
    n_cross = len(cross_state)
    pct_cross = n_cross / n_total * 100 if n_total > 0 else 0

    stats_text = f"{n_total} districts\n{n_cross} cross-state ({pct_cross:.0f}%)"
    ax.text(
        0.02, 0.98,
        stats_text,
        transform=ax.transAxes,
        fontsize=9,
        verticalalignment='top',
        bbox=dict(boxstyle='round', facecolor='white', alpha=0.8)
    )


def generate_regional_zooms(
    data_dir: Path,
    input_dir: Path,
    year: int,
    geoid_to_index: Dict[str, int],
    geoid_to_district: Dict[str, int],
    cross_state_districts: set,
    output_file: Path,
    dpi: int = 300
):
    """Generate 4-panel regional zoom figure."""
    print(f"\nGenerating regional zooms...")

    # Define regions
    regions = [
        {
            'name': 'Northeast',
            'title': 'A. Northeast (NY-NJ-CT-PA)',
            'states': ['NY', 'NJ', 'CT', 'PA']
        },
        {
            'name': 'Midwest',
            'title': 'B. Midwest (IL-IN-OH-MI)',
            'states': ['IL', 'IN', 'OH', 'MI']
        },
        {
            'name': 'South',
            'title': 'C. South (VA-NC-TN-KY)',
            'states': ['VA', 'NC', 'TN', 'KY']
        },
        {
            'name': 'West',
            'title': 'D. West (CA-NV-OR-WA)',
            'states': ['CA', 'NV', 'OR', 'WA']
        }
    ]

    # Create 2x2 figure
    fig, axes = plt.subplots(2, 2, figsize=(16, 12))
    axes = axes.flatten()

    for i, region in enumerate(regions):
        print(f"\n  Panel {i+1}: {region['name']}...")

        # Load tracts for this region
        tracts_gdf = load_tract_geometries_for_states(
            data_dir,
            year,
            region['states'],
            geoid_to_index
        )

        # Compute district geometries
        districts_gdf = compute_regional_districts(
            tracts_gdf,
            geoid_to_district,
            cross_state_districts
        )

        print(f"    {len(districts_gdf)} districts in region")

        # Load state boundaries
        states_gdf = load_state_boundaries(data_dir, year, region['states'])

        # Plot panel
        plot_regional_panel(
            axes[i],
            districts_gdf,
            states_gdf,
            region['title']
        )

    # Add legend to figure
    legend_elements = [
        Patch(facecolor='#2E7D96', edgecolor='white', label='Within-State Districts'),
        Patch(facecolor='#C44E52', edgecolor='white', label='Cross-State Districts')
    ]
    fig.legend(
        handles=legend_elements,
        loc='lower center',
        ncol=2,
        fontsize=11,
        frameon=True,
        fancybox=True,
        shadow=True,
        bbox_to_anchor=(0.5, -0.01)
    )

    # Overall title
    fig.suptitle(
        'Regional Analysis: Cross-State Districts in Four US Regions',
        fontsize=16,
        fontweight='bold',
        y=0.98
    )

    # Adjust layout
    plt.tight_layout(rect=[0, 0.02, 1, 0.96])

    # Save
    print(f"\n  Saving to {output_file}...")
    plt.savefig(output_file, dpi=dpi, bbox_inches='tight')
    plt.close()

    print(f"  [OK] Figure saved ({output_file.stat().st_size / 1e6:.1f} MB)")


def main():
    parser = argparse.ArgumentParser(description="Generate Figure 2 - Regional Zooms")
    parser.add_argument('--year', type=int, default=2020, help='Census year')
    parser.add_argument('--data-dir', type=Path, default=Path('data'), help='Data directory')
    parser.add_argument('--input-dir', type=Path, default=Path('outputs/experimental'), help='Input directory')
    parser.add_argument('--output-dir', type=Path, default=Path('research/13+national-redistricting/figures'), help='Output directory')
    parser.add_argument('--dpi', type=int, default=300, help='Figure DPI')
    args = parser.parse_args()

    # Create output directory
    args.output_dir.mkdir(parents=True, exist_ok=True)

    print("="*70)
    print("Figure 2: Regional Zooms (Paper #13)")
    print("="*70)

    # Load national data
    district_data = load_national_districts(args.input_dir, args.year)
    cross_state_districts = load_cross_state_analysis(args.input_dir, args.year)

    # Generate figure
    output_file = args.output_dir / f"figure2_regional_zooms_{args.year}.png"
    generate_regional_zooms(
        args.data_dir,
        args.input_dir,
        args.year,
        district_data['geoid_to_index'],
        district_data['geoid_to_district'],
        cross_state_districts,
        output_file,
        args.dpi
    )

    print("\n" + "="*70)
    print("Figure 2 complete!")
    print("="*70)
    print(f"Output: {output_file}")


if __name__ == '__main__':
    main()
