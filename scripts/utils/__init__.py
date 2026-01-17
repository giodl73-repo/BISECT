"""
Pipeline utility functions.

This package provides common utilities used across the redistricting pipeline,
eliminating code duplication and ensuring consistency.

Modules:
    config: State configuration loading
    paths: Path construction helpers
    args: Argument parsing utilities
    common: Skip logic, progress reporting, common helpers
    subprocess_helpers: Command building for subprocess calls

Example:
    >>> from scripts.utils import get_state_config, get_tract_file
    >>> config = get_state_config('2020')
    >>> tract_file = get_tract_file('california', '2020')
"""

# Config utilities
from .config import (
    get_state_config,
    get_state_config_safe,
)

# Path utilities
from .paths import (
    get_tract_file,
    get_places_file,
    get_adjacency_file,
    get_output_dir,
    get_state_output_dir,
    get_election_data_file,
    get_demographic_data_file,
)

# Argument parsing utilities
from .args import (
    add_common_pipeline_args,
    add_common_state_args,
    add_common_visualization_args,
    add_common_analysis_args,
)

# Common utilities
from .common import (
    should_skip_existing,
    report_progress,
    report_skip,
    report_success,
    report_failure,
    normalize_state_name,
    check_data_availability,
)

# Subprocess helpers
from .subprocess_helpers import (
    build_pipeline_command,
    build_command_string,
    add_common_flags_to_command,
    get_python_executable,
)

__all__ = [
    # Config
    'get_state_config',
    'get_state_config_safe',

    # Paths
    'get_tract_file',
    'get_places_file',
    'get_adjacency_file',
    'get_output_dir',
    'get_state_output_dir',
    'get_election_data_file',
    'get_demographic_data_file',

    # Args
    'add_common_pipeline_args',
    'add_common_state_args',
    'add_common_visualization_args',
    'add_common_analysis_args',

    # Common
    'should_skip_existing',
    'report_progress',
    'report_skip',
    'report_success',
    'report_failure',
    'normalize_state_name',
    'check_data_availability',

    # Subprocess
    'build_pipeline_command',
    'build_command_string',
    'add_common_flags_to_command',
    'get_python_executable',
]
