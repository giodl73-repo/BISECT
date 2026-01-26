"""
Database connection and session management.

Uses SQLAlchemy 2.0 with proper connection pooling and dependency injection.
Integrates with common-backend-utils for shared patterns.
"""
from sqlalchemy import create_engine
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker
from typing import Generator
from sqlalchemy.orm import Session

from app.config import settings


# Create SQLAlchemy engine with connection pooling
# For testing, this will be overridden in conftest.py
try:
    engine = create_engine(
        settings.database_url,
        pool_pre_ping=True,  # Verify connections before using
        pool_size=5,
        max_overflow=10,
        echo=settings.debug,  # Log SQL queries in debug mode
    )
except Exception:
    # During tests or when database is not available, create a dummy engine
    # Tests will override this in conftest.py
    engine = create_engine("sqlite:///:memory:")

# Session factory
SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)

# Base class for ORM models
Base = declarative_base()


# Database dependency (pattern from common-backend-utils)
def get_db() -> Generator[Session, None, None]:
    """
    Database session dependency for FastAPI endpoints.

    Usage:
        @app.get("/endpoint")
        def endpoint(db: Session = Depends(get_db)):
            ...
    """
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()
