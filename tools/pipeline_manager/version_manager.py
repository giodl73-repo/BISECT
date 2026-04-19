"""
Version Manager - Scan and manage pipeline output versions.

Provides functions to:
- List all versions in outputs/ directory
- Get version metadata (size, date, completion status)
- Delete versions
"""

import os
import shutil
from pathlib import Path
from datetime import datetime
from typing import List, Dict, Any


def get_outputs_dir() -> Path:
    """Get the outputs directory path."""
    return Path(__file__).parent.parent.parent / 'outputs'


def get_version_size(version_path: Path) -> tuple[int, str]:
    """
    Calculate total size of version directory.

    Args:
        version_path: Path to version directory

    Returns:
        Tuple of (size_bytes, size_human)
    """
    total_size = 0
    for dirpath, dirnames, filenames in os.walk(version_path):
        for f in filenames:
            fp = Path(dirpath) / f
            if fp.exists():
                total_size += fp.stat().st_size

    # Convert to human-readable
    for unit in ['B', 'KB', 'MB', 'GB', 'TB']:
        if total_size < 1024.0:
            return (total_size, f"{total_size:.1f} {unit}")
        total_size /= 1024.0
    return (total_size * 1024**5, f"{total_size:.1f} PB")


def get_version_years(version_path: Path) -> List[str]:
    """
    Get list of years present in version directory.

    Args:
        version_path: Path to version directory

    Returns:
        List of year strings (e.g., ['2020', '2010', '2000'])
    """
    years = []
    for year in ['2020', '2010', '2000']:
        year_dir = version_path / year
        if year_dir.exists() and year_dir.is_dir():
            years.append(year)
    return years


def get_version_completion_status(version_path: Path, years: List[str]) -> Dict[str, int]:
    """
    Check completion status for each year.

    Args:
        version_path: Path to version directory
        years: List of years to check

    Returns:
        Dict of {year: states_complete}
    """
    status = {}
    for year in years:
        marker_file = version_path / year / '.states_complete'
        if marker_file.exists():
            # Count lines in marker file (each line is a completed state)
            try:
                with open(marker_file, 'r') as f:
                    completed = len([line for line in f if line.strip()])
                status[year] = completed
            except Exception:
                status[year] = 0
        else:
            status[year] = 0
    return status


def has_dashboard(version_path: Path, years: List[str]) -> bool:
    """
    Check if version has generated dashboard.

    Args:
        version_path: Path to version directory
        years: List of years present

    Returns:
        True if dashboard exists for any year
    """
    for year in years:
        dashboard_file = version_path / year / 'index.html'
        if dashboard_file.exists():
            return True
    return False


def list_versions() -> List[Dict[str, Any]]:
    """
    List all versions in outputs/ directory with metadata.

    Returns:
        List of version dicts with metadata
    """
    outputs_dir = get_outputs_dir()
    if not outputs_dir.exists():
        return []

    versions = []
    for item in outputs_dir.iterdir():
        if not item.is_dir():
            continue

        # Skip special directories
        if item.name in ['data', 'artifacts']:
            continue

        # Get metadata
        years = get_version_years(item)
        if not years:
            # Not a valid version directory (no year subdirs)
            continue

        size_bytes, size_human = get_version_size(item)
        completion_status = get_version_completion_status(item, years)

        # Overall status
        all_complete = all(count >= 50 for count in completion_status.values())
        status = 'complete' if all_complete else 'partial'

        versions.append({
            'name': item.name,
            'path': str(item),
            'created': datetime.fromtimestamp(item.stat().st_mtime).isoformat(),
            'size_bytes': size_bytes,
            'size_human': size_human,
            'years': years,
            'states_complete': completion_status,
            'status': status,
            'has_dashboard': has_dashboard(item, years),
            'dashboard_url': f'/outputs/{item.name}/{years[0]}/index.html' if years else None
        })

    # Sort by creation date (newest first)
    versions.sort(key=lambda x: x['created'], reverse=True)

    return versions


def delete_version(version_name: str) -> tuple[bool, str]:
    """
    Delete a version directory.

    Args:
        version_name: Name of version to delete

    Returns:
        Tuple of (success, message)
    """
    outputs_dir = get_outputs_dir()
    version_path = outputs_dir / version_name

    if not version_path.exists():
        return (False, f"Version '{version_name}' not found")

    if not version_path.is_dir():
        return (False, f"'{version_name}' is not a directory")

    # Safety check - don't delete data or artifacts
    if version_name in ['data', 'artifacts']:
        return (False, f"Cannot delete special directory '{version_name}'")

    try:
        shutil.rmtree(version_path)
        return (True, f"Version '{version_name}' deleted successfully")
    except Exception as e:
        return (False, f"Failed to delete version: {str(e)}")


def get_error_log(version_name: str, year: str) -> tuple[bool, str]:
    """
    Get error log content for a version/year.

    Args:
        version_name: Version name
        year: Year (2020, 2010, 2000)

    Returns:
        Tuple of (success, log_content_or_error)
    """
    outputs_dir = get_outputs_dir()
    log_file = outputs_dir / version_name / year / 'error.log'

    if not log_file.exists():
        return (False, f"No error log found for {version_name}/{year}")

    try:
        with open(log_file, 'r', encoding='utf-8') as f:
            content = f.read()
        return (True, content)
    except Exception as e:
        return (False, f"Failed to read error log: {str(e)}")
