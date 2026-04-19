"""
Unit tests for scripts.utils.config module.
"""

import pytest
from scripts.utils.config import get_state_config, get_state_config_safe


class TestGetStateConfig:
    """Test get_state_config function."""

    def test_get_state_config_2020(self):
        """Test loading 2020 state configuration."""
        config = get_state_config('2020')

        assert config is not None
        assert isinstance(config, dict)
        assert len(config) == 50  # 50 states

        # Verify sample states
        assert 'CA' in config
        assert 'NY' in config
        assert 'TX' in config

        # Verify structure
        california = config['CA']
        assert 'name' in california
        assert 'districts' in california
        assert california['name'] == 'California'
        assert california['districts'] == 52  # 2020 apportionment

    def test_get_state_config_2010(self):
        """Test loading 2010 state configuration."""
        config = get_state_config('2010')

        assert config is not None
        assert isinstance(config, dict)
        assert len(config) == 50

        california = config['CA']
        assert california['districts'] == 53  # 2010 apportionment

    def test_get_state_config_2000(self):
        """Test loading 2000 state configuration."""
        config = get_state_config('2000')

        assert config is not None
        assert isinstance(config, dict)
        assert len(config) == 50

        california = config['CA']
        assert california['districts'] == 53  # 2000 apportionment

    def test_get_state_config_invalid_year(self):
        """Test that invalid year raises ValueError."""
        with pytest.raises(ValueError) as exc_info:
            get_state_config('2030')

        assert 'Unsupported year' in str(exc_info.value)
        assert '2030' in str(exc_info.value)

    def test_get_state_config_cache_consistency(self):
        """Test that multiple calls return consistent data."""
        config1 = get_state_config('2020')
        config2 = get_state_config('2020')

        # Should return same configuration
        assert config1['CA']['districts'] == config2['CA']['districts']

    def test_all_states_have_required_fields(self):
        """Test that all states have required fields."""
        config = get_state_config('2020')

        for state_code, state_data in config.items():
            assert 'name' in state_data, f"{state_code} missing 'name'"
            assert 'districts' in state_data, f"{state_code} missing 'districts'"
            assert isinstance(state_data['name'], str)
            assert isinstance(state_data['districts'], int)
            assert state_data['districts'] >= 1


class TestGetStateConfigSafe:
    """Test get_state_config_safe function."""

    def test_get_state_config_safe_valid_year(self):
        """Test safe loading with valid year."""
        config = get_state_config_safe('2020')

        assert config is not None
        assert isinstance(config, dict)
        assert len(config) == 50

    def test_get_state_config_safe_invalid_year(self):
        """Test safe loading with invalid year returns None."""
        config = get_state_config_safe('2030')

        assert config is None

    def test_get_state_config_safe_all_years(self):
        """Test safe loading works for all valid years."""
        for year in ['2000', '2010', '2020']:
            config = get_state_config_safe(year)
            assert config is not None
            assert len(config) == 50
