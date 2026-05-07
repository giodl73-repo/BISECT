"""
Cleanup utilities for test file management.

Provides functions to clean up test artifacts and temporary files.
"""

import shutil
from pathlib import Path


def cleanup_test_output(output_dir, keep_successful=False):
    """
    Clean up test output directory.

    Parameters
    ----------
    output_dir : Path or str
        Output directory to clean
    keep_successful : bool
        If True, keep outputs from successful tests

    Examples
    --------
    >>> from pathlib import Path
    >>> import tempfile
    >>> with tempfile.TemporaryDirectory() as tmpdir:
    ...     output_dir = Path(tmpdir) / 'output'
    ...     output_dir.mkdir()
    ...     cleanup_test_output(output_dir)
    """
    output_dir = Path(output_dir)

    if not output_dir.exists():
        return

    if keep_successful:
        # Only remove files marked as failures
        for file in output_dir.rglob('*_FAILED_*'):
            if file.is_file():
                file.unlink()
            elif file.is_dir():
                shutil.rmtree(file)
    else:
        # Remove entire directory
        shutil.rmtree(output_dir)


def cleanup_temporary_files(base_dir, patterns=None):
    """
    Clean up temporary files matching patterns.

    Parameters
    ----------
    base_dir : Path or str
        Base directory to search
    patterns : list of str, optional
        File patterns to remove (e.g., ['*.tmp', '*.temp'])
        Default: ['*.tmp', '*.temp', '*.cache']

    Examples
    --------
    >>> from pathlib import Path
    >>> import tempfile
    >>> with tempfile.TemporaryDirectory() as tmpdir:
    ...     base_dir = Path(tmpdir)
    ...     temp_file = base_dir / 'test.tmp'
    ...     temp_file.touch()
    ...     cleanup_temporary_files(base_dir, patterns=['*.tmp'])
    ...     not temp_file.exists()
    True
    """
    base_dir = Path(base_dir)

    if patterns is None:
        patterns = ['*.tmp', '*.temp', '*.cache']

    if not base_dir.exists():
        return

    for pattern in patterns:
        for file in base_dir.rglob(pattern):
            if file.is_file():
                file.unlink()


def cleanup_empty_directories(base_dir):
    """
    Remove empty directories recursively.

    Parameters
    ----------
    base_dir : Path or str
        Base directory to search

    Examples
    --------
    >>> from pathlib import Path
    >>> import tempfile
    >>> with tempfile.TemporaryDirectory() as tmpdir:
    ...     base_dir = Path(tmpdir)
    ...     empty_dir = base_dir / 'empty'
    ...     empty_dir.mkdir()
    ...     cleanup_empty_directories(base_dir)
    ...     not empty_dir.exists()
    True
    """
    base_dir = Path(base_dir)

    if not base_dir.exists():
        return

    # Walk bottom-up to remove nested empty directories
    for dirpath in sorted(base_dir.rglob('*'), reverse=True):
        if dirpath.is_dir() and not any(dirpath.iterdir()):
            dirpath.rmdir()


def cleanup_old_test_runs(output_base_dir, keep_recent=3):
    """
    Clean up old test run directories, keeping only recent ones.

    Parameters
    ----------
    output_base_dir : Path or str
        Base output directory containing test runs
    keep_recent : int
        Number of recent test runs to keep

    Examples
    --------
    >>> from pathlib import Path
    >>> import tempfile
    >>> with tempfile.TemporaryDirectory() as tmpdir:
    ...     base_dir = Path(tmpdir)
    ...     for i in range(5):
    ...         (base_dir / f'test_run_{i}').mkdir()
    ...     cleanup_old_test_runs(base_dir, keep_recent=2)
    ...     len(list(base_dir.iterdir()))
    2
    """
    output_base_dir = Path(output_base_dir)

    if not output_base_dir.exists():
        return

    # Get all test run directories
    test_runs = []
    for item in output_base_dir.iterdir():
        if item.is_dir():
            test_runs.append((item, item.stat().st_mtime))

    # Sort by modification time (newest first)
    test_runs.sort(key=lambda x: x[1], reverse=True)

    # Remove old runs
    for test_run, _ in test_runs[keep_recent:]:
        shutil.rmtree(test_run)


def cleanup_by_age(base_dir, max_age_days=7):
    """
    Remove files older than specified age.

    Parameters
    ----------
    base_dir : Path or str
        Base directory to search
    max_age_days : int
        Maximum age in days

    Examples
    --------
    >>> from pathlib import Path
    >>> import tempfile
    >>> import time
    >>> with tempfile.TemporaryDirectory() as tmpdir:
    ...     base_dir = Path(tmpdir)
    ...     old_file = base_dir / 'old.txt'
    ...     old_file.touch()
    ...     # File is new, so won't be removed
    ...     cleanup_by_age(base_dir, max_age_days=7)
    ...     old_file.exists()
    True
    """
    import time

    base_dir = Path(base_dir)

    if not base_dir.exists():
        return

    current_time = time.time()
    max_age_seconds = max_age_days * 24 * 60 * 60

    for file in base_dir.rglob('*'):
        if file.is_file():
            file_age = current_time - file.stat().st_mtime
            if file_age > max_age_seconds:
                file.unlink()


def cleanup_large_files(base_dir, max_size_mb=100):
    """
    Remove files larger than specified size.

    Parameters
    ----------
    base_dir : Path or str
        Base directory to search
    max_size_mb : int
        Maximum file size in megabytes

    Examples
    --------
    >>> from pathlib import Path
    >>> import tempfile
    >>> with tempfile.TemporaryDirectory() as tmpdir:
    ...     base_dir = Path(tmpdir)
    ...     small_file = base_dir / 'small.txt'
    ...     small_file.write_text('small')
    ...     cleanup_large_files(base_dir, max_size_mb=1)
    ...     small_file.exists()
    5
    True
    """
    base_dir = Path(base_dir)

    if not base_dir.exists():
        return

    max_size_bytes = max_size_mb * 1024 * 1024

    for file in base_dir.rglob('*'):
        if file.is_file():
            if file.stat().st_size > max_size_bytes:
                file.unlink()


def reset_test_directory(test_dir):
    """
    Reset test directory to clean state.

    Removes all contents but keeps directory structure.

    Parameters
    ----------
    test_dir : Path or str
        Test directory to reset

    Examples
    --------
    >>> from pathlib import Path
    >>> import tempfile
    >>> with tempfile.TemporaryDirectory() as tmpdir:
    ...     test_dir = Path(tmpdir) / 'test'
    ...     test_dir.mkdir()
    ...     (test_dir / 'file.txt').touch()
    ...     reset_test_directory(test_dir)
    ...     test_dir.exists() and not list(test_dir.iterdir())
    True
    """
    test_dir = Path(test_dir)

    if not test_dir.exists():
        return

    # Remove all files and subdirectories
    for item in test_dir.iterdir():
        if item.is_file():
            item.unlink()
        elif item.is_dir():
            shutil.rmtree(item)


def get_directory_size(directory):
    """
    Get total size of directory in bytes.

    Parameters
    ----------
    directory : Path or str
        Directory to measure

    Returns
    -------
    int
        Total size in bytes

    Examples
    --------
    >>> from pathlib import Path
    >>> import tempfile
    >>> with tempfile.TemporaryDirectory() as tmpdir:
    ...     test_dir = Path(tmpdir)
    ...     (test_dir / 'file.txt').write_text('hello')
    ...     size = get_directory_size(test_dir)
    ...     size > 0
    5
    True
    """
    directory = Path(directory)

    if not directory.exists():
        return 0

    total_size = 0
    for file in directory.rglob('*'):
        if file.is_file():
            total_size += file.stat().st_size

    return total_size


def cleanup_all(test_output_dir, temp_dir=None):
    """
    Comprehensive cleanup of all test artifacts.

    Parameters
    ----------
    test_output_dir : Path or str
        Test output directory
    temp_dir : Path or str, optional
        Temporary directory

    Examples
    --------
    >>> from pathlib import Path
    >>> import tempfile
    >>> with tempfile.TemporaryDirectory() as tmpdir:
    ...     test_output = Path(tmpdir) / 'output'
    ...     test_output.mkdir()
    ...     cleanup_all(test_output)
    ...     not test_output.exists()
    True
    """
    # Clean up test output
    cleanup_test_output(test_output_dir, keep_successful=False)

    # Clean up temporary files
    if temp_dir:
        cleanup_temporary_files(temp_dir)
        cleanup_empty_directories(temp_dir)
