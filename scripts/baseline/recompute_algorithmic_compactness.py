"""
Recompute compactness scores for algorithmic districts with proper projection.

The original us_2020_v1 results computed compactness in geographic coordinates
(NAD83 degrees), not projected coordinates (meters). This script recomputes
the scores with proper Albers Equal Area projection.

Input:
- State-level district assignments from outputs/us_2020_v1/states/*/final_assignments.pkl
- State tract shapefiles from data/raw/

Output:
- Corrected us_district_summary.csv with properly projected compactness scores
"""

import argparse
import pickle
import sys
from pathlib import Path

import geopandas as gpd
import numpy as np
import pandas as pd
from shapely.ops import unary_union

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from config_2020 import STATE_CONFIG_2020


def compute_polsby_popper(geometry):
    """Compute Polsby-Popper score."""
    area = geometry.area
    perimeter = geometry.length

    if perimeter == 0:
        return 0.0

    score = (4 * np.pi * area) / (perimeter ** 2)
    return min(1.0, score)


def compute_reock(geometry):
    """Compute Reock score (minimum bounding circle)."""
    area = geometry.area

    # Get centroid and maximum distance to any vertex
    centroid = geometry.centroid
    if geometry.geom_type == 'Polygon':
        coords = list(geometry.exterior.coords)
    elif geometry.geom_type == 'MultiPolygon':
        coords = []
        for poly in geometry.geoms:
            coords.extend(list(poly.exterior.coords))
    else:
        return 0.0

    max_dist = max(centroid.distance(gpd.points_from_xy([c[0]], [c[1]])[0]) for c in coords)
    circle_area = np.pi * (max_dist ** 2)

    if circle_area == 0:
        return 0.0

    return min(1.0, area / circle_area)


def recompute_state_compactness(state: str, assignments_file: Path, tracts_file: Path, year: int) -> pd.DataFrame:
    """
    Recompute compactness for one state with proper projection.

    Parameters
    ----------
    state : str
        State abbreviation (e.g., 'AL')
    assignments_file : Path
        Path to final_assignments.pkl
    tracts_file : Path
        Path to state tracts parquet file
    year : int
        Census year (2010 or 2020)

    Returns
    -------
    pd.DataFrame
        District compactness scores with columns: state, district, polsby_popper, reock
    """
    # Load district assignments
    with open(assignments_file, 'rb') as f:
        assignments = pickle.load(f)

    # Load tracts
    tracts = gpd.read_parquet(tracts_file)

    # Add district assignments
    tracts['district'] = tracts.index.map(assignments)

    # Reproject to appropriate CRS
    if state == 'AK':
        tracts = tracts.to_crs(epsg=3338)  # Alaska Albers
    elif state == 'HI':
        tracts = tracts.to_crs(epsg=2784)  # Hawaii NAD83
    else:
        tracts = tracts.to_crs(epsg=5070)  # Contiguous US Albers

    # Compute compactness for each district
    results = []
    for district_id in sorted(tracts['district'].unique()):
        district_tracts = tracts[tracts['district'] == district_id]
        district_geom = unary_union(district_tracts.geometry)

        pp = compute_polsby_popper(district_geom)
        reock = compute_reock(district_geom)

        results.append({
            'state': state,
            'district': district_id,
            'polsby_popper': pp,
            'reock': reock,
        })

    return pd.DataFrame(results)


def main():
    parser = argparse.ArgumentParser(
        description='Recompute algorithmic district compactness with proper projection'
    )
    parser.add_argument(
        '--year',
        type=int,
        default=2020,
        choices=[2010, 2020],
        help='Census year (default: 2020)'
    )
    parser.add_argument(
        '--version',
        type=str,
        default='v1',
        help='Output version (default: v1)'
    )
    parser.add_argument(
        '--output-dir',
        type=Path,
        default=None,
        help='Output directory (default: outputs/us_{year}_{version})'
    )

    args = parser.parse_args()

    if args.output_dir is None:
        args.output_dir = Path(f'outputs/us_{args.year}_{args.version}')

    print("=" * 60)
    print("Recomputing Algorithmic District Compactness")
    print("=" * 60)
    print(f"Year: {args.year}")
    print(f"Version: {args.version}")
    print(f"Output: {args.output_dir}")
    print()

    # Load original summary to preserve other columns
    original_summary = pd.read_csv(args.output_dir / 'us_district_summary.csv')

    # Recompute compactness for each state
    all_compactness = []

    for state, config in sorted(STATE_CONFIG_2020.items()):
        if config['districts'] == 0:
            continue

        print(f"Processing {state}...", end=" ", flush=True)

        # Convert state name to full lowercase name (e.g., 'AL' -> 'alabama')
        state_full = config['name'].lower().replace(' ', '_')
        assignments_file = args.output_dir / 'states' / state_full / 'final_assignments.pkl'
        tracts_file = Path(f'data/raw/{state.lower()}_tracts_{args.year}.parquet')

        if not assignments_file.exists():
            print(f"SKIP (no assignments file)")
            continue

        if not tracts_file.exists():
            print(f"SKIP (no tracts file)")
            continue

        try:
            state_compactness = recompute_state_compactness(state, assignments_file, tracts_file, args.year)
            # Convert state code to full state name for merging
            state_compactness['state'] = config['name']
            all_compactness.append(state_compactness)
            print(f"OK ({len(state_compactness)} districts)")
        except Exception as e:
            print(f"ERROR: {e}")

    # Concatenate all results
    compactness_df = pd.concat(all_compactness, ignore_index=True)

    # Merge with original summary to replace PP/Reock columns
    updated_summary = original_summary.drop(columns=['polsby_popper', 'reock'])
    updated_summary = pd.merge(
        updated_summary,
        compactness_df,
        on=['state', 'district'],
        how='left'
    )

    # Reorder columns to match original
    column_order = list(original_summary.columns)
    updated_summary = updated_summary[column_order]

    # Save updated summary
    output_file = args.output_dir / 'us_district_summary_projected.csv'
    updated_summary.to_csv(output_file, index=False)

    print(f"\nOK Saved corrected summary: {output_file}")

    # Print comparison
    print("\n" + "=" * 60)
    print("Comparison: Original vs Projected")
    print("=" * 60)
    print(f"Original Mean PP:   {original_summary['polsby_popper'].mean():.4f}")
    print(f"Projected Mean PP:  {updated_summary['polsby_popper'].mean():.4f}")
    print(f"Original Mean Reock: {original_summary['reock'].mean():.4f}")
    print(f"Projected Mean Reock: {updated_summary['reock'].mean():.4f}")
    print("=" * 60)


if __name__ == '__main__':
    main()
