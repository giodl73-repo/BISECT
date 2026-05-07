"""
Test utilities package.

Provides reusable helpers for testing:
- assertions: Assertion functions for common validations
- validators: Data quality validators
- cleanup: Test artifact cleanup utilities
"""

from .assertions import (
    assert_valid_csv,
    assert_valid_png,
    assert_population_balanced,
    assert_graph_connected,
    assert_shares_sum_to_one,
    assert_values_in_range,
    assert_no_duplicates,
    assert_geoid_format,
    assert_compactness_valid,
    assert_directory_structure,
)

from .validators import (
    validate_tract_data,
    validate_adjacency_graph,
    validate_district_assignments,
    validate_political_analysis,
    validate_compactness_metrics,
    validate_output_directory_structure,
)

from .cleanup import (
    cleanup_test_output,
    cleanup_temporary_files,
    cleanup_empty_directories,
    cleanup_old_test_runs,
    cleanup_by_age,
    cleanup_large_files,
    reset_test_directory,
    get_directory_size,
    cleanup_all,
)

__all__ = [
    # Assertions
    'assert_valid_csv',
    'assert_valid_png',
    'assert_population_balanced',
    'assert_graph_connected',
    'assert_shares_sum_to_one',
    'assert_values_in_range',
    'assert_no_duplicates',
    'assert_geoid_format',
    'assert_compactness_valid',
    'assert_directory_structure',
    # Validators
    'validate_tract_data',
    'validate_adjacency_graph',
    'validate_district_assignments',
    'validate_political_analysis',
    'validate_compactness_metrics',
    'validate_output_directory_structure',
    # Cleanup
    'cleanup_test_output',
    'cleanup_temporary_files',
    'cleanup_empty_directories',
    'cleanup_old_test_runs',
    'cleanup_by_age',
    'cleanup_large_files',
    'reset_test_directory',
    'get_directory_size',
    'cleanup_all',
]
