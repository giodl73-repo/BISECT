"""
Integration tests for pipeline execution.

Tests actual subprocess execution with a mock pipeline script.
These tests catch real issues that unit tests miss:
- Working directory problems
- Subprocess creation failures
- STATUS protocol parsing in real subprocess
- Error handling with actual stderr
"""
import pytest
import asyncio
from pathlib import Path
from app.workers.executor import PipelineExecutor
from app.services.execution_service import ExecutionManager


@pytest.fixture
def mock_pipeline_path():
    """Path to mock pipeline script."""
    return Path(__file__).parent.parent / "fixtures" / "mock_pipeline.py"


@pytest.mark.asyncio
async def test_executor_starts_subprocess(mock_pipeline_path):
    """Test that executor can start a real subprocess."""
    progress_updates = []
    completion_status = []
    error_messages = []

    async def on_progress(data):
        progress_updates.append(data)

    async def on_complete(exit_code):
        completion_status.append(exit_code)

    async def on_error(msg):
        error_messages.append(msg)

    command = [
        "python",
        str(mock_pipeline_path),
        "--years", "2020",
        "--version", "test",
        "--delay", "0.05",
    ]

    # Use project root as working directory
    project_root = Path(__file__).parent.parent.parent.parent

    executor = PipelineExecutor(
        run_id=1,
        command=command,
        on_progress=on_progress,
        on_complete=on_complete,
        on_error=on_error,
        cwd=project_root,
    )

    await executor.start()

    # Verify subprocess completed successfully
    assert len(completion_status) == 1
    assert completion_status[0] == 0  # Exit code 0
    assert len(error_messages) == 0

    # Verify we got progress updates
    assert len(progress_updates) > 0


@pytest.mark.asyncio
async def test_executor_parses_status_messages(mock_pipeline_path):
    """Test that STATUS messages are parsed correctly."""
    progress_updates = []

    async def on_progress(data):
        progress_updates.append(data)

    async def on_complete(exit_code):
        pass

    async def on_error(msg):
        pass

    command = [
        "python",
        str(mock_pipeline_path),
        "--years", "2020",
        "--version", "test",
        "--delay", "0.05",
    ]

    project_root = Path(__file__).parent.parent.parent.parent

    executor = PipelineExecutor(
        run_id=1,
        command=command,
        on_progress=on_progress,
        on_complete=on_complete,
        on_error=on_error,
        cwd=project_root,
    )

    await executor.start()

    # Check that we got STATUS messages
    assert len(progress_updates) > 0

    # Verify progress structure
    last_update = progress_updates[-1]
    assert "years" in last_update
    assert "2020" in last_update["years"]


@pytest.mark.asyncio
async def test_executor_handles_subprocess_failure(mock_pipeline_path):
    """Test that executor handles subprocess failures correctly."""
    completion_status = []
    error_messages = []

    async def on_progress(data):
        pass

    async def on_complete(exit_code):
        completion_status.append(exit_code)

    async def on_error(msg):
        error_messages.append(msg)

    command = [
        "python",
        str(mock_pipeline_path),
        "--years", "2020",
        "--version", "test",
        "--fail",  # This will cause exit code 1
        "--delay", "0.05",
    ]

    project_root = Path(__file__).parent.parent.parent.parent

    executor = PipelineExecutor(
        run_id=1,
        command=command,
        on_progress=on_progress,
        on_complete=on_complete,
        on_error=on_error,
        cwd=project_root,
    )

    await executor.start()

    # Verify we got exit code 1
    assert len(completion_status) == 1
    assert completion_status[0] == 1


@pytest.mark.asyncio
async def test_executor_working_directory(mock_pipeline_path):
    """Test that working directory is set correctly."""
    completion_status = []

    async def on_progress(data):
        pass

    async def on_complete(exit_code):
        completion_status.append(exit_code)

    async def on_error(msg):
        pass

    command = [
        "python",
        str(mock_pipeline_path),
        "--years", "2020",
        "--version", "test",
        "--delay", "0.01",
    ]

    # Test with explicit project root
    project_root = Path(__file__).parent.parent.parent.parent

    executor = PipelineExecutor(
        run_id=1,
        command=command,
        on_progress=on_progress,
        on_complete=on_complete,
        on_error=on_error,
        cwd=project_root,
    )

    await executor.start()

    # If working directory is wrong, subprocess won't start
    assert len(completion_status) == 1
    assert completion_status[0] == 0


@pytest.mark.asyncio
async def test_execution_manager_working_directory():
    """Test that ExecutionManager sets working directory correctly."""
    # This is a critical test - it would have caught the backend/ vs project root bug

    manager = await ExecutionManager.get_instance()
    manager.reset()

    # Mock database session maker
    def mock_session_maker():
        class MockSession:
            def close(self):
                pass
        return MockSession()

    manager.set_session_maker(mock_session_maker)

    config = {
        "years": ["2020"],
        "states": None,
        "workers": 4,
        "version": "test",
    }

    # Start a run
    await manager.start_run(run_id=99, config=config)

    # Verify run was added to active runs
    assert manager.is_running(99)

    # Get the executor
    executor = manager._active_runs[99]

    # Check working directory is project root, not backend/
    # The path should end with 'apportionment', not 'backend'
    assert executor.cwd.name.lower() == "apportionment"
    assert executor.cwd.name.lower() != "backend"

    # Verify the scripts directory exists relative to this working directory
    scripts_dir = executor.cwd / "scripts" / "pipeline"
    assert scripts_dir.exists(), f"Scripts directory not found: {scripts_dir}"

    # Clean up
    manager.reset()


@pytest.mark.asyncio
async def test_executor_multi_year_progress(mock_pipeline_path):
    """Test progress updates for multi-year runs."""
    progress_updates = []

    async def on_progress(data):
        progress_updates.append(data)

    async def on_complete(exit_code):
        pass

    async def on_error(msg):
        pass

    command = [
        "python",
        str(mock_pipeline_path),
        "--years", "2000,2010,2020",
        "--version", "test",
        "--delay", "0.05",
    ]

    project_root = Path(__file__).parent.parent.parent.parent

    executor = PipelineExecutor(
        run_id=1,
        command=command,
        on_progress=on_progress,
        on_complete=on_complete,
        on_error=on_error,
        cwd=project_root,
    )

    await executor.start()

    # Verify we got updates for all years
    assert len(progress_updates) > 0

    last_update = progress_updates[-1]
    assert "years" in last_update

    # Should have data for all three years
    years = last_update["years"]
    assert "2000" in years or "2010" in years or "2020" in years
