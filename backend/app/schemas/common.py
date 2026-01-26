"""
Common Pydantic schemas used across the API.

Includes health check responses, error responses, and version information.
"""
from pydantic import BaseModel
from datetime import datetime
from typing import Optional


class HealthResponse(BaseModel):
    """Health check response model."""
    status: str  # "healthy" or "degraded"
    database: str  # "connected" or "disconnected"
    timestamp: datetime


class VersionResponse(BaseModel):
    """Version information response model."""
    version: str  # Application version (9.0.0)
    api_version: str  # API version (v1)
    wave: int  # Wave number (9)


class ErrorResponse(BaseModel):
    """Standard error response model (from common-backend-utils)."""
    message: str
    code: str
    details: Optional[dict] = None
