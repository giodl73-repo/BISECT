"""
Pipeline executor for running redistricting subprocess.

Simplified MVP version - handles basic subprocess execution and STATUS parsing.
Full implementation with watchdog, heartbeat monitoring, and file-based fallback
can be added in future iterations.
"""
import asyncio
import os
from pathlib import Path
from typing import Optional, Callable, Awaitable
import logging

from app.utils.status_parser import StatusParser

logger = logging.getLogger(__name__)


class PipelineExecutor:
    """
    Async wrapper for pipeline subprocess execution.

    MVP implementation - handles:
    - Subprocess creation and monitoring
    - STATUS protocol parsing from stdout
    - Basic cancellation

    Future enhancements:
    - File-based progress fallback
    - Watchdog for hung processes
    - Advanced error handling
    """

    def __init__(
        self,
        run_id: int,
        command: list[str],
        on_progress: Callable[[dict], Awaitable[None]],
        on_complete: Callable[[int], Awaitable[None]],
        on_error: Callable[[str], Awaitable[None]],
        cwd: Optional[Path] = None,
    ):
        """
        Initialize pipeline executor.

        Args:
            run_id: Database run ID
            command: Command to execute (e.g., ['python', 'script.py', '--args'])
            on_progress: Async callback for progress updates
            on_complete: Async callback on completion (receives exit code)
            on_error: Async callback on error (receives error message)
            cwd: Working directory for subprocess
        """
        self.run_id = run_id
        self.command = command
        self.on_progress = on_progress
        self.on_complete = on_complete
        self.on_error = on_error
        self.cwd = cwd or Path.cwd()

        self.process: Optional[asyncio.subprocess.Process] = None
        self._cancelled = False
        self._status_parser = StatusParser()
        self._status_messages = []

    async def start(self):
        """Start pipeline subprocess and monitor output."""
        try:
            # Create subprocess
            env = os.environ.copy()
            env["PYTHONUNBUFFERED"] = "1"  # Disable Python output buffering

            logger.info(f"Starting pipeline for run {self.run_id}")
            logger.info(f"Command: {' '.join(self.command)}")
            logger.info(f"Working directory: {self.cwd}")

            self.process = await asyncio.create_subprocess_exec(
                *self.command,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
                env=env,
                cwd=str(self.cwd),
            )

            logger.info(f"Process created for run {self.run_id}, PID: {self.process.pid}")

            # Monitor stdout and stderr concurrently
            await asyncio.gather(
                self._monitor_stdout(),
                self._monitor_stderr(),
                self._wait_for_completion(),
            )

        except Exception as e:
            error_msg = f"{type(e).__name__}: {str(e)}"
            logger.error(f"Pipeline executor error for run {self.run_id}: {error_msg}")
            logger.exception(f"Full traceback for run {self.run_id}")
            await self.on_error(error_msg)

    async def cancel(self, timeout: float = 5.0):
        """
        Cancel running pipeline with graceful shutdown.

        Args:
            timeout: Seconds to wait for graceful shutdown before force kill
        """
        if not self.process or self.process.returncode is not None:
            logger.warning(f"Cannot cancel run {self.run_id} - process not running")
            return

        self._cancelled = True
        logger.info(f"Cancelling run {self.run_id}")

        try:
            # Try graceful termination first
            self.process.terminate()

            # Wait for graceful shutdown
            try:
                await asyncio.wait_for(self.process.wait(), timeout=timeout)
                logger.info(f"Run {self.run_id} terminated gracefully")
            except asyncio.TimeoutError:
                # Force kill if graceful shutdown times out
                logger.warning(f"Run {self.run_id} did not terminate gracefully, forcing kill")
                self.process.kill()
                await self.process.wait()

        except Exception as e:
            logger.error(f"Error cancelling run {self.run_id}: {e}")

    async def _monitor_stdout(self):
        """Read stdout and parse STATUS messages."""
        if not self.process or not self.process.stdout:
            return

        while True:
            line = await self.process.stdout.readline()
            if not line:
                break

            line_str = line.decode('utf-8', errors='replace').strip()
            if not line_str:
                continue

            # Log all output for debugging
            logger.debug(f"[Run {self.run_id}] {line_str}")

            # Parse STATUS messages
            msg = self._status_parser.parse_line(line_str)
            if msg:
                self._status_messages.append(msg)

                # Aggregate progress and send update
                progress = self._status_parser.aggregate_progress(self._status_messages)
                await self.on_progress(progress)

    async def _monitor_stderr(self):
        """Read stderr for error messages."""
        if not self.process or not self.process.stderr:
            return

        error_lines = []
        while True:
            line = await self.process.stderr.readline()
            if not line:
                break

            line_str = line.decode('utf-8', errors='replace').strip()
            if line_str:
                logger.warning(f"[Run {self.run_id} STDERR] {line_str}")
                error_lines.append(line_str)

        # If we collected error output, report it
        if error_lines and not self._cancelled:
            await self.on_error("\n".join(error_lines[-10:]))  # Last 10 lines

    async def _wait_for_completion(self):
        """Wait for process to complete and call completion callback."""
        if not self.process:
            return

        exit_code = await self.process.wait()

        if not self._cancelled:
            logger.info(f"Run {self.run_id} completed with exit code {exit_code}")
            await self.on_complete(exit_code)
