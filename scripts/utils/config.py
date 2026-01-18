"""
Configuration loading utilities.

This module provides helper functions to load state configuration
for different census years, eliminating duplicated if/elif chains
across the codebase.
"""


def get_state_config(year):
    """
    Get state configuration for census year.

    Dynamically imports the appropriate state configuration module
    based on the census year.

    Args:
        year: Census year as string ('2000', '2010', '2020')

    Returns:
        dict: State configuration dictionary mapping state codes to config

    Raises:
        ValueError: If year is not supported
        ImportError: If config module cannot be imported

    Example:
        >>> config = get_state_config('2020')
        >>> california = config['CA']
        >>> print(california['name'])  # 'California'
        >>> print(california['districts'])  # 52
    """
    configs = {
        '2000': ('scripts.config_2000', 'STATE_CONFIG_2000'),
        '2010': ('scripts.config_2010', 'STATE_CONFIG_2010'),
        '2020': ('scripts.config_2020', 'STATE_CONFIG_2020'),
    }

    if year not in configs:
        raise ValueError(f"Unsupported year: {year}. Must be one of: {', '.join(configs.keys())}")

    module_path, config_name = configs[year]

    try:
        module = __import__(module_path, fromlist=[config_name])
        return getattr(module, config_name)
    except ImportError as e:
        raise ImportError(f"Could not import {config_name} from {module_path}: {e}")


def get_state_config_safe(year):
    """
    Get state configuration for census year, returning None on error.

    Safe version of get_state_config that returns None instead of raising
    exceptions. Useful for optional config loading.

    Args:
        year: Census year as string ('2000', '2010', '2020')

    Returns:
        dict or None: State configuration dictionary, or None if unavailable

    Example:
        >>> config = get_state_config_safe('2020')
        >>> if config:
        ...     process_states(config)
        ... else:
        ...     print("Config not available")
    """
    try:
        return get_state_config(year)
    except (ValueError, ImportError):
        return None


def build_state_name_to_districts_map(year):
    """
    Build a mapping from lowercase state names to district counts.

    This function derives district counts from the year-specific STATE_CONFIG
    dictionaries, eliminating the need for hardcoded DISTRICTS_PER_STATE constants.

    Args:
        year: Census year as string ('2000', '2010', '2020')

    Returns:
        dict: Mapping of lowercase state names (with underscores) to district counts
              Example: {'alabama': 7, 'california': 52, 'new_york': 26, ...}

    Raises:
        ValueError: If year is not supported
        ImportError: If config module cannot be imported

    Example:
        >>> districts = build_state_name_to_districts_map('2020')
        >>> print(districts['california'])  # 52
        >>> print(districts['new_york'])     # 26
    """
    from scripts.constants import STATE_ABBREV

    # Get the year-specific STATE_CONFIG
    state_config = get_state_config(year)

    # Build the mapping: lowercase_name -> districts
    name_to_districts = {}
    for lowercase_name, code in STATE_ABBREV.items():
        if code in state_config:
            name_to_districts[lowercase_name] = state_config[code]['districts']

    return name_to_districts
