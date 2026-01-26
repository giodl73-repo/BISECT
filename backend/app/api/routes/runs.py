"""
Run management API endpoints.

CRUD operations for managing redistricting pipeline runs.
"""
from typing import Optional
from fastapi import APIRouter, Depends, HTTPException, Query, status
from sqlalchemy.orm import Session

from app.database import get_db
from app.schemas.run import (
    RunCreate,
    RunResponse,
    RunDetailResponse,
    RunListResponse,
    RunProgressResponse,
    StateConfigResponse,
    StateInfo,
)
from app.services import run_service


router = APIRouter()


@router.post("", response_model=RunResponse, status_code=status.HTTP_201_CREATED)
async def create_run(
    run_create: RunCreate,
    db: Session = Depends(get_db),
):
    """
    Create a new pipeline run.

    Creates a run with pending status and initializes year tracking records.
    """
    try:
        run = run_service.create_run(db, run_create)
        return run
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to create run: {str(e)}",
        )


@router.get("", response_model=RunListResponse)
async def list_runs(
    status: Optional[str] = Query(None, description="Filter by status"),
    year: Optional[str] = Query(None, description="Filter by year"),
    version: Optional[str] = Query(None, description="Filter by version"),
    limit: int = Query(50, ge=1, le=100, description="Maximum results"),
    offset: int = Query(0, ge=0, description="Results to skip"),
    db: Session = Depends(get_db),
):
    """
    List all runs with optional filtering and pagination.

    Supports filtering by status, year, and version with pagination.
    """
    runs, total = run_service.list_runs(
        db,
        status=status,
        year=year,
        version=version,
        limit=limit,
        offset=offset,
    )

    return RunListResponse(
        runs=runs,
        total=total,
        limit=limit,
        offset=offset,
    )


@router.get("/{run_id}", response_model=RunDetailResponse)
async def get_run(
    run_id: int,
    db: Session = Depends(get_db),
):
    """
    Get detailed run information including progress and year details.

    Returns 404 if run not found.
    """
    run = run_service.get_run(db, run_id)
    if not run:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"Run {run_id} not found",
        )

    # Calculate duration if completed
    duration_seconds = None
    if run.completed_at and run.started_at:
        duration_seconds = int((run.completed_at - run.started_at).total_seconds())

    return RunDetailResponse(
        id=run.id,
        version=run.version,
        status=run.status,
        config=run.config,
        progress=run.progress,
        created_at=run.created_at,
        started_at=run.started_at,
        completed_at=run.completed_at,
        error_message=run.error_message,
        output_path=run.output_path,
        year_details=run.years,
        duration_seconds=duration_seconds,
    )


@router.delete("/{run_id}", status_code=status.HTTP_204_NO_CONTENT)
async def delete_run(
    run_id: int,
    db: Session = Depends(get_db),
):
    """
    Delete run metadata (not output files).

    Returns 404 if run not found.
    """
    deleted = run_service.delete_run(db, run_id)
    if not deleted:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"Run {run_id} not found",
        )


@router.get("/{run_id}/progress", response_model=RunProgressResponse)
async def get_run_progress(
    run_id: int,
    db: Session = Depends(get_db),
):
    """
    Get current run progress for polling.

    Optimized for frequent polling (every 2 seconds).
    Returns progress data including per-year status and ETA.
    """
    run = run_service.get_run(db, run_id)
    if not run:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"Run {run_id} not found",
        )

    # Build year progress dict
    years_progress = {}
    for year in run.years:
        years_progress[year.year] = {
            "status": year.status,
            "states_completed": year.states_completed,
            "states_total": year.states_total,
            "current_stage": year.current_stage,
        }

    # Calculate overall progress
    if run.years:
        total_states = sum(y.states_total for y in run.years)
        completed_states = sum(y.states_completed for y in run.years)
        overall_progress = completed_states / total_states if total_states > 0 else 0.0
    else:
        overall_progress = 0.0

    # Calculate ETA (if running)
    eta_seconds = None
    if run.status == "running" and run.started_at and overall_progress > 0:
        from datetime import datetime
        elapsed = (datetime.utcnow() - run.started_at).total_seconds()
        if overall_progress > 0:
            total_estimated = elapsed / overall_progress
            eta_seconds = int(total_estimated - elapsed)

    return RunProgressResponse(
        run_id=run.id,
        status=run.status,
        overall_progress=overall_progress,
        years=years_progress,
        eta_seconds=eta_seconds,
    )


@router.get("/config/states", response_model=StateConfigResponse)
async def get_state_config(
    year: str = Query("2020", description="Census year"),
):
    """
    Get state configuration for a census year.

    Returns all 50 states with district counts and FIPS codes.
    """
    from app.constants.states import STATE_FIPS, get_state_districts

    # Get year-specific districts
    try:
        state_districts = get_state_districts(year)
    except ValueError:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=f"Invalid year: {year}. Must be 2000, 2010, or 2020.",
        )

    # Build state list
    states = []
    for state_code, state_name in sorted(STATE_FIPS.items()):
        states.append(StateInfo(
            code=state_code,
            name=state_name,
            districts=state_districts.get(state_name, 1),
            fips=state_code,
        ))

    return StateConfigResponse(
        year=year,
        states=states,
    )
