"""
SQLAlchemy models for redistricting pipeline runs.

Metadata-only storage - district data remains in file system (Parquet/CSV).
"""
from datetime import datetime
from enum import Enum
from typing import Optional

from sqlalchemy import (
    Column,
    Integer,
    String,
    Text,
    DateTime,
    ForeignKey,
    Index,
    JSON,
)
from sqlalchemy.orm import relationship
from sqlalchemy.sql import func

from app.database import Base


class RunStatus(str, Enum):
    """Run execution status."""
    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"


class Run(Base):
    """
    Pipeline run metadata.

    Stores configuration and progress, but not district data (stored as files).
    """
    __tablename__ = "runs"

    id = Column(Integer, primary_key=True, index=True)
    version = Column(String(50), nullable=False, index=True)
    status = Column(String(20), nullable=False, default=RunStatus.PENDING, index=True)

    # Flexible JSON columns for config and progress
    # Using JSON (works with both PostgreSQL and SQLite for tests)
    # PostgreSQL will automatically use JSONB in production
    config = Column(JSON, nullable=False)  # {years, states, workers, dpi, partition_mode}
    progress = Column(JSON, nullable=True)  # {years: {...}, overall_progress, eta_seconds}

    # Process tracking for orphan detection
    process_pid = Column(Integer, nullable=True, index=True)

    # Timestamps
    created_at = Column(DateTime(timezone=True), server_default=func.now(), index=True)
    started_at = Column(DateTime(timezone=True), nullable=True)
    completed_at = Column(DateTime(timezone=True), nullable=True)

    # Error tracking
    error_message = Column(Text, nullable=True)

    # Output location
    output_path = Column(String(255), nullable=True)

    # Relationships
    years = relationship("RunYear", back_populates="run", cascade="all, delete-orphan")

    def __repr__(self):
        return f"<Run(id={self.id}, version={self.version}, status={self.status})>"


class RunYear(Base):
    """
    Per-year progress tracking within a run.

    Allows detailed monitoring of multi-year pipeline execution.
    """
    __tablename__ = "run_years"

    id = Column(Integer, primary_key=True, index=True)
    run_id = Column(Integer, ForeignKey("runs.id", ondelete="CASCADE"), nullable=False, index=True)

    year = Column(String(4), nullable=False)
    status = Column(String(20), nullable=False, default=RunStatus.PENDING)

    # Progress tracking
    states_completed = Column(Integer, default=0)
    states_total = Column(Integer, default=50)
    current_stage = Column(String(50), nullable=True)

    # Timestamps
    started_at = Column(DateTime(timezone=True), nullable=True)
    completed_at = Column(DateTime(timezone=True), nullable=True)

    # Error tracking
    error_message = Column(Text, nullable=True)

    # Relationships
    run = relationship("Run", back_populates="years")

    def __repr__(self):
        return f"<RunYear(id={self.id}, run_id={self.run_id}, year={self.year}, status={self.status})>"


# Create indexes for common query patterns
Index("idx_run_years_run", RunYear.run_id)
Index("idx_runs_status", Run.status)
Index("idx_runs_version", Run.version)
Index("idx_runs_created_desc", Run.created_at.desc())
