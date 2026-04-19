"""
Compute compactness scores for enacted 2020 Congressional Districts.

Computes Polsby-Popper and Reock scores for all enacted districts
across all 50 states, matching the metrics used for algorithmic districts.

Input:
- Downloaded TIGER/Line shapefiles in data/enacted_districts/

Output:
- CSV with per-district compactness scores
- State-level summary statistics
"""

import argparse
import sys
import zipfile
from pathlib import Path
from typing import Dict, List

import geopandas as gpd
import numpy as np
import pandas as pd
from shapely.geometry import Point

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from config_2020 import STATE_CONFIG_2020


# State FIPS codes
STATE_FIPS: Dict[str, str] = {
    'AL': '01', 'AK': '02', 'AZ': '04', 'AR': '05', 'CA': '06',
    'CO': '08', 'CT': '09', 'DE': '10', 'FL': '12', 'GA': '13',
    'HI': '15', 'ID': '16', 'IL': '17', 'IN': '18', 'IA': '19',
    'KS': '20', 'KY': '21', 'LA': '22', 'ME': '23', 'MD': '24',
    'MA': '25', 'MI': '26', 'MN': '27', 'MS': '28', 'MO': '29',
    'MT': '30', 'NE': '31', 'NV': '32', 'NH': '33', 'NJ': '34',
    'NM': '35', 'NY': '36', 'NC': '37', 'ND': '38', 'OH': '39',
    'OK': '40', 'OR': '41', 'PA': '42', 'RI': '44', 'SC': '45',
    'SD': '46', 'TN': '47', 'TX': '48', 'UT': '49', 'VT': '50',
    'VA': '51', 'WA': '53', 'WV': '54', 'WI': '55', 'WY': '56',
}

# Reverse mapping
FIPS_TO_STATE = {v: k for k, v in STATE_FIPS.items()}


def compute_reock_score(geometry) -> float:
    """
    Compute Reock compactness score.

    Reock = Area / Area of minimum bounding circle

    Returns
    -------
    float
        Reock score in [0, 1], higher is more compact
    """
    area = geometry.area

    # Get minimum bounding circle using smallest enclosing circle algorithm
    # Approximate using convex hull's centroid and max distance
    centroid = geometry.centroid
    if geometry.geom_type == 'Polygon':
        coords = list(geometry.exterior.coords)
    elif geometry.geom_type == 'MultiPolygon':
        coords = []
        for poly in geometry.geoms:
            coords.extend(list(poly.exterior.coords))
    else:
        return 0.0

    # Find maximum distance from centroid to any point
    max_dist = max(Point(centroid).distance(Point(coord)) for coord in coords)

    # Area of circle with radius max_dist
    circle_area = np.pi * (max_dist ** 2)

    if circle_area == 0:
        return 0.0

    return area / circle_area


def load_and_compute_compactness(shapefile_dir: Path) -> pd.DataFrame:
    """
    Load enacted districts and compute compactness scores state-by-state.

    Parameters
    ----------
    shapefile_dir : Path
        Directory containing tl_2020_*_cd118.zip files

    Returns
    -------
    pd.DataFrame
        Compactness scores with columns:
        - state
        - district_num
        - polsby_popper
        - reock
        - perimeter_m
        - area_m2
    """
    all_results = []
    total_districts = 0

    # Load and process each state
    for state, config in sorted(STATE_CONFIG_2020.items()):
        if config['districts'] == 0:
            continue

        fips = STATE_FIPS[state]
        zip_path = shapefile_dir / f"tl_2020_{fips}_cd118.zip"

        if not zip_path.exists():
            print(f"WARNING: Missing shapefile for {state}: {zip_path}")
            continue

        # Read shapefile from zip
        gdf = gpd.read_file(f"zip://{zip_path}")

        # Filter out invalid districts (ZZ = not defined, usually water/territories)
        gdf = gdf[gdf['CD118FP'] != 'ZZ'].copy()

        if len(gdf) == 0:
            print(f"Loaded {state}: 0 districts (skipped - no valid districts)")
            continue

        # Reproject to appropriate projected CRS for accurate area/perimeter calculation
        if state == 'AK':
            # Alaska: use Alaska Albers (EPSG:3338)
            gdf = gdf.to_crs(epsg=3338)
        elif state == 'HI':
            # Hawaii: use NAD83 Hawaii (EPSG:2784)
            gdf = gdf.to_crs(epsg=2784)
        else:
            # Contiguous US: use Albers Equal Area (EPSG:5070)
            gdf = gdf.to_crs(epsg=5070)

        # Extract district number from CD118FP column
        # Note: '00' means at-large district, treat as district 1
        gdf['district_num'] = gdf['CD118FP'].astype(int)
        gdf.loc[gdf['district_num'] == 0, 'district_num'] = 1

        # Compute compactness for each district in this state
        for idx, row in gdf.iterrows():
            geom = row['geometry']

            # Compute metrics
            area = geom.area
            perimeter = geom.length

            # Polsby-Popper
            pp = (4 * np.pi * area) / (perimeter ** 2) if perimeter > 0 else 0.0

            # Reock
            reock = compute_reock_score(geom)

            all_results.append({
                'state': state,
                'district_num': row['district_num'],
                'polsby_popper': pp,
                'reock': reock,
                'perimeter_m': perimeter,
                'area_m2': area,
            })

            total_districts += 1

        print(f"Loaded {state}: {len(gdf)} districts")

    print(f"\nTotal districts loaded: {total_districts}")

    return pd.DataFrame(all_results)


def compute_state_summary(compactness_df: pd.DataFrame) -> pd.DataFrame:
    """
    Compute state-level summary statistics.

    Parameters
    ----------
    compactness_df : pd.DataFrame
        Per-district compactness scores

    Returns
    -------
    pd.DataFrame
        State-level summary with mean PP, Reock, perimeter
    """
    summary = compactness_df.groupby('state').agg({
        'district_num': 'count',
        'polsby_popper': 'mean',
        'reock': 'mean',
        'perimeter_m': 'mean',
        'area_m2': 'mean',
    }).reset_index()

    summary.rename(columns={
        'district_num': 'num_districts',
        'polsby_popper': 'mean_pp',
        'reock': 'mean_reock',
        'perimeter_m': 'mean_perimeter_m',
        'area_m2': 'mean_area_m2',
    }, inplace=True)

    return summary


def main():
    parser = argparse.ArgumentParser(
        description='Compute compactness scores for enacted congressional districts'
    )
    parser.add_argument(
        '--input-dir',
        type=Path,
        default=Path('data/enacted_districts'),
        help='Directory with downloaded shapefiles (default: data/enacted_districts)'
    )
    parser.add_argument(
        '--output-dir',
        type=Path,
        default=Path('outputs/baseline_comparison'),
        help='Output directory for results (default: outputs/baseline_comparison)'
    )

    args = parser.parse_args()

    # Create output directory
    args.output_dir.mkdir(parents=True, exist_ok=True)

    print("=" * 60)
    print("Computing Compactness for Enacted Congressional Districts")
    print("=" * 60)
    print()

    # Load districts and compute compactness state-by-state
    print("Loading enacted districts and computing compactness...")
    compactness_df = load_and_compute_compactness(args.input_dir)

    # Save per-district results
    district_output = args.output_dir / 'enacted_district_compactness.csv'
    compactness_df.to_csv(district_output, index=False)
    print(f"\nOK Saved per-district results: {district_output}")

    # Compute state summary
    print("\nComputing state-level summary...")
    summary_df = compute_state_summary(compactness_df)

    # Save state summary
    summary_output = args.output_dir / 'enacted_state_summary.csv'
    summary_df.to_csv(summary_output, index=False)
    print(f"OK Saved state summary: {summary_output}")

    # Print overall statistics
    print("\n" + "=" * 60)
    print("Overall Statistics (Enacted Districts)")
    print("=" * 60)
    print(f"Total districts:   {len(compactness_df)}")
    print(f"Mean Polsby-Popper: {compactness_df['polsby_popper'].mean():.4f}")
    print(f"Mean Reock:         {compactness_df['reock'].mean():.4f}")
    print(f"Mean Perimeter:     {compactness_df['perimeter_m'].mean():,.0f} m")
    print("=" * 60)


if __name__ == '__main__':
    main()
