#!/usr/bin/env python3
"""
Terminal utilities for progress bar rendering and layout.

Provides functions for:
- Terminal size detection
- Unicode block character rendering (progress bars)
- Tree connector formatting
- String truncation and abbreviation
"""

import shutil
import sys


def get_terminal_size():
    """
    Get current terminal size in columns and lines.

    Returns:
        Tuple of (columns, lines)
        Default: (80, 24) if detection fails
    """
    try:
        size = shutil.get_terminal_size()
        return (size.columns, size.lines)
    except Exception:
        return (80, 24)  # Fallback default


def is_wide_terminal(min_width=120):
    """
    Check if terminal is wide enough for full display.

    Args:
        min_width: Minimum column width (default: 120)

    Returns:
        True if terminal is wide enough, False otherwise
    """
    columns, _ = get_terminal_size()
    return columns >= min_width


def create_progress_bar(completed, total, width=20):
    """
    Create a text-based progress bar using Unicode block characters.

    Args:
        completed: Number of items completed
        total: Total number of items
        width: Width of progress bar in characters (default: 20)

    Returns:
        String with progress bar like: ████████░░░░░░░░░░░░

    Examples:
        create_progress_bar(25, 50, 20) -> '██████████░░░░░░░░░░'
        create_progress_bar(50, 50, 20) -> '████████████████████'
        create_progress_bar(0, 50, 20) ->  '░░░░░░░░░░░░░░░░░░░░'
    """
    if total == 0:
        return '░' * width

    # Calculate filled portion
    filled = int((completed / total) * width)
    filled = max(0, min(filled, width))  # Clamp to [0, width]

    # Use ASCII characters for Windows compatibility
    filled_char = '#'
    empty_char = '-'

    bar = filled_char * filled + empty_char * (width - filled)
    return bar


def format_tree_connector(is_last, wide_terminal=True):
    """
    Format tree connector characters for hierarchical display.

    Args:
        is_last: True if this is the last child, False otherwise
        wide_terminal: True to use ASCII, False for minimal

    Returns:
        String with tree connector (ASCII for Windows compatibility)

    Examples:
        format_tree_connector(False, True) -> '  +- '
        format_tree_connector(True, True) ->  '  `- '
    """
    # Use ASCII for Windows compatibility
    return '  `- ' if is_last else '  +- '


def abbreviate_state_name(state_name, max_length=15):
    """
    Abbreviate state name if too long for narrow terminals.

    Args:
        state_name: Full state name (e.g., 'california', 'new_york')
        max_length: Maximum length (default: 15)

    Returns:
        Abbreviated state name

    Examples:
        abbreviate_state_name('california', 15) -> 'california'
        abbreviate_state_name('california', 5) -> 'CA'
        abbreviate_state_name('north_carolina', 10) -> 'N Carolina'
    """
    if len(state_name) <= max_length:
        return state_name

    # State name to abbreviation mapping
    state_abbrevs = {
        'alabama': 'AL', 'alaska': 'AK', 'arizona': 'AZ', 'arkansas': 'AR',
        'california': 'CA', 'colorado': 'CO', 'connecticut': 'CT', 'delaware': 'DE',
        'florida': 'FL', 'georgia': 'GA', 'hawaii': 'HI', 'idaho': 'ID',
        'illinois': 'IL', 'indiana': 'IN', 'iowa': 'IA', 'kansas': 'KS',
        'kentucky': 'KY', 'louisiana': 'LA', 'maine': 'ME', 'maryland': 'MD',
        'massachusetts': 'MA', 'michigan': 'MI', 'minnesota': 'MN', 'mississippi': 'MS',
        'missouri': 'MO', 'montana': 'MT', 'nebraska': 'NE', 'nevada': 'NV',
        'new_hampshire': 'NH', 'new_jersey': 'NJ', 'new_mexico': 'NM', 'new_york': 'NY',
        'north_carolina': 'NC', 'north_dakota': 'ND', 'ohio': 'OH', 'oklahoma': 'OK',
        'oregon': 'OR', 'pennsylvania': 'PA', 'rhode_island': 'RI', 'south_carolina': 'SC',
        'south_dakota': 'SD', 'tennessee': 'TN', 'texas': 'TX', 'utah': 'UT',
        'vermont': 'VT', 'virginia': 'VA', 'washington': 'WA', 'west_virginia': 'WV',
        'wisconsin': 'WI', 'wyoming': 'WY'
    }

    # Try 2-letter abbreviation
    abbrev = state_abbrevs.get(state_name.lower(), state_name)
    if len(abbrev) <= max_length:
        return abbrev

    # Truncate with ellipsis
    return state_name[:max_length-3] + '...'


def truncate_string(text, max_length, suffix='...'):
    """
    Truncate string if it exceeds max length.

    Args:
        text: String to truncate
        max_length: Maximum length
        suffix: Suffix to append if truncated (default: '...')

    Returns:
        Truncated string

    Examples:
        truncate_string('Political visualization', 15) -> 'Political vi...'
        truncate_string('Short', 15) -> 'Short'
    """
    if len(text) <= max_length:
        return text

    return text[:max_length - len(suffix)] + suffix


def format_stage_description(stage_desc, wide_terminal=True):
    """
    Format stage description based on terminal width.

    Args:
        stage_desc: Full stage description (e.g., 'political_visualization')
        wide_terminal: True for full text, False for abbreviated

    Returns:
        Formatted stage description

    Examples:
        format_stage_description('political_visualization', True) -> 'Political visualization'
        format_stage_description('political_visualization', False) -> 'Political'
    """
    # Replace underscores with spaces and title case
    formatted = stage_desc.replace('_', ' ').title()

    if not wide_terminal:
        # Truncate to first word
        formatted = formatted.split()[0]

    return formatted
