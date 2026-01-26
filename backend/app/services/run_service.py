"""
Service layer for run management.

Handles business logic for creating, reading, updating, and deleting runs.
"""
from datetime import datetime
from typing import List, Optional, Tuple
from sqlalchemy.orm import Session, joinedload
from sqlalchemy import desc

from app.models.run import Run, RunYear, RunStatus
from app.schemas.run import RunCreate


def create_run(db: Session, run_create: RunCreate) -> Run:
    """
    Create a new pipeline run with pending status.

    Args:
        db: Database session
        run_create: Run creation data

    Returns:
        Created run instance
    """
    # Build config dict from request
    config = {
        "years": run_create.years,
        "states": run_create.states,
        "workers": run_create.workers,
        "dpi": run_create.dpi,
        "partition_mode": run_create.partition_mode,
    }

    # Create run
    run = Run(
        version=run_create.version,
        status=RunStatus.PENDING,
        config=config,
        output_path=f"outputs/{run_create.version}",
    )

    db.add(run)
    db.flush()  # Get ID without committing

    # Create year records
    for year in run_create.years:
        run_year = RunYear(
            run_id=run.id,
            year=year,
            status=RunStatus.PENDING,
            states_total=50 if not run_create.states else len(run_create.states),
        )
        db.add(run_year)

    db.commit()
    db.refresh(run)

    return run


def get_run(db: Session, run_id: int) -> Optional[Run]:
    """
    Get run by ID with year details eagerly loaded.

    Args:
        db: Database session
        run_id: Run ID

    Returns:
        Run instance or None if not found
    """
    return (
        db.query(Run)
        .options(joinedload(Run.years))
        .filter(Run.id == run_id)
        .first()
    )


def list_runs(
    db: Session,
    status: Optional[str] = None,
    year: Optional[str] = None,
    version: Optional[str] = None,
    limit: int = 50,
    offset: int = 0,
) -> Tuple[List[Run], int]:
    """
    List runs with filtering and pagination.

    Args:
        db: Database session
        status: Filter by status
        year: Filter by year (searches in config JSON)
        version: Filter by version
        limit: Maximum number of results
        offset: Number of results to skip

    Returns:
        Tuple of (runs list, total count)
    """
    query = db.query(Run)

    # Apply filters
    if status:
        query = query.filter(Run.status == status)
    if version:
        query = query.filter(Run.version == version)
    if year:
        # Filter by year in config JSON
        query = query.filter(Run.config["years"].astext.contains(year))

    # Get total count
    total = query.count()

    # Apply pagination and ordering
    runs = (
        query.order_by(desc(Run.created_at))
        .offset(offset)
        .limit(limit)
        .all()
    )

    return runs, total


def update_run_status(
    db: Session,
    run_id: int,
    status: RunStatus,
    error_message: Optional[str] = None,
) -> Optional[Run]:
    """
    Update run status with appropriate timestamps.

    Args:
        db: Database session
        run_id: Run ID
        status: New status
        error_message: Optional error message for failed runs

    Returns:
        Updated run or None if not found
    """
    run = db.query(Run).filter(Run.id == run_id).first()
    if not run:
        return None

    run.status = status

    # Update timestamps based on status
    if status == RunStatus.RUNNING and not run.started_at:
        run.started_at = datetime.utcnow()
    elif status in (RunStatus.COMPLETED, RunStatus.FAILED, RunStatus.CANCELLED):
        if not run.completed_at:
            run.completed_at = datetime.utcnow()

    if error_message:
        run.error_message = error_message

    db.commit()
    db.refresh(run)

    return run


def update_run_progress(
    db: Session,
    run_id: int,
    progress: dict,
) -> Optional[Run]:
    """
    Update run progress from STATUS messages.

    Args:
        db: Database session
        run_id: Run ID
        progress: Progress data dict

    Returns:
        Updated run or None if not found
    """
    run = db.query(Run).filter(Run.id == run_id).first()
    if not run:
        return None

    run.progress = progress

    db.commit()
    db.refresh(run)

    return run


def update_run_pid(db: Session, run_id: int, process_pid: int) -> Optional[Run]:
    """
    Update run process PID for orphan detection.

    Args:
        db: Database session
        run_id: Run ID
        process_pid: Process ID

    Returns:
        Updated run or None if not found
    """
    run = db.query(Run).filter(Run.id == run_id).first()
    if not run:
        return None

    run.process_pid = process_pid

    db.commit()
    db.refresh(run)

    return run


def delete_run(db: Session, run_id: int) -> bool:
    """
    Delete run metadata (not output files).

    Args:
        db: Database session
        run_id: Run ID

    Returns:
        True if deleted, False if not found
    """
    run = db.query(Run).filter(Run.id == run_id).first()
    if not run:
        return False

    db.delete(run)
    db.commit()

    return True
