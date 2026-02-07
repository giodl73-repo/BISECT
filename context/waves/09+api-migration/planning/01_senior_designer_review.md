# Wave 9 Senior Designer Review

**Reviewer**: Senior Software Designer
**Date**: 2026-01-25
**Wave**: 9 - API Migration
**Status**: DESIGN REVIEW

---

## Executive Summary

Wave 9 proposes adding a FastAPI backend and React dashboard to the existing CLI-based redistricting pipeline. The goal is to make the system accessible to general users through a web interface while maintaining full backward compatibility with existing CLI tools.

**Overall Assessment**: The plan is architecturally sound with a clear separation of concerns. However, several design decisions warrant discussion and some risks need mitigation strategies.

**Recommendation**: **PROCEED WITH MODIFICATIONS** - The wave is well-scoped but would benefit from the structural improvements outlined below.

---

## 1. Architecture Review

### 1.1 Strengths

**Clean Separation of Concerns**
The proposed architecture correctly maintains the existing separation:
- `src/apportionment/` - Core library (untouched)
- `scripts/` - CLI tools (untouched)
- `api/` - New FastAPI backend (wrapper)
- `frontend/` - New React dashboard

This layered approach is correct - the API wraps the CLI, which uses the library. No existing code changes are required.

**Technology Stack**
The chosen technologies are appropriate:
- **FastAPI**: Modern, async-first, auto-docs, excellent for Python shops
- **React + TypeScript**: Industry standard, strong typing, good ecosystem
- **PostgreSQL**: Robust, JSON support for flexible schemas, good with SQLAlchemy
- **Tailwind CSS**: Rapid UI development, consistent styling

**Integration with Existing Tools**
The plan to integrate with the App Manager ecosystem (port 8002/3002) is smart - it provides consistent tooling across projects.

### 1.2 Concerns

**Subprocess Communication Complexity**
The plan relies heavily on parsing STATUS messages from subprocess stdout. This works but has limitations:
- **Buffering issues**: Python subprocess stdout can buffer unexpectedly
- **Error handling**: Subprocess crashes may not emit clean STATUS messages
- **Resource leaks**: Long-running processes need careful cleanup

**Recommendation**: Add a secondary communication channel via temporary files or a simple socket for heartbeat/progress. The STATUS protocol should be primary, with file-based progress as fallback.

**Database Overhead**
Storing all district data in PostgreSQL may be overkill given:
- Results already exist as Parquet/CSV files
- Geometries are large (MB per state)
- Query patterns are simple (filter by state/year/run)

**Recommendation**: Use database for run metadata and progress only. Store district data in existing file formats with database containing pointers to files. This avoids data duplication and leverages the existing efficient file formats.

**WebSocket Complexity**
WebSocket for progress streaming adds operational complexity:
- Connection management
- Reconnection handling
- State synchronization

**Recommendation**: Start with polling (every 2 seconds) for simplicity. Add WebSocket as E60 optimization if polling proves inadequate. Most pipeline runs are 1-4 hours - polling is sufficient.

### 1.3 Architectural Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Subprocess hangs | High | Medium | Add timeout + watchdog process |
| Database schema evolution | Medium | High | Design flexible JSON columns for extensibility |
| Frontend state sync | Medium | Medium | Use React Query with proper cache invalidation |
| Memory usage (large geometries) | Medium | Low | Stream geometries from files, don't load all to memory |

---

## 2. Design Patterns Analysis

### 2.1 Backend Patterns (Positive)

**Dependency Injection**
Using FastAPI's `Depends()` for database sessions is correct:
```python
@router.get("/{id}")
async def get_run(id: int, db: Session = Depends(get_db)):
```

**Pydantic Schemas**
Separating request/response schemas from ORM models is the right approach. This allows API stability independent of database schema changes.

**Service Layer**
The planned `services/` directory for business logic keeps routes thin and logic testable.

### 2.2 Backend Patterns (Needs Attention)

**Async vs Sync**
The existing pipeline is synchronous Python. Calling it from async FastAPI requires care:

```python
# WRONG - blocks event loop
@router.post("/runs")
async def create_run():
    subprocess.run([...])  # Blocks!

# CORRECT - use to_thread for CPU-bound
@router.post("/runs")
async def create_run():
    await asyncio.to_thread(subprocess.run, [...])

# BETTER - use asyncio subprocess
@router.post("/runs")
async def create_run():
    proc = await asyncio.create_subprocess_exec(...)
```

**Background Tasks**
For long-running pipeline execution, use `BackgroundTasks` or a proper task queue:

```python
@router.post("/runs")
async def create_run(
    run_create: RunCreate,
    background_tasks: BackgroundTasks,
    db: Session = Depends(get_db)
):
    run = run_service.create_run(db, run_create)
    background_tasks.add_task(execute_pipeline, run.id)
    return run
```

For true production, consider Celery or Redis Queue for:
- Task persistence across server restarts
- Worker scaling
- Task retry logic

### 2.3 Frontend Patterns (Positive)

**Container/Presentational Split**
The planned separation of data-fetching containers from presentational components is correct.

**Custom Hooks**
Using hooks like `useRuns()`, `useDistricts()` encapsulates data fetching logic well.

**TypeScript**
Strong typing will catch many bugs at compile time. The type definitions should mirror Pydantic schemas.

### 2.4 Frontend Patterns (Needs Attention)

**State Management**
The plan mentions React Query (TanStack Query) but doesn't specify global state management. Consider:

- **Server state**: React Query (correct)
- **Client state**: React Context or Zustand for:
  - Current selected run
  - Map visualization settings
  - User preferences

**Map Performance**
Rendering 435 district geometries with Leaflet can be slow. Patterns needed:
- Geometry simplification (reduce point count by 90%)
- Progressive loading (load visible states first)
- Canvas rendering (Leaflet.Canvas) for better performance

**Offline Support**
The existing static dashboard works offline. The new React dashboard should degrade gracefully:
- Cache district data in IndexedDB
- Show cached data when API unavailable
- Indicate stale data clearly

---

## 3. Integration Strategy Review

### 3.1 Pipeline Wrapping (Correct Approach)

The plan to wrap existing scripts without modification is correct:

```
API -> subprocess -> run_complete_redistricting.py -> existing pipeline
```

This provides:
- Zero changes to tested pipeline code
- Easy rollback if API fails
- CLI remains fully functional

### 3.2 STATUS Protocol Integration

The existing STATUS protocol is well-designed. The API needs:

1. **Parser**: Convert STATUS strings to structured data
2. **Aggregator**: Combine worker statuses into run progress
3. **Emitter**: Send to WebSocket/polling endpoint

Existing pattern:
```
STATUS:WORKER:2020:1:STATE:12/50:california:STAGE:3/7:district_maps
```

Needs to become:
```json
{
  "run_id": 1,
  "year": "2020",
  "workers": {
    "1": {
      "state": "california",
      "state_progress": "12/50",
      "stage": "district_maps",
      "stage_progress": "3/7"
    }
  }
}
```

### 3.3 Backward Compatibility Concerns

**CLI Must Remain Primary**
The CLI tools must continue to work identically. This means:
- No changes to argument parsing
- No changes to output formats
- No new dependencies required for CLI use

**Configuration Consistency**
The web UI will expose configuration options. These must map exactly to CLI arguments:
- `--version` -> version field
- `--year` -> years array
- `--workers` -> workers field
- `--dpi` -> dpi field
- `--partition-mode` -> partition_mode field

### 3.4 Result Storage Strategy

**Recommended Approach**: Hybrid

```
outputs/{version}/{year}/
  ├── data/                    # Existing: District CSVs, summaries
  ├── maps/                    # Existing: Generated PNGs
  ├── states/                  # Existing: Per-state outputs
  └── metadata.json            # NEW: Run metadata, pointers

database:
  runs (id, version, config, status, timestamps, metadata_path)
  districts (id, run_id, state, year, summary_metrics, file_path)
```

Benefits:
- No data duplication
- Existing file structure unchanged
- Database remains small and fast
- Can regenerate database from files if needed

---

## 4. Detailed Phase Feedback

### Phase 1: E58 - Project Setup

**Scope**: Correct. Setup and infrastructure first.

**Tasks Review**:
- Backend directory structure
- Frontend directory structure
- PostgreSQL connection
- PM2 configuration
- Common package integration

**Missing**:
- Docker Compose for local development (PostgreSQL + API + Frontend)
- Environment configuration (.env files)
- CORS configuration for development
- API versioning strategy (recommend /api/v1/)

**Recommendation**: Add Docker Compose setup as sub-task. This provides consistent development environment and simplifies onboarding.

### Phase 2: E59 - Database Schema & Core API

**Scope**: Appropriate but potentially large.

**Schema Concerns**:
1. The schema should be minimal initially:
   - `runs` table (metadata, config, status)
   - `run_years` table (per-year progress)
   - Defer `districts` table until Phase 3 (load from files)

2. Use JSON columns for flexibility:
   ```sql
   config JSONB,           -- Flexible configuration
   progress JSONB,         -- Flexible progress structure
   metrics JSONB           -- Flexible metrics storage
   ```

**API Endpoints**:
- CRUD for runs
- Progress endpoint (polling)
- Configuration endpoint (available options)

**Missing Endpoints**:
- Health check (`/health`)
- Version endpoint (`/version`)
- State configuration (`/api/v1/states/config`)

### Phase 2: E60 - Pipeline Integration

**Scope**: This is the most complex enhancement.

**Critical Path**:
1. Subprocess spawning with correct environment
2. STATUS message parsing and routing
3. Progress aggregation and storage
4. Completion handling (success/failure)
5. Cancellation support

**Recommended Sub-phases**:
1. Basic subprocess execution (no progress)
2. STATUS parsing to database
3. Progress endpoint
4. WebSocket streaming (optional)
5. Cancellation and cleanup

**Testing Strategy**:
- Unit tests for STATUS parser
- Integration tests with mock subprocess
- E2E tests with small state (VT)

### Phase 3: E61 - Frontend Core

**Scope**: Appropriate.

**Component Priority**:
1. Run list (table with filters)
2. Run detail (progress display)
3. Run create form
4. District table (sortable, filterable)

**Missing**:
- Navigation/layout component
- Error handling components
- Loading state components

**Recommendation**: Create `@/components/ui/` library first with: Button, Card, Table, Input, Select, Badge, Spinner, ErrorBanner.

### Phase 3: E62 - Interactive Map & Deployment

**Scope**: Potentially too large. Consider splitting.

**Map Complexity**:
- Leaflet integration: Medium
- GeoJSON rendering: Medium
- Alaska/Hawaii insets: High
- Color-by-metric: Medium
- Interactive tooltips: Low

**Recommendation**: Split into:
- **62a**: Basic map with district boundaries
- **62b**: Color themes and interactivity
- **62c**: Alaska/Hawaii insets (can be deferred)

**Deployment Tasks** (should be separate):
- PM2 configuration
- Testing
- Documentation

---

## 5. Enhancement Planning Recommendations

### Revised Enhancement Structure

**E58: Project Setup & Infrastructure** (12-16 hours)
- Directory structure (api/, frontend/)
- Docker Compose (postgres, api, frontend)
- Environment configuration
- PM2 ecosystem file
- Database connection and migrations
- CORS and API versioning
- Health endpoints

**E59: Run Management API** (16-20 hours)
- Database schema (runs, run_years)
- Alembic migrations
- Run CRUD endpoints
- State configuration endpoint
- Unit tests for all endpoints

**E60: Pipeline Execution Engine** (20-24 hours)
- Subprocess manager class
- STATUS protocol parser
- Progress aggregation service
- Background task execution
- Cancellation support
- Integration tests with VT

**E61: React Dashboard Core** (20-24 hours)
- Vite + TypeScript setup
- UI component library
- Run list page
- Run detail page with progress
- Run creation form
- API client with error handling

**E62: District Visualization** (16-20 hours)
- Leaflet map component
- GeoJSON district rendering
- Color-by-metric selection
- District tooltips
- District table with sorting
- Basic deployment (PM2)

**E62b (Optional/Future): Map Enhancements**
- Alaska/Hawaii insets
- Zoom to state
- Print/export
- Full deployment and documentation

### Estimated Total: 84-104 hours (vs original 72-88)

The increase accounts for:
- Docker setup (+4h)
- More thorough testing (+8h)
- UI component library (+4h)

---

## 6. Technical Recommendations

### 6.1 Database

```sql
-- Minimal viable schema
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

-- Per-year tracking
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

### 6.2 API Structure

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

### 6.3 Frontend Structure

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

### 6.4 Testing Requirements

**API Tests**:
- Unit: Service functions, parsers
- Integration: Endpoints with test database
- E2E: Full run lifecycle with VT

**Frontend Tests**:
- Unit: Utility functions, formatters
- Component: React Testing Library
- E2E: Playwright for critical flows

**Minimum Coverage**:
- 80% unit test coverage
- All API endpoints have integration tests
- Run creation -> completion flow has E2E test

---

## 7. Open Questions for Team

1. **Concurrent Runs**: Should the system support multiple simultaneous runs? The plan is unclear. Recommendation: Single active run initially, queue additional requests.

2. **User Authentication**: Is this needed? The plan doesn't mention it. Recommendation: Defer authentication to future wave if needed.

3. **Data Retention**: How long to keep run history? Recommendation: Keep metadata indefinitely, allow manual deletion of output files.

4. **Error Recovery**: What happens if server restarts during a run? Recommendation: Detect orphaned runs on startup, allow manual retry.

5. **Alaska/Hawaii Insets**: How important are these for MVP? Recommendation: Defer to post-MVP enhancement.

---

## 8. Success Criteria

### MVP (Minimum Viable Product)

1. User can create a new run via web form
2. User can see progress during execution
3. User can view completed run results (table)
4. User can see district map (basic, no AK/HI insets)
5. CLI tools work unchanged

### Full Wave Completion

1. All MVP criteria
2. Run cancellation works
3. Map has color-by-metric
4. Run history with filtering
5. PM2 deployment automated
6. 80% test coverage

---

## 9. Conclusion

Wave 9 is a well-conceived plan that adds significant value by making the redistricting system accessible to non-technical users. The architecture is sound, maintaining proper separation of concerns and backward compatibility.

**Key Improvements Recommended**:
1. Use polling before WebSocket (simpler)
2. Minimize database scope (metadata only, files for data)
3. Add Docker Compose for development
4. Split E62 (map is complex)
5. Create UI component library first

**Risk Mitigation**:
1. Add subprocess watchdog for hung processes
2. Use JSON columns for schema flexibility
3. Start with VT-only E2E tests for speed

The estimated timeline of 4-6 weeks is achievable with the revised enhancement structure. I recommend proceeding with the modifications outlined above.

---

**Signed**: Senior Software Designer
**Date**: 2026-01-25
