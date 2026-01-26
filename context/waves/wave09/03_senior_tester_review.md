# Senior Test Engineer Review - Wave 9 (API Migration)

**Reviewer**: Senior Test Engineer
**Date**: 2026-01-25
**Wave**: 9 (API-MIGRATION)
**Status**: REVIEW COMPLETE

---

## Executive Summary

Wave 9 introduces a significant architectural change - transitioning from CLI-only to web-based run management. From a testing perspective, this wave presents **moderate-to-high testing complexity** due to:

1. **New technology layers** (FastAPI, React, SQLAlchemy) requiring new test infrastructure
2. **Critical subprocess integration** (Enhancement 62) with high failure mode complexity
3. **Real-time progress polling** requiring state synchronization testing
4. **Cross-layer integration** between CLI, API, database, and frontend

**Overall Testing Risk Assessment**: **MEDIUM-HIGH**

Enhancement 62 (Pipeline Execution Engine) is the **highest testing risk** and should receive disproportionate testing attention.

---

## Test Coverage Assessment by Enhancement

### Enhancement 60: API Project Setup (LOW RISK)

**Current Test Plan**:
- Basic endpoint tests (health, docs)
- CORS configuration verification
- Basic error handling

**Assessment**: ADEQUATE

The enhancement plan covers essential setup verification. Additional recommendations:

| Test Type | Proposed Count | Recommended | Gap Analysis |
|-----------|---------------|-------------|--------------|
| Unit | 0 | 3-5 | Add config validation tests |
| Integration | 3-5 | 5-8 | Add CORS preflight tests |
| E2E | 0 | 0 | Not needed for setup |

**Recommended Additions**:
```python
# Test configuration loading from environment
def test_config_loads_from_env():
    """Verify configuration reads from environment variables."""

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

### Enhancement 61: Run Management API (MEDIUM RISK)

**Current Test Plan**:
- Unit tests for run service (10-15)
- Integration tests for API endpoints (8-10)
- Manual testing

**Assessment**: ADEQUATE with gaps

| Test Type | Proposed Count | Recommended | Gap Analysis |
|-----------|---------------|-------------|--------------|
| Unit | 10-15 | 15-18 | Add schema validation edge cases |
| Integration | 8-10 | 12-15 | Add concurrent access tests |
| E2E | 0 | 2-3 | Add API contract tests |

**Testing Gaps Identified**:

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

---

### Enhancement 62: Pipeline Execution Engine (HIGHEST RISK)

**Current Test Plan**:
- Unit tests for STATUS parser (15-20)
- Integration tests with Vermont (5-8)
- Manual testing

**Assessment**: INSUFFICIENT - Needs significant expansion

This is the **most critical enhancement** for testing. The subprocess integration has multiple failure modes that must be tested.

| Test Type | Proposed Count | Recommended | Gap Analysis |
|-----------|---------------|-------------|--------------|
| Unit | 15-20 | 25-30 | Add edge case parsing, error scenarios |
| Integration | 5-8 | 12-15 | Add failure recovery, concurrent scenarios |
| E2E | 1 (VT) | 3-5 | Add multi-state, error recovery |

**Critical Testing Gaps**:

1. **Missing: Subprocess Crash Recovery**
   ```python
   @pytest.mark.asyncio
   async def test_subprocess_crash_recovery():
       """Test handling when subprocess crashes mid-execution."""
       # Create executor that will crash
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

2. **Missing: Stdout Buffering Stress Test**
   ```python
   @pytest.mark.asyncio
   @pytest.mark.timeout(60)
   async def test_rapid_status_messages():
       """Test handling rapid STATUS messages (buffering stress test)."""
       # Script that outputs 1000 STATUS messages rapidly
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

3. **Missing: File Progress Fallback Test**
   ```python
   @pytest.mark.asyncio
   async def test_file_progress_fallback():
       """Test file-based progress when stdout is silent."""
       # Executor where stdout is blocked but file progress works
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

4. **Missing: Watchdog Timeout Test**
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

       # Start with watchdog
       await executor.start()

       # Should be killed by watchdog
       await asyncio.wait_for(error_called.wait(), timeout=60)
       assert 'hung' in str(executor.error_message).lower()
   ```

5. **Missing: Windows-Specific Cancellation Test**
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

6. **Missing: Concurrent Cancel During Progress Write**
   ```python
   @pytest.mark.asyncio
   async def test_cancel_during_progress_write():
       """Test cancellation during progress update doesn't corrupt state."""
       # Test race condition between progress update and cancellation
   ```

7. **Missing: Server Restart Orphan Detection**
   ```python
   def test_orphan_detection_on_startup(db_session):
       """Test orphaned runs are detected on server restart."""
       # Create a run with 'running' status but no active process
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

**STATUS Parser Edge Cases**:
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

---

### Enhancement 63: React Dashboard Core (MEDIUM RISK)

**Current Test Plan**:
- Unit tests for UI components (15-20)
- Integration tests with MSW (8-10)
- E2E tests (3-5)

**Assessment**: ADEQUATE with recommendations

| Test Type | Proposed Count | Recommended | Gap Analysis |
|-----------|---------------|-------------|--------------|
| Unit | 15-20 | 18-22 | Add accessibility tests |
| Integration | 8-10 | 10-12 | Add polling state transition tests |
| E2E | 3-5 | 5-7 | Add error recovery flows |

**Testing Gaps Identified**:

1. **Missing: Polling State Transitions**
   ```typescript
   describe('Progress Polling', () => {
     it('should start polling when run status is running', async () => {
       server.use(
         rest.get('/api/v1/runs/1', (req, res, ctx) => {
           return res(ctx.json({ id: 1, status: 'running' }));
         })
       );

       render(<RunDetail runId={1} />);

       // Verify polling started (refetchInterval is active)
       await waitFor(() => {
         expect(mockApiCalls.length).toBeGreaterThan(1);
       }, { timeout: 5000 });
     });

     it('should stop polling when run status changes to completed', async () => {
       let callCount = 0;
       server.use(
         rest.get('/api/v1/runs/1', (req, res, ctx) => {
           callCount++;
           return res(ctx.json({
             id: 1,
             status: callCount < 3 ? 'running' : 'completed'
           }));
         })
       );

       render(<RunDetail runId={1} />);

       // Wait for completion
       await waitFor(() => {
         expect(screen.getByText(/completed/i)).toBeInTheDocument();
       });

       // Wait and verify polling stopped
       const callsAtComplete = callCount;
       await new Promise(r => setTimeout(r, 5000));
       expect(callCount).toBe(callsAtComplete);  // No new calls
     });
   });
   ```

2. **Missing: Accessibility Tests**
   ```typescript
   import { axe, toHaveNoViolations } from 'jest-axe';
   expect.extend(toHaveNoViolations);

   describe('Accessibility', () => {
     it('RunList should have no accessibility violations', async () => {
       const { container } = render(<RunList />);
       await waitFor(() => screen.getByRole('table'));

       const results = await axe(container);
       expect(results).toHaveNoViolations();
     });

     it('Progress bar should have proper ARIA attributes', () => {
       render(<ProgressBar value={50} max={100} />);

       const progressbar = screen.getByRole('progressbar');
       expect(progressbar).toHaveAttribute('aria-valuenow', '50');
       expect(progressbar).toHaveAttribute('aria-valuemax', '100');
     });
   });
   ```

3. **Missing: Optimistic Update Rollback Test**
   ```typescript
   it('should rollback optimistic update on error', async () => {
     server.use(
       rest.post('/api/v1/runs/1/actions/cancel', (req, res, ctx) => {
         return res(ctx.status(500), ctx.json({ detail: 'Server error' }));
       })
     );

     render(<RunDetail runId={1} />);

     // Run should be running initially
     await waitFor(() => screen.getByText(/running/i));

     // Click cancel
     await userEvent.click(screen.getByRole('button', { name: /cancel/i }));

     // Should briefly show cancelled (optimistic)
     // Then revert to running (on error)
     await waitFor(() => {
       expect(screen.getByText(/running/i)).toBeInTheDocument();
     });

     // Should show error message
     expect(screen.getByText(/error/i)).toBeInTheDocument();
   });
   ```

4. **Missing: Network Disconnection Test**
   ```typescript
   it('should handle network disconnection gracefully', async () => {
     render(<RunList />);
     await waitFor(() => screen.getByRole('table'));

     // Simulate network disconnect
     server.use(
       rest.get('/api/v1/runs', (req, res) => {
         return res.networkError('Network error');
       })
     );

     // Trigger refetch
     await userEvent.click(screen.getByRole('button', { name: /refresh/i }));

     // Should show offline/error state
     await waitFor(() => {
       expect(screen.getByText(/connection error|offline/i)).toBeInTheDocument();
     });
   });
   ```

---

### Enhancement 64: District Visualization (MEDIUM RISK)

**Current Test Plan**:
- Unit tests (8-10)
- Integration tests (5-8)
- E2E tests (3-5)

**Assessment**: ADEQUATE with performance testing gaps

| Test Type | Proposed Count | Recommended | Gap Analysis |
|-----------|---------------|-------------|--------------|
| Unit | 8-10 | 10-12 | Add color scale edge cases |
| Integration | 5-8 | 8-10 | Add map interaction tests |
| E2E | 3-5 | 5-7 | Add performance regression tests |

**Testing Gaps Identified**:

1. **Missing: Map Rendering Performance Test**
   ```typescript
   describe('Map Performance', () => {
     it('should render 435 districts without jank', async ({ page }) => {
       // Measure Long Tasks during render
       const longTasks: number[] = [];

       await page.evaluate(() => {
         const observer = new PerformanceObserver((list) => {
           for (const entry of list.getEntries()) {
             if (entry.duration > 50) {
               window.longTasks.push(entry.duration);
             }
           }
         });
         window.longTasks = [];
         observer.observe({ entryTypes: ['longtask'] });
       });

       await page.goto('/runs/1/districts');
       await page.waitForSelector('.leaflet-interactive');

       const tasks = await page.evaluate(() => window.longTasks);

       // No individual task should block for more than 100ms
       expect(Math.max(...tasks, 0)).toBeLessThan(100);
     });

     it('should maintain smooth interaction FPS', async ({ page }) => {
       await page.goto('/runs/1/districts');
       await page.waitForSelector('.leaflet-container');

       // Measure FPS during pan
       const fps = await page.evaluate(async () => {
         const map = document.querySelector('.leaflet-container');
         let frames = 0;
         let start: number;

         return new Promise<number>((resolve) => {
           const countFrame = () => {
             if (!start) start = performance.now();
             frames++;

             if (performance.now() - start < 1000) {
               requestAnimationFrame(countFrame);
             } else {
               resolve(frames);
             }
           };

           // Simulate pan
           map.dispatchEvent(new MouseEvent('mousedown', { clientX: 200, clientY: 200 }));
           requestAnimationFrame(countFrame);
         });
       });

       expect(fps).toBeGreaterThanOrEqual(55);  // Allow slight drop from 60
     });
   });
   ```

2. **Missing: Geometry Lazy Loading Test**
   ```typescript
   it('should lazy load state geometries', async ({ page }) => {
     const geometryRequests: string[] = [];

     await page.route('**/geometry/**', (route) => {
       geometryRequests.push(route.request().url());
       route.continue();
     });

     await page.goto('/runs/1/districts');

     // Initially only visible states should be loaded
     await page.waitForSelector('.leaflet-interactive');
     const initialCount = geometryRequests.length;
     expect(initialCount).toBeLessThan(50);  // Not all 50 states

     // Zoom to a specific region
     await page.evaluate(() => {
       const map = (window as any).map;
       map.setView([34.0, -118.0], 8);  // California
     });

     // Should load additional geometries
     await page.waitForTimeout(1000);
     expect(geometryRequests.length).toBeGreaterThan(initialCount);
   });
   ```

3. **Missing: Color Scale Consistency Test**
   ```typescript
   describe('Color Scales', () => {
     it('should use consistent colors across views', async ({ page }) => {
       await page.goto('/runs/1/districts?colorBy=compactness');

       // Get colors from map
       const mapColors = await page.evaluate(() => {
         const features = document.querySelectorAll('.leaflet-interactive');
         return Array.from(features).slice(0, 10).map(f =>
           getComputedStyle(f).fill
         );
       });

       // Get colors from table (if colored)
       const tableColors = await page.evaluate(() => {
         const cells = document.querySelectorAll('[data-color]');
         return Array.from(cells).slice(0, 10).map(c =>
           c.getAttribute('data-color')
         );
       });

       // Colors should match for same districts
       expect(mapColors).toEqual(tableColors);
     });

     it('should handle missing compactness values', async ({ page }) => {
       // Mock district with null compactness
       await page.route('**/districts', (route) => {
         route.fulfill({
           json: {
             districts: [
               { id: 1, state: 'test', polsby_popper: null },
             ],
           },
         });
       });

       await page.goto('/runs/1/districts?colorBy=compactness');

       // Should render with default/fallback color, not crash
       await expect(page.locator('.leaflet-interactive')).toBeVisible();
     });
   });
   ```

---

## Risk-Based Testing Priority

Based on impact and likelihood analysis:

| Enhancement | Risk Level | Testing Priority | Recommended Effort |
|------------|------------|------------------|-------------------|
| Enhancement 62 (Pipeline Execution) | **HIGHEST** | 1 | 40% of testing effort |
| Enhancement 61 (Run Management) | MEDIUM | 2 | 20% of testing effort |
| Enhancement 63 (React Dashboard) | MEDIUM | 3 | 20% of testing effort |
| Enhancement 64 (Visualization) | MEDIUM | 4 | 15% of testing effort |
| Enhancement 60 (Project Setup) | LOW | 5 | 5% of testing effort |

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

## Test Data Strategy

### Recommended Approach

1. **Unit Tests**: Factory-generated mock data
   - Use Faker for random but reproducible data
   - Create focused factories for each entity type
   - Seed Faker for deterministic tests

2. **Integration Tests**: Fixture-based approach
   - Pre-created database fixtures
   - Transactional rollback between tests
   - Separate test database (SQLite for speed, PostgreSQL for CI)

3. **E2E Tests**: Mock-first with real integration option
   - MSW for most E2E tests (fast, deterministic)
   - One "smoke test" against real backend (VT only)

4. **Performance Tests**: Production-like data
   - 435 mock districts for map tests
   - 1000+ STATUS messages for parsing stress tests

### Test Data Location

```
tests/
├── api/
│   ├── fixtures/
│   │   ├── runs.json           # Sample run records
│   │   ├── districts.json      # Sample district data
│   │   └── progress.json       # Sample progress states
│   └── mocks/
│       ├── status_messages.py  # Generate STATUS sequences
│       └── subprocess.py       # Mock subprocess outputs
└── frontend/
    ├── __tests__/
    │   └── factories/
    │       ├── run.ts          # Run factory
    │       ├── district.ts     # District factory
    │       └── progress.ts     # Progress factory
    └── mocks/
        ├── handlers.ts         # MSW handlers
        └── data/
            ├── runs.json
            └── districts.geojson
```

---

## Performance Testing Recommendations

### Benchmarks to Establish

| Metric | Target | Test Method |
|--------|--------|-------------|
| API: List runs (50 records) | <100ms p99 | Load test with k6 |
| API: Get run progress | <50ms p99 | Polling simulation |
| API: Create run | <200ms p99 | Sequential requests |
| Frontend: Initial load | <3s | Lighthouse CI |
| Frontend: Map render (435 districts) | <3s | Performance profiling |
| Frontend: Map interaction | 60fps | Frame rate monitoring |
| Pipeline: STATUS parsing throughput | 10,000 msg/s | Unit benchmark |

### Load Testing Script (k6)

```javascript
// tests/load/api-load-test.js
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '30s', target: 10 },   // Ramp up
    { duration: '1m', target: 10 },    // Sustained
    { duration: '30s', target: 50 },   // Peak
    { duration: '30s', target: 0 },    // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(99)<100'],  // 99th percentile < 100ms
    http_req_failed: ['rate<0.01'],    // Error rate < 1%
  },
};

export default function () {
  // List runs
  let response = http.get('http://localhost:8002/api/v1/runs');
  check(response, {
    'list runs status 200': (r) => r.status === 200,
    'list runs fast': (r) => r.timings.duration < 100,
  });

  // Get single run
  response = http.get('http://localhost:8002/api/v1/runs/1');
  check(response, {
    'get run status 200': (r) => r.status === 200,
  });

  sleep(1);
}
```

---

## Regression Prevention Strategy

### Ensuring CLI Functionality Remains Intact

1. **Existing CLI Tests Must Pass**
   - All 340 existing tests must continue passing
   - Add regression marker to critical CLI tests

2. **CLI-API Compatibility Tests**
   ```python
   def test_api_matches_cli_output():
       """Verify API produces same results as CLI."""
       # Run CLI
       cli_result = subprocess.run([
           'python', 'scripts/pipeline/run_state_redistricting.py',
           '--year', '2020', '--state', 'VT', '--version', 'cli_test'
       ], capture_output=True)

       # Run via API
       api_result = client.post('/api/v1/runs', json={
           'version': 'api_test',
           'years': ['2020'],
           'states': ['VT'],
       })
       # Start and wait for completion...

       # Compare outputs
       cli_output = Path('outputs/cli_test/2020/states/vermont')
       api_output = Path('outputs/api_test/2020/states/vermont')

       # Compare key files
       assert compare_csv(cli_output / 'final_assignments.csv',
                         api_output / 'final_assignments.csv')
   ```

3. **STATUS Protocol Contract Tests**
   - Ensure STATUS message format doesn't change
   - Parser tests lock down expected format

---

## CI/CD Integration Recommendations

### Proposed Pipeline

```yaml
stages:
  - lint
  - unit-test
  - integration-test
  - build
  - e2e-test
  - performance-test
  - deploy

unit-test:
  script:
    - pytest tests/api/unit/ -v --cov=api
    - cd frontend && pnpm test:unit --coverage
  coverage:
    report: cobertura
    paths:
      - coverage.xml
      - frontend/coverage/cobertura-coverage.xml

integration-test:
  services:
    - postgres:15
  script:
    - pytest tests/api/integration/ -v
    - cd frontend && pnpm test:integration

e2e-test:
  script:
    - docker-compose up -d
    - cd frontend && pnpm test:e2e
  artifacts:
    when: on_failure
    paths:
      - frontend/playwright-report/

performance-test:
  rules:
    - if: $CI_COMMIT_BRANCH == "main"
  script:
    - docker-compose up -d
    - k6 run tests/load/api-load-test.js
```

### Quality Gates

| Gate | Threshold | Blocking |
|------|-----------|----------|
| Unit test coverage | 80% | Yes |
| Integration test pass | 100% | Yes |
| E2E critical flows | 100% | Yes |
| API p99 latency | <100ms | Warning |
| Map load time | <3s | Warning |
| Security scan | No high/critical | Yes |

---

## Test Metrics and Reporting

### Recommended Metrics

1. **Coverage Metrics**
   - Line coverage by component
   - Branch coverage for critical paths
   - Mutation testing score (optional)

2. **Quality Metrics**
   - Test pass rate
   - Flaky test rate (<1% target)
   - Mean time to fix failing tests

3. **Performance Metrics**
   - Test suite execution time
   - Individual test execution time
   - Resource usage during tests

### Dashboard Reporting

Create test dashboard showing:
- Coverage trends
- Test pass/fail history
- Flaky test tracking
- Performance regression alerts

---

## Action Items Summary

### Immediate (Before Implementation Starts)

1. Create TESTING_PATTERNS.md document (DONE - see companion document)
2. Set up test infrastructure scaffolding
3. Create mock data generators for new entities

### During Implementation

1. Require TDD for Enhancement 62 (Pipeline Execution)
2. Review test coverage at each PR
3. Run E2E tests against staging before merge

### Before Launch

1. Complete all critical test scenarios
2. Run load testing with production-like data
3. Perform security testing (SQL injection, XSS)
4. Verify all quality gates pass
5. Create test runbook for operations team

---

## Conclusion

Wave 9 testing requires significant investment, particularly for Enhancement 62 (Pipeline Execution Engine). The recommended testing approach emphasizes:

1. **Risk-based prioritization** - Focus effort on highest-risk components
2. **Comprehensive coverage** - Unit, integration, and E2E tests for each layer
3. **Performance validation** - Establish and enforce benchmarks
4. **Regression prevention** - Ensure CLI functionality remains intact

The provided TESTING_PATTERNS.md document establishes patterns and practices for consistent test implementation across the wave.

**Estimated Testing Effort**: 40-50 hours across all enhancements (approximately 30-35% of total development time).

---

*Reviewed by: Senior Test Engineer*
*Date: 2026-01-25*
