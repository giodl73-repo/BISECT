"""
Pydantic schemas for run management API.

Request and response models for creating, listing, and tracking runs.
"""
from datetime import datetime
from typing import List, Optional, Dict, Any
from pydantic import BaseModel, Field, ConfigDict, field_validator


class RunCreate(BaseModel):
    """Request schema for creating a new run."""
    version: str = Field(..., min_length=1, max_length=50, description="Version identifier (e.g., 'v1', 'test')")
    years: List[str] = Field(..., min_length=1, description="Census years to process (e.g., ['2020', '2010'])")
    states: Optional[List[str]] = Field(None, description="State codes to process (None = all 50 states)")
    workers: int = Field(4, ge=1, le=16, description="Number of parallel workers")
    dpi: int = Field(150, ge=72, le=600, description="Map resolution DPI")
    partition_mode: str = Field("edge-weighted", description="Partitioning mode")

    @field_validator("years")
    @classmethod
    def validate_years(cls, v):
        """Validate census years."""
        valid_years = {"2000", "2010", "2020"}
        for year in v:
            if year not in valid_years:
                raise ValueError(f"Invalid year: {year}. Must be one of {valid_years}")
        return v


class RunYearResponse(BaseModel):
    """Response schema for year-level progress."""
    model_config = ConfigDict(from_attributes=True)

    id: int
    year: str
    status: str
    states_completed: int
    states_total: int
    current_stage: Optional[str] = None
    started_at: Optional[datetime] = None
    completed_at: Optional[datetime] = None
    error_message: Optional[str] = None


class RunResponse(BaseModel):
    """Response schema for run summary."""
    model_config = ConfigDict(from_attributes=True)

    id: int
    version: str
    status: str
    created_at: datetime
    started_at: Optional[datetime] = None
    completed_at: Optional[datetime] = None
    error_message: Optional[str] = None
    output_path: Optional[str] = None


class RunDetailResponse(RunResponse):
    """Response schema for run details."""
    config: Dict[str, Any]
    progress: Optional[Dict[str, Any]] = None
    year_details: List[RunYearResponse] = []
    duration_seconds: Optional[int] = None


class RunListResponse(BaseModel):
    """Response schema for paginated run list."""
    runs: List[RunResponse]
    total: int
    limit: int
    offset: int


class RunProgressResponse(BaseModel):
    """Response schema for progress polling."""
    run_id: int
    status: str
    overall_progress: float = Field(0.0, ge=0.0, le=1.0, description="Overall completion (0.0 to 1.0)")
    years: Dict[str, Dict[str, Any]] = Field(default_factory=dict, description="Per-year progress")
    eta_seconds: Optional[int] = Field(None, description="Estimated time to completion in seconds")


class StateInfo(BaseModel):
    """State configuration information."""
    code: str = Field(..., description="Two-letter state code (e.g., 'CA')")
    name: str = Field(..., description="Lowercase state name (e.g., 'california')")
    districts: int = Field(..., description="Number of congressional districts")
    fips: str = Field(..., description="Two-digit FIPS code (e.g., '06')")


class StateConfigResponse(BaseModel):
    """Response schema for state configuration."""
    year: str
    states: List[StateInfo]
