# Enhancement 62: Pipeline Execution Engine

**Status**: PLANNED
**Wave**: Wave 9 (API-MIGRATION)
**Priority**: High
**Estimated Complexity**: High (HIGHEST RISK in Wave 9)
**Estimated Hours**: 28-34 hours (includes 16-18 hours comprehensive testing)
**Created**: 2026-01-25
**Risk Level**: **HIGH** (per Senior Engineer assessment)
**Testing Risk**: **HIGHEST** (per Senior Tester assessment - 40% of wave testing effort)

---

## Description

Implement the pipeline execution engine that spawns and manages the redistricting subprocess, parses STATUS protocol messages, aggregates progress, and handles cancellation. **This is the highest risk and most critical enhancement in Wave 9** as it bridges the async FastAPI world with the synchronous CLI pipeline. Following the Senior Designer's recommendation, includes file-based progress fallback for reliability.

**Senior Engineer Verdict**: This is the critical path component. Start early, test thoroughly with VT. The file-based fallback is essential - don't rely solely on stdout.

---

## Tasks

### Phase 1: Subprocess Manager Class (4-5 hours)

- [ ] Create `api/workers/executor.py` with PipelineExecutor class
  ```python
  class PipelineExecutor:
      """Async wrapper for pipeline subprocess execution."""

      def __init__(
          self,
          run_id: int,
          command: list[str],
          on_progress: Callable[[dict], Awaitable[None]],
          on_complete: Callable[[int], Awaitable[None]],
          on_error: Callable[[str], Awaitable[None]],
          progress_file: Path,  # File-based fallback
      ):
          self.run_id = run_id
          self.command = command
          self.process: Optional[asyncio.subprocess.Process] = None
          self._cancelled = False
          self._watchdog_task: Optional[asyncio.Task] = None

      async def start(self):
          """Start pipeline subprocess and monitor output."""

      async def cancel(self, timeout: float = 5.0):
          """Cancel running pipeline with graceful shutdown."""

      async def _monitor_stdout(self):
          """Read stdout and parse STATUS messages."""

      async def _monitor_stderr(self):
          """Read stderr for error messages."""

      async def _watchdog(self, heartbeat_timeout: float = 60.0):
          """Monitor for hung processes."""
  ```
- [ ] Implement async subprocess creation with proper environment
  ```python
  async def _create_process(self):
      env = os.environ.copy()
      env["MULTI_YEAR_SUBPROCESS"] = "1"
      env["TQDM_POSITION"] = "999"  # Disable tqdm

      self.process = await asyncio.create_subprocess_exec(
          *self.command,
          stdout=subprocess.PIPE,
          stderr=subprocess.PIPE,
          env=env,
          cwd=self.project_root,
      )
  ```
- [ ] Implement graceful cancellation with SIGTERM then SIGKILL
- [ ] Add timeout handling for long-running processes

### Phase 2: STATUS Protocol Parser (3-4 hours)

- [ ] Create `api/utils/status_parser.py` with regex-based parsing
  ```python
  class StatusParser:
      """Parse STATUS protocol messages from pipeline output."""

      # STATUS:YEAR:2020:COMPLETE:24/50
      YEAR_PATTERN = re.compile(r"STATUS:YEAR:(\d{4}):(\w+):?(.*)$")

      # STATUS:WORKER:2020:1:STATE:12/50:california:STAGE:3/7:district_maps
      WORKER_PATTERN = re.compile(
          r"STATUS:WORKER:(\d{4}):(\d+):STATE:(\d+)/(\d+):(\w+):STAGE:(\d+)/(\d+):(.+)$"
      )

      def parse_line(self, line: str) -> Optional[StatusMessage]:
          """Parse a single STATUS line."""

      def aggregate_progress(self, messages: List[StatusMessage]) -> dict:
          """Aggregate messages into progress structure."""
  ```
- [ ] Handle all STATUS message types:
  - `STATUS:YEAR:*` - Year-level progress
  - `STATUS:WORKER:*` - Worker-level progress
  - `STATUS:CENSUS:*` - Census processing progress
  - `STATUS:NATION:*` - National stage progress
- [ ] Convert parsed messages to progress dictionary
- [ ] Calculate overall progress percentage

### Phase 3: File-Based Progress Fallback (2-3 hours)

- [ ] Implement file-based progress writing per Senior Designer recommendation
  ```python
  class FileProgressWriter:
      """Write progress to file as fallback for stdout parsing."""

      def __init__(self, progress_file: Path):
          self.progress_file = progress_file

      async def write_progress(self, progress: dict):
          """Atomically write progress to JSON file."""
          temp_file = self.progress_file.with_suffix('.tmp')
          async with aiofiles.open(temp_file, 'w') as f:
              await f.write(json.dumps(progress, indent=2))
          temp_file.rename(self.progress_file)

      async def read_progress(self) -> Optional[dict]:
          """Read progress from file."""
  ```
- [ ] Write progress on each STATUS message
- [ ] Use atomic file operations (write to temp, rename)
- [ ] Progress file location: `outputs/{version}/runs_progress.json`

### Phase 4: Progress Aggregation Service (3-4 hours)

- [ ] Create `api/services/progress_service.py`
  ```python
  class ProgressService:
      """Aggregate and calculate progress metrics."""

      def calculate_overall_progress(self, year_progress: dict) -> float:
          """Calculate overall progress across all years."""

      def calculate_eta(
          self,
          progress: float,
          start_time: datetime,
          historical_rates: Optional[List[float]] = None
      ) -> Optional[int]:
          """Estimate time remaining in seconds."""

      def format_progress_response(
          self,
          run: Run,
          year_progress: dict,
      ) -> RunProgressResponse:
          """Format progress for API response."""
  ```
- [ ] Implement ETA calculation with smoothing
- [ ] Track historical progress rates for better estimates
- [ ] Handle multi-year parallel execution

### Phase 5: Background Task Execution (3-4 hours)

- [ ] Integrate with FastAPI BackgroundTasks
  ```python
  @router.post("/{run_id}/actions/start", response_model=RunResponse)
  async def start_run(
      run_id: int,
      background_tasks: BackgroundTasks,
      db: Session = Depends(get_db)
  ):
      """Start pipeline execution in background."""
      run = run_service.get_run(db, run_id)
      if run.status != RunStatus.PENDING:
          raise ConflictError(f"Cannot start run in {run.status} state")

      # Update status to running
      run_service.update_run_status(db, run_id, RunStatus.RUNNING)

      # Start execution in background
      background_tasks.add_task(execute_pipeline, run_id, run.config)

      return run
  ```
- [ ] Implement `execute_pipeline` background function
- [ ] Handle completion callback (update database)
- [ ] Handle error callback (store error message)

### Phase 6: Cancellation Support (2-3 hours)

- [ ] Implement cancellation endpoint
  ```python
  @router.post("/{run_id}/actions/cancel", response_model=RunResponse)
  async def cancel_run(run_id: int, db: Session = Depends(get_db)):
      """Cancel an active run."""
      run = run_service.get_run(db, run_id)
      if run.status != RunStatus.RUNNING:
          raise ConflictError(f"Cannot cancel run in {run.status} state")

      # Signal executor to cancel
      await pipeline_manager.cancel(run_id)

      # Update status
      run_service.update_run_status(db, run_id, RunStatus.CANCELLED)

      return run
  ```
- [ ] Implement pipeline manager singleton for tracking active runs
- [ ] Handle subprocess cleanup on cancellation
- [ ] Ensure partial outputs are preserved (not deleted)

### Phase 7: Watchdog for Hung Processes (2-3 hours)

- [ ] Implement heartbeat monitoring
  ```python
  async def _watchdog(self, heartbeat_timeout: float = 60.0):
      """Monitor for hung processes."""
      last_progress_time = datetime.now()

      while self.process and self.process.returncode is None:
          await asyncio.sleep(10)  # Check every 10 seconds

          current_progress_time = self.last_progress_update
          if current_progress_time == last_progress_time:
              elapsed = (datetime.now() - last_progress_time).seconds
              if elapsed > heartbeat_timeout:
                  logger.warning(f"Run {self.run_id} appears hung, no progress for {elapsed}s")
                  await self.on_error(f"Process hung - no progress for {elapsed} seconds")
                  await self.cancel()
                  return
          else:
              last_progress_time = current_progress_time
  ```
- [ ] Configure heartbeat timeout (default 60 seconds)
- [ ] Log warnings before killing process
- [ ] Update run status with hung process error

### Phase 8: Integration Tests (3-4 hours)

- [ ] Create integration tests with Vermont
  ```python
  @pytest.mark.integration
  async def test_execute_pipeline_vermont():
      """Test full pipeline execution with Vermont."""
      run = await run_service.create_run(db, RunCreate(
          version="test_e62",
          years=["2020"],
          states=["VT"],
          workers=1
      ))

      # Start execution
      await pipeline_manager.start(run.id)

      # Wait for completion (with timeout)
      await asyncio.wait_for(
          wait_for_completion(run.id),
          timeout=300  # 5 minutes for VT
      )

      # Verify completion
      updated_run = run_service.get_run(db, run.id)
      assert updated_run.status == RunStatus.COMPLETED
      assert updated_run.output_path is not None

  @pytest.mark.integration
  async def test_cancel_running_pipeline():
      """Test cancellation during execution."""

  @pytest.mark.integration
  async def test_progress_updates_during_execution():
      """Test that progress updates are received."""
  ```
- [ ] Mock tests for faster unit testing
- [ ] Test error scenarios (subprocess crash, invalid config)

---

## Architecture Changes

**New Files**:
```
api/
├── workers/
│   ├── __init__.py
│   └── executor.py              # PipelineExecutor class
├── services/
│   ├── pipeline_service.py      # Pipeline execution coordination
│   └── progress_service.py      # Progress aggregation
└── utils/
    ├── __init__.py
    └── status_parser.py         # STATUS protocol parser
```

**Modified Files**:
```
api/routers/runs.py              # Add start/cancel actions
api/main.py                      # Initialize pipeline manager
```

**Related DESIGN_PATTERNS.md Sections**:
- Section 5: Async Execution Pattern
- Integration Patterns: Pipeline Subprocess Management
- Integration Patterns: STATUS Protocol Bridge

---

## Testing Strategy

**Assessment from Senior Tester**: INSUFFICIENT - needs significant expansion

This is the **most critical enhancement** for testing. The subprocess integration has multiple failure modes that must be tested. **40% of total wave testing effort should be allocated here.**

**Test Coverage Target**: 80% minimum

### Unit Tests (25-30 tests)

**STATUS Parser Tests (10-12 tests)**:
- All message types (YEAR, WORKER, CENSUS, NATION)
- Edge cases (zero progress, complete, single state)
- Invalid messages (incomplete, wrong prefix, non-numeric, empty)

**Process Lifecycle Tests (8-10 tests)**:
- Subprocess creation with correct environment
- Subprocess crash recovery
- Stdout buffering stress test
- File progress fallback
- Watchdog timeout detection
- State machine transitions

**Utility Tests (5-8 tests)**:
- Progress aggregation
- ETA calculation with smoothing
- File-based progress read/write with atomic operations
- Windows-specific signal handling

### Integration Tests (12-15 tests)

**Execution Tests (6-8 tests)**:
- Full pipeline execution (VT, single year)
- Subprocess communication flow
- Progress updates during execution
- Error recovery (subprocess crash, invalid config)
- Concurrent execution prevention
- File-based fallback activation

**Cancellation Tests (3-4 tests)**:
- Cancellation during execution
- Concurrent cancel during progress write
- Windows-specific cancellation

**Server Lifecycle Tests (2-3 tests)**:
- Server restart orphan detection
- Cleanup verification on completion

### E2E Tests (3-5 tests)

- Full VT pipeline run through API
- Multi-state execution
- Error recovery flow

### Performance Tests (4-5 tests)

- Memory usage during long-running processes
- Subprocess overhead measurement
- File polling interval optimization
- VT benchmark (< 5 minutes)

### Manual Testing
1. Start run via API: `POST /api/v1/runs/{id}/actions/start`
2. Poll progress: `GET /api/v1/runs/{id}/progress`
3. Cancel run: `POST /api/v1/runs/{id}/actions/cancel`
4. Verify output files created
5. Test with hung process (kill subprocess manually)
6. Test server restart during active run

---

## Testing Assessment (from Senior Tester)

| Attribute | Value |
|-----------|-------|
| **Risk Rating** | **HIGHEST** |
| **Original Assessment** | **INSUFFICIENT - Needs significant expansion** |
| **Testing Priority** | 1 (highest in wave) |
| **Recommended Effort** | 40% of total testing effort |

### Gap Analysis

| Test Type | Originally Proposed | Recommended | Gap |
|-----------|---------------------|-------------|-----|
| Unit | 15-20 | 25-30 | Add edge case parsing, error scenarios |
| Integration | 5-8 | 12-15 | Add failure recovery, concurrent scenarios |
| E2E | 1 (VT) | 3-5 | Add multi-state, error recovery |

### Critical Testing Gaps (MUST ADDRESS)

**1. Subprocess Crash Recovery**
```python
@pytest.mark.asyncio
async def test_subprocess_crash_recovery():
    """Test handling when subprocess crashes mid-execution."""
    executor = PipelineExecutor(
        run_id=1,
        command=['python', '-c', 'import os; os._exit(1)'],  # Abnormal exit
        on_progress=AsyncMock(),
        on_error=AsyncMock(),
        on_complete=AsyncMock(),
    )

    await executor.start()

    executor.on_error.assert_awaited()
    # Verify database updated to failed state
```

**2. Stdout Buffering Stress Test**
```python
@pytest.mark.asyncio
@pytest.mark.timeout(60)
async def test_rapid_status_messages():
    """Test handling rapid STATUS messages (buffering stress test)."""
    script = '''
import sys
for i in range(1000):
    print(f"STATUS:YEAR:2020:COMPLETE:{i+1}/1000", flush=True)
'''
    progress_count = 0

    async def count_progress(p):
        nonlocal progress_count
        progress_count += 1

    executor = PipelineExecutor(
        run_id=1,
        command=['python', '-c', script],
        on_progress=count_progress,
        on_complete=AsyncMock(),
    )

    await executor.start()

    # Should capture most messages despite buffering
    assert progress_count >= 900  # Allow some loss due to aggregation
```

**3. File Progress Fallback Test**
```python
@pytest.mark.asyncio
async def test_file_progress_fallback():
    """Test file-based progress when stdout is silent."""
    executor = PipelineExecutor(
        run_id=1,
        command=['python', 'scripts/test_silent_stdout.py'],
        on_progress=AsyncMock(),
        progress_file=Path('/tmp/test_progress.json'),
    )

    # Write progress to file
    with open('/tmp/test_progress.json', 'w') as f:
        json.dump({'overall_progress': 0.5}, f)

    # Executor should read from file as fallback
    await asyncio.sleep(15)  # Wait for fallback timeout

    executor.on_progress.assert_awaited()
```

**4. Watchdog Timeout Test**
```python
@pytest.mark.asyncio
@pytest.mark.timeout(90)
async def test_watchdog_kills_hung_process():
    """Test watchdog terminates hung processes."""
    error_called = asyncio.Event()

    async def on_error(msg):
        error_called.set()

    executor = PipelineExecutor(
        run_id=1,
        command=['python', '-c', 'import time; time.sleep(3600)'],  # Hang
        on_progress=AsyncMock(),
        on_error=on_error,
        on_complete=AsyncMock(),
        heartbeat_timeout=10,  # 10 second timeout for test
    )

    await executor.start()

    # Should be killed by watchdog
    await asyncio.wait_for(error_called.wait(), timeout=60)
    assert 'hung' in str(executor.error_message).lower()
```

**5. Windows-Specific Cancellation Test**
```python
@pytest.mark.skipif(sys.platform != 'win32', reason='Windows-specific')
@pytest.mark.asyncio
async def test_windows_cancellation():
    """Test cancellation works correctly on Windows."""
    executor = PipelineExecutor(
        run_id=1,
        command=['python', '-c', 'import time; time.sleep(60)'],
        on_progress=AsyncMock(),
        on_complete=AsyncMock(),
    )

    task = asyncio.create_task(executor.start())
    await asyncio.sleep(0.5)  # Let process start

    await executor.cancel(timeout=5.0)

    # Process should be terminated
    assert executor.process.returncode is not None
```

**6. Server Restart Orphan Detection**
```python
def test_orphan_detection_on_startup(db_session):
    """Test orphaned runs are detected on server restart."""
    run = Run(
        version='orphan_test',
        status=RunStatus.RUNNING,
        process_pid=99999,  # Non-existent PID
    )
    db_session.add(run)
    db_session.commit()

    # Simulate server startup
    from api.services.pipeline_manager import detect_orphans
    orphans = detect_orphans(db_session)

    assert len(orphans) == 1
    assert orphans[0].id == run.id
```

### STATUS Parser Edge Cases (MUST TEST)

```python
@pytest.mark.parametrize('line,expected_type', [
    # Valid messages
    ('STATUS:YEAR:2020:COMPLETE:24/50', 'YEAR'),
    ('STATUS:WORKER:2020:1:STATE:12/50:california:STAGE:3/7:maps', 'WORKER'),

    # Edge cases
    ('STATUS:YEAR:2020:COMPLETE:0/50', 'YEAR'),  # Zero progress
    ('STATUS:YEAR:2020:COMPLETE:50/50', 'YEAR'),  # Complete
    ('STATUS:WORKER:2020:0:STATE:1/1:vermont:STAGE:7/7:done', 'WORKER'),  # Single state

    # Invalid messages (should return None)
    ('STATUS:', None),
    ('STATUS:UNKNOWN:TYPE', None),
    ('STATUS:YEAR:2020', None),  # Incomplete
    ('NOTASTATUS:YEAR:2020:COMPLETE:1/1', None),  # Wrong prefix
    ('', None),  # Empty
    ('STATUS:YEAR:2020:COMPLETE:abc/50', None),  # Non-numeric
])
def test_status_parser_edge_cases(line, expected_type):
    """Test STATUS parser handles all edge cases."""
    msg_type, data = parse_status_message(line)
    if expected_type is None:
        assert msg_type is None
    else:
        assert msg_type == expected_type
```

### Critical Test Scenarios Matrix

| Scenario | Unit | Integration | E2E | Priority |
|----------|------|-------------|-----|----------|
| VT single-year execution | - | Yes | Yes | Critical |
| Progress updates in API | Yes | Yes | - | Critical |
| Subprocess crash handling | Yes | Yes | - | Critical |
| Cancellation within 10s | Yes | Yes | - | Critical |
| File-based fallback | Yes | Yes | - | Critical |
| Watchdog kills hung process | Yes | Yes | - | Critical |
| Server restart orphans | Yes | Yes | - | High |
| Windows signal handling | Yes | Yes | - | High |
| STATUS parser edge cases | Yes | - | - | High |
| Concurrent cancel race | Yes | Yes | - | Medium |

### Quality Gates for This Enhancement

- [ ] Vermont single-year run completes successfully via API
- [ ] Progress updates appear within 5 seconds in API response
- [ ] Cancellation stops subprocess within 10 seconds
- [ ] File-based fallback activates when stdout is delayed >10s
- [ ] Watchdog kills hung process after configured timeout
- [ ] Orphaned runs detected on server restart
- [ ] All STATUS message types parsed correctly
- [ ] Test coverage >= 80%
- [ ] All critical test scenarios pass

---

## Dependencies

**Prerequisites**:
- Enhancement 60 (Project Setup) - REQUIRED
- Enhancement 61 (Run Management API) - REQUIRED
- Existing pipeline scripts must work unchanged

**Python Packages** (add to requirements.txt):
- aiofiles>=23.0.0 (async file operations)

**Blocks**: Enhancements 63, 64 (frontend needs working progress endpoint)

---

## Success Criteria

- [ ] Can start pipeline execution via API
- [ ] STATUS messages parsed and aggregated correctly
- [ ] Progress updates reflected in database
- [ ] File-based progress works as fallback
- [ ] ETA calculation provides reasonable estimates
- [ ] Cancellation stops subprocess gracefully
- [ ] Watchdog detects and handles hung processes
- [ ] VT integration test completes successfully
- [ ] All tests pass (unit + integration)

---

## Design Notes (from Senior Designer)

### Async vs Sync
The existing pipeline is synchronous Python. Use `asyncio.create_subprocess_exec` to run without blocking the event loop:
```python
# CORRECT - non-blocking subprocess
proc = await asyncio.create_subprocess_exec(...)
```

### File-Based Fallback
Python subprocess stdout can buffer unexpectedly. The file-based progress provides reliability:
- Primary: Parse STATUS from stdout
- Fallback: Read from progress JSON file
- Benefit: Progress survives stdout buffering issues

### Background Tasks vs Celery
For this project, FastAPI's `BackgroundTasks` is sufficient:
- Single active run at a time (no queue needed)
- Progress tracked via polling (not persistent queue)
- Simpler deployment (no Redis/Celery workers)

Future consideration: Add Celery if need:
- Task persistence across server restarts
- Worker scaling
- Task retry logic

### Subprocess Communication Flow
```
API (async) → subprocess (sync pipeline)
     ↑               ↓
     └─── STATUS protocol messages ────┘
     └─── Progress JSON file (fallback) ┘
```

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Subprocess stdout buffering | File-based progress fallback |
| Process hangs | Watchdog with configurable timeout |
| Server restart during run | Detect orphaned runs on startup, allow manual retry |
| Windows subprocess issues | Test on Windows, use proper signal handling |
| Memory leaks from long processes | Monitor memory, implement cleanup |

---

## Related Documentation

- [Wave 9 Plan](../waves/WAVE09-api-migration.md)
- [Design Patterns](../DESIGN_PATTERNS.md) - Section 5, Integration Patterns
- [Senior Designer Review](../waves/wave09/01_senior_designer_review.md) - Subprocess recommendations
- [Senior Engineer Review](../waves/wave09/02_senior_engineer_review.md) - Detailed implementation patterns
- [Senior Tester Review](../waves/wave09/03_senior_tester_review.md) - Critical testing requirements
- [STATUS Protocol](../CODING_PATTERNS.md) - STATUS message format
- [TESTING_PATTERNS.md](../TESTING_PATTERNS.md) - Async test patterns, subprocess mocking

---

## Engineering Notes (from Senior Engineer)

### Risk Assessment: **HIGHEST RISK**

This is the most complex and critical enhancement in Wave 9. All four identified risks must be addressed:

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Subprocess stdout buffering | High | Medium | `PYTHONUNBUFFERED=1` + file fallback |
| Process hangs indefinitely | High | Medium | Watchdog (60s timeout) |
| Server restart during run | Medium | Low | PID in database, orphan detection |
| Windows subprocess issues | Medium | High | Use `process.terminate()`, test on Windows |

### Subprocess Management Pattern (asyncio)

Per engineer recommendation, use `asyncio.create_subprocess_exec` for non-blocking execution:

```python
# CORRECT - Async subprocess
async def _create_process(self):
    env = os.environ.copy()
    env["MULTI_YEAR_SUBPROCESS"] = "1"
    env["TQDM_POSITION"] = "999"  # Disable tqdm
    env["PYTHONUNBUFFERED"] = "1"  # Critical: disable output buffering

    self.process = await asyncio.create_subprocess_exec(
        *self.command,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
        env=env,
        cwd=self.project_root,
    )

# Read lines without blocking event loop
async def _monitor_stdout(self):
    async for line in self.process.stdout:
        if line.startswith(b"STATUS:"):
            await self._handle_status(line.decode().strip())
```

**Critical Note**: Python's `asyncio` subprocess readline can still buffer. The file-based progress fallback is **essential**.

### STATUS Protocol Bridge Implementation

Reuse existing code from `scripts/utils/status_protocol.py`:

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

### File-Based Progress Fallback with Atomic Write Pattern

Per engineer recommendation, use atomic writes to prevent corruption:

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
        temp_file.rename(self.progress_file)  # Atomic on most filesystems

    async def read(self) -> Optional[dict]:
        """Read progress, return None if file doesn't exist."""
        if not self.progress_file.exists():
            return None
        async with aiofiles.open(self.progress_file, "r") as f:
            return json.loads(await f.read())
```

**Fallback Strategy**: Read from file if stdout silent for >10 seconds.

### Error Handling State Machine

Per engineer recommendation:

```
pending -> running -> completed
                   -> failed (subprocess error)
                   -> cancelled (user action)
                   -> timeout (watchdog killed)

On server start: running -> orphaned (allow retry)
```

### PipelineManager Singleton Pattern

Per engineer recommendation, use singleton for single active run:

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

### Windows-Specific Considerations

**Issue**: Windows doesn't support SIGTERM like Unix.

**Solution**:
```python
async def cancel(self, timeout: float = 5.0):
    """Cancel running pipeline with graceful shutdown."""
    if self.process is None:
        return

    # Cross-platform: terminate first
    self.process.terminate()

    try:
        await asyncio.wait_for(self.process.wait(), timeout=timeout)
    except asyncio.TimeoutError:
        # Force kill if graceful shutdown fails
        if sys.platform == "win32":
            # Windows: use taskkill
            os.system(f"taskkill /F /PID {self.process.pid}")
        else:
            self.process.kill()

        await self.process.wait()
```

### Integration Test with Vermont

Per engineer recommendation, always test with Vermont (smallest state):

```python
@pytest.mark.integration
@pytest.mark.timeout(300)  # 5 minute timeout for VT
async def test_execute_pipeline_vermont():
    """End-to-end test with Vermont (smallest state)."""
    run = await run_service.create_run(db, RunCreate(
        version="test_e62",
        years=["2020"],
        states=["VT"],
        workers=1
    ))

    # Start execution
    await pipeline_manager.start(run.id)

    # Wait for completion (with timeout)
    await asyncio.wait_for(
        wait_for_completion(run.id),
        timeout=300  # 5 minutes for VT
    )

    # Verify completion
    updated_run = run_service.get_run(db, run.id)
    assert updated_run.status == RunStatus.COMPLETED
    assert updated_run.output_path is not None

    # Verify output files exist
    output_path = Path(updated_run.output_path)
    assert (output_path / "2020" / "states" / "vermont" / "data").exists()
```

### Debugging Approaches

Per engineer recommendation, add verbose logging:

```python
# Add verbose logging to executor
logger.debug(f"Command: {' '.join(command)}")
logger.debug(f"Environment: {json.dumps(env, indent=2)}")

# Capture stderr for debugging
async for line in proc.stderr:
    logger.warning(f"[stderr] {line.decode().strip()}")

# Debug endpoint for raw progress
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

---

**Enhancement 62 Summary**: Implement async pipeline execution engine with STATUS parsing, file-based progress fallback, cancellation support, and watchdog monitoring.
