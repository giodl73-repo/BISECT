"""
Generate Figure 1: National Districts Map for Paper #13.

Shows all 435 districts colored by cross-state status:
  - Blue: Within-state districts (254 districts)
  - Red: Cross-state districts (181 districts)

Usage:
    python scripts/experimental/generate_figure1_national_map.py --year 2020
"""

import argparse
import pickle
from pathlib import Path
from typing import Dict

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

from config.download_sources import STATE_FIPS, STATE_NAMES


def load_national_districts(input_dir: Path, year: int) -> Dict:
    """Load national district assignments."""
    print(f"\nLoading national districts...")

    districts_file = input_dir / f"national_districts_{year}.pkl"
    with open(districts_file, 'rb') as f:
        data = pickle.load(f)

    print(f"  {data['n_districts']} districts")
    print(f"  {data['n_cross_state_districts']} cross-state districts")

    return data


def load_cross_state_analysis(input_dir: Path, year: int) -> Dict:
    """Load cross-state district analysis."""
    print(f"\nLoading cross-state analysis...")

    analysis_file = input_dir / f"cross_state_analysis_{year}.pkl"
    with open(analysis_file, 'rb') as f:
        data = pickle.load(f)

    cross_state_districts = set([d['district'] for d in data['cross_state_districts']])

    print(f"  {len(cross_state_districts)} cross-state districts identified")

    return cross_state_districts


def load_tract_geometries(data_dir: Path, year: int, geoid_to_index: Dict[str, int]) -> gpd.GeoDataFrame:
    """Load tract geometries for all states."""
    print(f"\nLoading tract geometries...")

    all_tracts = []

    for state_abbr, fips in tqdm(STATE_FIPS.items(), desc="Loading tracts"):
        state_name = STATE_NAMES[state_abbr]

        # Try GPKG first, then shapefile
        tract_file_gpkg = data_dir / f"{year}/tiger/tracts/{state_name}/tracts_{state_name}_{year}.gpkg"
        tract_file_shp = data_dir / f"{year}/tiger/tracts/tl_{year}_{fips}_tract/tl_{year}_{fips}_tract.shp"

        if tract_file_gpkg.exists():
            tract_file = tract_file_gpkg
        elif tract_file_shp.exists():
            tract_file = tract_file_shp
        else:
            continue

        gdf = gpd.read_file(tract_file)

        # Only keep tracts in our graph
        gdf = gdf[gdf['GEOID'].isin(geoid_to_index.keys())]

        all_tracts.append(gdf[['GEOID', 'geometry']])

    # Concatenate all states
    national_gdf = gpd.GeoDataFrame(
        pd.concat(all_tracts, ignore_index=True),
        crs=all_tracts[0].crs
    )

    print(f"  Loaded {len(national_gdf):,} tract geometries")

    return national_gdf


def compute_district_geometries(
    tracts_gdf: gpd.GeoDataFrame,
    geoid_to_district: Dict[str, int],
    cross_state_districts: set
) -> gpd.GeoDataFrame:
    """Compute district geometries with cross-state flags."""
    print(f"\nComputing district geometries...")

    # Add district assignments
    tracts_gdf = tracts_gdf.copy()
    tracts_gdf['district'] = tracts_gdf['GEOID'].map(geoid_to_district)
    tracts_gdf = tracts_gdf[tracts_gdf['district'].notna()]

    # Dissolve by district
    print(f"  Dissolving {len(tracts_gdf):,} tracts into 435 districts...")
    districts_gdf = tracts_gdf.dissolve(by='district', as_index=False)

    # Add cross-state flag
    districts_gdf['cross_state'] = districts_gdf['district'].isin(cross_state_districts)

    n_cross = districts_gdf['cross_state'].sum()
    n_within = len(districts_gdf) - n_cross

    print(f"  {n_cross} cross-state districts")
    print(f"  {n_within} within-state districts")

    return districts_gdf


def generate_national_map(
    districts_gdf: gpd.GeoDataFrame,
    output_file: Path,
    dpi: int = 300
) -> None:
    """
    Generate national districts map with cross-state highlighting.

    Args:
        districts_gdf: GeoDataFrame with district geometries and cross_state flag
        output_file: Output PNG file path
        dpi: Figure DPI
    """
    print(f"\nGenerating national map...")

    # Project to Albers Equal Area for US
    districts_proj = districts_gdf.to_crs('ESRI:102003')

    # Create figure
    fig, ax = plt.subplots(figsize=(16, 10))

    # Color by cross-state status
    # Within-state: blue, Cross-state: red
    within_state = districts_proj[~districts_proj['cross_state']]
    cross_state = districts_proj[districts_proj['cross_state']]

    # Plot within-state districts
    within_state.plot(
        ax=ax,
        color='#2E7D96',  # Blue
        edgecolor='white',
        linewidth=0.3,
        alpha=0.8
    )

    # Plot cross-state districts
    cross_state.plot(
        ax=ax,
        color='#C44E52',  # Red
        edgecolor='white',
        linewidth=0.3,
        alpha=0.8
    )

    # Remove axes
    ax.set_xlim(districts_proj.total_bounds[0], districts_proj.total_bounds[2])
    ax.set_ylim(districts_proj.total_bounds[1], districts_proj.total_bounds[3])
    ax.axis('off')

    # Add legend
    legend_elements = [
        Patch(facecolor='#2E7D96', edgecolor='white', label=f'Within-State Districts (n={len(within_state)})'),
        Patch(facecolor='#C44E52', edgecolor='white', label=f'Cross-State Districts (n={len(cross_state)})')
    ]
    ax.legend(
        handles=legend_elements,
        loc='lower right',
        fontsize=11,
        frameon=True,
        fancybox=True,
        shadow=True
    )

    # Add title
    ax.set_title(
        'National Redistricting Without State Boundaries: 435 Congressional Districts',
        fontsize=16,
        fontweight='bold',
        pad=20
    )

    # Adjust layout
    plt.tight_layout()

    # Save
    print(f"  Saving to {output_file}...")
    plt.savefig(output_file, dpi=dpi, bbox_inches='tight')
    plt.close()

    print(f"  [OK] Figure saved ({output_file.stat().st_size / 1e6:.1f} MB)")


def main():
    parser = argparse.ArgumentParser(description="Generate Figure 1 - National Districts Map")
    parser.add_argument('--year', type=int, default=2020, help='Census year')
    parser.add_argument('--data-dir', type=Path, default=Path('data'), help='Data directory')
    parser.add_argument('--input-dir', type=Path, default=Path('outputs/experimental'), help='Input directory')
    parser.add_argument('--output-dir', type=Path, default=Path('research/13+national-redistricting/figures'), help='Output directory')
    parser.add_argument('--dpi', type=int, default=300, help='Figure DPI')
    args = parser.parse_args()

    # Create output directory
    args.output_dir.mkdir(parents=True, exist_ok=True)

    print("="*70)
    print("Figure 1: National Districts Map (Paper #13)")
    print("="*70)

    # Load data
    district_data = load_national_districts(args.input_dir, args.year)
    cross_state_districts = load_cross_state_analysis(args.input_dir, args.year)
    tracts_gdf = load_tract_geometries(args.data_dir, args.year, district_data['geoid_to_index'])

    # Compute district geometries
    districts_gdf = compute_district_geometries(
        tracts_gdf,
        district_data['geoid_to_district'],
        cross_state_districts
    )

    # Generate figure
    output_file = args.output_dir / f"figure1_national_map_{args.year}.png"
    generate_national_map(districts_gdf, output_file, args.dpi)

    print("\n" + "="*70)
    print("Figure 1 complete!")
    print("="*70)
    print(f"Output: {output_file}")


if __name__ == '__main__':
    main()
