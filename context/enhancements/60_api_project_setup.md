# Enhancement 60: API Project Setup & Infrastructure

**Status**: PLANNED
**Wave**: Wave 9 (API-MIGRATION)
**Priority**: High
**Estimated Complexity**: Medium
**Estimated Hours**: 12-16 hours
**Created**: 2026-01-25

---

## Description

Set up the complete development infrastructure for the FastAPI backend and React frontend. This includes Docker Compose for local development, project scaffolding, database connection, and basic health endpoints. This enhancement creates the foundation that all subsequent Wave 9 enhancements build upon.

---

## Tasks

### Phase 1: Backend Directory Structure (3-4 hours)

- [ ] Create `api/` directory with FastAPI project structure
  ```
  api/
  ├── main.py              # FastAPI app factory, CORS, lifespan
  ├── config.py            # pydantic-settings configuration
  ├── database.py          # SQLAlchemy engine, session factory
  ├── models.py            # SQLAlchemy ORM models (placeholder)
  ├── schemas/
  │   ├── __init__.py
  │   └── common.py        # Shared schemas (health, errors)
  ├── routers/
  │   ├── __init__.py
  │   └── health.py        # /health, /version endpoints
  └── requirements.txt     # FastAPI, SQLAlchemy, pydantic-settings, etc.
  ```
- [ ] Implement `config.py` using pydantic-settings
  - Database URL from environment
  - CORS origins configuration
  - Debug mode flag
  - Version string from package
- [ ] Implement `database.py` with connection pool
  - Engine with pool_pre_ping
  - Session factory with proper cleanup
  - Dependency injection function (`get_db`)
- [ ] Implement basic `main.py`
  - FastAPI app with lifespan handler
  - CORS middleware configuration
  - Global exception handler
  - Router registration

### Phase 2: Frontend Directory Structure (2-3 hours)

- [ ] Create `frontend/` directory with Vite + TypeScript + Tailwind
  ```
  frontend/
  ├── src/
  │   ├── main.tsx         # Entry point
  │   ├── App.tsx          # Root component (placeholder routing)
  │   ├── api/
  │   │   └── client.ts    # Axios instance with base URL
  │   └── components/
  │       └── ui/
  │           └── .gitkeep # Placeholder for UI components
  ├── public/
  ├── index.html
  ├── package.json
  ├── tsconfig.json
  ├── vite.config.ts
  └── tailwind.config.js
  ```
- [ ] Configure Vite with API proxy for development
- [ ] Set up Tailwind CSS with base styles
- [ ] Configure TypeScript with strict mode
- [ ] Add ESLint and Prettier configuration

### Phase 3: Docker Compose Setup (3-4 hours)

- [ ] Create `docker-compose.yml` for local development
  ```yaml
  services:
    postgres:
      image: postgres:15
      environment:
        POSTGRES_USER: apportionment
        POSTGRES_PASSWORD: dev_password
        POSTGRES_DB: apportionment
      ports:
        - "5434:5432"
      volumes:
        - postgres_data:/var/lib/postgresql/data

    api:
      build: ./api
      ports:
        - "8002:8000"
      environment:
        DATABASE_URL: postgresql://apportionment:dev_password@postgres:5432/apportionment
      depends_on:
        - postgres
      volumes:
        - ./api:/app
        - ./outputs:/outputs
        - ./scripts:/scripts

    frontend:
      build: ./frontend
      ports:
        - "3002:3000"
      depends_on:
        - api
      volumes:
        - ./frontend:/app
        - /app/node_modules
  ```
- [ ] Create `api/Dockerfile` for development
- [ ] Create `frontend/Dockerfile` for development
- [ ] Add `.dockerignore` files
- [ ] Document Docker Compose workflow in README

### Phase 4: Environment Configuration (1-2 hours)

- [ ] Create `.env.example` files for both api/ and frontend/
- [ ] Configure environment variables
  - `DATABASE_URL` - PostgreSQL connection string
  - `CORS_ORIGINS` - Allowed origins for CORS
  - `DEBUG` - Debug mode flag
  - `VITE_API_URL` - Frontend API base URL
- [ ] Add `.env` to `.gitignore`
- [ ] Document environment setup

### Phase 5: PM2 Ecosystem Configuration (1-2 hours)

- [ ] Create `ecosystem.config.js` for production deployment
  ```javascript
  module.exports = {
    apps: [
      {
        name: 'apportionment-api',
        script: 'uvicorn',
        args: 'main:app --host 0.0.0.0 --port 8002',
        cwd: './api',
        interpreter: 'python',
      },
      {
        name: 'apportionment-frontend',
        script: 'npm',
        args: 'run preview -- --port 3002',
        cwd: './frontend',
      }
    ]
  };
  ```
- [ ] Add `start.bat` / `start.sh` convenience scripts
- [ ] Document PM2 deployment steps

### Phase 6: Health Endpoints (1-2 hours)

- [ ] Implement `/health` endpoint
  - Returns `{ "status": "healthy", "database": "connected" }`
  - Checks database connectivity
  - Returns 503 if database unavailable
- [ ] Implement `/version` endpoint
  - Returns `{ "version": "9.0.0", "api_version": "v1" }`
  - Includes build timestamp if available
- [ ] Implement `/api/v1/` prefix for all API routes
- [ ] Add OpenAPI documentation configuration

### Phase 7: Testing & Documentation (2-3 hours)

- [ ] Create `tests/api/test_health.py` - Health endpoint tests
- [ ] Create `tests/api/conftest.py` - Test fixtures (test database)
- [ ] Verify Docker Compose startup
- [ ] Verify database connectivity
- [ ] Verify CORS configuration
- [ ] Update documentation
  - README.md with setup instructions
  - CLAUDE.md with new development commands

---

## Architecture Changes

This enhancement creates new directories and files without modifying existing code.

**New Files**:
```
api/                        # FastAPI backend (new)
├── main.py
├── config.py
├── database.py
├── models.py
├── schemas/common.py
├── routers/health.py
├── requirements.txt
└── Dockerfile

frontend/                   # React frontend (new)
├── src/main.tsx
├── src/App.tsx
├── src/api/client.ts
├── package.json
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.js
└── Dockerfile

docker-compose.yml          # Local development
ecosystem.config.js         # PM2 production
.env.example               # Environment template
```

**Related DESIGN_PATTERNS.md Sections**:
- Section 1: Project Structure (API)
- Section 2: Database Session Pattern
- Section 4: Frontend Project Structure

---

## Testing Strategy

### Unit Tests (3-5 tests)
- `test_config_loads_from_environment()` - Config loading from environment variables
- `test_config_validation_errors()` - Invalid configuration handling
- `test_database_connection_pool()` - Database connectivity and pool settings
- `test_database_url_parsing()` - Database URL format validation

### Integration Tests (5-8 tests)
- `test_health_endpoint_returns_healthy()` - Health check
- `test_health_endpoint_database_down()` - Database failure handling
- `test_version_endpoint()` - Version info
- `test_cors_headers_present()` - CORS configuration
- `test_cors_preflight_request()` - CORS preflight returns correct headers

**Test Coverage Target**: 80% minimum

### Manual Testing
1. `docker-compose up` - Verify all services start
2. `curl http://localhost:8002/health` - Verify API health
3. `curl http://localhost:3002` - Verify frontend serves
4. Check API docs at `http://localhost:8002/docs`

### Recommended Test Additions (from Senior Tester)

```python
# Test configuration loading from environment
def test_config_loads_from_env():
    """Verify configuration reads from environment variables."""
    import os
    os.environ["DATABASE_URL"] = "postgresql://test:test@localhost/test"
    from api.config import Settings
    settings = Settings()
    assert "test" in settings.database_url

# Test CORS preflight requests
def test_cors_preflight_request(client):
    """Verify CORS preflight returns correct headers."""
    response = client.options('/api/v1/runs', headers={
        'Origin': 'http://localhost:3002',
        'Access-Control-Request-Method': 'POST',
    })
    assert response.headers['Access-Control-Allow-Origin'] == 'http://localhost:3002'
```

---

## Testing Assessment (from Senior Tester)

| Attribute | Value |
|-----------|-------|
| **Risk Rating** | LOW |
| **Original Assessment** | ADEQUATE |
| **Testing Priority** | 5 (lowest in wave) |
| **Recommended Effort** | 5% of total testing effort |

### Gap Analysis

| Test Type | Originally Proposed | Recommended | Gap |
|-----------|---------------------|-------------|-----|
| Unit | 0 | 3-5 | Add config validation tests |
| Integration | 3-5 | 5-8 | Add CORS preflight tests |
| E2E | 0 | 0 | Not needed for setup |

### Quality Gates for This Enhancement

- [ ] Health endpoint returns 200 with database status
- [ ] Version endpoint returns correct API version
- [ ] CORS preflight requests succeed from allowed origins
- [ ] Database connection pool is properly configured
- [ ] All configuration can be overridden via environment variables

---

## Dependencies

**Prerequisites**:
- Docker Desktop installed
- Node.js 18+ installed
- Python 3.11+ installed
- PostgreSQL client (optional, for direct DB access)

**Python Packages** (api/requirements.txt):
- fastapi>=0.100.0
- uvicorn[standard]>=0.23.0
- sqlalchemy>=2.0.0
- pydantic-settings>=2.0.0
- psycopg2-binary>=2.9.0
- alembic>=1.12.0
- pytest>=7.0.0
- httpx>=0.24.0 (for testing)

**Node Packages** (frontend/package.json):
- react>=18.2.0
- react-dom>=18.2.0
- typescript>=5.0.0
- vite>=5.0.0
- tailwindcss>=3.4.0
- axios>=1.6.0
- @tanstack/react-query>=5.0.0

**Blocks**: Enhancements 61, 62, 63, 64 (all subsequent Wave 9 work)

---

## Success Criteria

- [ ] `docker-compose up` starts all three services successfully
- [ ] `/health` returns 200 with database status
- [ ] `/version` returns API version information
- [ ] Frontend loads at `http://localhost:3002`
- [ ] API docs available at `http://localhost:8002/docs`
- [ ] CORS allows requests from frontend origin
- [ ] Environment variables properly configured
- [ ] PM2 ecosystem file works for production deployment
- [ ] All tests pass

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Docker networking issues on Windows | Document Windows-specific setup, test on Windows |
| Port conflicts (5434, 8002, 3002) | Make ports configurable via environment variables |
| Database connection pooling issues | Configure pool_pre_ping, proper connection limits |
| Node version incompatibilities | Document required Node version, use .nvmrc |

---

## Related Documentation

- [Wave 9 Plan](../waves/WAVE09-api-migration.md)
- [Design Patterns](../DESIGN_PATTERNS.md)
- [Senior Designer Review](../waves/wave09/01_senior_designer_review.md)
- [Senior Engineer Review](../waves/wave09/02_senior_engineer_review.md)
- [Senior Tester Review](../waves/wave09/03_senior_tester_review.md)
- [TESTING_PATTERNS.md](../TESTING_PATTERNS.md) - FastAPI test client setup patterns

---

## Engineering Notes (from Senior Engineer)

### Risk Assessment: LOW

This enhancement is well-scoped with minimal risk. Primary concerns are port conflicts and environment setup consistency.

### Docker Compose Configuration

Per engineer recommendation, the Docker Compose setup should include proper volume mounts and environment configuration:

```yaml
services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_USER: apportionment
      POSTGRES_PASSWORD: dev_password
      POSTGRES_DB: apportionment
    ports:
      - "5434:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U apportionment"]
      interval: 5s
      timeout: 5s
      retries: 5

  api:
    build: ./api
    ports:
      - "8002:8000"
    environment:
      DATABASE_URL: postgresql://apportionment:dev_password@postgres:5432/apportionment
      PYTHONUNBUFFERED: "1"  # Critical for subprocess output
    depends_on:
      postgres:
        condition: service_healthy
    volumes:
      - ./api:/app
      - ./outputs:/outputs
      - ./scripts:/scripts
      - ./src:/src  # For apportionment library access
```

### pydantic-settings Configuration Pattern

Use `pydantic-settings` for type-safe configuration:

```python
# api/config.py
from pydantic_settings import BaseSettings
from typing import List

class Settings(BaseSettings):
    # Database
    database_url: str = "postgresql://apportionment:dev@localhost:5434/apportionment"

    # CORS
    cors_origins: List[str] = ["http://localhost:3002"]

    # Debug
    debug: bool = False

    # Paths
    project_root: str = "."
    outputs_dir: str = "outputs"

    # Pipeline (for Enhancement 62)
    default_workers: int = 4
    watchdog_timeout: int = 60  # seconds
    progress_persist_interval: int = 5  # seconds

    class Config:
        env_file = ".env"
        env_file_encoding = "utf-8"

settings = Settings()
```

### Health Endpoint Implementation

Per engineer recommendation, include database connectivity check:

```python
# api/routers/health.py
from fastapi import APIRouter, Depends
from sqlalchemy.orm import Session
from sqlalchemy import text

router = APIRouter()

@router.get("/health")
async def health_check(db: Session = Depends(get_db)):
    try:
        # Verify database connectivity
        db.execute(text("SELECT 1"))
        db_status = "connected"
    except Exception:
        db_status = "disconnected"

    return {
        "status": "healthy" if db_status == "connected" else "degraded",
        "database": db_status,
        "timestamp": datetime.utcnow().isoformat()
    }

@router.get("/version")
async def version():
    return {
        "version": "9.0.0",
        "api_version": "v1",
        "wave": 9
    }
```

### Testing Strategy

**Testing Pyramid for Enhancement 60**:
- Unit tests: 2-3 tests (config loading, database connection)
- Integration tests: 3-4 tests (health endpoint, CORS, version endpoint)

Use `httpx` for async test client:

```python
# tests/api/conftest.py
import pytest
from httpx import AsyncClient

@pytest.fixture
async def client():
    async with AsyncClient(app=app, base_url="http://test") as client:
        yield client

# tests/api/test_health.py
@pytest.mark.asyncio
async def test_health_endpoint(client):
    response = await client.get("/health")
    assert response.status_code == 200
    assert response.json()["status"] in ["healthy", "degraded"]
```

### Development Tools

Per engineer recommendation:
- Use `ruff` for linting (faster than flake8)
- Use `uvicorn --reload` for hot-reload during development
- Use `pnpm` over npm for frontend (faster, better disk usage)

---

**Enhancement 60 Summary**: Create foundational infrastructure for Wave 9 with Docker Compose, FastAPI backend skeleton, React frontend scaffold, and health endpoints.
