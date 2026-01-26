"""
Tests for configuration loading.

Tests that configuration loads correctly from environment variables.
"""
import os
import pytest


def test_config_loads_from_environment():
    """Test that configuration reads from environment variables."""
    # Set test environment variable
    os.environ["DATABASE_URL"] = "postgresql://test:test@localhost/test"

    # Import after setting env var to get updated config
    from app.config import Settings
    settings = Settings()

    assert "test" in settings.database_url
    assert settings.database_url.startswith("postgresql://test:test")


def test_config_has_default_values():
    """Test that configuration has sensible default values."""
    from app.config import Settings
    settings = Settings()

    assert settings.api_version == "v1"
    assert settings.app_version == "9.0.0"
    assert settings.default_workers == 4
    assert settings.watchdog_timeout == 60


def test_config_cors_origins_is_list():
    """Test that CORS origins is a list."""
    from app.config import Settings
    settings = Settings()

    assert isinstance(settings.cors_origins, list)
    assert len(settings.cors_origins) > 0
