"""
Census data download and processing using TIGER/Line shapefiles.

This module uses pygris to download Census block geometries and population data.
"""

import os
from pathlib import Path
from typing import Optional
import warnings

import geopandas as gpd
import pandas as pd
from tqdm import tqdm

# Suppress pygris warnings
warnings.filterwarnings('ignore', category=UserWarning)


def download_blocks(
    state: str,
    year: int = 2020,
    county: Optional[str] = None,
    cache_dir: Optional[str] = None,
    output_dir: Optional[str] = None,
) -> gpd.GeoDataFrame:
    """
    Download Census block shapefiles from TIGER/Line using pygris.

    Parameters
    ----------
    state : str
        State name or FIPS code (e.g., 'CA' or '06' for California)
    year : int, default 2020
        Census year (2020, 2010, 2000)
    county : str, optional
        County FIPS code for downloading single county (e.g., '075' for San Francisco)
    cache_dir : str, optional
        Directory for pygris cache (default: ./cache)
    output_dir : str, optional
        Directory to save processed data (default: ./data/raw)

    Returns
    -------
    gpd.GeoDataFrame
        Block geometries with population and metadata

    Notes
    -----
    - California has approximately 700,000 census blocks
    - Download may take several minutes for full state
    - Results are cached to avoid repeated downloads
    """
    try:
        import pygris
        from pygris import blocks as get_blocks
    except ImportError:
        raise ImportError(
            "pygris is required for downloading Census data. "
            "Install it with: pip install pygris"
        )

    # Set cache directory
    if cache_dir is None:
        cache_dir = os.path.join(os.getcwd(), 'cache')
    os.environ['PYGRIS_CACHE_DIR'] = cache_dir
    pygris.cache = True

    # Download blocks
    print(f"Downloading {year} Census blocks for {state}...")
    if county:
        print(f"  County: {county}")

    try:
        blocks_gdf = get_blocks(
            state=state,
            year=year,
            county=county,
            cache=True
        )
    except Exception as e:
        raise RuntimeError(
            f"Failed to download blocks for {state}: {e}\n"
            f"Check state code and year are valid."
        )

    print(f"Downloaded {len(blocks_gdf):,} blocks")

    # Extract relevant columns based on year
    if year == 2020:
        geoid_col = 'GEOID20'
        name_col = 'NAME20'
        aland_col = 'ALAND20'
        awater_col = 'AWATER20'
        intptlat_col = 'INTPTLAT20'
        intptlon_col = 'INTPTLON20'
        statefp_col = 'STATEFP20'
        countyfp_col = 'COUNTYFP20'
        tractce_col = 'TRACTCE20'
        blockce_col = 'BLOCKCE20'
    elif year == 2010:
        geoid_col = 'GEOID10'
        name_col = 'NAME10'
        aland_col = 'ALAND10'
        awater_col = 'AWATER10'
        intptlat_col = 'INTPTLAT10'
        intptlon_col = 'INTPTLON10'
        statefp_col = 'STATEFP10'
        countyfp_col = 'COUNTYFP10'
        tractce_col = 'TRACTCE10'
        blockce_col = 'BLOCKCE10'
    else:  # 2000 and others
        geoid_col = 'GEOID'
        name_col = 'NAME'
        aland_col = 'ALAND'
        awater_col = 'AWATER'
        # 2000 may not have INTPTLAT/LON
        intptlat_col = 'INTPTLAT' if 'INTPTLAT' in blocks_gdf.columns else None
        intptlon_col = 'INTPTLON' if 'INTPTLON' in blocks_gdf.columns else None
        statefp_col = 'STATEFP'
        countyfp_col = 'COUNTYFP'
        tractce_col = 'TRACTCE'
        blockce_col = 'BLOCKCE'

    # Select columns
    cols_to_keep = [
        geoid_col, statefp_col, countyfp_col, tractce_col, blockce_col,
        name_col, aland_col, awater_col, 'geometry'
    ]
    if intptlat_col and intptlat_col in blocks_gdf.columns:
        cols_to_keep.extend([intptlat_col, intptlon_col])

    blocks_gdf = blocks_gdf[cols_to_keep].copy()

    # Rename columns to standardized names
    rename_dict = {
        geoid_col: 'GEOID',
        statefp_col: 'STATEFP',
        countyfp_col: 'COUNTYFP',
        tractce_col: 'TRACTCE',
        blockce_col: 'BLOCKCE',
        name_col: 'NAME',
        aland_col: 'ALAND',
        awater_col: 'AWATER',
    }
    if intptlat_col:
        rename_dict[intptlat_col] = 'INTPTLAT'
        rename_dict[intptlon_col] = 'INTPTLON'

    blocks_gdf = blocks_gdf.rename(columns=rename_dict)

    # Get population data
    print("Fetching population data...")
    blocks_gdf = _add_population_data(blocks_gdf, state, year)

    # Add computed fields
    blocks_gdf['total_area'] = blocks_gdf['ALAND'] + blocks_gdf['AWATER']
    blocks_gdf['water_fraction'] = blocks_gdf['AWATER'] / blocks_gdf['total_area'].replace(0, 1)

    # Validate geometries
    print("Validating geometries...")
    invalid = ~blocks_gdf.geometry.is_valid
    if invalid.sum() > 0:
        print(f"  Warning: {invalid.sum()} invalid geometries found, fixing...")
        blocks_gdf.loc[invalid, 'geometry'] = blocks_gdf.loc[invalid, 'geometry'].buffer(0)

    # Summary statistics
    print("\nDataset Summary:")
    print(f"  Total blocks: {len(blocks_gdf):,}")
    print(f"  Blocks with population > 0: {(blocks_gdf['population'] > 0).sum():,}")
    print(f"  Total population: {blocks_gdf['population'].sum():,}")
    print(f"  Total land area: {blocks_gdf['ALAND'].sum() / 1e6:,.0f} km²")

    # Save to parquet if output_dir specified
    if output_dir:
        output_dir = Path(output_dir)
        output_dir.mkdir(parents=True, exist_ok=True)

        county_suffix = f"_{county}" if county else ""
        filename = f"{state.lower()}_blocks_{year}{county_suffix}.parquet"
        output_path = output_dir / filename

        print(f"\nSaving to {output_path}...")
        blocks_gdf.to_parquet(output_path, compression='snappy')
        print(f"Saved {len(blocks_gdf):,} blocks ({output_path.stat().st_size / 1e6:.1f} MB)")

    return blocks_gdf


def _add_population_data(
    blocks_gdf: gpd.GeoDataFrame,
    state: str,
    year: int
) -> gpd.GeoDataFrame:
    """
    Add population data to blocks GeoDataFrame.

    For 2020, attempts to fetch from Census API P.L. 94-171 file.
    If API is unavailable, uses zero population (must be added manually).

    Parameters
    ----------
    blocks_gdf : gpd.GeoDataFrame
        Blocks with geometries
    state : str
        State code
    year : int
        Census year

    Returns
    -------
    gpd.GeoDataFrame
        Blocks with 'population' column added
    """
    if year == 2020:
        try:
            # Try to get population from Census API
            pop_data = _fetch_population_from_api(state, year)
            if pop_data is not None:
                blocks_gdf = blocks_gdf.merge(
                    pop_data[['GEOID', 'population']],
                    on='GEOID',
                    how='left'
                )
                blocks_gdf['population'] = blocks_gdf['population'].fillna(0).astype(int)
                print(f"  Successfully fetched population for {(blocks_gdf['population'] > 0).sum():,} blocks")
                return blocks_gdf
        except Exception as e:
            print(f"  Warning: Could not fetch population data from API: {e}")

    # Fallback: set population to 0 (user must provide population data separately)
    print("  Warning: Using zero population. You must add population data separately.")
    blocks_gdf['population'] = 0

    return blocks_gdf


def _fetch_population_from_api(state: str, year: int) -> Optional[pd.DataFrame]:
    """
    Fetch block-level population from Census API (P.L. 94-171).

    Parameters
    ----------
    state : str
        State code
    year : int
        Census year

    Returns
    -------
    pd.DataFrame or None
        DataFrame with GEOID and population, or None if fetch fails
    """
    try:
        from census import Census
        # Note: This requires a Census API key
        # Users should set CENSUS_API_KEY environment variable
        api_key = os.environ.get('CENSUS_API_KEY')
        if not api_key:
            print("  Note: Set CENSUS_API_KEY environment variable for automatic population download")
            return None

        c = Census(api_key, year=year)

        # P.L. 94-171 variables
        # P1_001N is total population (2020)
        state_fips = _state_to_fips(state)

        pop_data = c.pl.state_county_tract_block(
            ('NAME', 'P1_001N'),  # Total population
            state_fips,
            '*',  # All counties
            '*',  # All tracts
            '*'   # All blocks
        )

        df = pd.DataFrame(pop_data)
        # Construct GEOID: state + county + tract + block
        df['GEOID'] = df['state'] + df['county'] + df['tract'] + df['block']
        df['population'] = df['P1_001N'].astype(int)

        return df[['GEOID', 'population']]

    except ImportError:
        print("  Note: Install 'census' package for automatic population download: pip install census")
        return None
    except Exception as e:
        print(f"  API fetch failed: {e}")
        return None


def _state_to_fips(state: str) -> str:
    """Convert state abbreviation to FIPS code."""
    state_fips = {
        'CA': '06', 'NY': '36', 'TX': '48', 'FL': '12', 'IL': '17',
        'PA': '42', 'OH': '39', 'GA': '13', 'NC': '37', 'MI': '26',
        # Add more as needed
    }
    if len(state) == 2:
        return state_fips.get(state.upper(), state)
    return state


def load_blocks(file_path: str) -> gpd.GeoDataFrame:
    """
    Load blocks from saved parquet file.

    Parameters
    ----------
    file_path : str
        Path to parquet file

    Returns
    -------
    gpd.GeoDataFrame
        Block geometries and data
    """
    print(f"Loading blocks from {file_path}...")
    blocks_gdf = gpd.read_parquet(file_path)
    print(f"Loaded {len(blocks_gdf):,} blocks")
    return blocks_gdf


def filter_populated_blocks(
    blocks_gdf: gpd.GeoDataFrame,
    min_population: int = 1
) -> gpd.GeoDataFrame:
    """
    Filter to blocks with population >= threshold.

    Useful for reducing computational burden by excluding
    unpopulated blocks (forests, parks, water bodies, etc.)

    Parameters
    ----------
    blocks_gdf : gpd.GeoDataFrame
        All blocks
    min_population : int, default 1
        Minimum population threshold

    Returns
    -------
    gpd.GeoDataFrame
        Filtered blocks
    """
    original_count = len(blocks_gdf)
    filtered = blocks_gdf[blocks_gdf['population'] >= min_population].copy()
    print(f"Filtered from {original_count:,} to {len(filtered):,} blocks "
          f"({100 * len(filtered) / original_count:.1f}%)")
    return filtered
