"""
Voting Rights Act (VRA) utilities for race-aware redistricting.

Provides tools for:
- Loading tract-level demographic data
- Creating multi-constraint vertex weights (population + minority VAP)
- Calculating target weights for majority-minority districts
- Analyzing VRA compliance
"""

from typing import Tuple, Dict, List, Optional
from pathlib import Path
import numpy as np
import pandas as pd
import geopandas as gpd


def load_tract_demographics(
    state: str,
    year: int = 2020,
    data_dir: str = 'data'
) -> pd.DataFrame:
    """
    Load tract-level demographic data including race/ethnicity.

    Parameters
    ----------
    state : str
        State name (lowercase, e.g., 'alabama')
    year : int
        Census year
    data_dir : str
        Base data directory

    Returns
    -------
    pd.DataFrame
        Demographic data with columns:
        - GEOID: Tract identifier (11-character)
        - total_pop: Total population
        - white_non_hispanic: White non-Hispanic population
        - black_non_hispanic: Black non-Hispanic population
        - asian_non_hispanic: Asian non-Hispanic population
        - hispanic: Hispanic/Latino population
        - other: Other races

    Raises
    ------
    FileNotFoundError
        If demographic data file not found
    """
    demo_file = Path(data_dir) / str(year) / 'demographics' / f'{state}_demographics_{year}.csv'

    if not demo_file.exists():
        raise FileNotFoundError(f"Demographics file not found: {demo_file}")

    demographics = pd.read_csv(demo_file)

    # Ensure GEOID is 11-character format with leading zeros
    demographics['GEOID'] = demographics['GEOID'].astype(str).str.zfill(11)

    return demographics


def create_vra_vertex_weights(
    tracts_gdf: gpd.GeoDataFrame,
    demographics: pd.DataFrame
) -> Tuple[np.ndarray, pd.DataFrame]:
    """
    Create multi-constraint vertex weights for VRA-aware partitioning.

    Parameters
    ----------
    tracts_gdf : gpd.GeoDataFrame
        Tract geometries with GEOID and population
    demographics : pd.DataFrame
        Tract-level demographic data

    Returns
    -------
    vertex_weights : np.ndarray
        Multi-constraint weights, shape (n_tracts, 2)
        - Column 0: Total population
        - Column 1: Minority VAP (non-white population)
    tracts_with_demographics : pd.DataFrame
        Merged tract data with demographics

    Notes
    -----
    Minority VAP is calculated as total population minus white non-Hispanic population.
    This follows Section 2 VRA analysis conventions.
    """
    # Merge tracts with demographics
    tracts_gdf['GEOID'] = tracts_gdf['GEOID'].astype(str)
    merged = tracts_gdf.merge(demographics, on='GEOID', how='left')

    # Calculate minority percentages
    merged['pct_white'] = merged['white_non_hispanic'] / merged['total_pop']
    merged['pct_black'] = merged['black_non_hispanic'] / merged['total_pop']
    merged['pct_hispanic'] = merged['hispanic'] / merged['total_pop']
    merged['pct_asian'] = merged['asian_non_hispanic'] / merged['total_pop']
    merged['pct_other'] = merged['other'] / merged['total_pop']
    merged['pct_minority'] = 1.0 - merged['pct_white']

    # Fill NaN values (zero-population tracts)
    merged = merged.fillna(0)

    # Create multi-constraint weights
    # Constraint 1: Total population (for population balance)
    # Constraint 2: Minority population (for VRA compliance)
    total_pop = merged['total_pop'].values
    minority_pop = (merged['total_pop'] * merged['pct_minority']).values

    vertex_weights = np.column_stack([total_pop, minority_pop])

    return vertex_weights, merged


def calculate_vra_target_weights(
    total_pop: float,
    total_minority_pop: float,
    num_districts: int,
    target_mm_districts: int,
    mm_target_pct: float = 0.55
) -> List[List[float]]:
    """
    Calculate target weights for VRA-constrained partitioning.

    Strategy:
    - Create target_mm_districts with 55%+ minority (above 50% threshold with margin)
    - Distribute remaining minority population across other districts
    - Balance total population equally

    Parameters
    ----------
    total_pop : float
        Total state population
    total_minority_pop : float
        Total minority population
    num_districts : int
        Total number of districts
    target_mm_districts : int
        Target number of majority-minority districts
    mm_target_pct : float, default 0.55
        Target minority percentage for MM districts (55% = safe margin above 50%)

    Returns
    -------
    List[List[float]]
        Target weights for each district: [[pop_frac, minority_frac], ...]
        Each district has 2 constraint targets that sum to 1.0 across all districts

    Examples
    --------
    >>> # Alabama: 5M pop, 1.85M minority, 7 districts, target 2 MM
    >>> targets = calculate_vra_target_weights(5e6, 1.85e6, 7, 2, 0.55)
    >>> # Returns 7 districts with balanced pop + concentrated minority in 2 districts
    """
    ideal_pop_per_district = total_pop / num_districts
    pop_fraction_per_district = 1.0 / num_districts

    # Calculate minority allocation
    mm_minority_total = target_mm_districts * ideal_pop_per_district * mm_target_pct
    remaining_minority = total_minority_pop - mm_minority_total
    non_mm_districts = num_districts - target_mm_districts

    if non_mm_districts > 0 and remaining_minority > 0:
        non_mm_target_pct = remaining_minority / (non_mm_districts * ideal_pop_per_district)
    else:
        non_mm_target_pct = 0.0

    # Create target weights
    target_weights = []

    # MM districts: equal population, high minority concentration
    for i in range(target_mm_districts):
        minority_fraction = (mm_target_pct * ideal_pop_per_district) / total_minority_pop
        target_weights.append([pop_fraction_per_district, minority_fraction])

    # Non-MM districts: equal population, lower minority concentration
    for i in range(non_mm_districts):
        minority_fraction = (non_mm_target_pct * ideal_pop_per_district) / total_minority_pop
        target_weights.append([pop_fraction_per_district, minority_fraction])

    return target_weights


def analyze_mm_districts(
    tracts_with_demographics: pd.DataFrame,
    district_assignments: np.ndarray,
    mm_threshold: float = 0.50
) -> Dict:
    """
    Analyze majority-minority district compliance.

    Parameters
    ----------
    tracts_with_demographics : pd.DataFrame
        Tract data with demographics and district assignments
    district_assignments : np.ndarray
        District ID for each tract
    mm_threshold : float, default 0.50
        Minority percentage threshold for MM district classification

    Returns
    -------
    Dict
        Analysis results with keys:
        - mm_districts: List of MM district IDs
        - mm_count: Number of MM districts
        - districts: List of dicts with per-district demographics
    """
    tracts_with_demographics = tracts_with_demographics.copy()
    tracts_with_demographics['district'] = district_assignments

    results = {
        'mm_districts': [],
        'mm_count': 0,
        'districts': []
    }

    for district_id in sorted(tracts_with_demographics['district'].unique()):
        district_tracts = tracts_with_demographics[tracts_with_demographics['district'] == district_id]

        total_pop = district_tracts['total_pop'].sum()
        minority_pop = (district_tracts['total_pop'] * district_tracts['pct_minority']).sum()
        black_pop = district_tracts['black_non_hispanic'].sum()
        hispanic_pop = district_tracts['hispanic'].sum()
        asian_pop = district_tracts['asian_non_hispanic'].sum()

        pct_minority = minority_pop / total_pop if total_pop > 0 else 0
        pct_black = black_pop / total_pop if total_pop > 0 else 0
        pct_hispanic = hispanic_pop / total_pop if total_pop > 0 else 0
        pct_asian = asian_pop / total_pop if total_pop > 0 else 0

        is_mm = pct_minority > mm_threshold

        district_info = {
            'district': int(district_id),
            'total_pop': int(total_pop),
            'pct_minority': float(pct_minority),
            'pct_black': float(pct_black),
            'pct_hispanic': float(pct_hispanic),
            'pct_asian': float(pct_asian),
            'is_mm': bool(is_mm)
        }

        results['districts'].append(district_info)

        if is_mm:
            results['mm_districts'].append(int(district_id))
            results['mm_count'] += 1

    return results


def get_vra_target_mm_districts(state: str) -> Optional[int]:
    """
    Get recommended target number of MM districts for VRA compliance.

    Based on enacted 2020 plans and Section 2 litigation history.

    Parameters
    ----------
    state : str
        State name (lowercase)

    Returns
    -------
    int or None
        Recommended number of MM districts, or None if no specific target
    """
    # States with known VRA Section 2 issues (from enacted plans and litigation)
    vra_targets = {
        'alabama': 2,       # Allen v. Milligan (2023) - 2 Black-majority districts
        'louisiana': 2,     # Recent Section 2 litigation
        'georgia': 5,       # Historical VRA compliance (multiple Black-majority districts)
        'mississippi': 1,   # 1 Black-majority district standard
        'south_carolina': 2,  # Coastal + inland Black-majority districts
        'north_carolina': 3,  # Multiple minority opportunity districts
        'texas': 8,         # Hispanic-majority districts in South Texas, Dallas, Houston
        'florida': 5,       # Hispanic + Black-majority districts
        'arizona': 3,       # Hispanic + Native American districts
        'new_mexico': 2,    # Hispanic-majority districts
        'california': 24,   # Large minority populations (Hispanic, Asian, Black)
        'new_york': 10,     # NYC minorities (Black, Hispanic, Asian)
        'illinois': 4,      # Chicago minorities
        'maryland': 3,      # Baltimore + PG County Black-majority districts
        'virginia': 3,      # Northern VA + Hampton Roads minorities
        'new_jersey': 7,    # Newark, Camden, Jersey City minorities
    }

    return vra_targets.get(state.lower())
