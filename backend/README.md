# Apportionment Backend API

FastAPI backend for the Congressional Redistricting project, integrated with the unified App Manager system.

## Quick Start

### Development (via PM2 - Recommended)

The backend is managed by the centralized PM2 ecosystem in the App Manager:

```bash
# From C:\src\appmanager
pm2 start infrastructure/ecosystem.config.js --only apportionment-backend

# Or start all services
pm2 start infrastructure/ecosystem.config.js
```

### Local Development (Standalone)

```bash
# Install dependencies
cd backend
poetry install

# Set environment variables
cp .env.example .env

# Run server
poetry run uvicorn app.main:app --reload --port 8002
```

## Integration with App Manager

This backend uses shared packages from the App Manager system:

**Shared Python Package**:
- `common-backend-utils` - Exception handling, database patterns, pagination
  - Located: `C:\src\appmanager\packages\common-backend-utils`
  - Installed as path dependency in `pyproject.toml`

**Exceptions**:
```python
from common_backend_utils.exceptions import NotFoundException, ValidationException, ConflictException
```

**Database**:
```python
from common_backend_utils.deps import create_db_dependency
get_db = create_db_dependency(SessionLocal)
```

## API Documentation

Once running, view the interactive API documentation:
- Swagger UI: http://localhost:8002/docs
- ReDoc: http://localhost:8002/redoc

## Endpoints

### Health & Version
- `GET /health` - Health check with database status
- `GET /version` - API version information
- `GET /` - Root endpoint with API info

### API v1 (Coming in Enhancement 61)
- `POST /api/v1/runs` - Create new redistricting run
- `GET /api/v1/runs` - List all runs
- `GET /api/v1/runs/{id}` - Get run details
- `PATCH /api/v1/runs/{id}` - Update run (cancel, etc.)

## Testing

```bash
# Run all tests
poetry run pytest

# Run with coverage
poetry run pytest --cov=app --cov-report=html

# Run specific test file
poetry run pytest tests/test_health.py
```

## Project Structure

```
backend/
├── app/
│   ├── main.py              # FastAPI app factory, CORS, lifespan
│   ├── config.py            # pydantic-settings configuration
│   ├── database.py          # SQLAlchemy engine, session factory
│   ├── models/              # SQLAlchemy ORM models
│   ├── schemas/             # Pydantic request/response models
│   ├── services/            # Business logic layer
│   └── api/routes/          # API route handlers
├── alembic/                 # Database migrations
├── tests/                   # Pytest tests
└── pyproject.toml           # Poetry dependencies
```

## Configuration

Configuration is managed via environment variables using pydantic-settings.

See `.env.example` for all available options.

Key settings:
- `DATABASE_URL` - PostgreSQL connection string
- `CORS_ORIGINS` - Allowed CORS origins (JSON list)
- `ENVIRONMENT` - development/production
- `DEFAULT_WORKERS` - Default parallelism for runs

## Database

### Migrations (Enhancement 61)

```bash
# Create migration
poetry run alembic revision --autogenerate -m "description"

# Apply migrations
poetry run alembic upgrade head

# Rollback migration
poetry run alembic downgrade -1
```

## Wave 9 Enhancements

This API is built across multiple enhancements:
- **Enhancement 60**: Project setup (current) ✓
- **Enhancement 61**: Run management API
- **Enhancement 62**: Pipeline execution engine
- **Enhancement 63**: React dashboard integration
- **Enhancement 64**: District visualization

## PM2 Integration

This backend is registered in the centralized PM2 ecosystem at:
`C:\src\appmanager\infrastructure\ecosystem.config.js`

Configuration:
- **Name**: `apportionment-backend`
- **Port**: 8002
- **Database**: PostgreSQL on port 5434
- **Logs**: `C:\src\appmanager\infrastructure\logs/apportionment-backend-*.log`
