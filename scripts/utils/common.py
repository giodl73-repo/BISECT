"""
Common utility functions.

This module provides miscellaneous helper functions used across
the redistricting pipeline, including skip logic, progress reporting,
and file existence checks.
"""

import os
from pathlib import Path
from .status_protocol import StatusReporter


def should_skip_existing(output_file, force=False):
    """
    Check if output should be skipped (exists and not forcing).

    Args:
        output_file: Path to output file (Path or str)
        force: If True, don't skip even if file exists

    Returns:
        bool: True if should skip, False if should process

    Example:
        >>> output = Path('outputs/v1/2020/states/california/map.png')
        >>> if should_skip_existing(output, force=False):
        ...     print("[SKIP] File exists")
        ...     return
    """
    if isinstance(output_file, str):
        output_file = Path(output_file)

    return output_file.exists() and not force


def report_progress(msg, position=None):
    """
    Report progress using STATUS protocol or print.

    Reports progress differently based on context:
    - If position is set (or TQDM_POSITION env var), use STATUS protocol
    - Otherwise, print directly

    Args:
        msg: Progress message
        position: Progress bar position (optional, uses TQDM_POSITION if None)

    Example:
        >>> report_progress("Processing California")
        Processing California

        >>> # In subprocess with TQDM_POSITION=2
        >>> report_progress("Processing California")
        STATUS:2:Processing California
    """
    # Use unified StatusReporter (with explicit position if provided)
    reporter = StatusReporter(position=position)
    reporter.report(msg)


def report_skip(reason, position=None):
    """
    Report skip with consistent formatting.

    Args:
        reason: Reason for skipping
        position: Progress bar position (optional, uses TQDM_POSITION if None)

    Example:
        >>> report_skip("Output already exists")
        [SKIP] Output already exists
    """
    report_progress(f"[SKIP] {reason}", position)


def report_success(msg, position=None):
    """
    Report success with consistent formatting.

    Args:
        msg: Success message
        position: Progress bar position (optional, uses TQDM_POSITION if None)

    Example:
        >>> report_success("Redistricting complete")
        [OK] Redistricting complete
    """
    report_progress(f"[OK] {msg}", position)


def report_failure(msg, position=None):
    """
    Report failure with consistent formatting.

    Args:
        msg: Failure message
        position: Progress bar position (optional, uses TQDM_POSITION if None)

    Example:
        >>> report_failure("Missing tract data")
        [FAIL] Missing tract data
    """
    report_progress(f"[FAIL] {msg}", position)


def normalize_state_name(state):
    """
    Normalize state name to lowercase with underscores.

    Args:
        state: State name or code (any case, spaces or underscores)

    Returns:
        str: Normalized state name (lowercase with underscores)

    Example:
        >>> normalize_state_name('California')
        'california'
        >>> normalize_state_name('NEW YORK')
        'new_york'
        >>> normalize_state_name('North Dakota')
        'north_dakota'
    """
    return state.lower().replace(' ', '_')


def check_data_availability(state, year):
    """
    Check if required data files exist for state and year.

    Args:
        state: State name (case-insensitive)
        year: Census year as string ('2000', '2010', '2020')

    Returns:
        dict: Dictionary with keys 'tracts', 'adjacency', 'places'
              Values are True if file exists, False otherwise

    Example:
        >>> avail = check_data_availability('California', '2020')
        >>> if not avail['tracts']:
        ...     print("ERROR: Missing tract data")
    """
    from .paths import get_tract_file, get_adjacency_file, get_places_file

    state_normalized = normalize_state_name(state)

    return {
        'tracts': get_tract_file(state_normalized, year).exists(),
        'adjacency': get_adjacency_file(state_normalized, year).exists(),
        'places': get_places_file(state_normalized, year).exists()
    }
