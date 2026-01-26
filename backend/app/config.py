"""
Configuration for Apportionment FastAPI backend.

Uses pydantic-settings for type-safe configuration from environment variables.
"""
from pydantic_settings import BaseSettings
from typing import List


class Settings(BaseSettings):
    """Application settings loaded from environment variables."""

    # Database
    database_url: str = "postgresql://apportionment_user:apportionment_pass@localhost:5434/apportionment_db"

    # CORS
    cors_origins: List[str] = ["http://localhost:3002", "http://localhost:9000"]

    # Debug
    debug: bool = False
    environment: str = "development"
    log_level: str = "info"

    # Paths
    project_root: str = "."
    outputs_dir: str = "outputs"

    # API
    api_version: str = "v1"
    app_version: str = "9.0.0"

    # Pipeline execution settings (for Enhancement 62)
    default_workers: int = 4
    watchdog_timeout: int = 60  # seconds
    progress_persist_interval: int = 5  # seconds
    max_concurrent_runs: int = 1  # Single run initially

    class Config:
        env_file = ".env"
        env_file_encoding = "utf-8"


# Global settings instance
settings = Settings()
