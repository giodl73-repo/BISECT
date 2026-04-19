"""
Unit tests for scripts.utils.common module.
"""

import pytest
import os
import tempfile
from pathlib import Path
from scripts.utils.common import (
    should_skip_existing,
    report_progress,
    report_skip,
    report_success,
    report_failure,
    normalize_state_name,
    check_data_availability,
)


class TestShouldSkipExisting:
    """Test should_skip_existing function."""

    def test_file_exists_no_force(self):
        """Test that existing file with force=False returns True."""
        with tempfile.NamedTemporaryFile(delete=False) as tmp:
            tmp_path = Path(tmp.name)

        try:
            result = should_skip_existing(tmp_path, force=False)
            assert result is True
        finally:
            tmp_path.unlink()

    def test_file_exists_with_force(self):
        """Test that existing file with force=True returns False."""
        with tempfile.NamedTemporaryFile(delete=False) as tmp:
            tmp_path = Path(tmp.name)

        try:
            result = should_skip_existing(tmp_path, force=True)
            assert result is False
        finally:
            tmp_path.unlink()

    def test_file_not_exists_no_force(self):
        """Test that non-existent file returns False."""
        nonexistent = Path('/nonexistent/file.txt')
        result = should_skip_existing(nonexistent, force=False)
        assert result is False

    def test_file_not_exists_with_force(self):
        """Test that non-existent file with force returns False."""
        nonexistent = Path('/nonexistent/file.txt')
        result = should_skip_existing(nonexistent, force=True)
        assert result is False

    def test_accepts_string_path(self):
        """Test that string paths are accepted."""
        with tempfile.NamedTemporaryFile(delete=False) as tmp:
            tmp_path_str = tmp.name

        try:
            result = should_skip_existing(tmp_path_str, force=False)
            assert result is True
        finally:
            Path(tmp_path_str).unlink()


class TestReportProgress:
    """Test report_progress function."""

    def test_report_progress_without_position(self, capsys):
        """Test progress reporting without position."""
        report_progress("Test message")

        captured = capsys.readouterr()
        assert "Test message" in captured.out
        assert "STATUS:" not in captured.out

    def test_report_progress_with_position(self, capsys):
        """Test progress reporting with position."""
        report_progress("Test message", position=2)

        captured = capsys.readouterr()
        assert "STATUS:2:Test message" in captured.out

    def test_report_progress_reads_env(self, capsys, monkeypatch):
        """Test that position is read from TQDM_POSITION env var."""
        monkeypatch.setenv('TQDM_POSITION', '3')
        report_progress("Test message")

        captured = capsys.readouterr()
        assert "STATUS:3:Test message" in captured.out

    def test_report_progress_explicit_position_overrides_env(self, capsys, monkeypatch):
        """Test that explicit position overrides env var."""
        monkeypatch.setenv('TQDM_POSITION', '3')
        report_progress("Test message", position=5)

        captured = capsys.readouterr()
        assert "STATUS:5:Test message" in captured.out


class TestReportSkip:
    """Test report_skip function."""

    def test_report_skip_without_position(self, capsys):
        """Test skip reporting without position."""
        report_skip("File already exists")

        captured = capsys.readouterr()
        assert "[SKIP]" in captured.out
        assert "File already exists" in captured.out

    def test_report_skip_with_position(self, capsys):
        """Test skip reporting with position."""
        report_skip("File already exists", position=2)

        captured = capsys.readouterr()
        assert "STATUS:2:[SKIP] File already exists" in captured.out


class TestReportSuccess:
    """Test report_success function."""

    def test_report_success_without_position(self, capsys):
        """Test success reporting without position."""
        report_success("Task completed")

        captured = capsys.readouterr()
        assert "[OK]" in captured.out
        assert "Task completed" in captured.out

    def test_report_success_with_position(self, capsys):
        """Test success reporting with position."""
        report_success("Task completed", position=2)

        captured = capsys.readouterr()
        assert "STATUS:2:[OK] Task completed" in captured.out


class TestReportFailure:
    """Test report_failure function."""

    def test_report_failure_without_position(self, capsys):
        """Test failure reporting without position."""
        report_failure("Task failed")

        captured = capsys.readouterr()
        assert "[FAIL]" in captured.out
        assert "Task failed" in captured.out

    def test_report_failure_with_position(self, capsys):
        """Test failure reporting with position."""
        report_failure("Task failed", position=2)

        captured = capsys.readouterr()
        assert "STATUS:2:[FAIL] Task failed" in captured.out


class TestNormalizeStateName:
    """Test normalize_state_name function."""

    def test_normalize_lowercase(self):
        """Test that uppercase is converted to lowercase."""
        assert normalize_state_name('CALIFORNIA') == 'california'
        assert normalize_state_name('California') == 'california'
        assert normalize_state_name('california') == 'california'

    def test_normalize_spaces_to_underscores(self):
        """Test that spaces are converted to underscores."""
        assert normalize_state_name('New York') == 'new_york'
        assert normalize_state_name('North Dakota') == 'north_dakota'
        assert normalize_state_name('NEW MEXICO') == 'new_mexico'

    def test_normalize_already_normalized(self):
        """Test that already normalized names are unchanged."""
        assert normalize_state_name('new_york') == 'new_york'
        assert normalize_state_name('california') == 'california'

    def test_normalize_mixed_case_and_spaces(self):
        """Test normalization with mixed case and spaces."""
        assert normalize_state_name('North CAROLINA') == 'north_carolina'
        assert normalize_state_name('West Virginia') == 'west_virginia'

    def test_normalize_idempotent(self):
        """Test that normalizing twice gives same result."""
        state = 'New York'
        once = normalize_state_name(state)
        twice = normalize_state_name(once)
        assert once == twice


class TestCheckDataAvailability:
    """Test check_data_availability function."""

    def test_returns_dict_with_keys(self):
        """Test that function returns dictionary with expected keys."""
        result = check_data_availability('california', '2020')

        assert isinstance(result, dict)
        assert 'tracts' in result
        assert 'adjacency' in result
        assert 'places' in result

    def test_returns_boolean_values(self):
        """Test that dictionary values are booleans."""
        result = check_data_availability('california', '2020')

        assert isinstance(result['tracts'], bool)
        assert isinstance(result['adjacency'], bool)
        assert isinstance(result['places'], bool)

    def test_normalizes_state_name(self):
        """Test that state name is normalized before checking."""
        result1 = check_data_availability('California', '2020')
        result2 = check_data_availability('CALIFORNIA', '2020')
        result3 = check_data_availability('california', '2020')

        # All should give same result (checking same files)
        assert result1 == result2 == result3

    def test_nonexistent_state_returns_false(self):
        """Test that non-existent state files return False."""
        result = check_data_availability('nonexistent_state', '2020')

        assert result['tracts'] is False
        assert result['adjacency'] is False
        assert result['places'] is False

    def test_different_years(self):
        """Test data availability check for different years."""
        for year in ['2000', '2010', '2020']:
            result = check_data_availability('california', year)
            assert isinstance(result, dict)
            assert 'tracts' in result
            assert 'adjacency' in result
            assert 'places' in result


class TestProgressReportingConsistency:
    """Test consistency across progress reporting functions."""

    def test_all_report_functions_use_status_protocol(self, capsys, monkeypatch):
        """Test that all report functions use STATUS protocol when position set."""
        monkeypatch.setenv('TQDM_POSITION', '2')

        report_progress("Progress message")
        report_skip("Skip reason")
        report_success("Success message")
        report_failure("Failure message")

        captured = capsys.readouterr()
        output_lines = captured.out.strip().split('\n')

        # All should use STATUS protocol
        for line in output_lines:
            assert line.startswith('STATUS:2:')

    def test_all_report_functions_use_plain_output(self, capsys):
        """Test that all report functions use plain output without position."""
        report_progress("Progress message")
        report_skip("Skip reason")
        report_success("Success message")
        report_failure("Failure message")

        captured = capsys.readouterr()

        # None should use STATUS protocol
        assert 'STATUS:' not in captured.out
