"""
Mock analysis data generators for testing.

Generates realistic political, demographic, and compactness analysis results
matching the output structure of pipeline analysis scripts.
"""

import pandas as pd
import numpy as np
import random


def generate_mock_political_analysis(districts_df, state='alabama', seed=42):
    """
    Generate mock political analysis CSV (partisan lean).

    Parameters
    ----------
    districts_df : pandas.DataFrame
        District assignments with district numbers
    state : str
        State name (lowercase)
    seed : int
        Random seed for reproducibility

    Returns
    -------
    pandas.DataFrame
        Political analysis with columns: district, dem_votes, rep_votes,
        dem_share, rep_share, margin, winner

    Examples
    --------
    >>> from tests.mocks.mock_districts import generate_mock_districts
    >>> from tests.mocks.mock_tracts import generate_mock_tracts
    >>> tracts = generate_mock_tracts(num_tracts=200, state='alabama')
    >>> districts = generate_mock_districts(tracts, num_districts=7)
    >>> political = generate_mock_political_analysis(districts, state='alabama')
    >>> len(political)
    7
    >>> political['winner'].isin(['D', 'R']).all()
    True
    """
    np.random.seed(seed)
    random.seed(seed)

    num_districts = len(districts_df['district'].unique())

    # State lean affects base probabilities
    # Red states: higher R share, Blue states: higher D share
    red_states = ['alabama', 'wyoming', 'idaho', 'oklahoma', 'west_virginia']
    blue_states = ['vermont', 'california', 'massachusetts', 'new_york', 'hawaii']

    if state in red_states:
        base_dem_share = 0.35  # ~35% D in red states
    elif state in blue_states:
        base_dem_share = 0.65  # ~65% D in blue states
    else:
        base_dem_share = 0.50  # ~50% D in swing states

    political_data = []

    for district_num in range(1, num_districts + 1):
        district_tracts = districts_df[districts_df['district'] == district_num]
        total_votes = int(district_tracts['population'].sum() * 0.60)  # ~60% turnout

        # Add district-level variation (some districts more D, some more R)
        district_variation = np.random.uniform(-0.15, 0.15)
        dem_share = base_dem_share + district_variation
        dem_share = max(0.20, min(0.80, dem_share))  # Clamp to realistic range

        # Calculate votes
        dem_votes = int(total_votes * dem_share)
        rep_votes = total_votes - dem_votes

        # Calculate actual shares (may differ slightly due to rounding)
        actual_dem_share = dem_votes / total_votes
        actual_rep_share = rep_votes / total_votes

        # Margin (positive = D lead, negative = R lead)
        margin = actual_dem_share - actual_rep_share

        # Winner
        winner = 'D' if dem_votes > rep_votes else 'R'

        political_data.append({
            'district': district_num,
            'dem_votes': dem_votes,
            'rep_votes': rep_votes,
            'dem_share': actual_dem_share,
            'rep_share': actual_rep_share,
            'margin': margin,
            'winner': winner,
        })

    political_df = pd.DataFrame(political_data)
    return political_df


def generate_mock_demographic_analysis(districts_df, seed=42):
    """
    Generate mock demographic analysis CSV.

    Parameters
    ----------
    districts_df : pandas.DataFrame
        District assignments with population
    seed : int
        Random seed for reproducibility

    Returns
    -------
    pandas.DataFrame
        Demographic analysis with columns: district, population, white,
        black, hispanic, asian, other, diversity_index

    Examples
    --------
    >>> from tests.mocks.mock_districts import generate_mock_districts
    >>> from tests.mocks.mock_tracts import generate_mock_tracts
    >>> tracts = generate_mock_tracts(num_tracts=200, state='alabama')
    >>> districts = generate_mock_districts(tracts, num_districts=7)
    >>> demographics = generate_mock_demographic_analysis(districts)
    >>> len(demographics)
    7
    >>> abs(demographics['white'] + demographics['black'] + demographics['hispanic'] +
    ...     demographics['asian'] + demographics['other'] - 1.0).max() < 0.01
    True
    """
    np.random.seed(seed)

    num_districts = len(districts_df['district'].unique())
    demographic_data = []

    for district_num in range(1, num_districts + 1):
        district_tracts = districts_df[districts_df['district'] == district_num]
        population = district_tracts['population'].sum()

        # Generate realistic demographic composition
        # Start with base distribution, then add variation

        # White (non-Hispanic) - typically 50-75%
        white = np.random.uniform(0.45, 0.75)

        # Black (non-Hispanic) - typically 5-30%
        black = np.random.uniform(0.05, min(0.30, 1 - white))

        # Hispanic - typically 5-25%
        hispanic = np.random.uniform(0.05, min(0.25, 1 - white - black))

        # Asian - typically 2-15%
        asian = np.random.uniform(0.02, min(0.15, 1 - white - black - hispanic))

        # Other - remainder
        other = 1.0 - (white + black + hispanic + asian)
        other = max(0.01, other)  # At least 1%

        # Normalize to sum to 1.0
        total = white + black + hispanic + asian + other
        white /= total
        black /= total
        hispanic /= total
        asian /= total
        other /= total

        # Calculate diversity index (Simpson's Diversity Index)
        # Range: 0 (homogeneous) to ~0.8 (highly diverse)
        diversity_index = 1 - (white**2 + black**2 + hispanic**2 + asian**2 + other**2)

        demographic_data.append({
            'district': district_num,
            'population': population,
            'white': white,
            'black': black,
            'hispanic': hispanic,
            'asian': asian,
            'other': other,
            'diversity_index': diversity_index,
        })

    demographics_df = pd.DataFrame(demographic_data)
    return demographics_df


def generate_mock_compactness_analysis(districts_df, seed=42):
    """
    Generate mock compactness analysis CSV.

    Parameters
    ----------
    districts_df : pandas.DataFrame
        District assignments
    seed : int
        Random seed for reproducibility

    Returns
    -------
    pandas.DataFrame
        Compactness metrics with columns: district, polsby_popper, reock,
        area_sq_km, perimeter_km

    Examples
    --------
    >>> from tests.mocks.mock_districts import generate_mock_districts
    >>> from tests.mocks.mock_tracts import generate_mock_tracts
    >>> tracts = generate_mock_tracts(num_tracts=200, state='alabama')
    >>> districts = generate_mock_districts(tracts, num_districts=7)
    >>> compactness = generate_mock_compactness_analysis(districts)
    >>> len(compactness)
    7
    >>> (compactness['polsby_popper'] >= 0.0).all() and (compactness['polsby_popper'] <= 1.0).all()
    True
    """
    np.random.seed(seed)

    num_districts = len(districts_df['district'].unique())
    compactness_data = []

    for district_num in range(1, num_districts + 1):
        # Polsby-Popper score (0-1, higher is more compact)
        # Real districts typically 0.10-0.45
        polsby_popper = np.random.uniform(0.15, 0.45)

        # Reock score (0-1, higher is more compact)
        # Real districts typically 0.30-0.75
        reock = np.random.uniform(0.35, 0.75)

        # Area (square kilometers)
        # Congressional districts vary widely: 1,000 - 10,000 sq km typical
        area_sq_km = np.random.uniform(1000, 8000)

        # Perimeter (kilometers)
        # Derive from area and Polsby-Popper: PP = 4π*A / P^2
        # So P = sqrt(4π*A / PP)
        perimeter_km = np.sqrt(4 * np.pi * area_sq_km / polsby_popper)

        compactness_data.append({
            'district': district_num,
            'polsby_popper': polsby_popper,
            'reock': reock,
            'area_sq_km': area_sq_km,
            'perimeter_km': perimeter_km,
        })

    compactness_df = pd.DataFrame(compactness_data)
    return compactness_df


def generate_mock_urban_analysis(districts_df, seed=42):
    """
    Generate mock urban area analysis CSV.

    Parameters
    ----------
    districts_df : pandas.DataFrame
        District assignments
    seed : int
        Random seed for reproducibility

    Returns
    -------
    pandas.DataFrame
        Urban metrics with columns: district, urban_population, rural_population,
        urban_share, population_density

    Examples
    --------
    >>> from tests.mocks.mock_districts import generate_mock_districts
    >>> from tests.mocks.mock_tracts import generate_mock_tracts
    >>> tracts = generate_mock_tracts(num_tracts=200, state='alabama')
    >>> districts = generate_mock_districts(tracts, num_districts=7)
    >>> urban = generate_mock_urban_analysis(districts)
    >>> len(urban)
    7
    >>> (urban['urban_share'] >= 0.0).all() and (urban['urban_share'] <= 1.0).all()
    True
    """
    np.random.seed(seed)

    num_districts = len(districts_df['district'].unique())
    urban_data = []

    for district_num in range(1, num_districts + 1):
        district_tracts = districts_df[districts_df['district'] == district_num]
        total_population = district_tracts['population'].sum()

        # Urban share varies widely: 20-95%
        urban_share = np.random.uniform(0.20, 0.95)

        urban_population = int(total_population * urban_share)
        rural_population = total_population - urban_population

        # Population density (people per sq km)
        # Urban districts: 500-5000, Rural districts: 20-200
        if urban_share > 0.7:
            density = np.random.uniform(800, 3000)
        elif urban_share > 0.4:
            density = np.random.uniform(200, 800)
        else:
            density = np.random.uniform(30, 200)

        urban_data.append({
            'district': district_num,
            'urban_population': urban_population,
            'rural_population': rural_population,
            'urban_share': urban_share,
            'population_density': density,
        })

    urban_df = pd.DataFrame(urban_data)
    return urban_df


def validate_mock_political_analysis(political_df):
    """
    Validate mock political analysis data.

    Parameters
    ----------
    political_df : pandas.DataFrame
        Political analysis to validate

    Raises
    ------
    AssertionError
        If validation fails
    """
    # Check required columns
    required_cols = ['district', 'dem_votes', 'rep_votes', 'dem_share', 'rep_share', 'margin', 'winner']
    missing_cols = [col for col in required_cols if col not in political_df.columns]
    assert not missing_cols, f"Missing columns: {missing_cols}"

    # Check shares sum to ~1.0
    share_sums = political_df['dem_share'] + political_df['rep_share']
    assert (abs(share_sums - 1.0) < 0.01).all(), "Dem + Rep shares must sum to 1.0"

    # Check winner matches votes
    for _, row in political_df.iterrows():
        if row['dem_votes'] > row['rep_votes']:
            assert row['winner'] == 'D', f"District {row['district']}: D has more votes but winner is {row['winner']}"
        else:
            assert row['winner'] == 'R', f"District {row['district']}: R has more votes but winner is {row['winner']}"

    print(f"[OK] Political analysis validated: {len(political_df)} districts")


def validate_mock_demographic_analysis(demographics_df):
    """
    Validate mock demographic analysis data.

    Parameters
    ----------
    demographics_df : pandas.DataFrame
        Demographic analysis to validate

    Raises
    ------
    AssertionError
        If validation fails
    """
    # Check required columns
    required_cols = ['district', 'population', 'white', 'black', 'hispanic', 'asian', 'other', 'diversity_index']
    missing_cols = [col for col in required_cols if col not in demographics_df.columns]
    assert not missing_cols, f"Missing columns: {missing_cols}"

    # Check shares sum to ~1.0
    share_sums = (demographics_df['white'] + demographics_df['black'] +
                  demographics_df['hispanic'] + demographics_df['asian'] +
                  demographics_df['other'])
    assert (abs(share_sums - 1.0) < 0.01).all(), "Demographic shares must sum to 1.0"

    # Check diversity index range
    assert (demographics_df['diversity_index'] >= 0.0).all(), "Diversity index must be >= 0"
    assert (demographics_df['diversity_index'] <= 1.0).all(), "Diversity index must be <= 1"

    print(f"[OK] Demographic analysis validated: {len(demographics_df)} districts")


if __name__ == '__main__':
    # Test generation
    from tests.mocks.mock_tracts import generate_mock_tracts
    from tests.mocks.mock_districts import generate_mock_districts

    print("Generating mock analysis data...")

    # Generate base data
    tracts = generate_mock_tracts(num_tracts=200, state='alabama', year='2020')
    districts = generate_mock_districts(tracts, num_districts=7)

    # Test political analysis
    political = generate_mock_political_analysis(districts, state='alabama')
    validate_mock_political_analysis(political)
    print(f"  Political analysis: {len(political)} districts")
    print(f"    D seats: {(political['winner'] == 'D').sum()}")
    print(f"    R seats: {(political['winner'] == 'R').sum()}")
    print(f"    Mean D share: {political['dem_share'].mean():.2%}")

    # Test demographic analysis
    demographics = generate_mock_demographic_analysis(districts)
    validate_mock_demographic_analysis(demographics)
    print(f"  Demographic analysis: {len(demographics)} districts")
    print(f"    Mean white share: {demographics['white'].mean():.2%}")
    print(f"    Mean diversity: {demographics['diversity_index'].mean():.3f}")

    # Test compactness analysis
    compactness = generate_mock_compactness_analysis(districts)
    print(f"  Compactness analysis: {len(compactness)} districts")
    print(f"    Mean PP: {compactness['polsby_popper'].mean():.3f}")
    print(f"    Mean Reock: {compactness['reock'].mean():.3f}")

    # Test urban analysis
    urban = generate_mock_urban_analysis(districts)
    print(f"  Urban analysis: {len(urban)} districts")
    print(f"    Mean urban share: {urban['urban_share'].mean():.2%}")
    print(f"    Mean density: {urban['population_density'].mean():.0f} people/sq km")

    print("[OK] Mock analysis generation working correctly")
