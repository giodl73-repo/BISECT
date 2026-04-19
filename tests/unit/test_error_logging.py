"""
Unit tests for error logging utilities.

Tests error_logger.py and stage_tracker.py functionality.
"""

import pytest
import tempfile
from pathlib import Path
from scripts.utils.error_logger import ErrorLogger, get_error_logger
from scripts.utils.stage_tracker import StageTracker, get_stage_tracker


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


class TestStageTracker:
    """Test StageTracker functionality."""

    def test_stage_tracker_creation(self):
        """Test that StageTracker can be created."""
        with tempfile.TemporaryDirectory() as tmpdir:
            output_dir = Path(tmpdir)
            tracker = StageTracker(output_dir)

            assert tracker.output_dir == output_dir

    def test_stage_tracker_factory(self):
        """Test get_stage_tracker factory function."""
        with tempfile.TemporaryDirectory() as tmpdir:
            output_dir = Path(tmpdir)
            tracker = get_stage_tracker(output_dir)

            assert isinstance(tracker, StageTracker)

    def test_mark_stage_complete(self):
        """Test marking stage as complete."""
        with tempfile.TemporaryDirectory() as tmpdir:
            output_dir = Path(tmpdir)
            tracker = StageTracker(output_dir)

            tracker.mark_stage_complete('test_stage')

            assert tracker.is_stage_complete('test_stage')

            marker_file = output_dir / '.stage_test_stage'
            assert marker_file.exists()

            # Check marker content
            content = marker_file.read_text()
            assert 'Stage: test_stage' in content
            assert 'Completed:' in content

    def test_mark_stage_complete_with_metadata(self):
        """Test marking stage complete with metadata."""
        with tempfile.TemporaryDirectory() as tmpdir:
            output_dir = Path(tmpdir)
            tracker = StageTracker(output_dir)

            tracker.mark_stage_complete('test_stage', metadata={
                'tasks_completed': 5,
                'duration': '120s'
            })

            metadata = tracker.get_stage_metadata('test_stage')
            assert metadata is not None
            assert 'tasks_completed' in metadata
            assert metadata['tasks_completed'] == '5'

    def test_get_completed_stages(self):
        """Test getting list of completed stages."""
        with tempfile.TemporaryDirectory() as tmpdir:
            output_dir = Path(tmpdir)
            tracker = StageTracker(output_dir)

            tracker.mark_stage_complete('stage1')
            tracker.mark_stage_complete('stage2')
            tracker.mark_stage_complete('stage3')

            completed = tracker.get_completed_stages()
            assert len(completed) == 3
            assert 'stage1' in completed
            assert 'stage2' in completed
            assert 'stage3' in completed

    def test_clear_stage(self):
        """Test clearing a specific stage marker."""
        with tempfile.TemporaryDirectory() as tmpdir:
            output_dir = Path(tmpdir)
            tracker = StageTracker(output_dir)

            tracker.mark_stage_complete('test_stage')
            assert tracker.is_stage_complete('test_stage')

            tracker.clear_stage('test_stage')
            assert not tracker.is_stage_complete('test_stage')

    def test_clear_all(self):
        """Test clearing all stage markers."""
        with tempfile.TemporaryDirectory() as tmpdir:
            output_dir = Path(tmpdir)
            tracker = StageTracker(output_dir)

            tracker.mark_stage_complete('stage1')
            tracker.mark_stage_complete('stage2')
            assert len(tracker.get_completed_stages()) == 2

            tracker.clear_all()
            assert len(tracker.get_completed_stages()) == 0

    def test_is_stage_complete_false(self):
        """Test checking non-existent stage."""
        with tempfile.TemporaryDirectory() as tmpdir:
            output_dir = Path(tmpdir)
            tracker = StageTracker(output_dir)

            assert not tracker.is_stage_complete('nonexistent_stage')
