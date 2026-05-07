#!/usr/bin/env python3
"""
Error logging utility for the redistricting pipeline.

Provides thread-safe error logging with context managers for clean integration.
Logs are written to outputs/{version}/{year}/error.log with full tracebacks,
timestamps, and system context.

Usage:
    # Initialize logger
    logger = ErrorLogger(output_dir=Path('outputs/v1/2020'), version='v1', year=2020)

    # Log stage execution
    with logger.stage('national_post_processing'):
        # Your code here
        pass

    # Log exceptions manually
    try:
        risky_operation()
    except Exception as e:
        logger.log_exception('task_name', e)
        raise

    # Log warnings
    logger.log_warning('Missing optional data file: elections.csv')
"""

import logging
import sys
import traceback
from pathlib import Path
from datetime import datetime
from typing import Optional
from contextlib import contextmanager
import platform

# Optional: psutil for system context (memory, disk info)
try:
    import psutil
    PSUTIL_AVAILABLE = True
except ImportError:
    PSUTIL_AVAILABLE = False


class ErrorLogger:
    """Thread-safe error logger for pipeline execution."""

    def __init__(self, output_dir: Path, version: str, year: int):
        """
        Initialize error logger.

        Args:
            output_dir: Output directory for this year (e.g., outputs/v1/2020)
            version: Version identifier (e.g., 'v1', 'test')
            year: Census year (2000, 2010, 2020)
        """
        self.output_dir = Path(output_dir)
        self.version = version
        self.year = year
        self.log_file = self.output_dir / 'error.log'

        # Ensure directory exists
        self.output_dir.mkdir(parents=True, exist_ok=True)

        # Initialize Python logging
        self.logger = logging.getLogger(f'pipeline_{version}_{year}')
        self.logger.setLevel(logging.DEBUG)
        self.logger.propagate = False  # Don't propagate to root logger

        # Remove any existing handlers
        self.logger.handlers.clear()

        # Create file handler with UTF-8 encoding
        handler = logging.FileHandler(self.log_file, mode='a', encoding='utf-8', errors='replace')
        handler.setLevel(logging.DEBUG)

        # Create formatter (no built-in timestamp, we'll add our own)
        formatter = logging.Formatter('%(message)s')
        handler.setFormatter(formatter)

        self.logger.addHandler(handler)

        # Track error and warning counts
        self.error_count = 0
        self.warning_count = 0

        # Track if header has been written
        self.header_written = False

    def _write_header(self):
        """Write header information to log file (once)."""
        if self.header_written:
            return

        # Get system info
        python_version = f"{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}"

        # Try to get METIS version (optional)
        metis_version = "Unknown"
        try:
            # Check if METIS is available
            import subprocess
            result = subprocess.run(['gpmetis', '-help'], capture_output=True, text=True, timeout=2)
            if 'METIS' in result.stdout or 'METIS' in result.stderr:
                metis_version = "5.1.0"  # Known version for our setup
        except:
            pass

        header = f"""{'='*74}
Pipeline Error Log: {self.log_file}
Started: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}
Version: {self.version} | Year: {self.year} | Python: {python_version} | METIS: {metis_version}
{'='*74}

"""
        self.logger.info(header)
        self.header_written = True

    def _get_timestamp(self) -> str:
        """Get formatted timestamp."""
        return datetime.now().strftime('%Y-%m-%d %H:%M:%S')

    def _get_system_context(self) -> str:
        """Get system context information (memory, disk, etc.)."""
        try:
            if not PSUTIL_AVAILABLE:
                return f"""System Context:
  - Platform: {platform.system()} {platform.release()}
  - psutil not available (install for memory/disk info)"""

            # Memory info
            mem = psutil.virtual_memory()
            mem_free_gb = mem.available / (1024**3)
            mem_total_gb = mem.total / (1024**3)

            # Disk info (for output directory)
            disk = psutil.disk_usage(str(self.output_dir.parent))
            disk_free_gb = disk.free / (1024**3)

            context = f"""System Context:
  - Memory: {mem_free_gb:.1f} GB free / {mem_total_gb:.1f} GB total
  - Disk: {disk_free_gb:.0f} GB free
  - Platform: {platform.system()} {platform.release()}"""

            return context
        except Exception:
            return "System Context: [Unable to retrieve]"

    def log_stage_start(self, stage_name: str):
        """
        Log the start of a pipeline stage.

        Args:
            stage_name: Name of the stage (e.g., 'national_post_processing', 'state_redistricting')
        """
        self._write_header()
        timestamp = self._get_timestamp()
        self.logger.info(f"[{timestamp}] STAGE_START: {stage_name}")

    def log_stage_complete(self, stage_name: str):
        """
        Log the successful completion of a pipeline stage.

        Args:
            stage_name: Name of the stage
        """
        timestamp = self._get_timestamp()
        self.logger.info(f"[{timestamp}] STAGE_COMPLETE: {stage_name}\n")

    def log_stage_failed(self, stage_name: str):
        """
        Log the failure of a pipeline stage.

        Args:
            stage_name: Name of the stage
        """
        timestamp = self._get_timestamp()
        self.logger.error(f"[{timestamp}] STAGE_FAILED: {stage_name}\n")

    def log_task_start(self, task_name: str):
        """
        Log the start of a task within a stage.

        Args:
            task_name: Name of the task
        """
        timestamp = self._get_timestamp()
        self.logger.info(f"[{timestamp}] TASK_START: {task_name}")

    def log_exception(self, task_name: str, exception: Exception, context: Optional[dict] = None):
        """
        Log an exception with full traceback and context.

        Args:
            task_name: Name of the task that failed
            exception: The exception that was raised
            context: Optional dict with additional context (e.g., {'states_completed': '50/50'})
        """
        self._write_header()
        self.error_count += 1

        timestamp = self._get_timestamp()
        exc_type = type(exception).__name__
        exc_message = str(exception)
        exc_traceback = ''.join(traceback.format_tb(exception.__traceback__))

        error_msg = f"""[{timestamp}] ERROR: {task_name}
Exception: {exc_type}
Message: {exc_message}
Traceback:
{exc_traceback}"""

        # Add system context
        error_msg += "\n" + self._get_system_context()

        # Add custom context if provided
        if context:
            error_msg += "\nTask Context:"
            for key, value in context.items():
                error_msg += f"\n  - {key}: {value}"

        error_msg += "\n"

        self.logger.error(error_msg)

    def log_warning(self, message: str, context: Optional[dict] = None):
        """
        Log a warning (non-fatal issue).

        Args:
            message: Warning message
            context: Optional dict with additional context
        """
        self._write_header()
        self.warning_count += 1

        timestamp = self._get_timestamp()
        warning_msg = f"[{timestamp}] WARNING: {message}"

        if context:
            warning_msg += "\nContext:"
            for key, value in context.items():
                warning_msg += f"\n  - {key}: {value}"

        warning_msg += "\n"

        self.logger.warning(warning_msg)

    def log_command_failure(self, command: str, return_code: int, output: str,
                           task_name: Optional[str] = None, context: Optional[dict] = None):
        """
        Log a command failure with full output.

        Args:
            command: The command that was run
            return_code: The exit code returned
            output: Combined stdout/stderr output
            task_name: Name of the task (e.g., 'redistricting', 'summary')
            context: Optional dict with additional context
        """
        self._write_header()
        self.error_count += 1

        timestamp = self._get_timestamp()

        # Truncate output if too long (keep first and last parts)
        max_output_lines = 100
        output_lines = output.split('\n')
        if len(output_lines) > max_output_lines:
            truncated_output = '\n'.join(output_lines[:50])
            truncated_output += '\n\n... [output truncated - see full output above] ...\n\n'
            truncated_output += '\n'.join(output_lines[-50:])
        else:
            truncated_output = output

        error_msg = f"""[{timestamp}] COMMAND_FAILURE: {task_name or 'Unknown Task'}
Return Code: {return_code}
Command: {command}
Output:
{'-' * 70}
{truncated_output}
{'-' * 70}
"""

        # Add system context
        error_msg += "\n" + self._get_system_context()

        # Add custom context if provided
        if context:
            error_msg += "\nTask Context:"
            for key, value in context.items():
                error_msg += f"\n  - {key}: {value}"

        error_msg += "\n"

        self.logger.error(error_msg)

    def write_summary(self):
        """Write summary footer to log file."""
        if not self.header_written:
            return  # Nothing was logged

        summary = f"""{'='*74}
Pipeline Summary: {self.error_count} error(s), {self.warning_count} warning(s)
{'='*74}
"""
        self.logger.info(summary)

    def close(self):
        """Close all file handlers (important for Windows file cleanup)."""
        for handler in self.logger.handlers[:]:
            handler.close()
            self.logger.removeHandler(handler)

    @contextmanager
    def stage(self, stage_name: str):
        """
        Context manager for logging a pipeline stage.

        Usage:
            with logger.stage('national_post_processing'):
                # Your code here
                pass

        Args:
            stage_name: Name of the stage
        """
        self.log_stage_start(stage_name)
        try:
            yield
            self.log_stage_complete(stage_name)
        except Exception as e:
            self.log_stage_failed(stage_name)
            self.log_exception(stage_name, e)
            raise

    @contextmanager
    def task(self, task_name: str):
        """
        Context manager for logging a task within a stage.

        Usage:
            with logger.task('visualize_national_rounds'):
                # Your code here
                pass

        Args:
            task_name: Name of the task
        """
        self.log_task_start(task_name)
        try:
            yield
        except Exception as e:
            self.log_exception(task_name, e)
            raise


def get_error_logger(output_dir: Path, version: str, year: int) -> ErrorLogger:
    """
    Factory function to create an ErrorLogger instance.

    Args:
        output_dir: Output directory for this year
        version: Version identifier
        year: Census year

    Returns:
        ErrorLogger instance
    """
    return ErrorLogger(output_dir, version, year)
