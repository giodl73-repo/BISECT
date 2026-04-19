"""
Argument parsing utilities.

This module provides helper functions to add common argument groups
to argparse parsers, reducing duplication across pipeline scripts.
"""

import argparse


def add_common_pipeline_args(parser):
    """
    Add common pipeline arguments to argparse parser.

    Adds standard arguments used across most pipeline scripts:
    - --year: Census year
    - --dpi: Output map resolution
    - --print-only: Dry run mode
    - --debug: Debug mode
    - --position: Progress bar position

    Args:
        parser: argparse.ArgumentParser instance

    Returns:
        parser: The same parser with arguments added (for chaining)

    Example:
        >>> parser = argparse.ArgumentParser()
        >>> add_common_pipeline_args(parser)
        >>> args = parser.parse_args(['--year', '2020'])
    """
    parser.add_argument('--year', type=str, default='2020',
                       choices=['2000', '2010', '2020'],
                       help='Census year')
    parser.add_argument('--dpi', type=int, default=150,
                       help='DPI for output maps (default: 150)')
    parser.add_argument('--print-only', action='store_true',
                       help='Print what would be done without doing it')
    parser.add_argument('--debug', action='store_true',
                       help='Enable debug mode with detailed output')
    parser.add_argument('--position', type=int, default=1,
                       help='Progress bar position (for parallel execution)')
    return parser


def add_common_state_args(parser):
    """
    Add common state-related arguments to argparse parser.

    Adds arguments for state identification and versioning:
    - --state: State code or name
    - --version: Version identifier

    Args:
        parser: argparse.ArgumentParser instance

    Returns:
        parser: The same parser with arguments added (for chaining)

    Example:
        >>> parser = argparse.ArgumentParser()
        >>> add_common_state_args(parser)
        >>> args = parser.parse_args(['--state', 'CA', '--version', 'v1'])
    """
    parser.add_argument('--state', type=str,
                       help='State code (e.g., CA) or name (e.g., California)')
    parser.add_argument('--version', type=str, required=True,
                       help='Version identifier for this run (e.g., v1, edge_weighted)')
    return parser


def add_common_visualization_args(parser):
    """
    Add common visualization arguments to argparse parser.

    Adds arguments for visualization control:
    - --scope: State vs national visualization
    - --skip-rounds: Skip round progression maps
    - --force: Force regeneration of existing outputs

    Args:
        parser: argparse.ArgumentParser instance

    Returns:
        parser: The same parser with arguments added (for chaining)

    Example:
        >>> parser = argparse.ArgumentParser()
        >>> add_common_visualization_args(parser)
        >>> args = parser.parse_args(['--scope', 'national'])
    """
    parser.add_argument('--scope', type=str, default='state',
                       choices=['state', 'national'],
                       help='Scope of visualization (state or national)')
    parser.add_argument('--skip-rounds', action='store_true',
                       help='Skip round progression maps')
    parser.add_argument('--force', action='store_true',
                       help='Force regeneration of existing outputs')
    return parser


def add_common_analysis_args(parser):
    """
    Add common analysis arguments to argparse parser.

    Adds arguments for analysis control:
    - --census-year: Census year for data
    - --election-year: Election year for political analysis
    - --skip-political: Skip political analysis
    - --skip-demographic: Skip demographic analysis

    Args:
        parser: argparse.ArgumentParser instance

    Returns:
        parser: The same parser with arguments added (for chaining)

    Example:
        >>> parser = argparse.ArgumentParser()
        >>> add_common_analysis_args(parser)
        >>> args = parser.parse_args(['--census-year', '2020'])
    """
    parser.add_argument('--census-year', type=str,
                       choices=['2000', '2010', '2020'],
                       help='Census year for demographic data')
    parser.add_argument('--election-year', type=str,
                       help='Election year for political analysis (e.g., 2020)')
    parser.add_argument('--skip-political', action='store_true',
                       help='Skip political analysis')
    parser.add_argument('--skip-demographic', action='store_true',
                       help='Skip demographic analysis')
    return parser
