#!/usr/bin/env python3
"""
Calculate compactness metrics for congressional districts.

Computes Polsby-Popper and Reock scores for each district and adds
them to the district_summary.csv file.

Metrics:
- Polsby-Popper: 4π × area / perimeter² (range 0-1, 1 = perfect circle)
- Reock: area / minimum_bounding_circle_area (range 0-1)
- Convex Hull Ratio: area / convex_hull_area (range 0-1)
- Perimeter: Total boundary length in kilometers

Usage:
    python scripts/analyze_district_compactness.py <state_directory>

Example:
    python scripts/analyze_district_compactness.py outputs/compactness-testing/california
"""

import sys
import pickle
from pathlib import Path
import pandas as pd
import geopandas as gpd
import numpy as np
from shapely.ops import unary_union
from shapely.geometry import Point


def polsby_popper_score(geometry):
    """
    Calculate Polsby-Popper compactness score and perimeter.

    Formula: 4π × area / perimeter²
    Range: 0 to 1, where 1 is a perfect circle

    Parameters
    ----------
    geometry : shapely geometry
        District geometry

    Returns
    -------
    tuple (float, float)
        Polsby-Popper score and perimeter in meters
    """
    area = geometry.area
    perimeter = geometry.length

    if perimeter == 0:
        return 0.0, 0.0

    score = (4 * np.pi * area) / (perimeter ** 2)
    return min(1.0, score), perimeter  # Cap score at 1.0 due to numerical precision


def minimum_bounding_circle(geometry):
    """
    Compute minimum bounding circle using Welzl's algorithm approximation.

    For simplicity, we use the centroid and max distance to boundary
    as an approximation.

    Parameters
    ----------
    geometry : shapely geometry
        District geometry

    Returns
    -------
    float
        Radius of minimum bounding circle
    """
    centroid = geometry.centroid

    # Get boundary points
    if geometry.geom_type == 'Polygon':
        boundary_coords = list(geometry.exterior.coords)
    elif geometry.geom_type == 'MultiPolygon':
        boundary_coords = []
        for poly in geometry.geoms:
            boundary_coords.extend(list(poly.exterior.coords))
    else:
        return 0.0

    # Find maximum distance from centroid to any boundary point
    max_dist = 0.0
    for coord in boundary_coords:
        point = Point(coord)
        dist = centroid.distance(point)
        max_dist = max(max_dist, dist)

    return max_dist


def reock_score(geometry):
    """
    Calculate Reock compactness score.

    Formula: area / area_of_minimum_bounding_circle
    Range: 0 to 1, where 1 is a perfect circle

    Parameters
    ----------
    geometry : shapely geometry
        District geometry

    Returns
    -------
    float
        Reock score
    """
    area = geometry.area
    radius = minimum_bounding_circle(geometry)

    if radius == 0:
        return 0.0

    circle_area = np.pi * (radius ** 2)
    score = area / circle_area
    return min(1.0, score)  # Cap at 1.0


def convex_hull_ratio(geometry):
    """
    Calculate convex hull ratio.

    Formula: area / convex_hull_area
    Range: 0 to 1, where 1 means the shape is already convex

    Parameters
    ----------
    geometry : shapely geometry
        District geometry

    Returns
    -------
    float
        Convex hull ratio
    """
    area = geometry.area
    convex_area = geometry.convex_hull.area

    if convex_area == 0:
        return 0.0

    return area / convex_area


def calculate_metrics_for_state(state_dir):
    """
    Calculate compactness metrics for all districts in a state.

    Parameters
    ----------
    state_dir : Path
        Path to state output directory
    """
    state_dir = Path(state_dir)

    print(f"\n{'='*70}")
    print(f"Calculating Compactness Metrics")
    print(f"{'='*70}")
    print(f"State directory: {state_dir}")

    # Map state directory name to state code
    state_name = state_dir.name

    # State name to FIPS code mapping
    state_codes = {
        'alabama': 'al', 'alaska': 'ak', 'arizona': 'az', 'arkansas': 'ar',
        'california': 'ca', 'colorado': 'co', 'connecticut': 'ct', 'delaware': 'de',
        'florida': 'fl', 'georgia': 'ga', 'hawaii': 'hi', 'idaho': 'id',
        'illinois': 'il', 'indiana': 'in', 'iowa': 'ia', 'kansas': 'ks',
        'kentucky': 'ky', 'louisiana': 'la', 'maine': 'me', 'maryland': 'md',
        'massachusetts': 'ma', 'michigan': 'mi', 'minnesota': 'mn', 'mississippi': 'ms',
        'missouri': 'mo', 'montana': 'mt', 'nebraska': 'ne', 'nevada': 'nv',
        'new_hampshire': 'nh', 'new_jersey': 'nj', 'new_mexico': 'nm', 'new_york': 'ny',
        'north_carolina': 'nc', 'north_dakota': 'nd', 'ohio': 'oh', 'oklahoma': 'ok',
        'oregon': 'or', 'pennsylvania': 'pa', 'rhode_island': 'ri', 'south_carolina': 'sc',
        'south_dakota': 'sd', 'tennessee': 'tn', 'texas': 'tx', 'utah': 'ut',
        'vermont': 'vt', 'virginia': 'va', 'washington': 'wa', 'west_virginia': 'wv',
        'wisconsin': 'wi', 'wyoming': 'wy'
    }

    state_code = state_codes.get(state_name.lower(), state_name[:2].lower())
    print(f"State: {state_name} ({state_code.upper()})")

    # Load tract geometries from raw data
    tracts_file = Path('data/raw') / f'{state_code}_tracts_2020.parquet'

    if not tracts_file.exists():
        print(f"ERROR: Tract geometries file not found: {tracts_file}")
        return False

    print(f"Loading tract geometries from {tracts_file.name}...")
    tracts_gdf = gpd.read_parquet(tracts_file)
    print(f"  Loaded {len(tracts_gdf):,} tracts")
    print(f"  Original CRS: {tracts_gdf.crs}")

    # Reproject to Albers Equal Area (EPSG:5070) for accurate area/perimeter in meters
    # This is the standard projection for US-wide analysis
    if tracts_gdf.crs != 'EPSG:5070':
        print(f"  Reprojecting to EPSG:5070 (Albers Equal Area) for metric calculations...")
        tracts_gdf = tracts_gdf.to_crs('EPSG:5070')
        print(f"  Reprojected CRS: {tracts_gdf.crs}")

    # Load district assignments
    assignments_file = state_dir / 'final_assignments.pkl'

    if not assignments_file.exists():
        print(f"ERROR: District assignments not found: {assignments_file}")
        return False

    print(f"Loading district assignments from {assignments_file.name}...")
    with open(assignments_file, 'rb') as f:
        assignments = pickle.load(f)

    # Merge assignments into tracts GeoDataFrame
    # assignments is a dict: {tract_id: district_id}
    tracts_gdf['district'] = tracts_gdf.index.map(assignments)

    # Filter out tracts without assignments (shouldn't happen but be safe)
    tracts_gdf = tracts_gdf[tracts_gdf['district'].notna()].copy()

    districts = sorted(tracts_gdf['district'].unique())
    print(f"  Found {len(districts)} districts")

    # Calculate metrics for each district
    print(f"\nCalculating compactness metrics...")
    metrics_data = []

    for district_id in districts:
        print(f"  District {district_id}...", end=" ")

        # Get all tracts in this district
        district_tracts = tracts_gdf[tracts_gdf['district'] == district_id]

        # Dissolve into single geometry
        district_geom = unary_union(district_tracts.geometry)

        # Calculate metrics
        pp_score, perimeter_m = polsby_popper_score(district_geom)
        reock = reock_score(district_geom)
        convex_ratio = convex_hull_ratio(district_geom)

        # Convert perimeter from meters to kilometers
        perimeter_km = perimeter_m / 1000.0

        metrics_data.append({
            'district': district_id,
            'polsby_popper': round(pp_score, 4),
            'reock': round(reock, 4),
            'convex_hull_ratio': round(convex_ratio, 4),
            'perimeter_km': round(perimeter_km, 2)
        })

        print(f"PP={pp_score:.4f}, Reock={reock:.4f}, Convex={convex_ratio:.4f}, Perim={perimeter_km:.2f}km")

    # Create metrics dataframe
    metrics_df = pd.DataFrame(metrics_data)

    # Load existing district summary from data/ subdirectory
    summary_file = state_dir / 'data' / 'district_summary.csv'

    if not summary_file.exists():
        print(f"\nWARNING: district_summary.csv not found")
        print(f"Creating new summary with compactness metrics only")
        compactness_dir = state_dir / 'compactness'
        compactness_dir.mkdir(parents=True, exist_ok=True)
        output_file = compactness_dir / 'district_compactness.csv'
        metrics_df.to_csv(output_file, index=False)
        print(f"Saved to: {output_file}")
    else:
        print(f"\nMerging with existing district_summary.csv...")
        summary_df = pd.read_csv(summary_file)

        # Drop existing compactness columns if they exist (to avoid duplicates)
        compactness_cols = ['polsby_popper', 'reock', 'convex_hull_ratio', 'perimeter_km']
        for col in compactness_cols:
            if col in summary_df.columns:
                summary_df = summary_df.drop(columns=[col])

        # Merge metrics into summary
        summary_df = summary_df.merge(metrics_df, on='district', how='left')

        # Save updated summary
        summary_df.to_csv(summary_file, index=False)
        print(f"Updated: {summary_file}")

    # Print statistics
    print(f"\n{'='*70}")
    print(f"Compactness Statistics")
    print(f"{'='*70}")
    print(f"Polsby-Popper scores:")
    print(f"  Mean:   {metrics_df['polsby_popper'].mean():.4f}")
    print(f"  Median: {metrics_df['polsby_popper'].median():.4f}")
    print(f"  Min:    {metrics_df['polsby_popper'].min():.4f}")
    print(f"  Max:    {metrics_df['polsby_popper'].max():.4f}")

    print(f"\nReock scores:")
    print(f"  Mean:   {metrics_df['reock'].mean():.4f}")
    print(f"  Median: {metrics_df['reock'].median():.4f}")
    print(f"  Min:    {metrics_df['reock'].min():.4f}")
    print(f"  Max:    {metrics_df['reock'].max():.4f}")

    print(f"\nConvex hull ratios:")
    print(f"  Mean:   {metrics_df['convex_hull_ratio'].mean():.4f}")
    print(f"  Median: {metrics_df['convex_hull_ratio'].median():.4f}")
    print(f"  Min:    {metrics_df['convex_hull_ratio'].min():.4f}")
    print(f"  Max:    {metrics_df['convex_hull_ratio'].max():.4f}")

    print(f"\nPerimeters (km):")
    print(f"  Total:  {metrics_df['perimeter_km'].sum():.2f} km")
    print(f"  Mean:   {metrics_df['perimeter_km'].mean():.2f} km")
    print(f"  Median: {metrics_df['perimeter_km'].median():.2f} km")
    print(f"  Min:    {metrics_df['perimeter_km'].min():.2f} km")
    print(f"  Max:    {metrics_df['perimeter_km'].max():.2f} km")

    # Interpretation guide
    print(f"\n{'='*70}")
    print(f"Interpretation Guide (Polsby-Popper)")
    print(f"{'='*70}")
    print(f"  Gerrymandered:        0.05-0.15")
    print(f"  Current US Congress:  0.15-0.25")
    print(f"  Algorithmic:          0.20-0.35")
    print(f"  Iowa-style (compact): 0.30-0.45")
    print(f"  Perfect circle:       1.00")
    print(f"{'='*70}")

    return True


def main():
    """Calculate compactness metrics for a state."""

    if len(sys.argv) < 2:
        print("Usage: python scripts/analyze_district_compactness.py <state_directory>")
        print("Example: python scripts/analyze_district_compactness.py outputs/compactness-testing/california")
        return 1

    state_dir = sys.argv[1]

    try:
        success = calculate_metrics_for_state(state_dir)
        return 0 if success else 1
    except Exception as e:
        print(f"\nERROR: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == '__main__':
    sys.exit(main())
