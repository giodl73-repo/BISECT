"""
Path construction utilities.

This module provides helper functions to construct standard file paths
used throughout the redistricting pipeline. Centralizes path logic to
ensure consistency and reduce duplication.
"""

from pathlib import Path


def get_tract_file(state, year):
    """
    Get census tract file path for state and year.

    Args:
        state: State name (case-insensitive, spaces/underscores normalized)
        year: Census year as string ('2000', '2010', '2020')

    Returns:
        Path: Path to tract parquet file

    Example:
        >>> path = get_tract_file('California', '2020')
        >>> print(path)
        data/tracts/2020/california_tracts_2020.parquet
    """
    state_normalized = state.lower().replace(' ', '_')
    return Path(f'data/tracts/{year}/{state_normalized}_tracts_{year}.parquet')


def get_places_file(state, year):
    """
    Get places file path for state and year.

    Args:
        state: State name (case-insensitive, spaces/underscores normalized)
        year: Census year as string ('2000', '2010', '2020')

    Returns:
        Path: Path to places parquet file

    Example:
        >>> path = get_places_file('New York', '2020')
        >>> print(path)
        data/tracts/2020/new_york_places_2020.parquet
    """
    state_normalized = state.lower().replace(' ', '_')
    return Path(f'data/tracts/{year}/{state_normalized}_places_{year}.parquet')


def get_adjacency_file(state, year):
    """
    Get adjacency graph file path for state and year.

    Args:
        state: State name (case-insensitive, spaces/underscores normalized)
        year: Census year as string ('2000', '2010', '2020')

    Returns:
        Path: Path to adjacency pickle file

    Example:
        >>> path = get_adjacency_file('california', '2020')
        >>> print(path)
        data/adjacency/2020/california_adjacency_2020.pkl
    """
    state_normalized = state.lower().replace(' ', '_')
    return Path(f'data/adjacency/{year}/{state_normalized}_adjacency_{year}.pkl')


def get_output_dir(year, version):
    """
    Get base output directory for a redistricting run.

    Args:
        year: Census year as string ('2000', '2010', '2020')
        version: Version identifier (e.g., 'v1', 'edge_weighted', 'test')

    Returns:
        Path: Path to output directory

    Example:
        >>> path = get_output_dir('2020', 'v1')
        >>> print(path)
        outputs/v1/2020
    """
    return Path(f'outputs/{version}/{year}')


def get_state_output_dir(state, year, version):
    """
    Get state-specific output directory.

    Args:
        state: State name (case-insensitive, spaces/underscores normalized)
        year: Census year as string ('2000', '2010', '2020')
        version: Version identifier (e.g., 'v1', 'edge_weighted', 'test')

    Returns:
        Path: Path to state output directory

    Example:
        >>> path = get_state_output_dir('California', '2020', 'v1')
        >>> print(path)
        outputs/v1/2020/states/california
    """
    state_normalized = state.lower().replace(' ', '_')
    return get_output_dir(year, version) / 'states' / state_normalized


def get_election_data_file(year):
    """
    Get processed election data file path.

    Args:
        year: Election year as string ('2020' currently supported)

    Returns:
        Path: Path to election data parquet file

    Example:
        >>> path = get_election_data_file('2020')
        >>> print(path)
        data/processed/elections/2020_president_tract.parquet
    """
    return Path(f'data/processed/elections/{year}_president_tract.parquet')


def get_demographic_data_file(year):
    """
    Get processed demographic data file path.

    Args:
        year: Census year as string ('2000', '2010', '2020')

    Returns:
        Path: Path to demographic data parquet file

    Example:
        >>> path = get_demographic_data_file('2020')
        >>> print(path)
        data/processed/demographics/2020_demographics_tract.parquet
    """
    return Path(f'data/processed/demographics/{year}_demographics_tract.parquet')
