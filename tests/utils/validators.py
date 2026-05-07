"""
Validators for test data quality.

Provides functions to validate data meets quality standards.
"""

import pandas as pd
import numpy as np
from pathlib import Path
import networkx as nx


def validate_tract_data(tracts_df):
    """
    Validate census tract data quality.

    Parameters
    ----------
    tracts_df : geopandas.GeoDataFrame
        Census tract data

    Returns
    -------
    dict
        Validation results with 'valid' (bool) and 'errors' (list)

    Examples
    --------
    >>> from tests.mocks.mock_tracts import generate_mock_tracts
    >>> tracts = generate_mock_tracts(num_tracts=50)
    >>> result = validate_tract_data(tracts)
    >>> result['valid']
    True
    """
    errors = []

    # Check required columns
    required_cols = ['GEOID', 'NAME', 'population', 'geometry']
    missing_cols = [col for col in required_cols if col not in tracts_df.columns]
    if missing_cols:
        errors.append(f"Missing required columns: {missing_cols}")

    # Check GEOID is string
    if 'GEOID' in tracts_df.columns and tracts_df['GEOID'].dtype != object:
        errors.append(f"GEOID must be string type, got {tracts_df['GEOID'].dtype}")

    # Check population is positive
    if 'population' in tracts_df.columns:
        if not (tracts_df['population'] > 0).all():
            errors.append("Some populations are not positive")

    # Check geometries are valid
    if 'geometry' in tracts_df.columns:
        if not tracts_df.geometry.is_valid.all():
            invalid_count = (~tracts_df.geometry.is_valid).sum()
            errors.append(f"{invalid_count} invalid geometries")

    return {
        'valid': len(errors) == 0,
        'errors': errors
    }


def validate_adjacency_graph(graph, num_tracts):
    """
    Validate adjacency graph quality.

    Parameters
    ----------
    graph : networkx.Graph
        Adjacency graph
    num_tracts : int
        Expected number of tracts

    Returns
    -------
    dict
        Validation results

    Examples
    --------
    >>> from tests.mocks.mock_tracts import generate_mock_tracts
    >>> from tests.mocks.mock_adjacency import generate_mock_adjacency
    >>> tracts = generate_mock_tracts(num_tracts=50)
    >>> graph = generate_mock_adjacency(tracts)
    >>> result = validate_adjacency_graph(graph, 50)
    >>> result['valid']
    True
    """
    errors = []

    # Check node count
    if len(graph.nodes) != num_tracts:
        errors.append(f"Expected {num_tracts} nodes, found {len(graph.nodes)}")

    # Check all nodes present (no gaps)
    expected_nodes = set(range(num_tracts))
    actual_nodes = set(graph.nodes)
    if expected_nodes != actual_nodes:
        missing = expected_nodes - actual_nodes
        extra = actual_nodes - expected_nodes
        if missing:
            errors.append(f"Missing nodes: {missing}")
        if extra:
            errors.append(f"Extra nodes: {extra}")

    # Check graph is connected
    if not nx.is_connected(graph):
        num_components = nx.number_connected_components(graph)
        errors.append(f"Graph is disconnected: {num_components} components")

    # Check no self-loops
    if any(u == v for u, v in graph.edges):
        errors.append("Graph has self-loops")

    # Check connectivity
    if len(graph.nodes) > 0:
        avg_degree = sum(dict(graph.degree()).values()) / len(graph.nodes)
        if avg_degree < 2:
            errors.append(f"Low average degree: {avg_degree:.2f}")

    return {
        'valid': len(errors) == 0,
        'errors': errors,
        'node_count': len(graph.nodes),
        'edge_count': len(graph.edges),
        'components': nx.number_connected_components(graph) if len(graph.nodes) > 0 else 0,
    }


def validate_district_assignments(districts_df, num_districts, tolerance=0.01):
    """
    Validate district assignment quality.

    Parameters
    ----------
    districts_df : pandas.DataFrame
        District assignments
    num_districts : int
        Expected number of districts
    tolerance : float
        Population balance tolerance

    Returns
    -------
    dict
        Validation results

    Examples
    --------
    >>> from tests.mocks.mock_tracts import generate_mock_tracts
    >>> from tests.mocks.mock_districts import generate_mock_districts
    >>> tracts = generate_mock_tracts(num_tracts=200)
    >>> districts = generate_mock_districts(tracts, num_districts=7)
    >>> result = validate_district_assignments(districts, num_districts=7)
    >>> result['valid']
    True
    """
    errors = []

    # Check required columns
    required_cols = ['tract_index', 'district', 'population']
    missing_cols = [col for col in required_cols if col not in districts_df.columns]
    if missing_cols:
        errors.append(f"Missing required columns: {missing_cols}")
        return {'valid': False, 'errors': errors}

    # Check all districts present
    unique_districts = sorted(districts_df['district'].unique())
    expected_districts = list(range(1, num_districts + 1))
    if unique_districts != expected_districts:
        missing = set(expected_districts) - set(unique_districts)
        if missing:
            errors.append(f"Missing districts: {missing}")

    # Check no tracts assigned to multiple districts
    if len(districts_df) != len(districts_df['tract_index'].unique()):
        errors.append("Some tracts assigned to multiple districts")

    # Check population balance
    total_population = districts_df['population'].sum()
    target_pop = total_population / num_districts

    district_pops = districts_df.groupby('district')['population'].sum()
    max_deviation = abs(district_pops - target_pop).max() / target_pop

    if max_deviation > tolerance:
        errors.append(f"Population imbalance: {max_deviation:.2%} > {tolerance:.2%}")

    return {
        'valid': len(errors) == 0,
        'errors': errors,
        'num_districts': len(unique_districts),
        'max_deviation': max_deviation,
        'total_population': total_population,
    }


def validate_political_analysis(political_df, num_districts):
    """
    Validate political analysis quality.

    Parameters
    ----------
    political_df : pandas.DataFrame
        Political analysis data
    num_districts : int
        Expected number of districts

    Returns
    -------
    dict
        Validation results

    Examples
    --------
    >>> from tests.mocks.mock_tracts import generate_mock_tracts
    >>> from tests.mocks.mock_districts import generate_mock_districts
    >>> from tests.mocks.mock_analysis import generate_mock_political_analysis
    >>> tracts = generate_mock_tracts(num_tracts=200)
    >>> districts = generate_mock_districts(tracts, num_districts=7)
    >>> political = generate_mock_political_analysis(districts)
    >>> result = validate_political_analysis(political, num_districts=7)
    >>> result['valid']
    True
    """
    errors = []

    # Check row count
    if len(political_df) != num_districts:
        errors.append(f"Expected {num_districts} rows, found {len(political_df)}")

    # Check required columns
    required_cols = ['district', 'dem_votes', 'rep_votes', 'dem_share', 'rep_share', 'margin', 'winner']
    missing_cols = [col for col in required_cols if col not in political_df.columns]
    if missing_cols:
        errors.append(f"Missing required columns: {missing_cols}")
        return {'valid': False, 'errors': errors}

    # Check shares sum to 1.0
    share_sums = political_df['dem_share'] + political_df['rep_share']
    if not (abs(share_sums - 1.0) < 0.01).all():
        errors.append("Dem + Rep shares don't sum to 1.0")

    # Check winner matches votes
    for _, row in political_df.iterrows():
        if row['dem_votes'] > row['rep_votes'] and row['winner'] != 'D':
            errors.append(f"District {row['district']}: D has more votes but winner is {row['winner']}")
        elif row['rep_votes'] > row['dem_votes'] and row['winner'] != 'R':
            errors.append(f"District {row['district']}: R has more votes but winner is {row['winner']}")

    return {
        'valid': len(errors) == 0,
        'errors': errors,
        'd_seats': (political_df['winner'] == 'D').sum() if 'winner' in political_df.columns else None,
        'r_seats': (political_df['winner'] == 'R').sum() if 'winner' in political_df.columns else None,
    }


def validate_compactness_metrics(compactness_df, num_districts):
    """
    Validate compactness metrics quality.

    Parameters
    ----------
    compactness_df : pandas.DataFrame
        Compactness analysis data
    num_districts : int
        Expected number of districts

    Returns
    -------
    dict
        Validation results

    Examples
    --------
    >>> from tests.mocks.mock_tracts import generate_mock_tracts
    >>> from tests.mocks.mock_districts import generate_mock_districts
    >>> from tests.mocks.mock_analysis import generate_mock_compactness_analysis
    >>> tracts = generate_mock_tracts(num_tracts=200)
    >>> districts = generate_mock_districts(tracts, num_districts=7)
    >>> compactness = generate_mock_compactness_analysis(districts)
    >>> result = validate_compactness_metrics(compactness, num_districts=7)
    >>> result['valid']
    True
    """
    errors = []

    # Check row count
    if len(compactness_df) != num_districts:
        errors.append(f"Expected {num_districts} rows, found {len(compactness_df)}")

    # Check required columns
    required_cols = ['district', 'polsby_popper', 'reock', 'area_sq_km', 'perimeter_km']
    missing_cols = [col for col in required_cols if col not in compactness_df.columns]
    if missing_cols:
        errors.append(f"Missing required columns: {missing_cols}")
        return {'valid': False, 'errors': errors}

    # Check Polsby-Popper in [0, 1]
    if not (compactness_df['polsby_popper'] >= 0.0).all():
        errors.append("Some Polsby-Popper values < 0")
    if not (compactness_df['polsby_popper'] <= 1.0).all():
        errors.append("Some Polsby-Popper values > 1")

    # Check Reock in [0, 1]
    if not (compactness_df['reock'] >= 0.0).all():
        errors.append("Some Reock values < 0")
    if not (compactness_df['reock'] <= 1.0).all():
        errors.append("Some Reock values > 1")

    # Check area and perimeter are positive
    if not (compactness_df['area_sq_km'] > 0).all():
        errors.append("Some areas are not positive")
    if not (compactness_df['perimeter_km'] > 0).all():
        errors.append("Some perimeters are not positive")

    return {
        'valid': len(errors) == 0,
        'errors': errors,
        'mean_polsby_popper': compactness_df['polsby_popper'].mean(),
        'mean_reock': compactness_df['reock'].mean(),
    }


def validate_output_directory_structure(output_dir):
    """
    Validate output directory structure.

    Parameters
    ----------
    output_dir : Path or str
        Output directory to validate

    Returns
    -------
    dict
        Validation results

    Examples
    --------
    >>> from pathlib import Path
    >>> import tempfile
    >>> with tempfile.TemporaryDirectory() as tmpdir:
    ...     output_dir = Path(tmpdir)
    ...     (output_dir / 'csvs').mkdir()
    ...     (output_dir / 'maps').mkdir()
    ...     result = validate_output_directory_structure(output_dir)
    ...     result['valid']
    True
    """
    output_dir = Path(output_dir)
    errors = []

    # Check base directory exists
    if not output_dir.exists():
        errors.append(f"Output directory not found: {output_dir}")
        return {'valid': False, 'errors': errors}

    # Check expected subdirectories
    expected_subdirs = ['csvs', 'maps']
    missing_subdirs = []

    for subdir in expected_subdirs:
        subdir_path = output_dir / subdir
        if not subdir_path.exists():
            missing_subdirs.append(subdir)

    if missing_subdirs:
        errors.append(f"Missing subdirectories: {missing_subdirs}")

    return {
        'valid': len(errors) == 0,
        'errors': errors,
        'directory': str(output_dir),
    }
