# Enhancement 61: Run Management API

**Status**: PLANNED
**Wave**: Wave 9 (API-MIGRATION)
**Priority**: High
**Estimated Complexity**: Medium
**Estimated Hours**: 16-20 hours
**Created**: 2026-01-25

---

## Description

Implement the database schema and core API endpoints for managing redistricting pipeline runs. This includes creating runs, listing runs with filters, viewing run details, and tracking per-year progress. Following the Senior Designer's recommendation, the database stores metadata only - district data remains in existing file formats (Parquet/CSV).

---

## Tasks

### Phase 1: Database Schema Design (3-4 hours)

- [ ] Create minimal viable schema following Senior Designer recommendation
  ```sql
  -- Run metadata (not district data)
  CREATE TABLE runs (
      id SERIAL PRIMARY KEY,
      version VARCHAR(50) NOT NULL,
      status VARCHAR(20) NOT NULL DEFAULT 'pending',
      config JSONB NOT NULL,           -- Flexible: years, states, workers, dpi, partition_mode
      progress JSONB,                   -- Flexible: per-year/worker progress
      created_at TIMESTAMP DEFAULT NOW(),
      started_at TIMESTAMP,
      completed_at TIMESTAMP,
      error_message TEXT,
      output_path VARCHAR(255)          -- Path to outputs/{version}/
  );

  CREATE INDEX idx_runs_status ON runs(status);
  CREATE INDEX idx_runs_version ON runs(version);
  CREATE INDEX idx_runs_created_at ON runs(created_at DESC);

  -- Per-year tracking for detailed progress
  CREATE TABLE run_years (
      id SERIAL PRIMARY KEY,
      run_id INTEGER REFERENCES runs(id) ON DELETE CASCADE,
      year VARCHAR(4) NOT NULL,
      status VARCHAR(20) NOT NULL DEFAULT 'pending',
      states_completed INTEGER DEFAULT 0,
      states_total INTEGER DEFAULT 50,
      current_stage VARCHAR(50),
      started_at TIMESTAMP,
      completed_at TIMESTAMP,
      error_message TEXT
  );

  CREATE INDEX idx_run_years_run ON run_years(run_id);
  ```
- [ ] Implement SQLAlchemy models in `api/models.py`
  - `Run` model with JSON columns
  - `RunYear` model with relationship
  - RunStatus enum
- [ ] Create Alembic migration for initial schema
  - `alembic init alembic`
  - Create migration script
  - Test upgrade/downgrade

### Phase 2: Pydantic Schemas (2-3 hours)

- [ ] Create request schemas in `api/schemas/run.py`
  ```python
  class RunCreate(BaseModel):
      version: str = Field(..., min_length=1, max_length=50)
      years: List[str] = Field(..., min_items=1)
      states: Optional[List[str]] = None  # None = all states
      workers: int = Field(4, ge=1, le=16)
      dpi: int = Field(150, ge=72, le=600)
      partition_mode: str = Field("edge-weighted")
  ```
- [ ] Create response schemas
  ```python
  class RunResponse(BaseModel):
      id: int
      version: str
      status: RunStatus
      years: List[str]
      states: Optional[List[str]]
      created_at: datetime
      started_at: Optional[datetime]
      completed_at: Optional[datetime]
      error_message: Optional[str]

      class Config:
          from_attributes = True

  class RunDetailResponse(RunResponse):
      config: dict
      progress: Optional[dict]
      year_details: List[RunYearResponse]
      duration_seconds: Optional[int]

  class RunListResponse(BaseModel):
      runs: List[RunResponse]
      total: int
      limit: int
      offset: int
  ```
- [ ] Create filter schemas for list queries

### Phase 3: Run Service Layer (3-4 hours)

- [ ] Implement `api/services/run_service.py`
  ```python
  def create_run(db: Session, run_create: RunCreate) -> Run:
      """Create a new run with pending status."""

  def get_run(db: Session, run_id: int) -> Optional[Run]:
      """Get run by ID with year details."""

  def list_runs(
      db: Session,
      status: Optional[str] = None,
      year: Optional[str] = None,
      version: Optional[str] = None,
      limit: int = 50,
      offset: int = 0
  ) -> Tuple[List[Run], int]:
      """List runs with filtering and pagination."""

  def update_run_status(db: Session, run_id: int, status: RunStatus) -> Run:
      """Update run status with timestamp."""

  def update_run_progress(db: Session, run_id: int, progress: dict) -> Run:
      """Update run progress from STATUS messages."""

  def delete_run(db: Session, run_id: int) -> bool:
      """Delete run (metadata only, not output files)."""
  ```
- [ ] Implement year-level progress tracking
- [ ] Add query optimization (eager loading, indexes)

### Phase 4: Run CRUD Endpoints (4-5 hours)

- [ ] Implement `api/routers/runs.py`
  ```python
  @router.post("", response_model=RunResponse, status_code=201)
  async def create_run(run_create: RunCreate, db: Session = Depends(get_db)):
      """Create a new pipeline run."""

  @router.get("", response_model=RunListResponse)
  async def list_runs(
      status: Optional[str] = Query(None),
      year: Optional[str] = Query(None),
      version: Optional[str] = Query(None),
      limit: int = Query(50, ge=1, le=100),
      offset: int = Query(0, ge=0),
      db: Session = Depends(get_db)
  ):
      """List all runs with optional filtering."""

  @router.get("/{run_id}", response_model=RunDetailResponse)
  async def get_run(run_id: int, db: Session = Depends(get_db)):
      """Get detailed run information including progress."""

  @router.delete("/{run_id}", status_code=204)
  async def delete_run(run_id: int, db: Session = Depends(get_db)):
      """Delete run metadata (not output files)."""
  ```
- [ ] Add proper error responses (404, 409, 422)
- [ ] Register router in main.py with /api/v1/runs prefix

### Phase 5: Progress Polling Endpoint (2-3 hours)

- [ ] Implement polling-based progress endpoint
  ```python
  @router.get("/{run_id}/progress", response_model=RunProgressResponse)
  async def get_run_progress(run_id: int, db: Session = Depends(get_db)):
      """Get current run progress (for polling)."""
      # Returns:
      # {
      #   "run_id": 1,
      #   "status": "running",
      #   "overall_progress": 0.45,
      #   "years": {
      #     "2020": {"status": "running", "states_completed": 24, "states_total": 50},
      #     "2010": {"status": "pending", "states_completed": 0, "states_total": 50}
      #   },
      #   "eta_seconds": 3600
      # }
  ```
- [ ] Add ETA calculation based on progress rate
- [ ] Optimize for frequent polling (2 second intervals)

### Phase 6: State Configuration Endpoint (2-3 hours)

- [ ] Implement state configuration endpoint
  ```python
  @router.get("/config/states", response_model=StateConfigResponse)
  async def get_state_config(year: str = Query("2020")):
      """Get state configuration for a census year."""
      # Returns:
      # {
      #   "year": "2020",
      #   "states": [
      #     {"code": "AL", "name": "alabama", "districts": 7, "fips": "01"},
      #     {"code": "AK", "name": "alaska", "districts": 1, "fips": "02"},
      #     ...
      #   ]
      # }
  ```
- [ ] Load configuration from existing `scripts/config_{year}.py`
- [ ] Support all three census years (2000, 2010, 2020)

### Phase 7: Testing (3-4 hours)

- [ ] Unit tests for run service
  ```python
  def test_create_run():
      """Test run creation with valid data."""

  def test_create_run_validation():
      """Test validation errors for invalid data."""

  def test_list_runs_with_filters():
      """Test filtering by status, year, version."""

  def test_update_run_progress():
      """Test progress update from STATUS messages."""
  ```
- [ ] Integration tests for API endpoints
  ```python
  def test_create_run_endpoint(client):
      response = client.post("/api/v1/runs", json={...})
      assert response.status_code == 201

  def test_list_runs_pagination(client):
      # Create 25 runs
      response = client.get("/api/v1/runs?limit=10&offset=10")
      assert len(response.json()["runs"]) == 10
  ```
- [ ] Test database migrations (upgrade/downgrade)

---

## Architecture Changes

**New Files**:
```
api/
├── models.py                    # SQLAlchemy models (Run, RunYear)
├── schemas/
│   └── run.py                   # Pydantic schemas
├── services/
│   └── run_service.py           # Business logic
├── routers/
│   └── runs.py                  # API endpoints
└── alembic/
    └── versions/
        └── 001_initial_schema.py
```

**Modified Files**:
```
api/main.py                      # Register runs router
```

**Database Tables**:
- `runs` - Run metadata and configuration
- `run_years` - Per-year progress tracking

**Related DESIGN_PATTERNS.md Sections**:
- Section 2: REST Endpoint Design
- Section 3: Request/Response Schemas
- Section 7: Database Session Pattern
- Database Patterns: Schema Design, Query Optimization

---

## Testing Strategy

**Test Coverage Target**: 80% minimum

### Unit Tests (15-20 tests)
- Run service CRUD operations
- Schema validation (valid/invalid data)
- Progress calculation
- ETA estimation
- State configuration loading
- Schema validation edge cases (empty strings, too-long values, empty arrays)

### Integration Tests (12-15 tests)
- All CRUD endpoints (create, read, list, delete)
- Filtering and pagination
- Pagination boundary conditions (empty results, last page, invalid offset)
- Progress polling
- Error responses (404, 409, 422)
- Database migrations
- Concurrent access tests (race conditions)
- Status transition validation

### E2E Tests (2-3 tests)
- API contract tests
- Full run lifecycle through API

### Manual Testing
1. Create run via API: `POST /api/v1/runs`
2. List runs with filters: `GET /api/v1/runs?status=running`
3. Get run details: `GET /api/v1/runs/{id}`
4. Poll progress: `GET /api/v1/runs/{id}/progress`
5. Verify OpenAPI docs at `/docs`

### Test Effort Estimate
- Unit tests: 15-18 tests
- Integration tests: 12-15 tests
- Total estimated effort: 20% of wave testing time

---

## Testing Assessment (from Senior Tester)

| Attribute | Value |
|-----------|-------|
| **Risk Rating** | MEDIUM |
| **Original Assessment** | ADEQUATE with gaps |
| **Testing Priority** | 2 |
| **Recommended Effort** | 20% of total testing effort |

### Gap Analysis

| Test Type | Originally Proposed | Recommended | Gap |
|-----------|---------------------|-------------|-----|
| Unit | 10-15 | 15-18 | Add schema validation edge cases |
| Integration | 8-10 | 12-15 | Add concurrent access tests |
| E2E | 0 | 2-3 | Add API contract tests |

### Testing Gaps Identified (from Senior Tester)

1. **Missing: Concurrent Access Tests**
   ```python
   @pytest.mark.asyncio
   async def test_concurrent_run_creation():
       """Test creating multiple runs simultaneously."""
       tasks = [
           client.post('/api/v1/runs', json={'version': f'v{i}', 'years': ['2020']})
           for i in range(10)
       ]
       responses = await asyncio.gather(*tasks)
       assert all(r.status_code == 201 for r in responses)
       assert len(set(r.json()['id'] for r in responses)) == 10  # All unique IDs
   ```

2. **Missing: Pagination Edge Cases**
   ```python
   def test_pagination_boundary_conditions(client):
       """Test pagination at boundaries."""
       # Create 25 runs
       for i in range(25):
           run_factory(version=f'v{i}')

       # Test offset beyond total
       response = client.get('/api/v1/runs?offset=100')
       assert response.json()['runs'] == []
       assert response.json()['total'] == 25

       # Test limit exceeds remaining
       response = client.get('/api/v1/runs?offset=20&limit=50')
       assert len(response.json()['runs']) == 5
   ```

3. **Missing: Schema Validation Edge Cases**
   ```python
   @pytest.mark.parametrize('invalid_data,expected_error', [
       ({'version': '', 'years': ['2020']}, 'version'),  # Empty string
       ({'version': 'a' * 100, 'years': ['2020']}, 'version'),  # Too long
       ({'version': 'v1', 'years': []}, 'years'),  # Empty years
       ({'version': 'v1', 'years': ['1999']}, 'years'),  # Invalid year
       ({'version': 'v1', 'years': ['2020'], 'workers': 0}, 'workers'),  # Zero workers
       ({'version': 'v1', 'years': ['2020'], 'workers': 100}, 'workers'),  # Too many workers
   ])
   def test_validation_edge_cases(client, invalid_data, expected_error):
       """Test validation catches edge cases."""
       response = client.post('/api/v1/runs', json=invalid_data)
       assert response.status_code == 422
       assert expected_error in str(response.json())
   ```

4. **Missing: State Configuration Year Validation**
   ```python
   @pytest.mark.parametrize('year,expected_count', [
       ('2000', 50),  # All 50 states
       ('2010', 50),
       ('2020', 50),
   ])
   def test_state_config_by_year(client, year, expected_count):
       """Test state configuration returns correct data for each year."""
       response = client.get(f'/api/v1/config/states?year={year}')
       assert len(response.json()['states']) == expected_count
   ```

### Quality Gates for This Enhancement

- [ ] All CRUD operations work correctly
- [ ] Pagination handles edge cases (empty, last page, beyond total)
- [ ] Schema validation catches all invalid inputs
- [ ] Concurrent requests don't cause race conditions
- [ ] Status transitions follow valid state machine
- [ ] Test coverage >= 80%

---

## Dependencies

**Prerequisites**:
- Enhancement 60 (Project Setup) - REQUIRED
- PostgreSQL database running
- Alembic configured

**Python Packages** (add to requirements.txt):
- alembic>=1.12.0 (if not already added)

**Blocks**: Enhancement 62 (Pipeline Execution Engine)

---

## Success Criteria

- [ ] Database migrations run successfully
- [ ] Can create runs with all configuration options
- [ ] Can list runs with filtering by status/year/version
- [ ] Can paginate large run lists (50+ runs)
- [ ] Progress endpoint returns structured progress data
- [ ] State configuration endpoint returns all states for all years
- [ ] All endpoints return proper error responses
- [ ] All tests pass (unit + integration)
- [ ] API documentation complete in OpenAPI

---

## Design Notes (from Senior Designer)

### Metadata-Only Database
The database stores only run metadata, not district data:
- **Rationale**: District data already exists as Parquet/CSV files (efficient formats)
- **Benefit**: Database stays small and fast
- **Approach**: Store `output_path` pointing to file system

### JSON Columns for Flexibility
Using JSONB columns for `config` and `progress`:
- **Rationale**: Schema evolution without migrations
- **Benefit**: Can add new config options without DB changes
- **Tradeoff**: Less type safety, use Pydantic for validation

### Progress Data Structure
```json
{
  "years": {
    "2020": {
      "status": "running",
      "states_completed": 24,
      "states_total": 50,
      "current_stage": "district_maps",
      "workers": {
        "0": {"state": "california", "stage": "3/7"},
        "1": {"state": "florida", "stage": "5/7"}
      }
    }
  },
  "overall_progress": 0.45,
  "eta_seconds": 3600
}
```

---

## Related Documentation

- [Wave 9 Plan](../waves/WAVE09-api-migration.md)
- [Design Patterns](../DESIGN_PATTERNS.md) - Sections 2, 3, 7
- [Senior Designer Review](../waves/wave09/01_senior_designer_review.md) - Database recommendations
- [Senior Engineer Review](../waves/wave09/02_senior_engineer_review.md) - Engineering implementation details
- [Senior Tester Review](../waves/wave09/03_senior_tester_review.md) - Testing recommendations
- [TESTING_PATTERNS.md](../TESTING_PATTERNS.md) - API testing patterns

---

## Engineering Notes (from Senior Engineer)

### Risk Assessment: MEDIUM

Straightforward CRUD implementation with medium risk around schema evolution and query performance.

### Database Schema Recommendations

Per engineer review, add `process_pid` column for orphan detection on server restart:

```sql
CREATE TABLE runs (
    id SERIAL PRIMARY KEY,
    version VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    config JSONB NOT NULL,
    progress JSONB,
    process_pid INTEGER,  -- NEW: For orphan detection
    created_at TIMESTAMP DEFAULT NOW(),
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    error_message TEXT,
    output_path VARCHAR(255)
);

-- Indexes for common query patterns
CREATE INDEX idx_runs_status ON runs(status);
CREATE INDEX idx_runs_version ON runs(version);
CREATE INDEX idx_runs_created_at ON runs(created_at DESC);
CREATE INDEX idx_runs_pid ON runs(process_pid) WHERE process_pid IS NOT NULL;
```

**JSONB Benefits**:
1. Avoids migration churn as STATUS protocol evolves
2. Flexible schema for config and progress structures
3. PostgreSQL JSONB provides indexing and querying capabilities

### SQLAlchemy Session Management Pattern

Use proper dependency injection with cleanup:

```python
# api/database.py
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker, Session
from contextlib import contextmanager

engine = create_engine(
    settings.database_url,
    pool_pre_ping=True,  # Test connection before use
    pool_size=5,
    max_overflow=10,
)
SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)

def get_db():
    """Dependency injection for database sessions."""
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()
```

### Query Optimization Notes

**Eager Loading for RunYear**:
```python
def get_run(db: Session, run_id: int) -> Optional[Run]:
    """Get run with year details eagerly loaded."""
    return db.query(Run).options(
        joinedload(Run.years)
    ).filter(Run.id == run_id).first()
```

**Pagination Pattern**:
```python
def list_runs(
    db: Session,
    status: Optional[str] = None,
    limit: int = 50,
    offset: int = 0
) -> Tuple[List[Run], int]:
    """List runs with filtering and pagination."""
    query = db.query(Run)

    if status:
        query = query.filter(Run.status == status)

    total = query.count()
    runs = query.order_by(Run.created_at.desc()).offset(offset).limit(limit).all()

    return runs, total
```

### Progress Caching for Polling

For frequent polling (every 2 seconds), use in-memory caching:

```python
class ProgressCache:
    """In-memory cache for progress polling."""

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

### Testing Approach

**Unit Tests** (run service):
```python
def test_create_run():
    db = MagicMock()
    run = run_service.create_run(db, RunCreate(
        version="test",
        years=["2020"],
        workers=4
    ))
    assert run.status == "pending"
    assert run.config["years"] == ["2020"]

def test_list_runs_pagination():
    # Create mock runs
    # Verify pagination returns correct slice
```

**Integration Tests** (API endpoints):
```python
async def test_create_run_endpoint(client, db):
    response = await client.post("/api/v1/runs", json={
        "version": "test_v1",
        "years": ["2020"],
        "states": ["VT"],
        "workers": 1
    })
    assert response.status_code == 201
    assert response.json()["status"] == "pending"
```

---

**Enhancement 61 Summary**: Implement database schema and core API endpoints for run management with metadata-only storage and polling-based progress tracking.
