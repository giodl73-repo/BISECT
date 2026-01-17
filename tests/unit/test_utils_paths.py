"""
Unit tests for scripts.utils.paths module.
"""

import pytest
from pathlib import Path
from scripts.utils.paths import (
    get_tract_file,
    get_places_file,
    get_adjacency_file,
    get_output_dir,
    get_state_output_dir,
    get_election_data_file,
    get_demographic_data_file,
)


class TestGetTractFile:
    """Test get_tract_file function."""

    def test_get_tract_file_basic(self):
        """Test basic tract file path construction."""
        path = get_tract_file('california', '2020')

        assert isinstance(path, Path)
        assert path.as_posix() == 'data/tracts/2020/california_tracts_2020.parquet'

    def test_get_tract_file_case_insensitive(self):
        """Test that state names are normalized to lowercase."""
        path1 = get_tract_file('California', '2020')
        path2 = get_tract_file('CALIFORNIA', '2020')
        path3 = get_tract_file('california', '2020')

        assert path1 == path2 == path3

    def test_get_tract_file_spaces_to_underscores(self):
        """Test that spaces are converted to underscores."""
        path = get_tract_file('New York', '2020')

        assert path.as_posix() == 'data/tracts/2020/new_york_tracts_2020.parquet'

    def test_get_tract_file_all_years(self):
        """Test tract file paths for all census years."""
        for year in ['2000', '2010', '2020']:
            path = get_tract_file('california', year)
            assert path.as_posix() == f'data/tracts/{year}/california_tracts_{year}.parquet'


class TestGetPlacesFile:
    """Test get_places_file function."""

    def test_get_places_file_basic(self):
        """Test basic places file path construction."""
        path = get_places_file('california', '2020')

        assert isinstance(path, Path)
        assert path.as_posix() == 'data/tracts/2020/california_places_2020.parquet'

    def test_get_places_file_normalization(self):
        """Test state name normalization."""
        path = get_places_file('North Dakota', '2020')

        assert path.as_posix() == 'data/tracts/2020/north_dakota_places_2020.parquet'


class TestGetAdjacencyFile:
    """Test get_adjacency_file function."""

    def test_get_adjacency_file_basic(self):
        """Test basic adjacency file path construction."""
        path = get_adjacency_file('california', '2020')

        assert isinstance(path, Path)
        assert path.as_posix() == 'data/adjacency/2020/california_adjacency_2020.pkl'

    def test_get_adjacency_file_all_years(self):
        """Test adjacency file paths for all census years."""
        for year in ['2000', '2010', '2020']:
            path = get_adjacency_file('texas', year)
            assert path.as_posix() == f'data/adjacency/{year}/texas_adjacency_{year}.pkl'


class TestGetOutputDir:
    """Test get_output_dir function."""

    def test_get_output_dir_basic(self):
        """Test basic output directory construction."""
        path = get_output_dir('2020', 'v1')

        assert isinstance(path, Path)
        assert path.as_posix() == 'outputs/v1/2020'

    def test_get_output_dir_version_variations(self):
        """Test output directory with different version formats."""
        path1 = get_output_dir('2020', 'v1')
        path2 = get_output_dir('2020', 'edge_weighted')
        path3 = get_output_dir('2020', 'test')

        assert path1.as_posix() == 'outputs/v1/2020'
        assert path2.as_posix() == 'outputs/edge_weighted/2020'
        assert path3.as_posix() == 'outputs/test/2020'

    def test_get_output_dir_all_years(self):
        """Test output directory for all census years."""
        for year in ['2000', '2010', '2020']:
            path = get_output_dir(year, 'v1')
            assert path.as_posix() == f'outputs/v1/{year}'


class TestGetStateOutputDir:
    """Test get_state_output_dir function."""

    def test_get_state_output_dir_basic(self):
        """Test basic state output directory construction."""
        path = get_state_output_dir('california', '2020', 'v1')

        assert isinstance(path, Path)
        assert path.as_posix() == 'outputs/v1/2020/states/california'

    def test_get_state_output_dir_normalization(self):
        """Test state name normalization in output directories."""
        path1 = get_state_output_dir('California', '2020', 'v1')
        path2 = get_state_output_dir('CALIFORNIA', '2020', 'v1')
        path3 = get_state_output_dir('california', '2020', 'v1')

        assert path1 == path2 == path3

    def test_get_state_output_dir_spaces(self):
        """Test space to underscore conversion."""
        path = get_state_output_dir('New York', '2020', 'v1')

        assert path.as_posix() == 'outputs/v1/2020/states/new_york'

    def test_get_state_output_dir_structure(self):
        """Test that state directory is under states/ subdirectory."""
        path = get_state_output_dir('texas', '2010', 'v2')

        assert path.parent.name == 'states'
        assert path.as_posix() == 'outputs/v2/2010/states/texas'


class TestGetElectionDataFile:
    """Test get_election_data_file function."""

    def test_get_election_data_file_2020(self):
        """Test election data file path for 2020."""
        path = get_election_data_file('2020')

        assert isinstance(path, Path)
        assert path.as_posix() == 'data/processed/elections/2020_president_tract.parquet'

    def test_get_election_data_file_structure(self):
        """Test election data file structure."""
        path = get_election_data_file('2020')

        assert path.parent.name == 'elections'
        assert path.name == '2020_president_tract.parquet'


class TestGetDemographicDataFile:
    """Test get_demographic_data_file function."""

    def test_get_demographic_data_file_basic(self):
        """Test demographic data file path construction."""
        path = get_demographic_data_file('2020')

        assert isinstance(path, Path)
        assert path.as_posix() == 'data/processed/demographics/2020_demographics_tract.parquet'

    def test_get_demographic_data_file_all_years(self):
        """Test demographic data file paths for all census years."""
        for year in ['2000', '2010', '2020']:
            path = get_demographic_data_file(year)
            assert path.as_posix() == f'data/processed/demographics/{year}_demographics_tract.parquet'

    def test_get_demographic_data_file_structure(self):
        """Test demographic data file structure."""
        path = get_demographic_data_file('2020')

        assert path.parent.name == 'demographics'
        assert 'demographics_tract' in path.name


class TestPathConsistency:
    """Test consistency across path functions."""

    def test_year_consistency(self):
        """Test that year appears consistently in all paths."""
        year = '2020'
        state = 'california'

        tract_path = get_tract_file(state, year)
        adjacency_path = get_adjacency_file(state, year)
        output_path = get_output_dir(year, 'v1')

        # All paths should contain the year
        assert year in str(tract_path)
        assert year in str(adjacency_path)
        assert year in str(output_path)

    def test_state_normalization_consistency(self):
        """Test that state normalization is consistent across functions."""
        states = ['California', 'NEW YORK', 'north dakota']

        for state in states:
            tract = get_tract_file(state, '2020')
            places = get_places_file(state, '2020')
            adjacency = get_adjacency_file(state, '2020')
            output = get_state_output_dir(state, '2020', 'v1')

            # Extract state name from each path using Path.name
            state_in_tract = tract.name.split('_tracts')[0]
            state_in_places = places.name.split('_places')[0]
            state_in_adjacency = adjacency.name.split('_adjacency')[0]
            state_in_output = output.name

            # All should be identical (lowercase, underscores)
            assert state_in_tract == state_in_places == state_in_adjacency == state_in_output
