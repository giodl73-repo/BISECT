"""
Mock district assignments generator for testing.

Generates population-balanced district assignments matching the output
structure of METIS graph partitioning.
"""

import pandas as pd
import numpy as np
import random


def generate_mock_districts(tracts_df, num_districts=7, tolerance=0.005, seed=42):
    """
    Generate mock district assignments with population balance.

    Parameters
    ----------
    tracts_df : geopandas.GeoDataFrame
        Mock tract data
    num_districts : int
        Number of districts to create
    tolerance : float
        Population balance tolerance (default: ±0.5%)
    seed : int
        Random seed for reproducibility

    Returns
    -------
    pandas.DataFrame
        District assignments with columns: tract_index, district, population

    Examples
    --------
    >>> from tests.mocks.mock_tracts import generate_mock_tracts
    >>> tracts = generate_mock_tracts(num_tracts=200)
    >>> districts = generate_mock_districts(tracts, num_districts=7)
    >>> len(districts['district'].unique())
    7
    >>> validate_mock_districts(districts, num_districts=7)
    """
    np.random.seed(seed)
    random.seed(seed)

    num_tracts = len(tracts_df)
    total_population = tracts_df['population'].sum()
    target_pop = total_population / num_districts

    # Sort tracts by population for better balance
    sorted_indices = tracts_df['population'].argsort().values

    # Assign tracts to districts using greedy algorithm
    district_assignments = np.zeros(num_tracts, dtype=int)
    district_populations = np.zeros(num_districts, dtype=int)

    # Assign each tract to least-populated district
    for tract_idx in sorted_indices[::-1]:  # Start with largest tracts
        tract_pop = tracts_df.iloc[tract_idx]['population']

        # Find district with lowest population
        min_district = np.argmin(district_populations)

        # Assign tract to this district (1-indexed)
        district_assignments[tract_idx] = min_district + 1
        district_populations[min_district] += tract_pop

    # Create DataFrame
    assignments_df = pd.DataFrame({
        'tract_index': range(num_tracts),
        'district': district_assignments,
        'population': tracts_df['population'].values,
        'GEOID': tracts_df['GEOID'].values,
    })

    return assignments_df


def generate_mock_rounds_hierarchy(tracts_df, num_districts=7, seed=42):
    """
    Generate mock rounds hierarchy showing recursive bisection process.

    Parameters
    ----------
    tracts_df : geopandas.GeoDataFrame
        Mock tract data
    num_districts : int
        Number of districts
    seed : int
        Random seed

    Returns
    -------
    pandas.DataFrame
        Rounds hierarchy with columns: round, parent_id, child_left, child_right,
        num_tracts, population

    Examples
    --------
    >>> from tests.mocks.mock_tracts import generate_mock_tracts
    >>> tracts = generate_mock_tracts(num_tracts=200)
    >>> rounds = generate_mock_rounds_hierarchy(tracts, num_districts=7)
    >>> len(rounds)  # Number of splits
    6
    """
    np.random.seed(seed)

    # Calculate number of rounds needed
    num_rounds = num_districts - 1  # Each split creates one additional region

    rounds_data = []
    total_population = tracts_df['population'].sum()

    # Round 1: Initial split
    split_1_pop = total_population // 2
    rounds_data.append({
        'round': 1,
        'parent_id': 0,
        'child_left': 1,
        'child_right': 2,
        'num_tracts': len(tracts_df),
        'population': total_population,
        'target_districts_left': num_districts // 2,
        'target_districts_right': num_districts - (num_districts // 2),
    })

    # Subsequent rounds: Continue bisecting
    current_round = 2
    next_region_id = 3

    for round_num in range(2, num_rounds + 1):
        # Mock: Create split record
        parent_id = round_num
        child_left = next_region_id
        child_right = next_region_id + 1
        next_region_id += 2

        # Mock population (roughly half of remaining)
        mock_population = total_population // (2 ** round_num)

        rounds_data.append({
            'round': round_num,
            'parent_id': parent_id,
            'child_left': child_left,
            'child_right': child_right,
            'num_tracts': len(tracts_df) // (2 ** round_num),
            'population': mock_population,
            'target_districts_left': 1,
            'target_districts_right': 1,
        })

    rounds_df = pd.DataFrame(rounds_data)
    return rounds_df


def generate_mock_district_summary(districts_df, num_districts=7):
    """
    Generate mock district summary CSV.

    Parameters
    ----------
    districts_df : pandas.DataFrame
        District assignments
    num_districts : int
        Number of districts

    Returns
    -------
    pandas.DataFrame
        District summary with metrics

    Examples
    --------
    >>> from tests.mocks.mock_tracts import generate_mock_tracts
    >>> tracts = generate_mock_tracts(num_tracts=200)
    >>> districts = generate_mock_districts(tracts, num_districts=7)
    >>> summary = generate_mock_district_summary(districts, num_districts=7)
    >>> len(summary)
    7
    """
    summary_data = []

    for district_num in range(1, num_districts + 1):
        district_tracts = districts_df[districts_df['district'] == district_num]

        # Calculate metrics
        population = district_tracts['population'].sum()
        num_tracts = len(district_tracts)

        # Mock compactness scores (realistic ranges)
        polsby_popper = np.random.uniform(0.15, 0.45)
        reock = np.random.uniform(0.35, 0.75)

        # Mock area and perimeter
        area_sq_km = np.random.uniform(1000, 5000)
        perimeter_km = np.random.uniform(200, 800)

        summary_data.append({
            'district': district_num,
            'population': population,
            'num_tracts': num_tracts,
            'polsby_popper': polsby_popper,
            'reock': reock,
            'area_sq_km': area_sq_km,
            'perimeter_km': perimeter_km,
        })

    summary_df = pd.DataFrame(summary_data)
    return summary_df


def validate_mock_districts(districts_df, num_districts, tolerance=0.01):
    """
    Validate mock district assignments.

    Parameters
    ----------
    districts_df : pandas.DataFrame
        District assignments
    num_districts : int
        Expected number of districts
    tolerance : float
        Population balance tolerance

    Raises
    ------
    AssertionError
        If validation fails
    """
    # Check all districts present
    unique_districts = sorted(districts_df['district'].unique())
    expected_districts = list(range(1, num_districts + 1))
    assert unique_districts == expected_districts, \
        f"Missing districts: {set(expected_districts) - set(unique_districts)}"

    # Check no tracts assigned to multiple districts
    assert len(districts_df) == len(districts_df['tract_index'].unique()), \
        "Some tracts assigned to multiple districts"

    # Check population balance
    total_population = districts_df['population'].sum()
    target_pop = total_population / num_districts

    district_pops = districts_df.groupby('district')['population'].sum()
    max_deviation = abs(district_pops - target_pop).max() / target_pop

    assert max_deviation <= tolerance, \
        f"Population imbalance: {max_deviation:.2%} > {tolerance:.2%}"

    print(f"[OK] Mock districts validated:")
    print(f"  Districts: {num_districts}")
    print(f"  Tracts: {len(districts_df)}")
    print(f"  Total population: {total_population:,}")
    print(f"  Target population: {target_pop:,.0f}")
    print(f"  Max deviation: {max_deviation:.2%}")


if __name__ == '__main__':
    # Test generation
    from tests.mocks.mock_tracts import generate_mock_tracts

    print("Generating mock district assignments...")

    # Small dataset (1 district)
    tracts_small = generate_mock_tracts(num_tracts=50, state='vermont')
    districts_small = generate_mock_districts(tracts_small, num_districts=1)
    validate_mock_districts(districts_small, num_districts=1, tolerance=0.01)
    print("  Small districts (1): OK")

    # Medium dataset (7 districts)
    tracts_medium = generate_mock_tracts(num_tracts=200, state='alabama')
    districts_medium = generate_mock_districts(tracts_medium, num_districts=7)
    validate_mock_districts(districts_medium, num_districts=7, tolerance=0.01)
    print("  Medium districts (7): OK")

    # Test rounds hierarchy
    rounds = generate_mock_rounds_hierarchy(tracts_medium, num_districts=7)
    print(f"  Rounds hierarchy: {len(rounds)} rounds")

    # Test district summary
    summary = generate_mock_district_summary(districts_medium, num_districts=7)
    print(f"  District summary: {len(summary)} districts")
    print(f"    Mean PP: {summary['polsby_popper'].mean():.3f}")
    print(f"    Mean Reock: {summary['reock'].mean():.3f}")

    print("[OK] Mock district generation working correctly")
