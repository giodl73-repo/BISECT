"""
Path construction utilities.

This module provides helper functions to construct standard file paths
used throughout the redistricting pipeline. Centralizes path logic to
ensure consistency and reduce duplication.

Raw census data: data/Census {year}/
Processed census data: outputs/{version}/data/{year}/
Redistricting results: outputs/{version}/results/{year}/

As of Enhancement 52, all processed census data is version-specific to enable
independent algorithm experiments and preprocessing variants.
"""

from pathlib import Path


def get_version_dir(version):
    """
    Get version base directory.

    Args:
        version: Version identifier (e.g., 'v1', 'test', 'edge_weighted')

    Returns:
        Path: Path to version directory

    Example:
        >>> path = get_version_dir('v1')
        >>> print(path)
        outputs/v1
    """
    return Path(f'outputs/{version}')


def get_census_data_dir(version, year):
    """
    Get census data directory for a version and year.

    Args:
        version: Version identifier (e.g., 'v1', 'test', 'edge_weighted')
        year: Census year as string ('2000', '2010', '2020')

    Returns:
        Path: Path to census data directory

    Example:
        >>> path = get_census_data_dir('v1', '2020')
        >>> print(path)
        outputs/v1/data/2020
    """
    return get_version_dir(version) / 'data' / year


def get_results_dir(version, year):
    """
    Get results directory for a version and year.

    Args:
        version: Version identifier (e.g., 'v1', 'test', 'edge_weighted')
        year: Census year as string ('2000', '2010', '2020')

    Returns:
        Path: Path to results directory

    Example:
        >>> path = get_results_dir('v1', '2020')
        >>> print(path)
        outputs/v1/results/2020
    """
    return get_version_dir(version) / 'results' / year


def get_tract_file(state, year, version):
    """
    Get census tract file path for state, year, and version.

    DEPRECATED: Use get_unit_file() with resolution='tract' for new code.

    Args:
        state: State name (case-insensitive, spaces/underscores normalized)
        year: Census year as string ('2000', '2010', '2020')
        version: Version identifier (e.g., 'v1', 'test', 'edge_weighted')

    Returns:
        Path: Path to tract parquet file (processed data)

    Example:
        >>> path = get_tract_file('California', '2020', 'v1')
        >>> print(path)
        outputs/v1/data/2020/units/california_tracts_2020.parquet
    """
    return get_unit_file(state, year, version, resolution='tract')


def get_unit_file(state, year, version, resolution='tract'):
    """
    Get census unit file path for state, year, version, and resolution.

    Args:
        state: State name (case-insensitive, spaces/underscores normalized)
        year: Census year as string ('2000', '2010', '2020')
        version: Version identifier (e.g., 'v1', 'test', 'edge_weighted')
        resolution: Geographic resolution ('tract', 'block_group', 'block')

    Returns:
        Path: Path to unit parquet file (processed data)

    Examples:
        >>> path = get_unit_file('California', '2020', 'v1', 'tract')
        >>> print(path)
        outputs/v1/data/2020/units/california_tracts_2020.parquet

        >>> path = get_unit_file('California', '2020', 'v1', 'block_group')
        >>> print(path)
        outputs/v1/data/2020/units/california_block_groups_2020.parquet
    """
    state_normalized = state.lower().replace(' ', '_')
    resolution_suffix = 'tracts' if resolution == 'tract' else f'{resolution}s'
    return get_census_data_dir(version, year) / 'units' / f'{state_normalized}_{resolution_suffix}_{year}.parquet'


def get_places_file(state, year, version):
    """
    Get places file path for state, year, and version.

    Args:
        state: State name (case-insensitive, spaces/underscores normalized)
        year: Census year as string ('2000', '2010', '2020')
        version: Version identifier (e.g., 'v1', 'test', 'edge_weighted')

    Returns:
        Path: Path to places parquet file (processed data)

    Example:
        >>> path = get_places_file('New York', '2020', 'v1')
        >>> print(path)
        outputs/v1/data/2020/places/new_york_places_2020.parquet
    """
    state_normalized = state.lower().replace(' ', '_')
    return get_census_data_dir(version, year) / 'places' / f'{state_normalized}_places_{year}.parquet'


def get_adjacency_file(state, year, version, resolution='tract'):
    """
    Get adjacency graph file path for state, year, version, and resolution.

    Args:
        state: State name (case-insensitive, spaces/underscores normalized)
        year: Census year as string ('2000', '2010', '2020')
        version: Version identifier (e.g., 'v1', 'test', 'edge_weighted')
        resolution: Geographic resolution ('tract', 'block_group', 'block') - default 'tract' for backward compatibility

    Returns:
        Path: Path to adjacency pickle file (processed data)

    Examples:
        >>> path = get_adjacency_file('california', '2020', 'v1', 'tract')
        >>> print(path)
        outputs/v1/data/2020/adjacency/california_adjacency_2020.pkl

        >>> path = get_adjacency_file('california', '2020', 'v1', 'block')
        >>> print(path)
        outputs/v1/data/2020/adjacency/california_block_adjacency_2020.pkl
    """
    state_normalized = state.lower().replace(' ', '_')
    if resolution == 'tract':
        # Backward compatibility: tracts use old naming without resolution prefix
        return get_census_data_dir(version, year) / 'adjacency' / f'{state_normalized}_adjacency_{year}.pkl'
    else:
        # New format: include resolution in filename
        return get_census_data_dir(version, year) / 'adjacency' / f'{state_normalized}_{resolution}_adjacency_{year}.pkl'


def get_election_data_file(year, version):
    """
    Get processed election data file path for year and version.

    Args:
        year: Election year as string ('2020' currently supported)
        version: Version identifier (e.g., 'v1', 'test', 'edge_weighted')

    Returns:
        Path: Path to election data parquet file (processed data)

    Example:
        >>> path = get_election_data_file('2020', 'v1')
        >>> print(path)
        outputs/v1/data/2020/elections/2020_president_tract.parquet
    """
    return get_census_data_dir(version, year) / 'elections' / f'{year}_president_tract.parquet'


def get_demographic_data_file(year, version):
    """
    Get processed demographic data file path for year and version.

    Args:
        year: Census year as string ('2000', '2010', '2020')
        version: Version identifier (e.g., 'v1', 'test', 'edge_weighted')

    Returns:
        Path: Path to demographic data parquet file (processed data)

    Example:
        >>> path = get_demographic_data_file('2020', 'v1')
        >>> print(path)
        outputs/v1/data/2020/demographics/2020_demographics_tract.parquet
    """
    return get_census_data_dir(version, year) / 'demographics' / f'{year}_demographics_tract.parquet'


def get_output_dir(version, year):
    """
    Get base results directory for a redistricting run.

    Args:
        version: Version identifier (e.g., 'v1', 'test', 'edge_weighted')
        year: Census year as string ('2000', '2010', '2020')

    Returns:
        Path: Path to results directory

    Example:
        >>> path = get_output_dir('v1', '2020')
        >>> print(path)
        outputs/v1/results/2020

    Note:
        Parameter order changed in Enhancement 52: version before year for consistency
    """
    return get_results_dir(version, year)


def get_state_output_dir(state, version, year):
    """
    Get state-specific results directory.

    Args:
        state: State name (case-insensitive, spaces/underscores normalized)
        version: Version identifier (e.g., 'v1', 'test', 'edge_weighted')
        year: Census year as string ('2000', '2010', '2020')

    Returns:
        Path: Path to state results directory

    Example:
        >>> path = get_state_output_dir('California', 'v1', '2020')
        >>> print(path)
        outputs/v1/results/2020/states/california

    Note:
        Parameter order changed in Enhancement 52: version before year for consistency
    """
    state_normalized = state.lower().replace(' ', '_')
    return get_results_dir(version, year) / 'states' / state_normalized
