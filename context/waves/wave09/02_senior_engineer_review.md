# Wave 9 Senior Engineer Review

**Reviewer**: Senior Software Engineer
**Date**: 2026-01-25
**Wave**: 9 - API Migration
**Status**: ENGINEERING REVIEW

---

## Executive Summary

This review evaluates Wave 9 from an engineering perspective, focusing on implementation feasibility, code quality, integration challenges, and technical risks. The proposed architecture is sound, building on proven patterns from the existing CLI pipeline while introducing modern web technologies.

**Overall Assessment**: **PROCEED** - The technical plan is well-structured with appropriate complexity allocation. The Senior Designer's recommendations significantly de-risk the implementation.

**Key Engineering Verdict**:
- E60 (Setup): Low risk, well-scoped
- E61 (Run API): Medium risk, straightforward CRUD
- E62 (Pipeline Execution): **HIGH RISK** - Most complex, critical path
- E63 (React Dashboard): Medium risk, standard patterns
- E64 (Visualization): Medium risk, performance-sensitive

---

## 1. Technical Implementation Review

### 1.1 Technology Stack Assessment

**FastAPI (Backend)** - Excellent Choice
- Async-first aligns with subprocess management needs
- Automatic OpenAPI documentation reduces frontend integration friction
- Pydantic validation catches errors at API boundary
- Mature ecosystem with good SQLAlchemy integration

**React + TypeScript + Vite (Frontend)** - Excellent Choice
- TypeScript catches type mismatches early (API/frontend contract)
- Vite provides fast development iteration
- React Query (TanStack Query) handles server state elegantly
- Tailwind enables rapid UI development

**PostgreSQL (Database)** - Appropriate Choice
- JSONB columns provide schema flexibility (matches Senior Designer recommendation)
- Good indexing for query patterns (status, version, created_at)
- Overkill for just run metadata, but provides growth path if needed

**SQLAlchemy + Alembic (ORM/Migrations)** - Standard Choice
- Version-controlled schema changes critical for team development
- Session management patterns well-documented

### 1.2 Database Schema Analysis

The proposed schema is minimal and appropriate:

```sql
-- GOOD: Simple, indexed, flexible
runs (id, version, status, config JSONB, progress JSONB, timestamps, output_path)
run_years (id, run_id FK, year, status, states_completed, states_total, timestamps)
```

**Engineering Notes**:
1. **JSONB for config/progress** - Correct decision. Avoids migration churn as STATUS protocol evolves.
2. **No district storage** - Correct. District data is 10-100MB per state. Keep in Parquet/CSV files.
3. **output_path column** - Good. Links run to file system results.

**Missing consideration**: Add `process_pid` column to `runs` table for orphan detection on server restart:
```sql
ALTER TABLE runs ADD COLUMN process_pid INTEGER;
CREATE INDEX idx_runs_pid ON runs(process_pid) WHERE process_pid IS NOT NULL;
```

### 1.3 Subprocess Management Analysis

This is the **highest risk** component. The existing CLI uses:
- `subprocess.Popen` with line-buffered stdout
- STATUS protocol for progress (text-based, parse with regex)
- `ProcessMonitor` class with daemon threads
- `readline()` loop (NOT `for line in stdout:` which blocks)

**Critical Implementation Pattern** (from `pipeline_orchestrator.py`):
```python
# CORRECT - Non-blocking readline
while True:
    line = self.proc.stdout.readline()
    if not line:
        if self.proc.poll() is not None:
            break
        continue
    process(line)
```

**For FastAPI async context**, must use `asyncio.create_subprocess_exec`:
```python
# CORRECT - Async subprocess
proc = await asyncio.create_subprocess_exec(
    *command,
    stdout=asyncio.subprocess.PIPE,
    stderr=asyncio.subprocess.PIPE,
    env=env
)

# Read lines without blocking event loop
async for line in proc.stdout:
    if line.startswith(b"STATUS:"):
        await handle_status(line.decode())
```

**Potential Issue**: Python's `asyncio` subprocess readline can still buffer. The file-based progress fallback (Senior Designer recommendation) is **essential**.

### 1.4 Error Handling Analysis

**Current CLI error handling**:
- Return codes (0=success, non-zero=failure)
- Error messages to stderr
- `.states_complete` marker files for resumption

**API must handle**:
1. **Subprocess crash** - Process dies unexpectedly
2. **Timeout** - Process hangs (watchdog)
3. **Server restart** - Orphaned runs
4. **User cancellation** - SIGTERM gracefully, SIGKILL fallback

**Recommended error state machine**:
```
pending -> running -> completed
                   -> failed (subprocess error)
                   -> cancelled (user action)
                   -> timeout (watchdog killed)

On server start: running -> orphaned (allow retry)
```

---

## 2. Integration Engineering

### 2.1 CLI Integration Strategy

**Principle**: Zero changes to existing CLI scripts.

The API wraps:
```
scripts/pipeline/run_complete_redistricting.py --version {v} --year {y} --workers {w}
```

**Implementation**:
```python
# api/services/pipeline_service.py
def build_command(config: dict) -> list[str]:
    cmd = [
        sys.executable,
        str(Path("scripts/pipeline/run_complete_redistricting.py")),
        "--version", config["version"],
    ]
    for year in config["years"]:
        cmd.extend(["--year", year])
    if config.get("states"):
        cmd.extend(["--states", ",".join(config["states"])])
    cmd.extend([
        "--workers", str(config.get("workers", 4)),
        "--dpi", str(config.get("dpi", 150)),
        "--partition-mode", config.get("partition_mode", "edge-weighted"),
    ])
    return cmd
```

**Environment setup**:
```python
env = os.environ.copy()
env["TQDM_POSITION"] = "999"  # Suppress tqdm bars
env["MULTI_YEAR_SUBPROCESS"] = "1"  # Enable STATUS protocol
env["PYTHONUNBUFFERED"] = "1"  # Disable output buffering
```

### 2.2 STATUS Protocol Bridge

The existing `StatusReader` class in `scripts/utils/status_protocol.py` parses all message types. For the API:

```python
# api/utils/status_bridge.py
from scripts.utils.status_protocol import parse_status_message

class StatusBridge:
    """Bridge STATUS protocol to database/WebSocket."""

    def __init__(self, run_id: int, db: Session, ws_manager: Optional[WebSocketManager]):
        self.run_id = run_id
        self.db = db
        self.ws_manager = ws_manager
        self.progress = {}  # Accumulated progress

    async def process_line(self, line: str):
        msg_type, data = parse_status_message(line)
        if msg_type is None:
            return

        # Update progress structure
        self._update_progress(msg_type, data)

        # Persist to database (batch every N updates for performance)
        await self._persist_progress()

        # Broadcast to WebSocket clients (if implemented)
        if self.ws_manager:
            await self.ws_manager.broadcast(self.run_id, self.progress)

    def _update_progress(self, msg_type: str, data: dict):
        """Aggregate STATUS messages into progress structure."""
        year = data.get("year")

        if msg_type == "YEAR":
            self.progress.setdefault("years", {})[year] = {
                "states_completed": data["completed"],
                "states_total": data["total"],
                "status": "running"
            }

        elif msg_type == "WORKER":
            year_progress = self.progress.setdefault("years", {}).setdefault(year, {})
            workers = year_progress.setdefault("workers", {})
            workers[str(data["worker_id"])] = {
                "state": data["state_name"],
                "state_num": data["state_num"],
                "stage": f"{data['stage']}/{data['stage_total']}",
                "stage_name": data["stage_desc"]
            }

        # ... handle other message types
```

### 2.3 File-Based Progress Fallback

**Implementation** (per Senior Designer recommendation):

```python
# api/utils/file_progress.py
import json
import aiofiles
from pathlib import Path

class FileProgressManager:
    """Atomic file-based progress for reliability."""

    def __init__(self, outputs_dir: Path, version: str):
        self.progress_file = outputs_dir / version / "api_progress.json"
        self.progress_file.parent.mkdir(parents=True, exist_ok=True)

    async def write(self, progress: dict):
        """Atomic write: temp file + rename."""
        temp_file = self.progress_file.with_suffix(".tmp")
        async with aiofiles.open(temp_file, "w") as f:
            await f.write(json.dumps(progress, indent=2))
        temp_file.rename(self.progress_file)

    async def read(self) -> Optional[dict]:
        """Read progress, return None if file doesn't exist."""
        if not self.progress_file.exists():
            return None
        async with aiofiles.open(self.progress_file, "r") as f:
            return json.loads(await f.read())
```

### 2.4 Concurrent Pipeline Execution

**Initial scope**: Single active run at a time.

**Implementation**:
```python
# api/services/pipeline_manager.py
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
            self.executor = PipelineExecutor(run_id, config)
            asyncio.create_task(self._execute_and_cleanup(run_id))
            return True

    async def cancel_run(self, run_id: int) -> bool:
        async with self._lock:
            if self.active_run != run_id:
                raise NotFoundError("Run", run_id)

            if self.executor:
                await self.executor.cancel()
            return True

    async def _execute_and_cleanup(self, run_id: int):
        """Execute pipeline and clean up on completion."""
        try:
            await self.executor.run()
        finally:
            async with self._lock:
                self.active_run = None
                self.executor = None
```

---

## 3. Code Quality & Maintainability

### 3.1 Proposed Code Organization

**Backend** (`api/`):
```
api/
|-- main.py              # App factory, lifespan, middleware
|-- config.py            # pydantic-settings (DATABASE_URL, CORS, DEBUG)
|-- database.py          # Engine, session factory, get_db dependency
|-- models.py            # SQLAlchemy models (Run, RunYear)
|-- schemas/
|   |-- __init__.py
|   |-- run.py           # RunCreate, RunResponse, RunProgressResponse
|   |-- common.py        # Pagination, ErrorResponse
|-- routers/
|   |-- __init__.py
|   |-- health.py        # /health, /version
|   |-- runs.py          # /api/v1/runs CRUD + actions
|   |-- files.py         # /api/v1/files (serve maps/data)
|-- services/
|   |-- __init__.py
|   |-- run_service.py   # Run CRUD operations
|   |-- pipeline_service.py  # Build commands, validate config
|   |-- progress_service.py  # Progress aggregation, ETA
|-- workers/
|   |-- __init__.py
|   |-- executor.py      # PipelineExecutor (async subprocess)
|   |-- manager.py       # PipelineManager (singleton)
|-- utils/
|   |-- __init__.py
|   |-- status_bridge.py # STATUS protocol parser bridge
|   |-- file_progress.py # File-based progress fallback
|   |-- exceptions.py    # Custom HTTP exceptions
```

**Frontend** (`frontend/src/`):
```
frontend/src/
|-- main.tsx             # Entry, QueryClientProvider
|-- App.tsx              # Router, Layout
|-- api/
|   |-- client.ts        # Axios instance, interceptors
|   |-- runs.ts          # Run API functions
|   |-- districts.ts     # District API functions
|-- components/
|   |-- ui/              # Button, Card, Table, Input, Badge, Spinner, ErrorBanner
|   |-- layout/          # Layout, Navbar, Breadcrumbs
|-- features/
|   |-- runs/
|   |   |-- RunList.tsx
|   |   |-- RunDetail.tsx
|   |   |-- RunForm.tsx
|   |   |-- RunProgress.tsx
|   |   |-- hooks.ts     # useRuns, useRun, useCreateRun
|   |-- districts/
|       |-- DistrictsPage.tsx
|       |-- DistrictMap.tsx
|       |-- DistrictTable.tsx
|       |-- hooks.ts
|-- types/
|   |-- run.ts
|   |-- district.ts
|-- utils/
    |-- format.ts        # formatNumber, formatDuration, formatPartisanLean
```

### 3.2 Testing Strategy

**API Testing Pyramid**:

```
         /\
        /E2E\        3-5 tests (VT full run, cancellation)
       /------\
      /Integ   \     15-20 tests (endpoints with test DB)
     /----------\
    /    Unit    \   30-40 tests (services, parsers, utils)
   /--------------\
```

**Unit Test Examples** (`tests/api/unit/`):
```python
# test_status_bridge.py
def test_parse_year_message():
    bridge = StatusBridge(run_id=1, db=None, ws_manager=None)
    bridge.process_line_sync("STATUS:YEAR:2020:COMPLETE:24/50")
    assert bridge.progress["years"]["2020"]["states_completed"] == 24

def test_parse_worker_message():
    bridge = StatusBridge(run_id=1, db=None, ws_manager=None)
    bridge.process_line_sync("STATUS:WORKER:2020:1:STATE:12/50:california:STAGE:3/7:maps")
    assert bridge.progress["years"]["2020"]["workers"]["1"]["state"] == "california"
```

**Integration Test Examples** (`tests/api/integration/`):
```python
# test_runs_api.py
async def test_create_run(client, db):
    response = await client.post("/api/v1/runs", json={
        "version": "test_v1",
        "years": ["2020"],
        "states": ["VT"],
        "workers": 1
    })
    assert response.status_code == 201
    assert response.json()["status"] == "pending"

async def test_start_run(client, db):
    # Create run
    create_resp = await client.post("/api/v1/runs", json={...})
    run_id = create_resp.json()["id"]

    # Start run
    start_resp = await client.post(f"/api/v1/runs/{run_id}/actions/start")
    assert start_resp.status_code == 200
    assert start_resp.json()["status"] == "running"
```

**E2E Test Examples** (`tests/api/e2e/`):
```python
# test_full_pipeline.py
@pytest.mark.e2e
@pytest.mark.timeout(300)  # 5 minute timeout for VT
async def test_vt_full_pipeline(client, db):
    """End-to-end test with Vermont (smallest state)."""
    # Create and start
    run = await create_and_start_run(client, {
        "version": "e2e_test",
        "years": ["2020"],
        "states": ["VT"],
        "workers": 1
    })

    # Poll until complete
    while True:
        progress = await client.get(f"/api/v1/runs/{run['id']}/progress")
        if progress.json()["status"] in ("completed", "failed"):
            break
        await asyncio.sleep(5)

    # Verify completion
    assert progress.json()["status"] == "completed"

    # Verify output files exist
    output_path = Path(progress.json()["output_path"])
    assert (output_path / "2020" / "states" / "vermont" / "data").exists()
```

**Frontend Testing**:
```typescript
// RunTable.test.tsx
describe('RunTable', () => {
  it('renders runs correctly', () => {
    render(<RunTable runs={mockRuns} onRowClick={jest.fn()} />);
    expect(screen.getByText('v1')).toBeInTheDocument();
  });

  it('handles empty state', () => {
    render(<RunTable runs={[]} onRowClick={jest.fn()} />);
    expect(screen.getByText('No runs found')).toBeInTheDocument();
  });
});
```

### 3.3 Logging and Observability

**Logging Pattern** (FastAPI):
```python
import logging
from datetime import datetime

logger = logging.getLogger("apportionment.api")

# Structured logging
logger.info("Run started", extra={
    "run_id": run_id,
    "version": config["version"],
    "years": config["years"],
    "timestamp": datetime.utcnow().isoformat()
})

# In executor
logger.debug("STATUS message received", extra={
    "run_id": self.run_id,
    "msg_type": msg_type,
    "data": data
})
```

**Error Logging**:
```python
@app.exception_handler(Exception)
async def global_exception_handler(request: Request, exc: Exception):
    logger.error(
        "Unhandled exception",
        extra={
            "path": request.url.path,
            "method": request.method,
            "error_type": type(exc).__name__,
            "error_message": str(exc)
        },
        exc_info=True
    )
    return JSONResponse(status_code=500, content={"detail": "Internal server error"})
```

### 3.4 Configuration Management

```python
# api/config.py
from pydantic_settings import BaseSettings
from typing import List, Optional

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

    # Pipeline
    default_workers: int = 4
    watchdog_timeout: int = 60  # seconds
    progress_persist_interval: int = 5  # seconds

    class Config:
        env_file = ".env"
        env_file_encoding = "utf-8"

settings = Settings()
```

---

## 4. Performance & Scalability

### 4.1 Database Performance

**Query patterns and indexes**:
```sql
-- Primary queries
SELECT * FROM runs WHERE status = 'running';  -- idx_runs_status
SELECT * FROM runs WHERE version = 'v1' ORDER BY created_at DESC;  -- idx_runs_version, idx_runs_created_at
SELECT * FROM runs WHERE id = ?;  -- PRIMARY KEY

-- Pagination
SELECT * FROM runs ORDER BY created_at DESC LIMIT 50 OFFSET 0;  -- idx_runs_created_at
```

**Optimization**: For progress polling (every 2 seconds), use in-memory caching:
```python
# api/services/progress_service.py
from functools import lru_cache
from datetime import datetime, timedelta

class ProgressCache:
    def __init__(self, ttl_seconds: int = 1):
        self._cache = {}
        self._timestamps = {}
        self._ttl = timedelta(seconds=ttl_seconds)

    def get(self, run_id: int) -> Optional[dict]:
        if run_id in self._cache:
            if datetime.now() - self._timestamps[run_id] < self._ttl:
                return self._cache[run_id]
        return None

    def set(self, run_id: int, progress: dict):
        self._cache[run_id] = progress
        self._timestamps[run_id] = datetime.now()
```

### 4.2 File I/O Patterns

**Geometry files** (largest data):
- State GeoJSON: 1-50 MB each
- Total US: ~500 MB

**Optimization**:
1. **Simplify geometries** server-side (reduce 90% of points)
2. **Lazy loading** per state (don't load all 50 at once)
3. **HTTP caching** headers (ETag, Cache-Control)

```python
# api/routers/files.py
from fastapi.responses import FileResponse
import hashlib

@router.get("/geometry/{version}/{year}/{state}")
async def get_geometry(version: str, year: str, state: str):
    file_path = outputs_dir / version / year / "states" / state / "geometry.geojson"

    if not file_path.exists():
        raise HTTPException(404, "Geometry not found")

    # Generate ETag from file modification time
    mtime = file_path.stat().st_mtime
    etag = hashlib.md5(f"{file_path}:{mtime}".encode()).hexdigest()

    return FileResponse(
        file_path,
        media_type="application/geo+json",
        headers={
            "ETag": etag,
            "Cache-Control": "public, max-age=3600"  # Cache 1 hour
        }
    )
```

### 4.3 Memory Usage

**Pipeline execution memory**:
- The CLI pipeline manages its own memory (forked process)
- API server only stores progress dict (~10KB per active run)

**Map rendering memory** (frontend):
- 435 districts x ~1000 points each = ~4M coordinates
- At 8 bytes per coordinate = ~32MB
- Browser can handle this with proper GeoJSON handling

**Optimization for large datasets**:
```typescript
// Simplify on the fly if needed
import simplify from '@turf/simplify';

const simplifiedGeojson = simplify(geojson, { tolerance: 0.01 });
```

### 4.4 WebSocket Connection Management

**Initial approach**: Polling (per Senior Designer recommendation)
- 2 second interval
- Automatic stop when run completes
- ~1800 requests per hour per active run (manageable)

**If WebSocket needed later**:
```python
# api/utils/ws_manager.py
class WebSocketManager:
    def __init__(self):
        self.connections: Dict[int, Set[WebSocket]] = {}  # run_id -> connections

    async def connect(self, run_id: int, ws: WebSocket):
        await ws.accept()
        self.connections.setdefault(run_id, set()).add(ws)

    async def disconnect(self, run_id: int, ws: WebSocket):
        if run_id in self.connections:
            self.connections[run_id].discard(ws)

    async def broadcast(self, run_id: int, data: dict):
        if run_id not in self.connections:
            return

        dead = set()
        for ws in self.connections[run_id]:
            try:
                await ws.send_json(data)
            except:
                dead.add(ws)

        for ws in dead:
            self.connections[run_id].discard(ws)
```

### 4.5 React Rendering Performance

**435 districts on Leaflet map**:
```typescript
// Use Canvas renderer instead of SVG for better performance
import { MapContainer, TileLayer, GeoJSON } from 'react-leaflet';
import L from 'leaflet';

// Prefer canvas for many features
const renderer = L.canvas({ padding: 0.5 });

function DistrictMap({ districts }) {
  return (
    <MapContainer center={[39.8, -98.6]} zoom={4}>
      <TileLayer url="..." />
      <GeoJSON
        data={geojsonData}
        renderer={renderer}  // Use canvas
        style={getStyle}
      />
    </MapContainer>
  );
}
```

**Memoization for expensive computations**:
```typescript
const geojsonData = useMemo(() => ({
  type: 'FeatureCollection',
  features: districts.map(d => ({
    type: 'Feature',
    id: d.id,
    geometry: d.geometry,
    properties: d
  }))
}), [districts]);  // Only recompute when districts change
```

---

## 5. Risk Analysis

### 5.1 Technical Risks by Enhancement

| Enhancement | Risk Level | Key Risk | Mitigation |
|-------------|------------|----------|------------|
| 60 - Setup | Low | Port conflicts | Configurable ports via env |
| 61 - Run API | Medium | Schema evolution | JSONB columns, migrations |
| 62 - Execution | **High** | Subprocess hangs | Watchdog + file fallback |
| 63 - Dashboard | Medium | State sync bugs | React Query invalidation |
| 64 - Maps | Medium | Performance | Simplification, canvas |

### 5.2 E62 Deep Dive (Highest Risk)

**Risk 1: Subprocess stdout buffering**
- **Impact**: Progress updates delayed or lost
- **Likelihood**: Medium
- **Mitigation**:
  - `PYTHONUNBUFFERED=1` environment variable
  - File-based progress fallback (write every 5 seconds)
  - Read from file if stdout silent for >10 seconds

**Risk 2: Process hangs indefinitely**
- **Impact**: Run stuck in "running" state forever
- **Likelihood**: Medium
- **Mitigation**:
  - Watchdog task (check progress every 10 seconds)
  - Kill after 60 seconds of no progress
  - Mark run as "timeout" status

**Risk 3: Server restart during execution**
- **Impact**: Orphaned runs, inconsistent state
- **Likelihood**: Low
- **Mitigation**:
  - Store process PID in database
  - On startup, check for orphaned PIDs
  - Mark orphaned runs, allow manual retry

**Risk 4: Windows subprocess issues**
- **Impact**: Different signal handling (no SIGTERM)
- **Likelihood**: High (Windows is development target)
- **Mitigation**:
  - Use `process.terminate()` (cross-platform)
  - Test on Windows explicitly
  - Use `taskkill /F /PID` fallback on Windows

### 5.3 Integration Risk Matrix

| Integration Point | Components | Risk | Mitigation |
|-------------------|------------|------|------------|
| CLI <-> API | subprocess | Medium | Use exact same command as CLI |
| STATUS <-> Database | parsing | Low | Use existing StatusReader |
| API <-> Frontend | REST | Low | TypeScript types match Pydantic |
| Files <-> Database | paths | Low | Store absolute paths |
| Markers <-> State | files | Low | Check marker files in get_run |

---

## 6. Build Order Recommendation

### 6.1 Recommended Implementation Sequence

```
Week 1-2: E60 (Setup)
  |-- Docker Compose working (postgres, API skeleton)
  |-- Frontend scaffold (Vite + Tailwind + empty routes)
  |-- Health endpoints verified
  |-- CI/CD for tests

Week 2-3: E61 (Run API)
  |-- Database schema + migrations
  |-- Run CRUD endpoints
  |-- Progress polling endpoint
  |-- Unit + integration tests

Week 3-4: E62 (Execution Engine)  ** CRITICAL PATH **
  |-- PipelineExecutor class
  |-- StatusBridge integration
  |-- File-based fallback
  |-- Watchdog implementation
  |-- VT integration test

Week 4-5: E63 (React Dashboard)
  |-- UI component library
  |-- Run list + detail pages
  |-- Progress display with polling
  |-- Run creation form

Week 5-6: E64 (Visualization)
  |-- Leaflet map component
  |-- Color-by-metric
  |-- District table
  |-- PM2 deployment
  |-- E2E tests
```

### 6.2 Parallelization Opportunities

```
E60 ----+
                   |
E61 ----+---- E62
                   |
                   +---- E63 (can start UI while 62 in progress)
                         |
                         +---- E64 (after 63 complete)
```

Enhancements 63 and 62 can be developed in parallel by different developers:
- Developer A: E62 (backend execution)
- Developer B: E63 (frontend, mock API responses)

---

## 7. Tooling Recommendations

### 7.1 Development Tools

**Backend**:
- `uvicorn --reload` for development
- `pytest-asyncio` for async tests
- `pytest-cov` for coverage
- `httpx` for async test client
- `ruff` for linting (faster than flake8)

**Frontend**:
- `pnpm` over npm (faster, better disk usage)
- `@tanstack/react-query-devtools` for debugging
- `msw` for API mocking in tests
- `@playwright/test` for E2E

**Database**:
- `pgAdmin` or `DBeaver` for database inspection
- `alembic history` to visualize migrations

### 7.2 Debugging Approaches

**Subprocess debugging**:
```python
# Add verbose logging to executor
logger.debug(f"Command: {' '.join(command)}")
logger.debug(f"Environment: {json.dumps(env, indent=2)}")

# Capture stderr for debugging
proc = await asyncio.create_subprocess_exec(
    *command,
    stdout=asyncio.subprocess.PIPE,
    stderr=asyncio.subprocess.PIPE,  # Capture for debugging
)

# Log stderr
async for line in proc.stderr:
    logger.warning(f"[stderr] {line.decode().strip()}")
```

**Progress debugging**:
```python
# Add debug endpoint for raw progress
@router.get("/{run_id}/debug/progress")
async def debug_progress(run_id: int, db: Session = Depends(get_db)):
    """Debug endpoint: raw progress from file and database."""
    file_progress = await FileProgressManager.read()
    db_progress = run_service.get_run(db, run_id).progress
    return {
        "file": file_progress,
        "database": db_progress,
        "timestamp": datetime.utcnow().isoformat()
    }
```

### 7.3 Performance Benchmarks

**Target metrics**:
| Metric | Target | Measurement |
|--------|--------|-------------|
| API response time | <100ms | p99 latency |
| Progress poll overhead | <50ms | Including DB query |
| Map initial load | <3s | Time to first paint |
| Map interaction | 60fps | No jank on pan/zoom |

---

## 8. Documentation Updates Required

### 8.1 CLAUDE.md Updates

Add to "Common Commands":
```bash
# API Development
cd api && uvicorn main:app --reload --port 8002   # API dev server
cd frontend && pnpm dev                             # Frontend dev server
docker-compose up -d postgres                       # Database only
docker-compose up                                   # Full stack

# API Testing
pytest tests/api/ -v                                # All API tests
pytest tests/api/unit/ -v                           # Unit only
pytest tests/api/e2e/ -v --timeout=300              # E2E (slow)
```

Add to "Recent Changes":
```
- **2026-XX-XX**: Wave 9 - FastAPI backend and React dashboard
  - API: /api/v1/runs for run management
  - Frontend: http://localhost:3002
  - Docker Compose for local development
```

### 8.2 ARCHITECTURE.md Updates

See updated ARCHITECTURE.md file with new sections:
- API Layer Architecture
- Frontend Architecture
- Integration Patterns
- Deployment Architecture

### 8.3 CODING_PATTERNS.md Updates

See updated CODING_PATTERNS.md file with new sections:
- FastAPI Patterns
- SQLAlchemy Patterns
- React Patterns
- Async Subprocess Patterns

---

## 9. Conclusion

Wave 9 is technically sound and well-scoped. The Senior Designer's recommendations (polling over WebSocket, metadata-only database, file-based fallback) significantly reduce implementation risk.

**Key Success Factors**:
1. **E62 is critical path** - Start early, test thoroughly with VT
2. **STATUS protocol is proven** - Reuse existing parsing code
3. **File-based fallback is essential** - Don't rely solely on stdout
4. **Test on Windows early** - Subprocess behavior differs

**Estimated Timeline**: 4-6 weeks is achievable with the phased approach:
- Weeks 1-2: Infrastructure (Enhancements 60-61)
- Weeks 3-4: Execution engine (E62)
- Weeks 4-6: Frontend (Enhancements 63-64)

The wave maintains full backward compatibility with CLI tools and adds significant value by making the redistricting system accessible via web interface.

---

**Signed**: Senior Software Engineer
**Date**: 2026-01-25
