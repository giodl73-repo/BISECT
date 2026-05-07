"""
Unit tests for scripts.utils.paths module.

Tests for Enhancement 52: Per-Version Census Data Structure.
All path functions now require version parameter and use the new structure:
- Census data: outputs/{version}/data/{year}/
- Results: outputs/{version}/results/{year}/
"""

import pytest
from pathlib import Path
from scripts.utils.paths import (
    get_version_dir,
    get_census_data_dir,
    get_results_dir,
    get_tract_file,
    get_places_file,
    get_adjacency_file,
    get_output_dir,
    get_state_output_dir,
    get_election_data_file,
    get_demographic_data_file,
)


class TestGetVersionDir:
    """Test get_version_dir function."""

    def test_get_version_dir_basic(self):
        """Test basic version directory construction."""
        path = get_version_dir('v1')

        assert isinstance(path, Path)
        assert path.as_posix() == 'outputs/v1'

    def test_get_version_dir_variations(self):
        """Test version directory with different version formats."""
        path1 = get_version_dir('v1')
        path2 = get_version_dir('test')
        path3 = get_version_dir('edge_weighted')

        assert path1.as_posix() == 'outputs/v1'
        assert path2.as_posix() == 'outputs/test'
        assert path3.as_posix() == 'outputs/edge_weighted'


class TestGetCensusDataDir:
    """Test get_census_data_dir function."""

    def test_get_census_data_dir_basic(self):
        """Test basic census data directory construction."""
        path = get_census_data_dir('v1', '2020')

        assert isinstance(path, Path)
        assert path.as_posix() == 'outputs/v1/data/2020'

    def test_get_census_data_dir_all_years(self):
        """Test census data directory for all census years."""
        for year in ['2000', '2010', '2020']:
            path = get_census_data_dir('v1', year)
            assert path.as_posix() == f'outputs/v1/data/{year}'

    def test_get_census_data_dir_all_versions(self):
        """Test census data directory for different versions."""
        for version in ['v1', 'v2', 'test']:
            path = get_census_data_dir(version, '2020')
            assert version in path.as_posix()
            assert path.as_posix() == f'outputs/{version}/data/2020'

    def test_get_census_data_dir_structure(self):
        """Test that census data directory has correct structure."""
        path = get_census_data_dir('v1', '2020')

        # Should be: outputs/v1/data/2020
        assert path.parent.name == 'data'
        assert path.parent.parent.name == 'v1'


class TestGetResultsDir:
    """Test get_results_dir function."""

    def test_get_results_dir_basic(self):
        """Test basic results directory construction."""
        path = get_results_dir('v1', '2020')

        assert isinstance(path, Path)
        assert path.as_posix() == 'outputs/v1/results/2020'

    def test_get_results_dir_all_years(self):
        """Test results directory for all census years."""
        for year in ['2000', '2010', '2020']:
            path = get_results_dir('v1', year)
            assert path.as_posix() == f'outputs/v1/results/{year}'

    def test_get_results_dir_structure(self):
        """Test that results directory has correct structure."""
        path = get_results_dir('v1', '2020')

        # Should be: outputs/v1/results/2020
        assert path.parent.name == 'results'
        assert path.parent.parent.name == 'v1'


class TestGetTractFile:
    """Test get_tract_file function."""

    def test_get_tract_file_basic(self):
        """Test basic tract file path construction with version."""
        path = get_tract_file('california', '2020', 'v1')

        assert isinstance(path, Path)
        assert path.as_posix() == 'outputs/v1/data/2020/units/california_tracts_2020.parquet'

    def test_get_tract_file_case_insensitive(self):
        """Test that state names are normalized to lowercase."""
        path1 = get_tract_file('California', '2020', 'v1')
        path2 = get_tract_file('CALIFORNIA', '2020', 'v1')
        path3 = get_tract_file('california', '2020', 'v1')

        assert path1 == path2 == path3

    def test_get_tract_file_spaces_to_underscores(self):
        """Test that spaces are converted to underscores."""
        path = get_tract_file('New York', '2020', 'v1')

        assert path.as_posix() == 'outputs/v1/data/2020/units/new_york_tracts_2020.parquet'

    def test_get_tract_file_all_years(self):
        """Test tract file paths for all census years."""
        for year in ['2000', '2010', '2020']:
            path = get_tract_file('california', year, 'v1')
            assert path.as_posix() == f'outputs/v1/data/{year}/units/california_tracts_{year}.parquet'

    def test_get_tract_file_version_specific(self):
        """Test that tract files are version-specific."""
        path_v1 = get_tract_file('california', '2020', 'v1')
        path_v2 = get_tract_file('california', '2020', 'v2')

        assert path_v1 != path_v2
        assert 'v1' in path_v1.as_posix()
        assert 'v2' in path_v2.as_posix()


class TestGetPlacesFile:
    """Test get_places_file function."""

    def test_get_places_file_basic(self):
        """Test basic places file path construction with version."""
        path = get_places_file('california', '2020', 'v1')

        assert isinstance(path, Path)
        assert path.as_posix() == 'outputs/v1/data/2020/places/california_places_2020.parquet'

    def test_get_places_file_normalization(self):
        """Test state name normalization."""
        path = get_places_file('North Dakota', '2020', 'v1')

        assert path.as_posix() == 'outputs/v1/data/2020/places/north_dakota_places_2020.parquet'

    def test_get_places_file_version_specific(self):
        """Test that places files are version-specific."""
        path_v1 = get_places_file('california', '2020', 'v1')
        path_test = get_places_file('california', '2020', 'test')

        assert path_v1 != path_test
        assert 'v1' in path_v1.as_posix()
        assert 'test' in path_test.as_posix()


class TestGetAdjacencyFile:
    """Test get_adjacency_file function."""

    def test_get_adjacency_file_basic(self):
        """Test basic adjacency file path construction with version."""
        path = get_adjacency_file('california', '2020', 'v1')

        assert isinstance(path, Path)
        assert path.as_posix() == 'outputs/v1/data/2020/adjacency/california_adjacency_2020.pkl'

    def test_get_adjacency_file_all_years(self):
        """Test adjacency file paths for all census years."""
        for year in ['2000', '2010', '2020']:
            path = get_adjacency_file('texas', year, 'v1')
            assert path.as_posix() == f'outputs/v1/data/{year}/adjacency/texas_adjacency_{year}.pkl'

    def test_get_adjacency_file_version_specific(self):
        """Test that adjacency files are version-specific."""
        path_v1 = get_adjacency_file('texas', '2020', 'v1')
        path_v2 = get_adjacency_file('texas', '2020', 'v2')

        assert path_v1 != path_v2
        assert 'v1' in path_v1.as_posix()
        assert 'v2' in path_v2.as_posix()


class TestGetElectionDataFile:
    """Test get_election_data_file function."""

    def test_get_election_data_file_2020(self):
        """Test election data file path for 2020 with version."""
        path = get_election_data_file('2020', 'v1')

        assert isinstance(path, Path)
        assert path.as_posix() == 'outputs/v1/data/2020/elections/2020_president_tract.parquet'

    def test_get_election_data_file_structure(self):
        """Test election data file structure."""
        path = get_election_data_file('2020', 'v1')

        assert path.parent.name == 'elections'
        assert path.name == '2020_president_tract.parquet'

    def test_get_election_data_file_version_specific(self):
        """Test that election files are version-specific."""
        path_v1 = get_election_data_file('2020', 'v1')
        path_v2 = get_election_data_file('2020', 'v2')

        assert path_v1 != path_v2
        assert 'v1' in path_v1.as_posix()
        assert 'v2' in path_v2.as_posix()


class TestGetDemographicDataFile:
    """Test get_demographic_data_file function."""

    def test_get_demographic_data_file_basic(self):
        """Test demographic data file path construction with version."""
        path = get_demographic_data_file('2020', 'v1')

        assert isinstance(path, Path)
        assert path.as_posix() == 'outputs/v1/data/2020/demographics/2020_demographics_tract.parquet'

    def test_get_demographic_data_file_all_years(self):
        """Test demographic data file paths for all census years."""
        for year in ['2000', '2010', '2020']:
            path = get_demographic_data_file(year, 'v1')
            assert path.as_posix() == f'outputs/v1/data/{year}/demographics/{year}_demographics_tract.parquet'

    def test_get_demographic_data_file_structure(self):
        """Test demographic data file structure."""
        path = get_demographic_data_file('2020', 'v1')

        assert path.parent.name == 'demographics'
        assert 'demographics_tract' in path.name

    def test_get_demographic_data_file_version_specific(self):
        """Test that demographic files are version-specific."""
        path_v1 = get_demographic_data_file('2020', 'v1')
        path_test = get_demographic_data_file('2020', 'test')

        assert path_v1 != path_test
        assert 'v1' in path_v1.as_posix()
        assert 'test' in path_test.as_posix()


class TestGetOutputDir:
    """Test get_output_dir function (alias for get_results_dir)."""

    def test_get_output_dir_basic(self):
        """Test basic output directory construction."""
        path = get_output_dir('v1', '2020')

        assert isinstance(path, Path)
        assert path.as_posix() == 'outputs/v1/results/2020'

    def test_get_output_dir_parameter_order(self):
        """Test that parameter order is (version, year)."""
        path = get_output_dir('v1', '2020')

        # Should be version first, year second
        assert path.as_posix() == 'outputs/v1/results/2020'

    def test_get_output_dir_version_variations(self):
        """Test output directory with different version formats."""
        path1 = get_output_dir('v1', '2020')
        path2 = get_output_dir('edge_weighted', '2020')
        path3 = get_output_dir('test', '2020')

        assert path1.as_posix() == 'outputs/v1/results/2020'
        assert path2.as_posix() == 'outputs/edge_weighted/results/2020'
        assert path3.as_posix() == 'outputs/test/results/2020'

    def test_get_output_dir_all_years(self):
        """Test output directory for all census years."""
        for year in ['2000', '2010', '2020']:
            path = get_output_dir('v1', year)
            assert path.as_posix() == f'outputs/v1/results/{year}'


class TestGetStateOutputDir:
    """Test get_state_output_dir function."""

    def test_get_state_output_dir_basic(self):
        """Test basic state output directory construction."""
        path = get_state_output_dir('california', 'v1', '2020')

        assert isinstance(path, Path)
        assert path.as_posix() == 'outputs/v1/results/2020/states/california'

    def test_get_state_output_dir_parameter_order(self):
        """Test that parameter order is (state, version, year)."""
        path = get_state_output_dir('california', 'v1', '2020')

        # Should be state, version, year
        assert path.as_posix() == 'outputs/v1/results/2020/states/california'

    def test_get_state_output_dir_normalization(self):
        """Test state name normalization in output directories."""
        path1 = get_state_output_dir('California', 'v1', '2020')
        path2 = get_state_output_dir('CALIFORNIA', 'v1', '2020')
        path3 = get_state_output_dir('california', 'v1', '2020')

        assert path1 == path2 == path3

    def test_get_state_output_dir_spaces(self):
        """Test space to underscore conversion."""
        path = get_state_output_dir('New York', 'v1', '2020')

        assert path.as_posix() == 'outputs/v1/results/2020/states/new_york'

    def test_get_state_output_dir_structure(self):
        """Test that state directory is under states/ subdirectory."""
        path = get_state_output_dir('texas', 'v2', '2010')

        assert path.parent.name == 'states'
        assert path.as_posix() == 'outputs/v2/results/2010/states/texas'


class TestPathConsistency:
    """Test consistency across path functions."""

    def test_year_consistency(self):
        """Test that year appears consistently in all paths."""
        year = '2020'
        state = 'california'
        version = 'v1'

        tract_path = get_tract_file(state, year, version)
        adjacency_path = get_adjacency_file(state, year, version)
        output_path = get_output_dir(version, year)

        # All paths should contain the year
        assert year in str(tract_path)
        assert year in str(adjacency_path)
        assert year in str(output_path)

    def test_version_consistency(self):
        """Test that version appears consistently in all paths."""
        version = 'v1'
        year = '2020'
        state = 'california'

        tract_path = get_tract_file(state, year, version)
        adjacency_path = get_adjacency_file(state, year, version)
        output_path = get_output_dir(version, year)
        election_path = get_election_data_file(year, version)

        # All paths should contain the version
        assert version in str(tract_path)
        assert version in str(adjacency_path)
        assert version in str(output_path)
        assert version in str(election_path)

    def test_state_normalization_consistency(self):
        """Test that state normalization is consistent across functions."""
        states = ['California', 'NEW YORK', 'north dakota']
        version = 'v1'

        for state in states:
            tract = get_tract_file(state, '2020', version)
            places = get_places_file(state, '2020', version)
            adjacency = get_adjacency_file(state, '2020', version)
            output = get_state_output_dir(state, version, '2020')

            # Extract state name from each path using Path.name
            state_in_tract = tract.name.split('_tracts')[0]
            state_in_places = places.name.split('_places')[0]
            state_in_adjacency = adjacency.name.split('_adjacency')[0]
            state_in_output = output.name

            # All should be identical (lowercase, underscores)
            assert state_in_tract == state_in_places == state_in_adjacency == state_in_output

    def test_census_data_vs_results_separation(self):
        """Test that census data and results are properly separated."""
        version = 'v1'
        year = '2020'

        # Census data paths should contain 'data'
        tract_path = get_tract_file('california', year, version)
        adjacency_path = get_adjacency_file('california', year, version)
        election_path = get_election_data_file(year, version)

        assert 'data' in str(tract_path)
        assert 'data' in str(adjacency_path)
        assert 'data' in str(election_path)

        # Results paths should contain 'results'
        results_path = get_results_dir(version, year)
        output_path = get_output_dir(version, year)
        state_path = get_state_output_dir('california', version, year)

        assert 'results' in str(results_path)
        assert 'results' in str(output_path)
        assert 'results' in str(state_path)

    def test_version_isolation(self):
        """Test that different versions produce different paths."""
        versions = ['v1', 'v2', 'test']
        year = '2020'
        state = 'california'

        paths_by_version = {}
        for version in versions:
            paths_by_version[version] = {
                'tract': get_tract_file(state, year, version),
                'adjacency': get_adjacency_file(state, year, version),
                'results': get_results_dir(version, year),
                'state': get_state_output_dir(state, version, year),
            }

        # All paths should be different across versions
        v1_paths = paths_by_version['v1']
        v2_paths = paths_by_version['v2']
        test_paths = paths_by_version['test']

        assert v1_paths['tract'] != v2_paths['tract'] != test_paths['tract']
        assert v1_paths['adjacency'] != v2_paths['adjacency'] != test_paths['adjacency']
        assert v1_paths['results'] != v2_paths['results'] != test_paths['results']
        assert v1_paths['state'] != v2_paths['state'] != test_paths['state']

    def test_same_version_same_year_produces_identical_paths(self):
        """Test that calling functions multiple times with same args produces identical results."""
        version = 'v1'
        year = '2020'
        state = 'california'

        # Call each function twice
        tract1 = get_tract_file(state, year, version)
        tract2 = get_tract_file(state, year, version)

        adjacency1 = get_adjacency_file(state, year, version)
        adjacency2 = get_adjacency_file(state, year, version)

        results1 = get_results_dir(version, year)
        results2 = get_results_dir(version, year)

        # Should be identical
        assert tract1 == tract2
        assert adjacency1 == adjacency2
        assert results1 == results2
