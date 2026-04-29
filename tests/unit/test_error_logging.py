"""
Unit tests for error_logger.py.

The companion TestStageTracker class was removed 2026-04-29 when stage_tracker
was archived to archive/python-pipeline-final/scripts/utils/ — its only
production callers (the Python pipeline orchestrators) were retired in Plan 02.
"""

import pytest
import tempfile
from pathlib import Path
from scripts.utils.error_logger import ErrorLogger, get_error_logger


class TestErrorLogger:
    """Test ErrorLogger functionality."""

    def test_error_logger_creation(self):
        """Test that ErrorLogger can be created."""
        with tempfile.TemporaryDirectory() as tmpdir:
            output_dir = Path(tmpdir)
            logger = ErrorLogger(output_dir, 'test', 2020)

            assert logger.output_dir == output_dir
            assert logger.version == 'test'
            assert logger.year == 2020
            assert logger.log_file == output_dir / 'error.log'
            assert logger.error_count == 0
            assert logger.warning_count == 0

            logger.close()  # Close handlers for Windows cleanup

    def test_error_logger_factory(self):
        """Test get_error_logger factory function."""
        with tempfile.TemporaryDirectory() as tmpdir:
            output_dir = Path(tmpdir)
            logger = get_error_logger(output_dir, 'test', 2020)

            assert isinstance(logger, ErrorLogger)
            assert logger.version == 'test'

            logger.close()  # Close handlers for Windows cleanup

    def test_log_warning(self):
        """Test logging warnings."""
        with tempfile.TemporaryDirectory() as tmpdir:
            output_dir = Path(tmpdir)
            logger = ErrorLogger(output_dir, 'test', 2020)

            logger.log_warning('Test warning message')

            assert logger.warning_count == 1
            assert logger.log_file.exists()

            # Check log content
            log_content = logger.log_file.read_text()
            assert 'WARNING: Test warning message' in log_content

            logger.close()  # Close handlers for Windows cleanup

    def test_log_exception(self):
        """Test logging exceptions."""
        with tempfile.TemporaryDirectory() as tmpdir:
            output_dir = Path(tmpdir)
            logger = ErrorLogger(output_dir, 'test', 2020)

            try:
                raise ValueError("Test error")
            except Exception as e:
                logger.log_exception('test_task', e)

            assert logger.error_count == 1
            assert logger.log_file.exists()

            # Check log content
            log_content = logger.log_file.read_text()
            assert 'ERROR: test_task' in log_content
            assert 'ValueError' in log_content
            assert 'Test error' in log_content

            logger.close()  # Close handlers for Windows cleanup

    def test_stage_context_manager_success(self):
        """Test stage context manager with successful execution."""
        with tempfile.TemporaryDirectory() as tmpdir:
            output_dir = Path(tmpdir)
            logger = ErrorLogger(output_dir, 'test', 2020)

            with logger.stage('test_stage'):
                pass  # Successful execution

            log_content = logger.log_file.read_text()
            assert 'STAGE_START: test_stage' in log_content
            assert 'STAGE_COMPLETE: test_stage' in log_content

            logger.close()  # Close handlers for Windows cleanup

    def test_stage_context_manager_failure(self):
        """Test stage context manager with exception."""
        with tempfile.TemporaryDirectory() as tmpdir:
            output_dir = Path(tmpdir)
            logger = ErrorLogger(output_dir, 'test', 2020)

            with pytest.raises(ValueError):
                with logger.stage('test_stage'):
                    raise ValueError("Stage failed")

            log_content = logger.log_file.read_text()
            assert 'STAGE_START: test_stage' in log_content
            assert 'STAGE_FAILED: test_stage' in log_content
            assert 'ERROR: test_stage' in log_content

            logger.close()  # Close handlers for Windows cleanup

    def test_task_context_manager(self):
        """Test task context manager."""
        with tempfile.TemporaryDirectory() as tmpdir:
            output_dir = Path(tmpdir)
            logger = ErrorLogger(output_dir, 'test', 2020)

            with logger.task('test_task'):
                pass

            log_content = logger.log_file.read_text()
            assert 'TASK_START: test_task' in log_content

            logger.close()  # Close handlers for Windows cleanup

    def test_write_summary(self):
        """Test writing log summary."""
        with tempfile.TemporaryDirectory() as tmpdir:
            output_dir = Path(tmpdir)
            logger = ErrorLogger(output_dir, 'test', 2020)

            logger.log_warning('Warning 1')
            logger.log_warning('Warning 2')

            try:
                raise ValueError("Error 1")
            except Exception as e:
                logger.log_exception('task1', e)

            logger.write_summary()

            log_content = logger.log_file.read_text()
            assert 'Pipeline Summary: 1 error(s), 2 warning(s)' in log_content

            logger.close()  # Close handlers for Windows cleanup

