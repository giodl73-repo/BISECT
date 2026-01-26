"""
Tests for run service layer.

Unit tests for run creation, listing, updating, and deletion.
"""
import pytest
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker
from sqlalchemy.pool import StaticPool

from app.database import Base
from app.models.run import Run, RunYear, RunStatus
from app.schemas.run import RunCreate
from app.services import run_service


@pytest.fixture
def db_session():
    """Create an in-memory SQLite database for testing."""
    engine = create_engine(
        "sqlite:///:memory:",
        connect_args={"check_same_thread": False},
        poolclass=StaticPool,
    )
    Base.metadata.create_all(bind=engine)
    SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)

    session = SessionLocal()
    try:
        yield session
    finally:
        session.close()


def test_create_run(db_session):
    """Test creating a run with valid data."""
    run_create = RunCreate(
        version="test_v1",
        years=["2020"],
        states=["VT"],
        workers=1,
        dpi=150,
        partition_mode="edge-weighted",
    )

    run = run_service.create_run(db_session, run_create)

    assert run.id is not None
    assert run.version == "test_v1"
    assert run.status == RunStatus.PENDING
    assert run.config["years"] == ["2020"]
    assert run.config["states"] == ["VT"]
    assert run.config["workers"] == 1
    assert run.output_path == "outputs/test_v1"
    assert len(run.years) == 1
    assert run.years[0].year == "2020"
    assert run.years[0].states_total == 1  # Only VT


def test_create_run_multiple_years(db_session):
    """Test creating a run with multiple years."""
    run_create = RunCreate(
        version="multi_year",
        years=["2020", "2010", "2000"],
        workers=4,
    )

    run = run_service.create_run(db_session, run_create)

    assert len(run.years) == 3
    years_list = [y.year for y in run.years]
    assert "2020" in years_list
    assert "2010" in years_list
    assert "2000" in years_list
    # All states when states=None
    assert all(y.states_total == 50 for y in run.years)


def test_get_run(db_session):
    """Test retrieving a run by ID."""
    run_create = RunCreate(
        version="get_test",
        years=["2020"],
        workers=2,
    )

    created_run = run_service.create_run(db_session, run_create)
    retrieved_run = run_service.get_run(db_session, created_run.id)

    assert retrieved_run is not None
    assert retrieved_run.id == created_run.id
    assert retrieved_run.version == "get_test"
    assert len(retrieved_run.years) > 0


def test_get_run_not_found(db_session):
    """Test retrieving a non-existent run."""
    result = run_service.get_run(db_session, 99999)
    assert result is None


def test_list_runs(db_session):
    """Test listing runs without filters."""
    # Create multiple runs
    for i in range(5):
        run_create = RunCreate(
            version=f"v{i}",
            years=["2020"],
            workers=2,
        )
        run_service.create_run(db_session, run_create)

    runs, total = run_service.list_runs(db_session)

    assert total == 5
    assert len(runs) == 5


def test_list_runs_with_status_filter(db_session):
    """Test filtering runs by status."""
    # Create runs with different statuses
    run1 = run_service.create_run(db_session, RunCreate(version="v1", years=["2020"]))
    run2 = run_service.create_run(db_session, RunCreate(version="v2", years=["2020"]))

    # Update one to running
    run_service.update_run_status(db_session, run2.id, RunStatus.RUNNING)

    # Filter by pending
    pending_runs, pending_total = run_service.list_runs(db_session, status="pending")
    assert pending_total == 1
    assert pending_runs[0].id == run1.id

    # Filter by running
    running_runs, running_total = run_service.list_runs(db_session, status="running")
    assert running_total == 1
    assert running_runs[0].id == run2.id


def test_list_runs_with_version_filter(db_session):
    """Test filtering runs by version."""
    run_service.create_run(db_session, RunCreate(version="v1", years=["2020"]))
    run_service.create_run(db_session, RunCreate(version="v2", years=["2020"]))
    run_service.create_run(db_session, RunCreate(version="v1", years=["2010"]))

    runs, total = run_service.list_runs(db_session, version="v1")

    assert total == 2
    assert all(r.version == "v1" for r in runs)


def test_list_runs_pagination(db_session):
    """Test pagination of run list."""
    # Create 25 runs
    for i in range(25):
        run_service.create_run(db_session, RunCreate(version=f"v{i}", years=["2020"]))

    # Get first page
    page1, total = run_service.list_runs(db_session, limit=10, offset=0)
    assert len(page1) == 10
    assert total == 25

    # Get second page
    page2, _ = run_service.list_runs(db_session, limit=10, offset=10)
    assert len(page2) == 10

    # Get third page
    page3, _ = run_service.list_runs(db_session, limit=10, offset=20)
    assert len(page3) == 5  # Only 5 remaining


def test_update_run_status(db_session):
    """Test updating run status."""
    run = run_service.create_run(db_session, RunCreate(version="status_test", years=["2020"]))

    # Update to running
    updated_run = run_service.update_run_status(db_session, run.id, RunStatus.RUNNING)
    assert updated_run.status == RunStatus.RUNNING
    assert updated_run.started_at is not None

    # Update to completed
    updated_run = run_service.update_run_status(db_session, run.id, RunStatus.COMPLETED)
    assert updated_run.status == RunStatus.COMPLETED
    assert updated_run.completed_at is not None


def test_update_run_status_with_error(db_session):
    """Test updating run status with error message."""
    run = run_service.create_run(db_session, RunCreate(version="error_test", years=["2020"]))

    updated_run = run_service.update_run_status(
        db_session,
        run.id,
        RunStatus.FAILED,
        error_message="Test error message",
    )

    assert updated_run.status == RunStatus.FAILED
    assert updated_run.error_message == "Test error message"
    assert updated_run.completed_at is not None


def test_update_run_progress(db_session):
    """Test updating run progress."""
    run = run_service.create_run(db_session, RunCreate(version="progress_test", years=["2020"]))

    progress_data = {
        "overall_progress": 0.45,
        "years": {
            "2020": {
                "status": "running",
                "states_completed": 24,
                "states_total": 50,
            }
        },
        "eta_seconds": 3600,
    }

    updated_run = run_service.update_run_progress(db_session, run.id, progress_data)

    assert updated_run.progress == progress_data
    assert updated_run.progress["overall_progress"] == 0.45


def test_delete_run(db_session):
    """Test deleting a run."""
    run = run_service.create_run(db_session, RunCreate(version="delete_test", years=["2020"]))
    run_id = run.id

    # Delete run
    deleted = run_service.delete_run(db_session, run_id)
    assert deleted is True

    # Verify run is gone
    result = run_service.get_run(db_session, run_id)
    assert result is None


def test_delete_run_not_found(db_session):
    """Test deleting a non-existent run."""
    deleted = run_service.delete_run(db_session, 99999)
    assert deleted is False
