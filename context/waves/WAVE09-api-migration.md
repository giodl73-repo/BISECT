# Wave 9: API Migration

**Date**: 2026-01-25 (Planned)
**Focus**: Add FastAPI backend and React dashboard to existing pipeline without changing core functionality
**Status**: PLANNED
**Estimated Duration**: 4-6 weeks
**Enhancements**: 60, 61, 62, 63, 64
**Phases**:
- Phase 1: Enhancement 60 - Project Setup & Infrastructure (PLANNED)
- Phase 2: Enhancements 61, 62 - Backend API & Pipeline Integration (PLANNED)
- Phase 3: Enhancements 63, 64 - React Dashboard & Visualization (PLANNED)

---

## Goals

1. Create FastAPI backend wrapping existing Python pipeline
2. Set up PostgreSQL database for run metadata (not district data - use files)
3. Build React dashboard for running and viewing redistricting analyses
4. Integrate with App Manager system (port 8002 backend, 3002 frontend)
5. Enable web-based pipeline execution and monitoring
6. Provide interactive visualization of results
7. Maintain backward compatibility with existing CLI tools

---

## Success Metrics (Revised per Senior Designer and Tester)

### MVP (Minimum Viable Product)

| Metric | Target | Notes |
|--------|--------|-------|
| User can create run via web | Yes | Form with year, version, states, workers |
| Progress visible during execution | Yes | Polling-based (2 second intervals) |
| Completed run results viewable | Yes | District table with sorting/filtering |
| Basic district map | Yes | Leaflet, no AK/HI insets initially |
| CLI tools unchanged | Yes | Zero modifications to existing scripts |

### Full Wave Completion

| Metric | Target | Notes |
|--------|--------|-------|
| All MVP criteria | Yes | - |
| Run cancellation | Yes | Graceful subprocess termination |
| Color-by-metric maps | Yes | Compactness, partisan lean, demographics |
| Run history with filtering | Yes | By status, year, version |
| PM2 deployment automated | Yes | Ecosystem file for all services |
| Test coverage | 80%+ | Unit + integration + E2E with VT |

### Quality Gates (from Senior Tester)

| Gate | Threshold | Blocking |
|------|-----------|----------|
| Unit test coverage | 80% minimum | Yes |
| Integration test pass | 100% | Yes |
| E2E critical flows | 100% | Yes |
| API p99 latency | <100ms | Warning |
| Map load time | <3s | Warning |
| Security scan | No high/critical | Yes |

---

## Testing Strategy (from Senior Tester)

**Overall Testing Risk Assessment**: MEDIUM-HIGH

Enhancement 62 (Pipeline Execution Engine) is the **highest testing risk** and should receive disproportionate testing attention.

### Test Pyramid

```
         /\
        /E2E\        10-15 tests (critical flows, VT integration)
       /------\
      /  Integ  \     40-50 tests (API endpoints, component integration)
     /----------\
    /    Unit    \   70-90 tests (services, parsers, components, utilities)
   /--------------\
```

### Effort Allocation by Enhancement

| Enhancement | Risk Level | Testing Priority | Recommended Effort |
|------------|------------|------------------|-------------------|
| Enhancement 62 (Pipeline Execution) | **HIGHEST** | 1 | 40% of testing effort |
| Enhancement 61 (Run Management) | MEDIUM | 2 | 20% of testing effort |
| Enhancement 63 (React Dashboard) | MEDIUM | 3 | 20% of testing effort |
| Enhancement 64 (Visualization) | MEDIUM | 4 | 15% of testing effort |
| Enhancement 60 (Project Setup) | LOW | 5 | 5% of testing effort |

### Testing Timeline

**Estimated Total Testing Effort**: 40-50 hours (30-35% of total development time)

| Phase | Weeks | Testing Focus |
|-------|-------|---------------|
| Phase 1 (Setup) | 1-2 | Infrastructure tests, health endpoint tests |
| Phase 2 (Backend) | 2-4 | API unit/integration tests, pipeline execution tests |
| Phase 3 (Frontend) | 4-6 | Component tests, E2E tests, performance tests |

### Critical Test Scenarios (Must Pass Before Launch)

1. **Pipeline Execution**
   - [ ] Vermont single-year run completes successfully
   - [ ] Progress updates appear in API response
   - [ ] Cancellation stops subprocess within 10 seconds
   - [ ] File-based fallback works when stdout is delayed
   - [ ] Watchdog kills hung process after timeout

2. **API Reliability**
   - [ ] 100 concurrent API requests handled without errors
   - [ ] Database connection pool handles load
   - [ ] Error responses don't leak internal details

3. **Frontend Stability**
   - [ ] Run creation works end-to-end
   - [ ] Progress polling updates UI correctly
   - [ ] Error boundaries catch component failures
   - [ ] Works on latest Chrome, Firefox, Edge

---

## Test Coverage Requirements by Enhancement

| Enhancement | Unit Tests | Integration Tests | E2E Tests | Total |
|-------------|------------|-------------------|-----------|-------|
| Enhancement 60 | 3-5 | 5-8 | 0 | 8-13 |
| Enhancement 61 | 15-18 | 12-15 | 2-3 | 29-36 |
| Enhancement 62 | 25-30 | 12-15 | 3-5 | 40-50 |
| Enhancement 63 | 18-22 | 10-12 | 5-7 | 33-41 |
| Enhancement 64 | 10-12 | 8-10 | 5-7 | 23-29 |
| **Total** | **71-87** | **47-60** | **15-22** | **133-169** |

---

## Architecture Overview (Revised per Senior Designer)

**Backend (FastAPI - Port 8002)**:
- Wraps existing Python pipeline scripts via subprocess
- Stores run metadata only in PostgreSQL (not district data)
- Provides REST API for frontend (polling, not WebSocket initially)
- Uses asyncio subprocess for non-blocking execution
- Background tasks for pipeline execution

**Frontend (React - Port 3002)**:
- Built with Vite + TypeScript + Tailwind
- UI component library first (`@/components/ui/`)
- Uses React Query (TanStack Query) for server state
- Polling-based progress updates (2 second intervals)
- Leaflet for map visualization

**Database (PostgreSQL - Port 5434)**:
- Run metadata and progress only
- JSON columns for flexible config/progress storage
- File paths to existing Parquet/CSV outputs
- No geometry storage (too large, use files)

---

## Design Decisions (per Senior Designer Review)

### Polling over WebSocket
**Decision**: Start with polling (every 2 seconds) for progress updates.
**Rationale**: Simpler implementation, sufficient for 1-4 hour runs, easier debugging.
**Future**: Add WebSocket in future enhancement if polling proves inadequate.

### Metadata-Only Database
**Decision**: Store only run metadata in PostgreSQL, not district data.
**Rationale**:
- District data already exists as Parquet/CSV files
- Geometries are too large (MB per state)
- Avoids data duplication
- Database stays small and fast
- Can regenerate from files if needed

### File-Based Fallback for Progress
**Decision**: Add file-based progress as fallback to STATUS protocol parsing.
**Rationale**: Subprocess stdout can buffer unexpectedly, need reliable progress tracking.
**Implementation**: Write progress to `outputs/runs_progress.json` as secondary channel.

### UI Component Library First
**Decision**: Build reusable UI components before feature components.
**Components**: Button, Card, Table, Input, Select, Badge, Spinner, ErrorBanner.
**Rationale**: Ensures consistent styling, faster feature development.

### Deferred AK/HI Insets
**Decision**: Basic map first, Alaska/Hawaii insets in future enhancement.
**Rationale**: Inset positioning is complex, not required for MVP.

---

## Phases (Revised per Senior Designer)

### Phase 1: Enhancement 60 - Project Setup & Infrastructure
**Status**: PLANNED
**Estimated Effort**: 12-16 hours

Set up complete development environment with Docker Compose.

**Tasks**:
- Backend directory structure (api/)
- Frontend directory structure (frontend/)
- Docker Compose for local development (PostgreSQL + API + Frontend)
- Environment configuration (.env files)
- PM2 ecosystem configuration
- Database connection with SQLAlchemy
- CORS configuration for development
- API versioning (/api/v1/)
- Health check endpoints (/health, /version)

### Phase 2: Backend Implementation (Enhancements 61, 62)
**Status**: PLANNED
**Estimated Effort**: 36-44 hours

Database schema, API endpoints, and pipeline integration.

**Enhancement 61 - Run Management API** (16-20 hours):
- Database schema (runs, run_years tables only)
- Alembic migrations
- JSON columns for config and progress flexibility
- Run CRUD endpoints
- Progress polling endpoint
- State configuration endpoint
- Unit tests for all endpoints

**Enhancement 62 - Pipeline Execution Engine** (20-24 hours):
- Subprocess manager class (async execution)
- STATUS protocol parser (regex-based)
- File-based progress fallback
- Progress aggregation service
- Background task execution
- Cancellation support (SIGTERM with timeout)
- Watchdog for hung processes
- Integration tests with VT

### Phase 3: Frontend & Deployment (Enhancements 63, 64)
**Status**: PLANNED
**Estimated Effort**: 36-44 hours

Frontend components and production deployment.

**Enhancement 63 - React Dashboard Core** (20-24 hours):
- Vite + TypeScript + Tailwind setup
- UI component library (Button, Card, Table, etc.)
- Navigation/layout component
- Run list page with filters
- Run detail page with progress display
- Run creation form with validation
- API client with error handling
- Error boundaries and loading states
- React Query integration

**Enhancement 64 - District Visualization** (16-20 hours):
- Leaflet map component
- GeoJSON district rendering
- Color-by-metric selection (compactness, partisan, demographic)
- District tooltips with stats
- District table with sorting
- Basic deployment (PM2 ecosystem)
- E2E tests for critical flows

---

## Dependencies

**Prerequisites**:
- Completed Wave 8 (Wave Manager Improvements) - Complete
- Stable pipeline with all features
- PostgreSQL available (via Docker or system)
- PM2 process manager configured

**Blocking Issues**: None identified

---

## Risks & Mitigation (Revised per Senior Designer)

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Subprocess hangs | High | Medium | Add timeout + watchdog process, file-based heartbeat |
| Database schema evolution | Medium | High | Use JSON columns for flexibility, Alembic migrations |
| Frontend state sync | Medium | Medium | React Query with proper cache invalidation |
| Memory usage (large geometries) | Medium | Low | Stream from files, don't load all to memory |
| Buffering issues in subprocess | Medium | Medium | File-based progress fallback |
| Error recovery on server restart | Medium | Low | Detect orphaned runs on startup, allow manual retry |

---

## Open Questions (from Senior Designer)

1. **Concurrent Runs**: Single active run initially, queue additional requests
2. **User Authentication**: Deferred to future wave if needed
3. **Data Retention**: Keep metadata indefinitely, allow manual deletion of output files
4. **Error Recovery**: Detect orphaned runs on startup, allow manual retry
5. **Alaska/Hawaii Insets**: Deferred to post-MVP enhancement

---

## Technical Recommendations (from Senior Designer)

### Database Schema (Minimal Viable)

```sql
CREATE TABLE runs (
    id SERIAL PRIMARY KEY,
    version VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    config JSONB NOT NULL,
    progress JSONB,
    created_at TIMESTAMP DEFAULT NOW(),
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    error_message TEXT,
    output_path VARCHAR(255)
);

CREATE INDEX idx_runs_status ON runs(status);
CREATE INDEX idx_runs_version ON runs(version);

CREATE TABLE run_years (
    id SERIAL PRIMARY KEY,
    run_id INTEGER REFERENCES runs(id) ON DELETE CASCADE,
    year VARCHAR(4) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    states_completed INTEGER DEFAULT 0,
    states_total INTEGER DEFAULT 50,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    error_message TEXT
);

CREATE INDEX idx_run_years_run ON run_years(run_id);
```

### API Structure

```
api/
├── main.py              # App factory, middleware, exception handlers
├── config.py            # pydantic-settings configuration
├── database.py          # Engine, session factory
├── models.py            # SQLAlchemy models (simple, one file)
├── schemas/
│   ├── run.py           # Run schemas
│   └── common.py        # Shared schemas (pagination, errors)
├── routers/
│   ├── runs.py          # /api/v1/runs
│   ├── health.py        # /health, /version
│   └── files.py         # /api/v1/files (map/data serving)
├── services/
│   ├── run_service.py   # Run CRUD
│   └── pipeline.py      # Pipeline execution
└── workers/
    └── executor.py      # Subprocess management
```

### Frontend Structure

```
frontend/
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── api/
│   │   ├── client.ts    # Axios instance
│   │   └── runs.ts      # Run API
│   ├── components/
│   │   └── ui/          # Reusable: Button, Card, Table, etc.
│   ├── features/
│   │   ├── runs/        # Run list, detail, form
│   │   └── districts/   # District table, map
│   ├── hooks/
│   │   └── useApi.ts    # React Query hooks
│   └── types/
│       └── index.ts     # TypeScript interfaces
├── package.json
├── tsconfig.json
├── vite.config.ts
└── tailwind.config.js
```

---

## Related Enhancements

- [Enhancement 60](../enhancements/60_api_project_setup.md) - Project Setup & Infrastructure (planned)
- [Enhancement 61](../enhancements/61_run_management_api.md) - Run Management API (planned)
- [Enhancement 62](../enhancements/62_pipeline_execution_engine.md) - Pipeline Execution Engine (planned)
- [Enhancement 63](../enhancements/63_react_dashboard_core.md) - React Dashboard Core (planned)
- [Enhancement 64](../enhancements/64_district_visualization.md) - District Visualization (planned)

---

## Notes

- This wave maintains full backward compatibility with CLI tools
- Web interface is additive, not replacement
- Senior Designer review completed 2026-01-25 with PROCEED WITH MODIFICATIONS recommendation
- Estimated timeline of 4-6 weeks is achievable with revised enhancement structure

---

## Design Documents

- [Senior Designer Review](wave09/01_senior_designer_review.md) - Design review with recommendations
- [Senior Engineer Review](wave09/02_senior_engineer_review.md) - Engineering review with implementation details
- [Senior Tester Review](wave09/03_senior_tester_review.md) - Testing strategy and quality gates
- [Design Patterns](../DESIGN_PATTERNS.md) - API and frontend patterns for implementation

---

## Technical Assessment (from Senior Engineer)

### Risk Ratings by Enhancement

| Enhancement | Risk Level | Key Risk | Mitigation |
|-------------|------------|----------|------------|
| 60 - Setup | **Low** | Port conflicts | Configurable ports via env |
| 61 - Run API | **Medium** | Schema evolution | JSONB columns, migrations |
| 62 - Execution | **HIGH** | Subprocess hangs | Watchdog + file fallback |
| 63 - Dashboard | **Medium** | State sync bugs | React Query invalidation |
| 64 - Maps | **Medium** | Performance | Simplification, canvas |

### Enhancement 62 Deep Dive (Highest Risk)

**Risk 1: Subprocess stdout buffering**
- **Impact**: Progress updates delayed or lost
- **Likelihood**: Medium
- **Mitigation**: `PYTHONUNBUFFERED=1` env var + file-based progress fallback

**Risk 2: Process hangs indefinitely**
- **Impact**: Run stuck in "running" state forever
- **Likelihood**: Medium
- **Mitigation**: Watchdog task (check every 10s), kill after 60s of no progress

**Risk 3: Server restart during execution**
- **Impact**: Orphaned runs, inconsistent state
- **Likelihood**: Low
- **Mitigation**: Store process PID in database, detect orphaned PIDs on startup

**Risk 4: Windows subprocess issues**
- **Impact**: Different signal handling (no SIGTERM)
- **Likelihood**: High (Windows is development target)
- **Mitigation**: Use `process.terminate()`, test on Windows explicitly, `taskkill /F /PID` fallback

---

## Implementation Timeline (from Senior Engineer)

### Recommended 4-6 Week Build Order

```
Week 1-2: Enhancement 60 (Setup)
  |-- Docker Compose working (postgres, API skeleton)
  |-- Frontend scaffold (Vite + Tailwind + empty routes)
  |-- Health endpoints verified
  |-- CI/CD for tests

Week 2-3: Enhancement 61 (Run API)
  |-- Database schema + migrations
  |-- Run CRUD endpoints
  |-- Progress polling endpoint
  |-- Unit + integration tests

Week 3-4: Enhancement 62 (Execution Engine)  ** CRITICAL PATH **
  |-- PipelineExecutor class
  |-- StatusBridge integration
  |-- File-based fallback
  |-- Watchdog implementation
  |-- VT integration test

Week 4-5: Enhancement 63 (React Dashboard)
  |-- UI component library
  |-- Run list + detail pages
  |-- Progress display with polling
  |-- Run creation form

Week 5-6: Enhancement 64 (Visualization)
  |-- Leaflet map component
  |-- Color-by-metric
  |-- District table
  |-- PM2 deployment
  |-- E2E tests
```

---

## Parallelization Opportunities

Enhancements 62 and 63 can be developed concurrently by different developers:

```
Enhancement 60 ----+
                   |
Enhancement 61 ----+---- Enhancement 62 (Developer A: backend execution)
                   |
                   +---- Enhancement 63 (Developer B: frontend, mock API responses)
                         |
                         +---- Enhancement 64 (after 63 complete)
```

**Benefits**:
- Reduce calendar time from 6 weeks to 4-5 weeks
- Developer B can build UI with mock data while Developer A builds execution engine
- Integration testing happens in weeks 4-5

---

## Key Engineering Decisions (from Senior Engineer)

### 1. Single Active Run Pattern (PipelineManager Singleton)

**Rationale**: Simplifies state management, avoids resource contention.

```python
class PipelineManager:
    """Singleton managing active pipeline execution."""

    _instance = None
    _lock = asyncio.Lock()

    def __init__(self):
        self.active_run: Optional[int] = None
        self.executor: Optional[PipelineExecutor] = None

    async def start_run(self, run_id: int, config: dict) -> bool:
        async with self._lock:
            if self.active_run is not None:
                raise ConflictError(f"Run {self.active_run} already active")
            self.active_run = run_id
            # ...
```

### 2. STATUS Protocol Bridge Reusing Existing Code

**Rationale**: The existing `StatusReader` class already handles all message types.

```python
from scripts.utils.status_protocol import parse_status_message

class StatusBridge:
    """Bridge STATUS protocol to database/WebSocket."""

    async def process_line(self, line: str):
        msg_type, data = parse_status_message(line)
        if msg_type is None:
            return
        self._update_progress(msg_type, data)
        await self._persist_progress()
```

### 3. Windows Signal Handling Considerations

**Issue**: Windows doesn't support SIGTERM like Unix.
**Solution**: Use `process.terminate()` (cross-platform) with `taskkill /F /PID` fallback.

### 4. File-Based Progress with Atomic Writes

**Rationale**: Python subprocess stdout can buffer unexpectedly.

```python
async def write_progress(self, progress: dict):
    """Atomic write: temp file + rename."""
    temp_file = self.progress_file.with_suffix(".tmp")
    async with aiofiles.open(temp_file, "w") as f:
        await f.write(json.dumps(progress, indent=2))
    temp_file.rename(self.progress_file)
```

### 5. Process PID Tracking for Orphan Detection

**Recommendation**: Add `process_pid` column to `runs` table:
```sql
ALTER TABLE runs ADD COLUMN process_pid INTEGER;
CREATE INDEX idx_runs_pid ON runs(process_pid) WHERE process_pid IS NOT NULL;
```

---

## Development Tooling (from Senior Engineer)

### Backend Tools
- `uvicorn --reload` for development
- `pytest-asyncio` for async tests
- `pytest-cov` for coverage
- `httpx` for async test client
- `ruff` for linting (faster than flake8)

### Frontend Tools
- `pnpm` over npm (faster, better disk usage)
- `@tanstack/react-query-devtools` for debugging
- `msw` for API mocking in tests
- `@playwright/test` for E2E

### Database Tools
- `pgAdmin` or `DBeaver` for database inspection
- `alembic history` to visualize migrations

### Performance Benchmarks (Target Metrics)

| Metric | Target | Measurement |
|--------|--------|-------------|
| API response time | <100ms | p99 latency |
| Progress poll overhead | <50ms | Including DB query |
| Map initial load | <3s | Time to first paint |
| Map interaction | 60fps | No jank on pan/zoom |

---

**Wave 9 Summary**: Transform apportionment project from CLI-only to full web application with interactive dashboard and real-time pipeline execution, using polling-based progress updates and file-backed data storage.
