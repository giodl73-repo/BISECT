"""
County Fragmentation Analysis

Analyzes how enacted congressional redistricting plans split counties
across multiple districts, destroying county autonomy and identity.

Key question: How many districts does each large county currently span?

For each census year (2000, 2010, 2020):
- Load enacted redistricting plans (shapefiles)
- Overlay with county boundaries
- Calculate fragmentation metrics:
  * Number of districts county spans
  * Population in each district piece
  * Fragmentation index

Compares:
- Current system: Large counties fragmented across many districts
- County system: Each county = autonomous entity

Usage:
    python scripts/analyze_county_fragmentation.py --year 2020 --state CA
    python scripts/analyze_county_fragmentation.py --year 2020  # All states
"""

import sys
from pathlib import Path
from typing import Dict, List, Tuple
import pandas as pd
import geopandas as gpd
from collections import defaultdict

# Add src to path
PROJECT_ROOT = Path(__file__).parent.parent.parent.parent
sys.path.insert(0, str(PROJECT_ROOT / 'src'))


def load_county_data(year: int) -> pd.DataFrame:
    """Load county population data."""
    county_file = PROJECT_ROOT / f'outputs/data/{year}/counties/all_counties_{year}.csv'
    if not county_file.exists():
        print(f"ERROR: County data not found: {county_file}")
        sys.exit(1)
    return pd.read_csv(county_file)


def load_enacted_districts(year: int, state: str) -> gpd.GeoDataFrame:
    """
    Load enacted redistricting plan for a state.

    DATA SOURCES NEEDED:
    1. Census Bureau TIGER/Line Congressional Districts:
       - URL: https://www2.census.gov/geo/tiger/TIGER{year}/CD/
       - Files: tl_{year}_us_cd116.zip (for 2020), cd113 (2010), cd110 (2000)
       - Format: Shapefile with congressional district boundaries

    2. Dave's Redistricting App (DRA) Plans:
       - URL: https://davesredistricting.org/
       - Export enacted plans as GeoJSON or Shapefile
       - Per-state historical enacted plans available

    3. Census 2020 Redistricting Data:
       - URL: https://www2.census.gov/geo/tiger/TIGER2020PL/
       - State Legislative District (SLD) files if needed

    TODO: Download and cache these shapefiles in:
          data/enacted/{year}/tl_{year}_us_cd.shp

    Args:
        year: Census year (2000, 2010, 2020)
        state: State abbreviation

    Returns:
        GeoDataFrame with district boundaries, or None
    """
    # Possible paths for enacted plans
    possible_paths = [
        PROJECT_ROOT / f'data/enacted/{year}/tl_{year}_us_cd.shp',
        PROJECT_ROOT / f'data/enacted/{year}/{state.lower()}_districts_{year}.shp',
        PROJECT_ROOT / f'data/redistricting/{year}/enacted/{state}_congressional_{year}.shp',
    ]

    for path in possible_paths:
        if path.exists():
            try:
                gdf = gpd.read_file(path)
                # Filter to specific state if loading national file
                if state and 'STATEFP' in gdf.columns:
                    from scripts.data.census_config import STATE_FIPS
                    state_fips = STATE_FIPS.get(state, '')
                    gdf = gdf[gdf['STATEFP'] == state_fips]
                return gdf
            except Exception as e:
                print(f"  Warning: Could not read {path}: {e}")

    return None


def load_county_geometries(year: int, state: str) -> gpd.GeoDataFrame:
    """
    Load county geometries for a state.

    DATA SOURCES:
    1. Census Bureau TIGER/Line County Boundaries:
       - URL: https://www2.census.gov/geo/tiger/TIGER{year}/COUNTY/
       - File: tl_{year}_us_county.zip
       - Format: Shapefile with county boundaries and FIPS codes

    2. Alternative: State-specific county files:
       - URL: https://www2.census.gov/geo/tiger/TIGER{year}/COUNTY/tl_{year}_{state_fips}_county.zip
       - Per-state downloads for smaller file sizes

    TODO: Download and cache in:
          data/tiger/counties/tl_{year}_us_county.shp

    Then merge with population data from:
          outputs/data/{year}/counties/all_counties_{year}.csv

    Args:
        year: Census year
        state: State abbreviation

    Returns:
        GeoDataFrame with county boundaries and populations, or None
    """
    # Try loading cached county geometries
    possible_paths = [
        PROJECT_ROOT / f'data/tiger/counties/tl_{year}_us_county.shp',
        PROJECT_ROOT / f'data/{year}/tiger/counties/tl_{year}_us_county.shp',
    ]

    for path in possible_paths:
        if path.exists():
            try:
                gdf = gpd.read_file(path)

                # Merge with population data
                county_csv = PROJECT_ROOT / f'outputs/data/{year}/counties/all_counties_{year}.csv'
                if county_csv.exists():
                    pop_df = pd.read_csv(county_csv)
                    gdf = gdf.merge(pop_df, left_on='GEOID', right_on='fips', how='left')

                    # Filter to specific state if requested
                    if state:
                        from scripts.data.census_config import STATE_FIPS
                        state_fips = STATE_FIPS.get(state, '')
                        gdf = gdf[gdf['STATEFP'] == state_fips]

                    return gdf

            except Exception as e:
                print(f"  Warning: Could not read {path}: {e}")

    return None


def calculate_fragmentation(
    counties_gdf: gpd.GeoDataFrame,
    districts_gdf: gpd.GeoDataFrame
) -> pd.DataFrame:
    """
    Calculate county fragmentation metrics.

    For each county:
    - Count how many districts it intersects
    - Calculate population in each district piece
    - Compute fragmentation index

    Args:
        counties_gdf: County boundaries with populations
        districts_gdf: District boundaries

    Returns:
        DataFrame with fragmentation metrics per county
    """
    results = []

    for idx, county in counties_gdf.iterrows():
        county_geom = county.geometry
        county_pop = county['population']
        county_fips = county['fips']

        # Find all districts that intersect this county
        intersecting = districts_gdf[districts_gdf.geometry.intersects(county_geom)]

        num_districts = len(intersecting)

        if num_districts == 0:
            continue

        # Calculate area/population in each district (approximation)
        district_pieces = []
        for _, district in intersecting.iterrows():
            intersection = county_geom.intersection(district.geometry)
            intersection_area = intersection.area
            county_area = county_geom.area

            # Approximate population in this piece
            if county_area > 0:
                pop_fraction = intersection_area / county_area
                pop_in_piece = county_pop * pop_fraction
            else:
                pop_in_piece = 0

            district_pieces.append({
                'district_id': district.get('DISTRICT', district.get('CD', 'unknown')),
                'population': pop_in_piece,
                'area_fraction': intersection_area / county_area if county_area > 0 else 0
            })

        # Fragmentation index (1 = fully contained, higher = more fragmented)
        # Use Herfindahl-Hirschman Index (sum of squared shares)
        shares = [p['area_fraction'] for p in district_pieces]
        hhi = sum(s**2 for s in shares)
        fragmentation_index = 1 / hhi if hhi > 0 else num_districts

        results.append({
            'fips': county_fips,
            'state': county['state'],
            'population': county_pop,
            'num_districts': num_districts,
            'district_pieces': district_pieces,
            'fragmentation_index': fragmentation_index
        })

    return pd.DataFrame(results)


def analyze_state_fragmentation(year: int, state: str):
    """
    Analyze county fragmentation for a state.

    Args:
        year: Census year
        state: State abbreviation
    """
    print(f"Analyzing {state} ({year})")
    print("-" * 80)

    # Load data
    enacted = load_enacted_districts(year, state)
    if enacted is None:
        print(f"  Enacted districts not found for {state} {year}")
        print(f"  Need: Shapefiles of enacted congressional districts")
        return None

    counties = load_county_geometries(year, state)
    if counties is None:
        print(f"  County geometries not found for {state} {year}")
        print(f"  Need: County boundary shapefiles with populations")
        return None

    # Calculate fragmentation
    fragmentation = calculate_fragmentation(counties, enacted)

    # Report results
    print(f"\n{state} County Fragmentation Summary:")
    print(f"  Total counties: {len(fragmentation)}")

    # Most fragmented counties
    most_fragmented = fragmentation.nlargest(10, 'num_districts')
    print(f"\n  Most fragmented counties (spans most districts):")
    for _, county in most_fragmented.iterrows():
        print(f"    {county['fips']}: {county['population']:>10,} people across {county['num_districts']} districts")
        print(f"      Fragmentation index: {county['fragmentation_index']:.2f}")

    # Counties fully contained (num_districts = 1)
    contained = fragmentation[fragmentation['num_districts'] == 1]
    print(f"\n  Counties fully within single district: {len(contained)} ({len(contained)/len(fragmentation)*100:.1f}%)")

    return fragmentation


def compare_to_county_system(year: int):
    """
    Compare current fragmentation to county-based system.

    Args:
        year: Census year
    """
    print(f"\nComparison: Current vs County-Based System ({year})")
    print("=" * 80)

    counties = load_county_data(year)

    # Threshold for analysis (counties that would get direct representation)
    threshold = 1_500_000  # 1.5M people

    large_counties = counties[counties['population'] >= threshold].copy()
    large_counties = large_counties.sort_values('population', ascending=False)

    print(f"\nLarge counties (>{threshold:,} people): {len(large_counties)}")
    print()

    print("Current System (Estimated Fragmentation):")
    print("  County                        Pop          Expected Districts  Status")
    print("-" * 80)

    ideal_district = 761_169

    for _, county in large_counties.head(20).iterrows():
        expected_districts = int(round(county['population'] / ideal_district))
        if expected_districts > 1:
            status = f"FRAGMENTED (likely {expected_districts}+ pieces)"
        else:
            status = "May be contained"

        print(f"  {county['fips']} ({county['state']})  {county['population']:>12,}  "
              f"{expected_districts:>2d}              {status}")

    print()
    print("County-Based System:")
    print("  Each large county = AUTONOMOUS ENTITY")
    print("  Los Angeles County (10M) = single political unit with 13 representatives")
    print("  Cook County (5.3M) = single political unit with 7 representatives")
    print("  No fragmentation - county identity preserved")
    print()


def run_analysis(year: int, states: List[str] = None):
    """
    Run fragmentation analysis.

    Args:
        year: Census year
        states: List of states to analyze (None = show comparison only)
    """
    print(f"County Fragmentation Analysis ({year})")
    print("=" * 80)
    print()

    if states:
        # Analyze specific states
        for state in states:
            result = analyze_state_fragmentation(year, state)
            if result is not None:
                print()
    else:
        # Show comparison
        compare_to_county_system(year)


if __name__ == '__main__':
    import argparse

    parser = argparse.ArgumentParser(
        description='Analyze how enacted plans fragment counties'
    )
    parser.add_argument('--year', type=int, required=True,
                       help='Census year (2000, 2010, 2020)')
    parser.add_argument('--states', type=str, nargs='+',
                       help='States to analyze (omit for comparison mode)')

    args = parser.parse_args()

    run_analysis(args.year, args.states)
