#!/usr/bin/env python3
"""
Centralized download configuration and sources.

This module provides single source of truth for:
- STATE_FIPS mappings
- Census API configurations
- Download URL templates
- State lists

Created: 2026-01-18 (Enhancement #48)
"""

# =============================================================================
# STATE FIPS CODE MAPPINGS
# =============================================================================

# Standard 2-digit FIPS codes (most common format)
STATE_FIPS = {
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
    'DC': '11'
}

# Reverse mapping (FIPS to state code)
FIPS_TO_STATE = {v: k for k, v in STATE_FIPS.items()}

# =============================================================================
# STATE LISTS
# =============================================================================

# All 50 states (no DC)
ALL_STATES = [
    'AL', 'AK', 'AZ', 'AR', 'CA', 'CO', 'CT', 'DE', 'FL', 'GA',
    'HI', 'ID', 'IL', 'IN', 'IA', 'KS', 'KY', 'LA', 'ME', 'MD',
    'MA', 'MI', 'MN', 'MS', 'MO', 'MT', 'NE', 'NV', 'NH', 'NJ',
    'NM', 'NY', 'NC', 'ND', 'OH', 'OK', 'OR', 'PA', 'RI', 'SC',
    'SD', 'TN', 'TX', 'UT', 'VT', 'VA', 'WA', 'WV', 'WI', 'WY'
]

# All 50 states + DC
ALL_STATES_WITH_DC = ALL_STATES + ['DC']

# State code to full name mapping (lowercase with underscores for file naming)
STATE_NAMES = {
    'AL': 'alabama', 'AK': 'alaska', 'AZ': 'arizona', 'AR': 'arkansas',
    'CA': 'california', 'CO': 'colorado', 'CT': 'connecticut', 'DE': 'delaware',
    'FL': 'florida', 'GA': 'georgia', 'HI': 'hawaii', 'ID': 'idaho',
    'IL': 'illinois', 'IN': 'indiana', 'IA': 'iowa', 'KS': 'kansas',
    'KY': 'kentucky', 'LA': 'louisiana', 'ME': 'maine', 'MD': 'maryland',
    'MA': 'massachusetts', 'MI': 'michigan', 'MN': 'minnesota', 'MS': 'mississippi',
    'MO': 'missouri', 'MT': 'montana', 'NE': 'nebraska', 'NV': 'nevada',
    'NH': 'new_hampshire', 'NJ': 'new_jersey', 'NM': 'new_mexico', 'NY': 'new_york',
    'NC': 'north_carolina', 'ND': 'north_dakota', 'OH': 'ohio', 'OK': 'oklahoma',
    'OR': 'oregon', 'PA': 'pennsylvania', 'RI': 'rhode_island', 'SC': 'south_carolina',
    'SD': 'south_dakota', 'TN': 'tennessee', 'TX': 'texas', 'UT': 'utah',
    'VT': 'vermont', 'VA': 'virginia', 'WA': 'washington', 'WV': 'west_virginia',
    'WI': 'wisconsin', 'WY': 'wyoming', 'DC': 'district_of_columbia'
}

# States by region (useful for batching/testing)
STATES_BY_REGION = {
    'northeast': ['CT', 'ME', 'MA', 'NH', 'RI', 'VT', 'NJ', 'NY', 'PA'],
    'midwest': ['IL', 'IN', 'MI', 'OH', 'WI', 'IA', 'KS', 'MN', 'MO', 'NE', 'ND', 'SD'],
    'south': ['DE', 'FL', 'GA', 'MD', 'NC', 'SC', 'VA', 'WV', 'AL', 'KY', 'MS', 'TN', 'AR', 'LA', 'OK', 'TX'],
    'west': ['AZ', 'CO', 'ID', 'MT', 'NV', 'NM', 'UT', 'WY', 'AK', 'CA', 'HI', 'OR', 'WA'],
}

# Small test states (fast downloads, good for testing)
TEST_STATES = ['VT', 'DE', 'WY', 'AK', 'ND', 'SD']

# =============================================================================
# CENSUS API CONFIGURATIONS
# =============================================================================

# Census API configurations by year
# Variables for demographic data (age, race, sex)
CENSUS_CONFIGS = {
    2000: {
        'api_base': 'https://api.census.gov/data/2000/dec/sf1',
        'variables': {
            'total_pop': 'P001001',          # Total population
            'male': 'P012002',               # Male
            'female': 'P012026',             # Female
            'total_pop_race': 'P001001',     # Total (for validation)
            'white_non_hispanic': 'P005003', # White alone (not Hispanic)
            'black_non_hispanic': 'P005004', # Black alone (not Hispanic)
            'asian_non_hispanic': 'P005006', # Asian alone (not Hispanic)
            'hispanic': 'P008010',           # Hispanic or Latino (any race)
        },
        'table': 'sf1',
        'description': '2000 Decennial Census SF1'
    },
    2010: {
        'api_base': 'https://api.census.gov/data/2010/dec/sf1',
        'variables': {
            'total_pop': 'P001001',          # Total population
            'male': 'P012002',               # Male
            'female': 'P012026',             # Female
            'total_pop_race': 'P001001',     # Total (for validation)
            'white_non_hispanic': 'P005003', # White alone (not Hispanic)
            'black_non_hispanic': 'P005004', # Black alone (not Hispanic)
            'asian_non_hispanic': 'P005006', # Asian alone (not Hispanic)
            'hispanic': 'P008010',           # Hispanic or Latino (any race)
        },
        'table': 'sf1',
        'description': '2010 Decennial Census SF1'
    },
    2020: {
        'api_base': 'https://api.census.gov/data/2020/dec/dhc',
        'variables': {
            'total_pop': 'P12_001N',         # Total population
            'male': 'P12_002N',              # Male
            'female': 'P12_026N',            # Female
            'total_pop_race': 'P5_001N',     # Total (for validation)
            'white_non_hispanic': 'P5_003N', # White alone (not Hispanic)
            'black_non_hispanic': 'P5_004N', # Black alone (not Hispanic)
            'asian_non_hispanic': 'P5_006N', # Asian alone (not Hispanic)
            'hispanic': 'P5_010N',           # Hispanic or Latino (any race)
        },
        'table': 'dhc',
        'description': '2020 Decennial Census DHC'
    }
}

# =============================================================================
# DOWNLOAD URL TEMPLATES
# =============================================================================

# Census TIGER/Line (tract geometries)
TIGER_TRACT_URLS = {
    2000: 'https://www2.census.gov/geo/tiger/TIGER2010/TRACT/2000/tl_2010_{fips}_tract00.zip',
    2010: 'https://www2.census.gov/geo/tiger/TIGER2010/TRACT/2010/tl_2010_{fips}_tract10.zip',
    2020: 'https://www2.census.gov/geo/tiger/TIGER2020/TRACT/tl_2020_{fips}_tract.zip'
}

# Census TIGER/Line (place/city boundaries)
TIGER_PLACE_URLS = {
    2000: 'https://www2.census.gov/geo/tiger/TIGER2010/PLACE/2000/tl_2010_{fips}_place00.zip',
    2010: 'https://www2.census.gov/geo/tiger/TIGER2010/PLACE/2010/tl_2010_{fips}_place10.zip',
    2020: 'https://www2.census.gov/geo/tiger/TIGER2020/PLACE/tl_2020_{fips}_place.zip'
}

# Census TIGER/Line (congressional districts - for baseline comparison)
TIGER_CD_URLS = {
    2000: {
        'cd107': 'https://www2.census.gov/geo/tiger/TIGER2010/CD/107/tl_2010_us_cd107.zip',  # 2001-2003
        'fallback': [
            'https://www2.census.gov/geo/tiger/TIGER2010/CD/107/tl_2010_us_cd107.zip',
            'https://www2.census.gov/geo/tiger/TIGER2000/cd106.zip'
        ]
    },
    2010: {
        'cd112': 'https://www2.census.gov/geo/tiger/TIGER2012/CD/tl_2012_us_cd112.zip',  # 2011-2013
        'cd113': 'https://www2.census.gov/geo/tiger/TIGER2013/CD/tl_2013_us_cd113.zip',  # 2013-2015
        'fallback': [
            'https://www2.census.gov/geo/tiger/TIGER2012/CD/tl_2012_us_cd112.zip',
            'https://www2.census.gov/geo/tiger/TIGER2013/CD/tl_2013_us_cd113.zip'
        ]
    },
    2020: {
        'cd118': 'https://www2.census.gov/geo/tiger/TIGER2022/CD/tl_2022_us_cd118.zip',  # 2023-2025 (post-2020 redistricting)
        'cd117': 'https://www2.census.gov/geo/tiger/TIGER2020/CD/tl_2020_us_cd116.zip',  # 2019-2021 (pre-2020 redistricting)
        'fallback': [
            'https://www2.census.gov/geo/tiger/TIGER2022/CD/tl_2022_us_cd118.zip',
            'https://www2.census.gov/geo/tiger/TIGER2020/CD/tl_2020_us_cd116.zip'
        ]
    }
}

# Metro area (CBSA) boundaries
TIGER_CBSA_URLS = {
    2000: 'https://www2.census.gov/geo/tiger/TIGER2010/CBSA/2000/tl_2010_us_cbsa00.zip',
    2010: 'https://www2.census.gov/geo/tiger/TIGER2010/CBSA/2010/tl_2010_us_cbsa10.zip',
    2020: 'https://www2.census.gov/geo/tiger/TIGER2020/CBSA/tl_2020_us_cbsa.zip'
}

# Harvard Dataverse (election data)
HARVARD_DATAVERSE = {
    'doi': 'doi:10.7910/DVN/Z8TSH3',
    'base_url': 'https://dataverse.harvard.edu/api/datasets/:persistentId',
    'description': 'Tract-level presidential election results (2016, 2020)',
    'license': 'CC0 1.0 Universal'
}

# =============================================================================
# UTILITY FUNCTIONS
# =============================================================================

def get_fips_2digit(state_code):
    """
    Get 2-digit FIPS code for a state.

    Args:
        state_code: 2-letter state code (e.g., 'CA', 'TX')

    Returns:
        2-digit FIPS code (e.g., '06', '48')

    Raises:
        ValueError: If state code is invalid
    """
    state_code = state_code.upper()
    if state_code not in STATE_FIPS:
        raise ValueError(f"Invalid state code: {state_code}")
    return STATE_FIPS[state_code]


def get_fips_3digit(state_code):
    """
    Get 3-digit FIPS code for a state (used in some datasets).

    Args:
        state_code: 2-letter state code (e.g., 'CA', 'TX')

    Returns:
        3-digit FIPS code (e.g., '060', '480')

    Raises:
        ValueError: If state code is invalid
    """
    fips_2 = get_fips_2digit(state_code)
    return fips_2.ljust(3, '0')  # Pad with '0' to make 3 digits


def get_state_from_fips(fips_code):
    """
    Get state code from FIPS code (2 or 3 digit).

    Args:
        fips_code: FIPS code (e.g., '06', '060')

    Returns:
        State code (e.g., 'CA')

    Raises:
        ValueError: If FIPS code is invalid
    """
    # Normalize to 2 digits
    fips_2 = fips_code[:2] if len(fips_code) > 2 else fips_code
    fips_2 = fips_2.zfill(2)  # Ensure 2 digits with leading zeros

    if fips_2 not in FIPS_TO_STATE:
        raise ValueError(f"Invalid FIPS code: {fips_code}")
    return FIPS_TO_STATE[fips_2]


def get_census_config(year):
    """
    Get Census API configuration for a specific year.

    Args:
        year: Census year (2000, 2010, or 2020)

    Returns:
        Dictionary with api_base, variables, table, description

    Raises:
        ValueError: If year is not supported
    """
    year_int = int(year)
    if year_int not in CENSUS_CONFIGS:
        raise ValueError(f"Census year {year} not supported. Available: {list(CENSUS_CONFIGS.keys())}")
    return CENSUS_CONFIGS[year_int]


def get_tiger_tract_url(state_code, year):
    """
    Get TIGER/Line tract shapefile URL for a state and year.

    Args:
        state_code: 2-letter state code (e.g., 'CA')
        year: Census year (2000, 2010, or 2020)

    Returns:
        Download URL string

    Raises:
        ValueError: If state or year is invalid
    """
    year_int = int(year)
    if year_int not in TIGER_TRACT_URLS:
        raise ValueError(f"Year {year} not supported for TIGER tracts")

    fips = get_fips_2digit(state_code)
    return TIGER_TRACT_URLS[year_int].format(fips=fips)


def get_tiger_place_url(state_code, year):
    """
    Get TIGER/Line place shapefile URL for a state and year.

    Args:
        state_code: 2-letter state code (e.g., 'CA')
        year: Census year (2000, 2010, or 2020)

    Returns:
        Download URL string

    Raises:
        ValueError: If state or year is invalid
    """
    year_int = int(year)
    if year_int not in TIGER_PLACE_URLS:
        raise ValueError(f"Year {year} not supported for TIGER places")

    fips = get_fips_2digit(state_code)
    return TIGER_PLACE_URLS[year_int].format(fips=fips)


def get_tiger_cd_url(year, congress=None):
    """
    Get TIGER/Line congressional district shapefile URL.

    Args:
        year: Census year (2000, 2010, or 2020)
        congress: Optional congress number (e.g., 107, 112, 118)

    Returns:
        Primary URL or list of fallback URLs

    Raises:
        ValueError: If year is invalid
    """
    year_int = int(year)
    if year_int not in TIGER_CD_URLS:
        raise ValueError(f"Year {year} not supported for congressional districts")

    config = TIGER_CD_URLS[year_int]

    # If specific congress requested
    if congress:
        key = f'cd{congress}'
        if key in config:
            return config[key]

    # Return fallback list (try each URL in order)
    return config.get('fallback', [])


def get_cbsa_url(year):
    """
    Get TIGER/Line CBSA (metro area) shapefile URL.

    Args:
        year: Census year (2010 or 2020)

    Returns:
        Download URL string

    Raises:
        ValueError: If year is not supported
    """
    year_int = int(year)
    if year_int not in TIGER_CBSA_URLS:
        raise ValueError(f"Year {year} not supported for CBSA data. Available: {list(TIGER_CBSA_URLS.keys())}")
    return TIGER_CBSA_URLS[year_int]


# =============================================================================
# VALIDATION
# =============================================================================

def validate_state_code(state_code):
    """
    Validate a state code.

    Args:
        state_code: 2-letter state code

    Returns:
        True if valid

    Raises:
        ValueError: If invalid
    """
    state_code = state_code.upper()
    if state_code not in STATE_FIPS:
        raise ValueError(
            f"Invalid state code: {state_code}. "
            f"Must be one of: {', '.join(sorted(STATE_FIPS.keys()))}"
        )
    return True


def validate_year(year, available_years=None):
    """
    Validate a census year.

    Args:
        year: Year (string or int)
        available_years: Optional list of valid years (default: 2000, 2010, 2020)

    Returns:
        Integer year if valid

    Raises:
        ValueError: If invalid
    """
    year_int = int(year)
    if available_years is None:
        available_years = [2000, 2010, 2020]

    if year_int not in available_years:
        raise ValueError(
            f"Invalid year: {year}. "
            f"Must be one of: {', '.join(str(y) for y in available_years)}"
        )
    return year_int
