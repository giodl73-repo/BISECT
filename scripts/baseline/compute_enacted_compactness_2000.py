"""
Compute compactness scores for enacted 2000 Congressional Districts.

Computes Polsby-Popper and Reock scores for all enacted districts (108th Congress)
across all 50 states, matching the metrics used for algorithmic districts.

Note: Using CD108 (108th Congress, 2003-2005) which uses the same district boundaries
as CD107 (2001-2003), both based on 2000 Census redistricting.

Input:
- Downloaded NHGIS or TIGER/Line shapefile for CD108
- Expected location: data/enacted_districts/2000/

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

from config_2000 import STATE_CONFIG_2000


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


def detect_district_column(gdf):
    """
    Detect which column contains the district number.

    NHGIS and TIGER/Line use different column names.
    Common patterns: CD108FP, CD108, DISTRICT, DISTRICTID, etc.
    """
    possible_columns = ['CD108FP', 'CD108', 'DISTRICT', 'DISTRICTID', 'CD', 'CONG_DIST']

    for col in gdf.columns:
        col_upper = col.upper()
        if col_upper in possible_columns or 'CD' in col_upper and '108' in col_upper:
            print(f"[OK] Detected district column: {col}")
            return col

    # Fallback: look for any column with 'CD' or 'DISTRICT'
    for col in gdf.columns:
        col_upper = col.upper()
        if 'DISTRICT' in col_upper or col_upper == 'CD':
            print(f"[OK] Using district column: {col}")
            return col

    print("[WARN] Could not auto-detect district column")
    print(f"       Available columns: {list(gdf.columns)}")
    return None


def load_and_compute_compactness(shapefile_path: Path) -> pd.DataFrame:
    """
    Load enacted districts from nationwide file and compute compactness scores.

    Parameters
    ----------
    shapefile_path : Path
        Path to CD108 shapefile (zip or directory)

    Returns
    -------
    pd.DataFrame
        Compactness scores with columns:
        - state_code
        - state_name
        - district_num
        - polsby_popper
        - reock
        - perimeter_m
        - area_m2
    """
    print(f"Loading CD108 shapefile (108th Congress, 2000 Census boundaries)...")
    print(f"File: {shapefile_path}\n")

    # Read shapefile (from zip or directory)
    if shapefile_path.suffix == '.zip':
        gdf = gpd.read_file(f"zip://{shapefile_path}")
    else:
        gdf = gpd.read_file(shapefile_path)

    print(f"Loaded {len(gdf)} congressional districts")
    print(f"Columns: {list(gdf.columns)}\n")

    # Detect district column
    district_col = detect_district_column(gdf)
    if district_col is None:
        print("ERROR: Could not detect district number column")
        print("Please specify manually or check the shapefile structure")
        sys.exit(1)

    # Filter out invalid districts (ZZ, 98, 99 = not defined, usually water/territories)
    invalid_values = ['ZZ', '98', '99', 98, 99]
    gdf = gdf[~gdf[district_col].isin(invalid_values)].copy()

    print(f"After filtering: {len(gdf)} valid districts\n")

    # Add state code from STATEFP (or similar column)
    statefp_col = None
    for col in gdf.columns:
        col_upper = col.upper()
        if 'STATEFP' in col_upper or col_upper in ['STATE_FIPS', 'STATEFIPS', 'FIPS', 'ST_FIPS']:
            statefp_col = col
            print(f"[OK] Found state FIPS column: {col}")
            break

    if statefp_col:
        # Extract just the FIPS code (first 2 digits if longer)
        fips_values = gdf[statefp_col].astype(str).str[:2]
        gdf['state_code'] = fips_values.map(FIPS_TO_STATE)
    else:
        print("[WARN] No STATEFP column found, trying alternative methods...")
        # Try to extract from GEOID or similar
        for col in gdf.columns:
            if 'GEOID' in col.upper() or 'GISJOIN' in col.upper():
                # First 2 digits are usually state FIPS
                gdf['state_code'] = gdf[col].astype(str).str[:3].str[-2:].map(FIPS_TO_STATE)
                print(f"[OK] Extracted state FIPS from {col}")
                break

    # Add state name from config
    def get_state_name(state_code):
        if pd.isna(state_code):
            return None
        return STATE_CONFIG_2000.get(state_code.upper() if isinstance(state_code, str) else state_code, {}).get('name', state_code)

    gdf['state_name'] = gdf['state_code'].map(get_state_name)

    # Extract district number
    # Note: '00' or '0' means at-large district, treat as district 1
    try:
        gdf['district_num'] = pd.to_numeric(gdf[district_col], errors='coerce')
        gdf.loc[gdf['district_num'] == 0, 'district_num'] = 1
    except Exception as e:
        print(f"[WARN] Error converting district numbers: {e}")
        gdf['district_num'] = 1

    all_results = []
    total_districts = 0

    # Group by state and process
    valid_states = [s for s in gdf['state_code'].unique() if pd.notna(s)]
    for state_code in sorted(valid_states):

        state_districts = gdf[gdf['state_code'] == state_code].copy()

        if len(state_districts) == 0:
            continue

        # Reproject to appropriate projected CRS for accurate area/perimeter calculation
        if state_code == 'AK':
            # Alaska: use Alaska Albers (EPSG:3338)
            state_districts = state_districts.to_crs(epsg=3338)
        elif state_code == 'HI':
            # Hawaii: use NAD83 Hawaii (EPSG:2784)
            state_districts = state_districts.to_crs(epsg=2784)
        else:
            # Contiguous US: use Albers Equal Area (EPSG:5070)
            state_districts = state_districts.to_crs(epsg=5070)

        # Compute compactness for each district in this state
        for idx, row in state_districts.iterrows():
            geom = row['geometry']
            district_num = row['district_num']
            state_name = row['state_name']

            # Compute area and perimeter
            area = geom.area  # m^2
            perimeter = geom.length  # m

            # Polsby-Popper
            if perimeter > 0:
                pp = (4 * np.pi * area) / (perimeter ** 2)
            else:
                pp = 0.0

            # Reock
            reock = compute_reock_score(geom)

            all_results.append({
                'state_code': state_code,
                'state_name': state_name,
                'district_num': int(district_num),
                'polsby_popper': pp,
                'reock': reock,
                'area_m2': area,
                'perimeter_m': perimeter
            })

            total_districts += 1

        print(f"Processed {state_code}: {len(state_districts)} districts")

    print(f"\nTotal districts processed: {total_districts}")

    if total_districts != 435:
        print(f"\n[WARN] Expected 435 districts, got {total_districts}")
        print(f"       This may be OK if some states/territories are excluded")

    return pd.DataFrame(all_results)


def main():
    parser = argparse.ArgumentParser(
        description='Compute compactness for enacted 2000 congressional districts (CD108)'
    )
    parser.add_argument(
        '--shapefile',
        type=Path,
        help='Path to CD108 shapefile (zip file or directory)'
    )
    parser.add_argument(
        '--output',
        type=Path,
        default=Path('data/enacted_districts/2000/enacted_compactness_2000.csv'),
        help='Output CSV file (default: data/enacted_districts/2000/enacted_compactness_2000.csv)'
    )

    args = parser.parse_args()

    # Auto-detect shapefile if not provided
    if not args.shapefile:
        search_paths = [
            Path('data/enacted_districts/2000'),
            Path('data/geography'),
        ]

        for search_dir in search_paths:
            if not search_dir.exists():
                continue

            # Look for CD108 files
            candidates = list(search_dir.rglob('*cd108*.zip')) + \
                        list(search_dir.rglob('*cd108*.shp')) + \
                        list(search_dir.rglob('*CD108*.zip')) + \
                        list(search_dir.rglob('*CD108*.shp'))

            if candidates:
                args.shapefile = candidates[0]
                print(f"[OK] Auto-detected shapefile: {args.shapefile}\n")
                break

    if not args.shapefile:
        print("ERROR: No shapefile specified and auto-detection failed")
        print("\nPlease specify shapefile path:")
        print("  python scripts/baseline/compute_enacted_compactness_2000.py --shapefile <path>")
        print("\nSearched in:")
        print("  - data/enacted_districts/2000/")
        print("  - data/geography/")
        sys.exit(1)

    if not args.shapefile.exists():
        print(f"ERROR: Shapefile not found: {args.shapefile}")
        sys.exit(1)

    # Compute compactness
    df = load_and_compute_compactness(args.shapefile)

    # Save results
    args.output.parent.mkdir(parents=True, exist_ok=True)
    df.to_csv(args.output, index=False)

    print(f"\n{'=' * 60}")
    print(f"Compactness Results (2000 Census - CD108)")
    print(f"{'=' * 60}")
    print(f"\nSummary Statistics (Polsby-Popper):")
    print(f"  Mean:   {df['polsby_popper'].mean():.4f}")
    print(f"  Median: {df['polsby_popper'].median():.4f}")
    print(f"  Min:    {df['polsby_popper'].min():.4f}")
    print(f"  Max:    {df['polsby_popper'].max():.4f}")
    print(f"  Std:    {df['polsby_popper'].std():.4f}")

    print(f"\nSummary Statistics (Reock):")
    print(f"  Mean:   {df['reock'].mean():.4f}")
    print(f"  Median: {df['reock'].median():.4f}")
    print(f"  Min:    {df['reock'].min():.4f}")
    print(f"  Max:    {df['reock'].max():.4f}")
    print(f"  Std:    {df['reock'].std():.4f}")

    print(f"\nResults saved to: {args.output}")
    print(f"{'=' * 60}")


if __name__ == '__main__':
    main()
