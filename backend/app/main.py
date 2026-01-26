"""
FastAPI application entry point.

Creates the FastAPI app with CORS, exception handling, and router registration.
Integrates with common-backend-utils for shared exception handling.
"""
from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from contextlib import asynccontextmanager
from datetime import datetime

from common_backend_utils.exceptions import AppException, create_exception_handler

from app.config import settings
from app.database import engine, Base
from app.api.routes import health, runs


@asynccontextmanager
async def lifespan(app: FastAPI):
    """
    Application lifespan context manager.

    Handles startup and shutdown events.
    """
    # Startup: Create database tables (in production, use Alembic migrations)
    # For development, create tables automatically
    # TODO: Use Alembic migrations in production
    Base.metadata.create_all(bind=engine)

    yield

    # Shutdown: Close database connections
    engine.dispose()


# Create FastAPI application
app = FastAPI(
    title="Apportionment API",
    description="Congressional redistricting API for running and visualizing algorithmic redistricting",
    version=settings.app_version,
    lifespan=lifespan,
    docs_url="/docs",
    redoc_url="/redoc",
)


# Configure CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.cors_origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


# Register exception handler from common-backend-utils
@app.exception_handler(AppException)
async def app_exception_handler(request: Request, exc: AppException):
    """Handle AppException from common-backend-utils."""
    return JSONResponse(
        status_code=exc.status_code,
        content=exc.to_dict()
    )


# Global exception handler for unhandled errors
@app.exception_handler(Exception)
async def global_exception_handler(request: Request, exc: Exception):
    """
    Global exception handler for unhandled errors.

    Returns a consistent error response format.
    """
    return JSONResponse(
        status_code=500,
        content={
            "message": "Internal server error" if not settings.debug else str(exc),
            "code": "INTERNAL_ERROR",
            "timestamp": datetime.utcnow().isoformat(),
        },
    )


# Register routers
app.include_router(health.router, prefix="", tags=["Health"])
app.include_router(runs.router, prefix="/api/v1/runs", tags=["Runs"])


@app.get("/")
async def root():
    """Root endpoint with API information."""
    return {
        "message": "Apportionment API",
        "version": settings.app_version,
        "docs": "/docs",
        "health": "/health",
    }
