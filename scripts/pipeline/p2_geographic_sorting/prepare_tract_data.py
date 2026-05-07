"""
Prepare Tract-Level Data for Geographic Sorting Analysis

This script combines:
1. Tract geometries (TIGER shapefiles)
2. Population data (Census PL 94-171)
3. Election results (2020 presidential)

Outputs a unified GeoDataFrame for each state with all necessary data.

Author: Claude
Date: 2026-02-07
"""

import geopandas as gpd
import pandas as pd
from pathlib import Path
import sys
from typing import Dict

# Add project root to path
project_root = Path(__file__).parents[3]
sys.path.insert(0, str(project_root))
sys.path.insert(0, str(project_root / "src"))

from scripts.config_2020 import STATE_CONFIG_2020
from scripts.config.download_sources import STATE_FIPS, STATE_NAMES

# Create helper dicts
NAME_TO_ABBREV = {v: k for k, v in STATE_NAMES.items()}  # lowercase_underscore -> abbrev
NAME_TO_FIPS = {name: STATE_FIPS[abbrev] for abbrev, name in STATE_NAMES.items()}  # lowercase_underscore -> FIPS

# Create STATES_2020 dict in expected format (lowercase_underscore_name -> districts)
STATES_2020 = {
    STATE_NAMES[abbrev]: config['districts']
    for abbrev, config in STATE_CONFIG_2020.items()
}


def load_tract_geometries(state_fips: str) -> gpd.GeoDataFrame:
    """Load TIGER tract shapefiles for a state."""
    tract_dir = Path(f"data/2020/tiger/tracts/tl_2020_{state_fips}_tract")
    shapefile = tract_dir / f"tl_2020_{state_fips}_tract.shp"

    if not shapefile.exists():
        return None

    gdf = gpd.read_file(shapefile)
    return gdf


def load_tract_population(state_name: str) -> pd.DataFrame:
    """
    Load tract-level population from Census PL 94-171 files.

    Uses the geo file which contains GEOID and total population.
    """
    state_abbr = state_name[:2]  # First 2 chars for state abbreviation
    geo_file = Path(f"data/2020/redistricting/{state_abbr}2020.pl/{state_abbr}geo2020.pl")

    if not geo_file.exists():
        # Try alternative naming
        fips = STATE_FIPS.get(state_name)
        if fips:
            geo_file = Path(f"data/2020/redistricting/{state_abbr}2020.pl/{state_abbr}geo2020.pl")

    if not geo_file.exists():
        return None

    # Read the geo file - it's pipe-delimited
    # Format: FILEID|STUSAB|SUMLEV|GEOVAR|GEOCOMP|CHARITER|CIFSN|LOGRECNO|GEOID|GEOCODE|REGION|DIVISION|STATE|STATENS|COUNTY|COUNTYCC|COUNTYNS|COUSUB|COUSUBCC|COUSUBNS|SUBMCD|SUBMCDCC|SUBMCDNS|ESTATE|ESTATECC|ESTATENS|CONCIT|CONCITCC|CONCITNS|PLACE|PLACECC|PLACENS|TRACT|BLKGRP|BLOCK|AIANHH|AIHHTLI|AIANHHFP|AIANHHCC|AIANHHNS|AITS|AITSFP|AITSCC|AITSNS|TTRACT|TBLKGRP|ANRC|ANRCCC|ANRCNS|CBSA|CBSACC|METDIV|CSA|NECTA|NECTADIV|CNECTA|CBSAPCI|NECTAPCI|UA|UATYPE|UR|CD116|CD118|CD119|CD120|CD121|SLDU18|SLDU22|SLDU24|SLDU26|SLDU28|SLDL18|SLDL22|SLDL24|SLDL26|SLDL28|VTD|VTDI|ZCTA|SDELM|SDSEC|SDUNI|PUMA|AREALAND|AREAWATR|BASENAME|NAME|FUNCSTAT|GCUNI|POP100|HU100|INTPTLAT|INTPTLON|LSADC|PARTFLAG|UGA

    try:
        # Read with specific columns we need
        df = pd.read_csv(
            geo_file,
            sep='|',
            encoding='latin1',
            dtype=str,
            usecols=['SUMLEV', 'GEOID', 'POP100', 'AREALAND', 'AREAWATR']
        )

        # Filter to tract level (SUMLEV = 140)
        df = df[df['SUMLEV'] == '140'].copy()

        # Convert population to int
        df['POP100'] = pd.to_numeric(df['POP100'], errors='coerce').fillna(0).astype(int)
        df['AREALAND'] = pd.to_numeric(df['AREALAND'], errors='coerce').fillna(0)
        df['AREAWATR'] = pd.to_numeric(df['AREAWATR'], errors='coerce').fillna(0)

        # Rename columns
        df = df.rename(columns={
            'GEOID': 'tract_geoid',
            'POP100': 'total_population',
            'AREALAND': 'land_area_sqm',
            'AREAWATR': 'water_area_sqm'
        })

        return df[['tract_geoid', 'total_population', 'land_area_sqm', 'water_area_sqm']]

    except Exception as e:
        print(f"Error reading geo file: {e}")
        return None


def load_tract_election_data() -> pd.DataFrame:
    """Load tract-level 2020 presidential election results."""
    election_file = Path("data/2020/elections/2020_president/tracts-2020-RLCR.csv")

    if not election_file.exists():
        return None

    # Read election data
    df = pd.read_csv(election_file, dtype={'tract_GEOID': str})

    # Select relevant columns
    df = df[[
        'tract_GEOID',
        'tract_state_fp',
        'tract_population',
        'G20PREDBID',  # Biden votes
        'G20PRERTRU'   # Trump votes
    ]].copy()

    # Rename for consistency
    df = df.rename(columns={
        'tract_GEOID': 'tract_geoid',
        'G20PREDBID': 'biden_votes',
        'G20PRERTRU': 'trump_votes'
    })

    # Convert tract_state_fp to zero-padded string for matching
    df['tract_state_fp'] = df['tract_state_fp'].astype(str).str.zfill(2)

    # Convert votes to numeric
    df['biden_votes'] = pd.to_numeric(df['biden_votes'], errors='coerce').fillna(0)
    df['trump_votes'] = pd.to_numeric(df['trump_votes'], errors='coerce').fillna(0)

    # Compute Democratic vote share
    df['total_votes'] = df['biden_votes'] + df['trump_votes']
    df['dem_vote_share'] = 0.0
    mask = df['total_votes'] > 0
    df.loc[mask, 'dem_vote_share'] = df.loc[mask, 'biden_votes'] / df.loc[mask, 'total_votes']

    return df


def prepare_state_data(state_name: str, state_fips: str) -> gpd.GeoDataFrame:
    """
    Prepare unified tract-level data for a state.

    Returns GeoDataFrame with:
    - tract_geoid
    - geometry
    - total_population
    - biden_votes, trump_votes, total_votes
    - dem_vote_share
    - pop_density (people per sq mile)
    """
    print(f"Preparing data for {state_name}...")

    # Load tract geometries
    geo_gdf = load_tract_geometries(state_fips)
    if geo_gdf is None:
        print(f"  [WARN] No geometry data found")
        return None

    print(f"  [OK] Loaded {len(geo_gdf)} tract geometries")

    # Population data will come from election file (tract_population column)
    # No need to load separately from Census geo files

    # Load election data (only needs to be loaded once, then filtered)
    election_df = load_tract_election_data()
    if election_df is None:
        print(f"  [WARN] No election data found")
        return None

    # Filter election data to this state
    state_election_df = election_df[election_df['tract_state_fp'] == state_fips].copy()
    print(f"  [OK] Loaded election results for {len(state_election_df)} tracts")

    # Merge all data
    # Start with geometries
    merged = geo_gdf.copy()

    # Merge election results (includes population)
    merged = merged.merge(
        state_election_df[['tract_geoid', 'tract_population', 'biden_votes', 'trump_votes', 'total_votes', 'dem_vote_share']],
        left_on='GEOID',
        right_on='tract_geoid',
        how='left'
    )

    # Rename tract_population to total_population
    merged = merged.rename(columns={'tract_population': 'total_population'})

    # Fill NaN values
    merged['total_population'] = pd.to_numeric(merged['total_population'], errors='coerce').fillna(0)
    merged['biden_votes'] = merged['biden_votes'].fillna(0)
    merged['trump_votes'] = merged['trump_votes'].fillna(0)
    merged['total_votes'] = merged['total_votes'].fillna(0)
    merged['dem_vote_share'] = merged['dem_vote_share'].fillna(0)

    # Compute population density (people per square mile)
    # Reproject to equal-area projection for accurate area calculation
    merged_proj = merged.to_crs("EPSG:5070")  # NAD83 / Conus Albers

    # Compute area in square miles
    area_sq_meters = merged_proj.geometry.area
    area_sq_miles = area_sq_meters / (1609.34 ** 2)

    # Population density
    merged['pop_density'] = 0.0
    mask = area_sq_miles > 0
    merged.loc[mask, 'pop_density'] = merged.loc[mask, 'total_population'] / area_sq_miles[mask]

    # Select final columns
    final_gdf = merged[[
        'GEOID',
        'geometry',
        'total_population',
        'biden_votes',
        'trump_votes',
        'total_votes',
        'dem_vote_share',
        'pop_density'
    ]].copy()

    final_gdf = final_gdf.rename(columns={'GEOID': 'tract_geoid'})

    print(f"  [OK] Final dataset: {len(final_gdf)} tracts with complete data")

    return final_gdf


def prepare_all_states():
    """Prepare tract data for all 50 states."""
    output_dir = Path("research/gerry-recursive-bisection/data/geographic_sorting/tracts")
    output_dir.mkdir(parents=True, exist_ok=True)

    success_count = 0
    failed_states = []

    for state_name, n_districts in STATES_2020.items():
        # Skip single-district states
        if n_districts == 1:
            print(f"Skipping {state_name} (single district)")
            continue

        state_fips = NAME_TO_FIPS.get(state_name)
        if state_fips is None:
            print(f"[WARN] No FIPS code for {state_name}")
            continue

        try:
            gdf = prepare_state_data(state_name, state_fips)

            if gdf is not None and len(gdf) > 0:
                # Save as GeoPackage
                output_file = output_dir / f"{state_name}_tracts_prepared.gpkg"
                gdf.to_file(output_file, driver="GPKG")
                print(f"  [OK] Saved to {output_file}")
                success_count += 1
            else:
                failed_states.append(state_name)

        except Exception as e:
            print(f"  [FAIL] Error: {e}")
            failed_states.append(state_name)
            import traceback
            traceback.print_exc()

    print()
    print("="*60)
    print(f"Data preparation complete!")
    print(f"  Success: {success_count} states")
    print(f"  Failed: {len(failed_states)} states")
    if failed_states:
        print(f"  Failed states: {', '.join(failed_states)}")
    print("="*60)


def main():
    """Main execution."""
    print("="*60)
    print("Preparing Tract-Level Data for Geographic Sorting Analysis")
    print("="*60)
    print()

    # Load election data once (used for all states)
    print("Loading national election data...")
    election_df = load_tract_election_data()
    if election_df is None:
        print("ERROR: Could not load election data")
        return

    print(f"[OK] Loaded election results for {len(election_df)} tracts nationally")
    print()

    # Prepare data for all states
    prepare_all_states()


if __name__ == "__main__":
    main()
