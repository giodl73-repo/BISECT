"""
Service for managing pipeline execution.

Handles starting, monitoring, and cancelling pipeline runs.
"""
import asyncio
import logging
from pathlib import Path
from typing import Dict, Optional
from sqlalchemy.orm import Session

from app.models.run import RunStatus
from app.services import run_service
from app.workers.executor import PipelineExecutor

logger = logging.getLogger(__name__)


class ExecutionManager:
    """
    Singleton manager for tracking active pipeline executions.

    Maintains a registry of running pipelines and provides methods
    to start, monitor, and cancel them.
    """

    _instance: Optional['ExecutionManager'] = None
    _lock = asyncio.Lock()

    def __init__(self):
        self._active_runs: Dict[int, PipelineExecutor] = {}
        self._db_session_maker = None  # Will be set by FastAPI app

    @classmethod
    async def get_instance(cls) -> 'ExecutionManager':
        """Get singleton instance."""
        if cls._instance is None:
            async with cls._lock:
                if cls._instance is None:
                    cls._instance = cls()
        return cls._instance

    def set_session_maker(self, session_maker):
        """Set database session maker for callbacks."""
        self._db_session_maker = session_maker

    async def start_run(self, run_id: int, config: dict):
        """
        Start pipeline execution for a run.

        Args:
            run_id: Run ID from database
            config: Run configuration dict
        """
        if run_id in self._active_runs:
            raise ValueError(f"Run {run_id} is already active")

        # Build command from config
        command = self._build_command(config)

        # Create progress callback
        async def on_progress(progress: dict):
            """Update progress in database."""
            if not self._db_session_maker:
                return

            db = self._db_session_maker()
            try:
                run_service.update_run_progress(db, run_id, progress)
            finally:
                db.close()

        # Create completion callback
        async def on_complete(exit_code: int):
            """Handle completion."""
            if not self._db_session_maker:
                return

            db = self._db_session_maker()
            try:
                if exit_code == 0:
                    run_service.update_run_status(db, run_id, RunStatus.COMPLETED)
                    logger.info(f"Run {run_id} completed successfully")
                else:
                    run_service.update_run_status(
                        db,
                        run_id,
                        RunStatus.FAILED,
                        error_message=f"Pipeline exited with code {exit_code}",
                    )
                    logger.error(f"Run {run_id} failed with exit code {exit_code}")

                # Remove from active runs
                self._active_runs.pop(run_id, None)
            finally:
                db.close()

        # Create error callback
        async def on_error(error_msg: str):
            """Handle error."""
            if not self._db_session_maker:
                return

            db = self._db_session_maker()
            try:
                run_service.update_run_status(
                    db,
                    run_id,
                    RunStatus.FAILED,
                    error_message=error_msg,
                )
                logger.error(f"Run {run_id} error: {error_msg}")

                # Remove from active runs
                self._active_runs.pop(run_id, None)
            finally:
                db.close()

        # Create executor
        # Set working directory to project root (parent of backend/)
        project_root = Path.cwd().parent if Path.cwd().name == "backend" else Path.cwd()

        executor = PipelineExecutor(
            run_id=run_id,
            command=command,
            on_progress=on_progress,
            on_complete=on_complete,
            on_error=on_error,
            cwd=project_root,
        )

        # Store in active runs
        self._active_runs[run_id] = executor

        # Start execution in background
        asyncio.create_task(executor.start())

        logger.info(f"Started execution for run {run_id}")

    async def cancel_run(self, run_id: int):
        """
        Cancel an active run.

        Args:
            run_id: Run ID to cancel
        """
        executor = self._active_runs.get(run_id)
        if not executor:
            raise ValueError(f"Run {run_id} is not active")

        await executor.cancel()
        self._active_runs.pop(run_id, None)

        logger.info(f"Cancelled run {run_id}")

    def is_running(self, run_id: int) -> bool:
        """Check if a run is currently active."""
        return run_id in self._active_runs

    def reset(self):
        """Reset manager state (for testing)."""
        self._active_runs.clear()

    def _build_command(self, config: dict) -> list[str]:
        """
        Build pipeline command from configuration.

        Args:
            config: Run configuration

        Returns:
            Command as list of strings
        """
        # Extract config values
        years = config.get("years", ["2020"])
        states = config.get("states")
        workers = config.get("workers", 4)
        dpi = config.get("dpi", 150)
        partition_mode = config.get("partition_mode", "edge-weighted")
        version = config.get("version", "test")

        # Build command
        # Using Python to run the pipeline script
        command = [
            "python",
            "scripts/pipeline/run_complete_redistricting.py",
            "--years", ",".join(years),
            "--workers", str(workers),
            "--dpi", str(dpi),
            "--partition-mode", partition_mode,
            "--version", version,
        ]

        # Add states filter if specified
        if states:
            command.extend(["--states", ",".join(states)])

        return command


# Global instance (will be initialized by FastAPI app)
execution_manager: Optional[ExecutionManager] = None


async def get_execution_manager() -> ExecutionManager:
    """Get or create execution manager instance."""
    global execution_manager
    if execution_manager is None:
        execution_manager = await ExecutionManager.get_instance()
    return execution_manager
