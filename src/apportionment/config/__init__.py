"""
Configuration management for redistricting runs.

This module provides configuration tracking for experimental variants,
allowing reproducible runs with full parameter documentation.
"""

from .run_config import RunConfig, write_config, read_config, validate_config

__all__ = ['RunConfig', 'write_config', 'read_config', 'validate_config']
