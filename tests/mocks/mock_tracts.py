"""
Mock census tract data generator for testing.

Generates realistic GeoDataFrames with tract-level data matching
the structure of real census data files.
"""

import geopandas as gpd
import pandas as pd
import numpy as np
from shapely.geometry import Polygon, Point
from shapely.affinity import translate
import random


def generate_mock_tracts(num_tracts=100, state='vermont', year='2020', seed=42):
    """
    Generate mock census tract GeoDataFrame.

    Parameters
    ----------
    num_tracts : int
        Number of tracts to generate
    state : str
        State name (lowercase)
    year : str
        Census year ('2000', '2010', or '2020')
    seed : int
        Random seed for reproducibility

    Returns
    -------
    geopandas.GeoDataFrame
        Mock tract data with realistic structure

    Examples
    --------
    >>> tracts = generate_mock_tracts(num_tracts=50, state='vermont')
    >>> len(tracts)
    50
    >>> 'GEOID' in tracts.columns
    True
    >>> tracts.geometry.is_valid.all()
    True
    """
    np.random.seed(seed)
    random.seed(seed)

    # Generate tract geometries in a grid pattern
    tracts_data = []
    grid_size = int(np.ceil(np.sqrt(num_tracts)))

    for i in range(num_tracts):
        # Grid position
        row = i // grid_size
        col = i % grid_size

        # Create square polygon (0.1 x 0.1 degrees)
        # Offset by row/col to create grid
        x_offset = col * 0.1
        y_offset = row * 0.1

        # Add small random variation to make less regular
        x_jitter = np.random.uniform(-0.01, 0.01)
        y_jitter = np.random.uniform(-0.01, 0.01)

        # Create polygon (square tract)
        coords = [
            (x_offset + x_jitter, y_offset + y_jitter),
            (x_offset + 0.1 + x_jitter, y_offset + y_jitter),
            (x_offset + 0.1 + x_jitter, y_offset + 0.1 + y_jitter),
            (x_offset + x_jitter, y_offset + 0.1 + y_jitter),
            (x_offset + x_jitter, y_offset + y_jitter),  # Close polygon
        ]
        geometry = Polygon(coords)

        # Generate GEOID (state FIPS + county + tract)
        # Format varies by year
        state_fips = _get_state_fips(state)
        county_code = str(random.randint(1, 9)).zfill(3)
        tract_code = str(i + 1).zfill(6)

        if year == '2000':
            geoid = f'{state_fips}{county_code}{tract_code}'
        else:
            geoid = f'{state_fips}{county_code}{tract_code}'

        # Generate realistic population (500-5000)
        # Log-normal distribution for realistic variation
        population = int(np.random.lognormal(7.8, 0.5))
        population = max(500, min(5000, population))

        # Tract name
        name = f'Census Tract {tract_code}'

        # Build tract record
        tract_record = {
            'GEOID': geoid,
            'NAME': name,
            'STATEFP': state_fips,
            'COUNTYFP': county_code,
            'TRACTCE': tract_code,
            'population': population,
            'geometry': geometry,
        }

        # Add year-specific fields
        if year == '2020':
            tract_record['ALAND'] = int(geometry.area * 1e10)  # Land area in sq meters
            tract_record['AWATER'] = 0
        elif year == '2010':
            tract_record['ALAND10'] = int(geometry.area * 1e10)
            tract_record['AWATER10'] = 0
        elif year == '2000':
            tract_record['ALAND00'] = int(geometry.area * 1e10)
            tract_record['AWATER00'] = 0

        tracts_data.append(tract_record)

    # Create GeoDataFrame
    gdf = gpd.GeoDataFrame(tracts_data, crs='EPSG:4326')

    # Ensure GEOID is string type
    gdf['GEOID'] = gdf['GEOID'].astype(str)

    return gdf


def generate_mock_places(num_places=10, state='vermont', year='2020', seed=42):
    """
    Generate mock census places (cities) GeoDataFrame.

    Parameters
    ----------
    num_places : int
        Number of places to generate
    state : str
        State name (lowercase)
    year : str
        Census year ('2000', '2010', or '2020')
    seed : int
        Random seed for reproducibility

    Returns
    -------
    geopandas.GeoDataFrame
        Mock places data with realistic structure
    """
    np.random.seed(seed)
    random.seed(seed)

    places_data = []
    state_fips = _get_state_fips(state)

    # Common city names
    city_names = [
        'Burlington', 'Montpelier', 'Rutland', 'Essex', 'Colchester',
        'Bennington', 'Brattleboro', 'Hartford', 'Springfield', 'Middlebury',
        'St. Albans', 'Winooski', 'Vergennes', 'Newport', 'Barre'
    ]

    for i in range(min(num_places, len(city_names))):
        # Random location
        x = np.random.uniform(0, 1)
        y = np.random.uniform(0, 1)

        # Create point geometry for city center
        geometry = Point(x, y)

        # Generate place ID
        place_code = str(i + 1).zfill(5)
        geoid = f'{state_fips}{place_code}'

        # Population (cities have more people)
        population = int(np.random.uniform(1000, 50000))

        place_record = {
            'GEOID': geoid,
            'NAME': city_names[i],
            'STATEFP': state_fips,
            'PLACEFP': place_code,
            'population': population,
            'geometry': geometry,
        }

        places_data.append(place_record)

    gdf = gpd.GeoDataFrame(places_data, crs='EPSG:4326')
    gdf['GEOID'] = gdf['GEOID'].astype(str)

    return gdf


def _get_state_fips(state_name):
    """
    Get state FIPS code from state name.

    Parameters
    ----------
    state_name : str
        State name (lowercase)

    Returns
    -------
    str
        Two-digit state FIPS code
    """
    # Partial mapping for common test states
    state_fips_map = {
        'vermont': '50',
        'delaware': '10',
        'alabama': '01',
        'california': '06',
        'wyoming': '56',
        'new_york': '36',
        'texas': '48',
    }

    return state_fips_map.get(state_name.lower(), '99')


def validate_mock_tracts(tracts_df):
    """
    Validate mock tracts DataFrame has required structure.

    Parameters
    ----------
    tracts_df : geopandas.GeoDataFrame
        Mock tracts data to validate

    Raises
    ------
    AssertionError
        If validation fails with clear error message
    """
    # Check required columns
    required_cols = ['GEOID', 'NAME', 'population', 'geometry']
    missing_cols = [col for col in required_cols if col not in tracts_df.columns]
    assert not missing_cols, f"Missing required columns: {missing_cols}"

    # Check GEOID is string
    assert tracts_df['GEOID'].dtype == object, "GEOID must be string type"

    # Check population is positive integer
    assert (tracts_df['population'] > 0).all(), "All populations must be positive"

    # Check geometries are valid
    assert tracts_df.geometry.is_valid.all(), "All geometries must be valid"

    # Check geometries are Polygons
    assert tracts_df.geometry.geom_type.isin(['Polygon', 'MultiPolygon']).all(), \
        "All geometries must be Polygon or MultiPolygon"

    print(f"[OK] Mock tracts validated: {len(tracts_df)} tracts")


if __name__ == '__main__':
    # Test generation
    print("Generating mock tracts...")

    # Small dataset (Vermont)
    tracts_small = generate_mock_tracts(num_tracts=50, state='vermont')
    validate_mock_tracts(tracts_small)
    print(f"  Small dataset: {len(tracts_small)} tracts")
    print(f"  Total population: {tracts_small['population'].sum():,}")
    print(f"  Mean population: {tracts_small['population'].mean():.0f}")

    # Medium dataset (Alabama)
    tracts_medium = generate_mock_tracts(num_tracts=200, state='alabama')
    validate_mock_tracts(tracts_medium)
    print(f"  Medium dataset: {len(tracts_medium)} tracts")
    print(f"  Total population: {tracts_medium['population'].sum():,}")

    # Test places generation
    places = generate_mock_places(num_places=10, state='vermont')
    print(f"  Places: {len(places)} cities")

    print("[OK] Mock tract generation working correctly")
