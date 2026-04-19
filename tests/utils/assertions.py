"""
Assertion helpers for testing.

Provides reusable assertion functions for common test validations.
"""

import pandas as pd
import numpy as np
from pathlib import Path
from PIL import Image


def assert_valid_csv(csv_file, required_columns=None, min_rows=1):
    """
    Assert that a CSV file is valid and well-formed.

    Parameters
    ----------
    csv_file : Path or str
        Path to CSV file
    required_columns : list of str, optional
        Required column names
    min_rows : int
        Minimum number of rows

    Raises
    ------
    AssertionError
        If CSV is invalid

    Examples
    --------
    >>> from pathlib import Path
    >>> assert_valid_csv(Path('output.csv'), required_columns=['district', 'population'])
    """
    csv_file = Path(csv_file)

    # Check file exists
    assert csv_file.exists(), f"CSV file not found: {csv_file}"

    # Check is CSV extension
    assert csv_file.suffix == '.csv', f"Not a CSV file: {csv_file}"

    # Try to read
    try:
        df = pd.read_csv(csv_file)
    except Exception as e:
        raise AssertionError(f"Failed to read CSV: {csv_file}. Error: {e}")

    # Check minimum rows
    assert len(df) >= min_rows, \
        f"CSV has {len(df)} rows, expected at least {min_rows}"

    # Check required columns
    if required_columns:
        missing_cols = [col for col in required_columns if col not in df.columns]
        assert not missing_cols, \
            f"CSV missing required columns: {missing_cols}"


def assert_valid_png(png_file, min_width=400, min_height=300, max_width=4000, max_height=4000):
    """
    Assert that a PNG file is valid.

    Parameters
    ----------
    png_file : Path or str
        Path to PNG file
    min_width : int
        Minimum image width
    min_height : int
        Minimum image height
    max_width : int
        Maximum image width
    max_height : int
        Maximum image height

    Raises
    ------
    AssertionError
        If PNG is invalid

    Examples
    --------
    >>> from pathlib import Path
    >>> assert_valid_png(Path('map.png'), min_width=800, min_height=600)
    """
    png_file = Path(png_file)

    # Check file exists
    assert png_file.exists(), f"PNG file not found: {png_file}"

    # Check is PNG extension
    assert png_file.suffix == '.png', f"Not a PNG file: {png_file}"

    # Try to open as image
    try:
        img = Image.open(png_file)
    except Exception as e:
        raise AssertionError(f"Failed to open PNG: {png_file}. Error: {e}")

    # Check format
    assert img.format == 'PNG', f"Invalid PNG format: {png_file}"

    # Check dimensions
    width, height = img.size

    assert width >= min_width, \
        f"PNG width {width} < minimum {min_width}"
    assert height >= min_height, \
        f"PNG height {height} < minimum {min_height}"
    assert width <= max_width, \
        f"PNG width {width} > maximum {max_width}"
    assert height <= max_height, \
        f"PNG height {height} > maximum {max_height}"


def assert_population_balanced(districts_df, num_districts, tolerance=0.01):
    """
    Assert that district populations are balanced within tolerance.

    Parameters
    ----------
    districts_df : pandas.DataFrame
        District assignments with 'district' and 'population' columns
    num_districts : int
        Expected number of districts
    tolerance : float
        Population balance tolerance (e.g., 0.01 = ±1%)

    Raises
    ------
    AssertionError
        If populations are not balanced

    Examples
    --------
    >>> import pandas as pd
    >>> districts = pd.DataFrame({
    ...     'district': [1, 1, 2, 2],
    ...     'population': [500, 500, 500, 500]
    ... })
    >>> assert_population_balanced(districts, num_districts=2, tolerance=0.01)
    """
    # Check all districts present
    unique_districts = sorted(districts_df['district'].unique())
    expected_districts = list(range(1, num_districts + 1))
    assert unique_districts == expected_districts, \
        f"Missing districts: {set(expected_districts) - set(unique_districts)}"

    # Calculate district populations
    total_population = districts_df['population'].sum()
    target_pop = total_population / num_districts

    district_pops = districts_df.groupby('district')['population'].sum()
    max_deviation = abs(district_pops - target_pop).max() / target_pop

    assert max_deviation <= tolerance, \
        f"Population imbalance: {max_deviation:.2%} > {tolerance:.2%}"


def assert_graph_connected(graph):
    """
    Assert that a graph is connected (single component).

    Parameters
    ----------
    graph : networkx.Graph
        Graph to check

    Raises
    ------
    AssertionError
        If graph is disconnected

    Examples
    --------
    >>> import networkx as nx
    >>> G = nx.Graph()
    >>> G.add_edges_from([(0, 1), (1, 2), (2, 3)])
    >>> assert_graph_connected(G)
    """
    import networkx as nx

    assert nx.is_connected(graph), \
        f"Graph is disconnected: {nx.number_connected_components(graph)} components"


def assert_shares_sum_to_one(df, share_columns, tolerance=0.01):
    """
    Assert that share columns sum to 1.0 for each row.

    Parameters
    ----------
    df : pandas.DataFrame
        DataFrame with share columns
    share_columns : list of str
        Column names containing shares
    tolerance : float
        Tolerance for sum (e.g., 0.01 = allow 0.99-1.01)

    Raises
    ------
    AssertionError
        If shares don't sum to 1.0

    Examples
    --------
    >>> import pandas as pd
    >>> df = pd.DataFrame({
    ...     'white': [0.6, 0.7],
    ...     'black': [0.3, 0.2],
    ...     'other': [0.1, 0.1]
    ... })
    >>> assert_shares_sum_to_one(df, ['white', 'black', 'other'])
    """
    # Calculate sum for each row
    share_sums = df[share_columns].sum(axis=1)

    # Check all rows sum to ~1.0
    assert (abs(share_sums - 1.0) < tolerance).all(), \
        f"Some rows don't sum to 1.0: {share_sums.tolist()}"


def assert_values_in_range(df, column, min_value, max_value):
    """
    Assert that all values in a column are within a range.

    Parameters
    ----------
    df : pandas.DataFrame
        DataFrame
    column : str
        Column name
    min_value : float
        Minimum allowed value
    max_value : float
        Maximum allowed value

    Raises
    ------
    AssertionError
        If values are out of range

    Examples
    --------
    >>> import pandas as pd
    >>> df = pd.DataFrame({'score': [0.2, 0.5, 0.8]})
    >>> assert_values_in_range(df, 'score', 0.0, 1.0)
    """
    assert column in df.columns, f"Column '{column}' not found in DataFrame"

    values = df[column]

    assert (values >= min_value).all(), \
        f"Some {column} values < {min_value}: min={values.min()}"

    assert (values <= max_value).all(), \
        f"Some {column} values > {max_value}: max={values.max()}"


def assert_no_duplicates(df, subset):
    """
    Assert that there are no duplicate rows based on subset of columns.

    Parameters
    ----------
    df : pandas.DataFrame
        DataFrame to check
    subset : list of str
        Columns to check for duplicates

    Raises
    ------
    AssertionError
        If duplicates are found

    Examples
    --------
    >>> import pandas as pd
    >>> df = pd.DataFrame({
    ...     'state': ['AL', 'AL', 'CA'],
    ...     'district': [1, 2, 1]
    ... })
    >>> assert_no_duplicates(df, ['state', 'district'])
    """
    duplicates = df[df.duplicated(subset=subset, keep=False)]

    assert len(duplicates) == 0, \
        f"Found {len(duplicates)} duplicate rows on {subset}"


def assert_geoid_format(df, geoid_column='GEOID'):
    """
    Assert that GEOID column has valid format.

    Parameters
    ----------
    df : pandas.DataFrame
        DataFrame with GEOID column
    geoid_column : str
        GEOID column name

    Raises
    ------
    AssertionError
        If GEOID format is invalid

    Examples
    --------
    >>> import pandas as pd
    >>> df = pd.DataFrame({'GEOID': ['01001000100', '01001000200']})
    >>> assert_geoid_format(df, 'GEOID')
    """
    assert geoid_column in df.columns, f"Column '{geoid_column}' not found"

    # Check is string type
    assert df[geoid_column].dtype == object, \
        f"{geoid_column} must be string type, got {df[geoid_column].dtype}"

    # Check length (11 digits for tract GEOIDs)
    lengths = df[geoid_column].str.len()
    assert (lengths == 11).all(), \
        f"{geoid_column} must be 11 digits, found lengths: {lengths.unique()}"

    # Check are numeric strings
    assert df[geoid_column].str.isdigit().all(), \
        f"{geoid_column} must contain only digits"


def assert_compactness_valid(df):
    """
    Assert that compactness metrics are valid.

    Parameters
    ----------
    df : pandas.DataFrame
        DataFrame with compactness columns

    Raises
    ------
    AssertionError
        If compactness metrics are invalid

    Examples
    --------
    >>> import pandas as pd
    >>> df = pd.DataFrame({
    ...     'polsby_popper': [0.3, 0.4],
    ...     'reock': [0.5, 0.6]
    ... })
    >>> assert_compactness_valid(df)
    """
    # Check Polsby-Popper in [0, 1]
    if 'polsby_popper' in df.columns:
        assert_values_in_range(df, 'polsby_popper', 0.0, 1.0)

    # Check Reock in [0, 1]
    if 'reock' in df.columns:
        assert_values_in_range(df, 'reock', 0.0, 1.0)

    # Check area and perimeter are positive
    if 'area_sq_km' in df.columns:
        assert (df['area_sq_km'] > 0).all(), "Area must be positive"

    if 'perimeter_km' in df.columns:
        assert (df['perimeter_km'] > 0).all(), "Perimeter must be positive"


def assert_directory_structure(base_dir, expected_subdirs):
    """
    Assert that a directory has expected subdirectories.

    Parameters
    ----------
    base_dir : Path or str
        Base directory to check
    expected_subdirs : list of str
        Expected subdirectory names

    Raises
    ------
    AssertionError
        If directory structure is invalid

    Examples
    --------
    >>> from pathlib import Path
    >>> assert_directory_structure(Path('output'), ['maps', 'csvs', 'data'])
    """
    base_dir = Path(base_dir)

    assert base_dir.exists(), f"Directory not found: {base_dir}"
    assert base_dir.is_dir(), f"Not a directory: {base_dir}"

    for subdir in expected_subdirs:
        subdir_path = base_dir / subdir
        assert subdir_path.exists(), f"Subdirectory not found: {subdir_path}"
        assert subdir_path.is_dir(), f"Not a directory: {subdir_path}"
