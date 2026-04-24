#!/usr/bin/env python3
"""
Unit tests for centralized download configuration.

Tests: scripts/config/download_sources.py
Created: 2026-01-18 (Enhancement #48)
"""

import pytest
from scripts.config.download_sources import (
    STATE_FIPS, FIPS_TO_STATE, ALL_STATES, ALL_STATES_WITH_DC,
    STATES_BY_REGION, TEST_STATES, CENSUS_CONFIGS,
    get_fips_2digit, get_fips_3digit, get_state_from_fips,
    get_census_config, get_tiger_tract_url, get_tiger_place_url,
    get_tiger_cd_url, get_cbsa_url,
    validate_state_code, validate_year
)


class TestStateFIPSMappings:
    """Test STATE_FIPS and reverse mapping."""

    def test_state_fips_has_50_states(self):
        """All 50 states + DC should be in STATE_FIPS."""
        assert len(STATE_FIPS) == 51  # 50 states + DC

    def test_state_fips_2_digits(self):
        """All FIPS codes should be 2 digits."""
        for state, fips in STATE_FIPS.items():
            assert len(fips) == 2, f"{state} has FIPS '{fips}' (not 2 digits)"
            assert fips.isdigit(), f"{state} has non-numeric FIPS '{fips}'"

    def test_reverse_mapping(self):
        """FIPS_TO_STATE should be reverse of STATE_FIPS."""
        assert len(FIPS_TO_STATE) == len(STATE_FIPS)
        for state, fips in STATE_FIPS.items():
            assert FIPS_TO_STATE[fips] == state

    def test_known_states(self):
        """Test a few known state/FIPS pairs."""
        assert STATE_FIPS['CA'] == '06'
        assert STATE_FIPS['TX'] == '48'
        assert STATE_FIPS['NY'] == '36'
        assert STATE_FIPS['VT'] == '50'
        assert STATE_FIPS['DC'] == '11'

    def test_case_insensitive_lookup(self):
        """State codes should work regardless of case."""
        # This tests the utility functions, not the dict itself
        assert get_fips_2digit('ca') == '06'
        assert get_fips_2digit('CA') == '06'
        assert get_fips_2digit('Ca') == '06'


class TestStateLists:
    """Test state list constants."""

    def test_all_states_count(self):
        """ALL_STATES should have 50 states."""
        assert len(ALL_STATES) == 50

    def test_all_states_with_dc_count(self):
        """ALL_STATES_WITH_DC should have 51."""
        assert len(ALL_STATES_WITH_DC) == 51
        assert 'DC' in ALL_STATES_WITH_DC
        assert 'DC' not in ALL_STATES

    def test_all_states_unique(self):
        """No duplicates in state lists."""
        assert len(ALL_STATES) == len(set(ALL_STATES))
        assert len(ALL_STATES_WITH_DC) == len(set(ALL_STATES_WITH_DC))

    def test_all_states_valid(self):
        """All states in lists should be in STATE_FIPS."""
        for state in ALL_STATES:
            assert state in STATE_FIPS, f"{state} not in STATE_FIPS"
        for state in ALL_STATES_WITH_DC:
            assert state in STATE_FIPS, f"{state} not in STATE_FIPS"

    def test_regions_cover_all_states(self):
        """STATES_BY_REGION should cover all 50 states."""
        all_region_states = []
        for region, states in STATES_BY_REGION.items():
            all_region_states.extend(states)

        assert len(all_region_states) == 50
        assert set(all_region_states) == set(ALL_STATES)

    def test_test_states_valid(self):
        """TEST_STATES should be valid states."""
        for state in TEST_STATES:
            assert state in STATE_FIPS


class TestCensusConfigs:
    """Test Census API configurations."""

    def test_census_configs_years(self):
        """CENSUS_CONFIGS should have 2000, 2010, 2020."""
        assert 2000 in CENSUS_CONFIGS
        assert 2010 in CENSUS_CONFIGS
        assert 2020 in CENSUS_CONFIGS

    def test_census_configs_structure(self):
        """Each config should have required keys."""
        for year, config in CENSUS_CONFIGS.items():
            assert 'api_base' in config
            assert 'variables' in config
            assert 'table' in config
            assert 'description' in config

            # api_base should be URL
            assert config['api_base'].startswith('https://')

            # variables should be dict
            assert isinstance(config['variables'], dict)
            assert len(config['variables']) > 0

    def test_census_configs_variables(self):
        """Each config should have standard demographic variables."""
        required_vars = [
            'total_pop', 'male', 'female',
            'white_non_hispanic', 'black_non_hispanic',
            'asian_non_hispanic', 'hispanic'
        ]

        for year, config in CENSUS_CONFIGS.items():
            for var in required_vars:
                assert var in config['variables'], \
                    f"Year {year} missing variable '{var}'"


class TestFIPSUtilities:
    """Test FIPS code utility functions."""

    def test_get_fips_2digit_valid(self):
        """get_fips_2digit should return 2-digit codes."""
        assert get_fips_2digit('CA') == '06'
        assert get_fips_2digit('TX') == '48'
        assert get_fips_2digit('VT') == '50'

    def test_get_fips_2digit_case_insensitive(self):
        """get_fips_2digit should handle any case."""
        assert get_fips_2digit('ca') == '06'
        assert get_fips_2digit('CA') == '06'
        assert get_fips_2digit('Ca') == '06'

    def test_get_fips_2digit_invalid(self):
        """get_fips_2digit should raise ValueError for invalid states."""
        with pytest.raises(ValueError, match="Invalid state code"):
            get_fips_2digit('XX')
        with pytest.raises(ValueError):
            get_fips_2digit('ZZ')

    def test_get_fips_3digit_valid(self):
        """get_fips_3digit should return 3-digit codes."""
        assert get_fips_3digit('CA') == '060'
        assert get_fips_3digit('TX') == '480'
        assert get_fips_3digit('VT') == '500'
        assert get_fips_3digit('AL') == '010'

    def test_get_state_from_fips_2digit(self):
        """get_state_from_fips should work with 2-digit codes."""
        assert get_state_from_fips('06') == 'CA'
        assert get_state_from_fips('48') == 'TX'
        assert get_state_from_fips('01') == 'AL'

    def test_get_state_from_fips_3digit(self):
        """get_state_from_fips should work with 3-digit codes."""
        assert get_state_from_fips('060') == 'CA'
        assert get_state_from_fips('480') == 'TX'
        assert get_state_from_fips('010') == 'AL'

    def test_get_state_from_fips_invalid(self):
        """get_state_from_fips should raise ValueError for invalid FIPS."""
        with pytest.raises(ValueError, match="Invalid FIPS code"):
            get_state_from_fips('99')
        with pytest.raises(ValueError):
            get_state_from_fips('00')


class TestCensusConfigUtilities:
    """Test Census config utility functions."""

    def test_get_census_config_valid(self):
        """get_census_config should return config for valid years."""
        config_2000 = get_census_config(2000)
        assert config_2000['api_base'] == 'https://api.census.gov/data/2000/dec/sf1'

        config_2020 = get_census_config('2020')  # Test string input
        assert config_2020['api_base'] == 'https://api.census.gov/data/2020/dec/dhc'

    def test_get_census_config_invalid(self):
        """get_census_config should raise ValueError for invalid years."""
        with pytest.raises(ValueError, match="not supported"):
            get_census_config(1990)
        with pytest.raises(ValueError):
            get_census_config(2025)


class TestTIGERURLUtilities:
    """Test TIGER/Line URL generation."""

    def test_get_tiger_tract_url_2020(self):
        """get_tiger_tract_url should generate correct 2020 URLs."""
        url = get_tiger_tract_url('CA', 2020)
        assert 'tl_2020_06_tract.zip' in url
        assert 'TIGER2020' in url

    def test_get_tiger_tract_url_2010(self):
        """get_tiger_tract_url should generate correct 2010 URLs."""
        url = get_tiger_tract_url('TX', 2010)
        assert 'tl_2010_48_tract10.zip' in url
        assert 'TIGER2010' in url

    def test_get_tiger_tract_url_invalid_year(self):
        """get_tiger_tract_url should raise ValueError for invalid years."""
        with pytest.raises(ValueError, match="not supported"):
            get_tiger_tract_url('CA', 1990)

    def test_get_tiger_tract_url_invalid_state(self):
        """get_tiger_tract_url should raise ValueError for invalid states."""
        with pytest.raises(ValueError, match="Invalid state code"):
            get_tiger_tract_url('XX', 2020)

    def test_get_tiger_place_url_2020(self):
        """get_tiger_place_url should generate correct 2020 URLs."""
        url = get_tiger_place_url('NY', 2020)
        assert 'tl_2020_36_place.zip' in url
        assert 'TIGER2020' in url

    def test_get_tiger_cd_url_2020(self):
        """get_tiger_cd_url should return URL or fallback list."""
        result = get_tiger_cd_url(2020)
        assert isinstance(result, list)
        assert len(result) > 0
        assert 'cd118' in result[0]

    def test_get_tiger_cd_url_with_congress(self):
        """get_tiger_cd_url should handle specific congress numbers."""
        url = get_tiger_cd_url(2020, congress=118)
        assert 'cd118' in url
        assert isinstance(url, str)  # Specific congress returns single URL

    def test_get_cbsa_url_2020(self):
        """get_cbsa_url should return valid URL for 2020."""
        url = get_cbsa_url(2020)
        assert 'cbsa' in url.lower()
        assert '2020' in url

    def test_get_cbsa_url_invalid_year(self):
        """get_cbsa_url should raise ValueError for invalid years."""
        with pytest.raises(ValueError, match="not supported"):
            get_cbsa_url(1990)  # 1990 not in supported CBSA years


class TestValidation:
    """Test validation functions."""

    def test_validate_state_code_valid(self):
        """validate_state_code should return True for valid states."""
        assert validate_state_code('CA') is True
        assert validate_state_code('ca') is True
        assert validate_state_code('DC') is True

    def test_validate_state_code_invalid(self):
        """validate_state_code should raise ValueError for invalid states."""
        with pytest.raises(ValueError, match="Invalid state code"):
            validate_state_code('XX')
        with pytest.raises(ValueError, match="Must be one of"):
            validate_state_code('ZZ')

    def test_validate_year_valid(self):
        """validate_year should return int for valid years."""
        assert validate_year(2020) == 2020
        assert validate_year('2020') == 2020
        assert validate_year(2010) == 2010

    def test_validate_year_invalid(self):
        """validate_year should raise ValueError for invalid years."""
        with pytest.raises(ValueError, match="Invalid year"):
            validate_year(1990)
        with pytest.raises(ValueError):
            validate_year(2025)

    def test_validate_year_custom_list(self):
        """validate_year should accept custom year list."""
        assert validate_year(2015, available_years=[2010, 2015, 2020]) == 2015

        with pytest.raises(ValueError):
            validate_year(2000, available_years=[2010, 2020])


class TestIntegration:
    """Integration tests combining multiple functions."""

    def test_roundtrip_state_fips(self):
        """Converting state -> FIPS -> state should return original."""
        for state in ALL_STATES:
            fips = get_fips_2digit(state)
            assert get_state_from_fips(fips) == state

    def test_all_states_have_tiger_urls(self):
        """All states should have valid TIGER URLs for 2020."""
        for state in ALL_STATES:
            url_tract = get_tiger_tract_url(state, 2020)
            assert url_tract.startswith('https://')

            url_place = get_tiger_place_url(state, 2020)
            assert url_place.startswith('https://')

    def test_all_years_have_census_configs(self):
        """All supported years should have census configs."""
        for year in [2000, 2010, 2020]:
            config = get_census_config(year)
            assert 'api_base' in config
            assert 'variables' in config
