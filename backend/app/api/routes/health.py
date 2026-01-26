"""
Health and version endpoints.

Provides health check and version information for monitoring and diagnostics.
"""
from fastapi import APIRouter, Depends, status
from sqlalchemy.orm import Session
from sqlalchemy import text
from datetime import datetime

from app.database import get_db
from app.config import settings
from app.schemas.common import HealthResponse, VersionResponse


router = APIRouter(tags=["Health"])


@router.get("/health", response_model=HealthResponse)
async def health_check(db: Session = Depends(get_db)):
    """
    Health check endpoint.

    Returns:
        - status: "healthy" if database is connected, "degraded" otherwise
        - database: "connected" or "disconnected"
        - timestamp: Current timestamp

    Status Codes:
        - 200: Service is healthy
        - 503: Service is degraded (database unavailable)
    """
    try:
        # Verify database connectivity with simple query
        db.execute(text("SELECT 1"))
        db_status = "connected"
        overall_status = "healthy"
        status_code = status.HTTP_200_OK
    except Exception:
        db_status = "disconnected"
        overall_status = "degraded"
        status_code = status.HTTP_503_SERVICE_UNAVAILABLE

    return HealthResponse(
        status=overall_status,
        database=db_status,
        timestamp=datetime.utcnow()
    )


@router.get("/version", response_model=VersionResponse)
async def version():
    """
    Version information endpoint.

    Returns:
        - version: Application version
        - api_version: API version prefix
        - wave: Wave number
    """
    return VersionResponse(
        version=settings.app_version,
        api_version=settings.api_version,
        wave=9
    )
