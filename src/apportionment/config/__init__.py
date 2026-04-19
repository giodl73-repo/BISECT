"""
Configuration management for redistricting runs.

This module provides configuration tracking for experimental variants,
allowing reproducible runs with full parameter documentation.
"""

from .run_config import (
    RunConfig, write_config, read_config, validate_config,
    VersionConfig, write_version_config, read_version_config, update_version_config_with_year
)

__all__ = [
    'RunConfig', 'write_config', 'read_config', 'validate_config',
    'VersionConfig', 'write_version_config', 'read_version_config', 'update_version_config_with_year'
]
