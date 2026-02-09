"""
Huntington-Hill Apportionment Module

Implements the Equal Proportions method used by the U.S. Census Bureau
to allocate congressional seats to states.

Generic implementation works with any list of entities (states, counties,
international regions, etc.) with populations.
"""

from .algorithm import (
    apportion,
    priority_value,
    validate_census_apportionment
)

__all__ = [
    'apportion',
    'priority_value',
    'validate_census_apportionment'
]
