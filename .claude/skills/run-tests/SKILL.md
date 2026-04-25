---
name: run-tests
description: Execute test suite with intelligent filtering and reporting. Run all tests or specific categories (unit/integration/E2E) with clear summaries, coverage options, and failure guidance.
allowed-tools:
  - Read
  - Bash
  - Grep
  - TodoWrite
user-invocable: true
---

# Run Tests

Execute test suite (151 tests, ~18s) with intelligent filtering, clear reporting, actionable guidance.

## Prerequisites
**Required**: pytest, pytest-playwright, tests in `tests/`, fixtures in `tests/fixtures/` + `tests/mocks/`
**Optional**: pytest-cov (coverage), Chromium (`playwright install chromium`, E2E)

## When to Use
User says "Run tests/all tests/unit tests/with coverage/test X code", when validating changes pre-commit, checking test health, generating coverage, after implementing features

## Workflow

### Step 1: Determine Scope
Ask user via `AskUserQuestion`:

**Test Type**: All (151, ~18s), Unit (110, ~7s), Integration (21, ~3s), E2E dashboard (20, ~8s), Specific component (by marker)
**Options**: With coverage, Verbose, Failed first, Show markers

### Step 2: Build Pytest Command

**By type**:
```bash
pytest tests/ -v                 # All
pytest tests/unit/ -v            # Unit
pytest tests/integration/ -v     # Integration
pytest tests/e2e/ -v            # E2E
```

**By marker**:
```bash
pytest tests/ -m redistricting -v   # Component markers: redistricting, political,
pytest tests/ -m political -v       # demographic, compactness, visualization, dashboard
```

**With coverage**:
```bash
pytest tests/ --cov=apportionment --cov-report=html -v
```

**Advanced**:
```bash
pytest tests/ --lf -v           # Last failed first
pytest tests/ --markers         # Show markers
pytest tests/ -v --tb=long      # Verbose tracebacks
pytest tests/ -v -s            # Show print statements
```

### Step 3: Execute
Use `Bash`:
```bash
cd apportionment
pytest [command from Step 2]
```

**Monitor**: Progress, counts (passed/failed/skipped), errors, time
**Handle issues**: Cancel → partial results, timeout → smaller subset, crash → `/debug-tests`

### Step 4: Parse Results
Extract via `Grep` if needed: Total run, passed, failed, skipped, time, coverage %

**Output patterns**:
```
====== 151 passed in 18.23s ======
====== 148 passed, 3 failed in 18.67s ======
====== 100 passed, 5 skipped in 15.32s ======
```

### Step 5: Report Summary

**All passed**:
```
[OK] All tests passed!
Summary: 151 run, 151 passed (100%), 0 failed, 0 skipped, 18.2s
Coverage: 92% (if --cov)
```

**Partial failures**:
```
[WARN] Some tests failed
Summary: 151 run, 148 passed (98%), 3 failed (2%), 0 skipped, 18.5s
Failed: test1, test2, test3
Next: /debug-tests to investigate
```

**Critical (all failed)**:
```
[FAIL] Test suite failed
Summary: 12 run, 0 passed, 12 failed (100%), 2.1s
Critical issue → Common: Import errors (PYTHONPATH), missing deps (requirements.txt), mock data (fixtures)
Next: /debug-tests for troubleshooting
```

### Step 6: Suggest Next Steps

**All passed**: Run with coverage (check %), ready to commit, consider full pipeline test
**Some failed**: `/debug-tests`, re-run failed (`pytest --lf -v`), check recent changes
**Many failed**: `/debug-tests` for systematic troubleshooting, check imports/mocks/data, verify env setup
**Coverage generated**: Open `htmlcov/index.html`, review uncovered sections, identify areas needing tests

## Test Categories

### By Type
**Unit** (110, 7s): Redistricting (10), METIS (27), Political (13), Demographic (13), Compactness (15), Visualization (18), Aggregation (14)
**Integration** (21, 3s): Single-state flows, national aggregation, multi-stage validation
**E2E Dashboard** (20, 8s): Dashboard functionality (9), artifact validation (11)

### By Marker
`redistricting`, `political`, `demographic`, `compactness`, `visualization`, `aggregation`, `dashboard`, `unit`, `integration`, `slow` (>10s)

## Coverage Reporting

**With --cov**: HTML → `htmlcov/index.html`, open: `start htmlcov/index.html` (Win) / `open htmlcov/index.html` (Mac)

**Interpretation**: 90-100% excellent, 70-90% good, 50-70% moderate (needs work), <50% low (needs tests)

**Check**: `apportionment/partition/` (core), `apportionment/data/`, `apportionment/visualization/` (scripts/ excluded)

## Quick Commands
```bash
pytest tests/ -v                                         # All (~18s)
pytest tests/unit/ -v                                    # Unit (~7s)
pytest tests/integration/ -v                             # Integration (~3s)
pytest tests/e2e/ -v                                     # E2E (~8s)
pytest tests/ --cov=apportionment --cov-report=html -v   # Coverage
pytest tests/ --lf -v                                    # Last failed first
pytest tests/ --markers                                  # Show markers
pytest tests/ -v --tb=long                               # Verbose tracebacks
pytest tests/ -m redistricting -v                        # Specific component
pytest tests/ -m political -v                            # Political tests
pytest tests/ --pdb -v                                   # Debugger (stop on fail)
pytest tests/ -v -s                                      # Show prints
```

## Troubleshooting

**Import Errors**: `ModuleNotFoundError: apportionment` → `export PYTHONPATH="${PYTHONPATH}:$(pwd)"`
**Playwright**: Browser not found → `playwright install chromium`
**Mock Data**: FileNotFoundError → Auto-generated by conftest.py, persist → `/debug-tests`
**Coverage**: --cov not recognized → `pip install pytest-cov`
**Slow Tests**: E2E timeout → `--headed --slowmo` or subset: `pytest tests/e2e/test_run_dashboard.py -v`

## Performance

**Times**: All ~18s, Unit ~7s, Integration ~3s, E2E ~8s

**Parallel** (faster):
```bash
pip install pytest-xdist
pytest tests/ -n 4 -v  # 4 workers
```

**Optimization**: Mock data once per session (session-scoped), E2E reuse mock run, unit tests use lightweight mocks

## What You'll Get
Test results summary (pass/fail stats), execution time, failure details (if any), coverage report (if --cov) in `htmlcov/`, next step recommendations, quick re-run/debug commands

## Next Steps

**All passed**: Commit with confidence, run coverage if not done, consider full pipeline test
**Some failed**: `/debug-tests` to investigate, re-run `pytest --lf -v`, check recent changes
**Coverage low**: Identify uncovered in `htmlcov/index.html`, write tests for uncovered functions, focus on critical paths
**CI/CD**: Add pytest to workflow, use exit codes, upload coverage to Codecov

## Related Skills
`/debug-tests` (debug test failures), `/enhancement-implement` (Phase 4 includes testing), `/pipeline-debug` (pipeline failures)

## Examples

**Ex 1 - All tests**: User "Run all tests" → Ask confirm → `pytest tests/ -v` → 151 passed, 18.2s → [OK] Ready to commit

**Ex 2 - Unit + coverage**: User "Run unit tests with coverage" → Ask confirm → `pytest tests/unit/ --cov=apportionment --cov-report=html -v` → 110 passed, 7.5s, 92% → [OK] Open htmlcov/

**Ex 3 - With failures**: User "Run tests" → Ask confirm → `pytest tests/ -v` → 148 passed, 3 failed, 18.7s → [WARN] List failures → `/debug-tests`

**Ex 4 - Component**: User "Test political analysis" → Recognize "political" → `pytest tests/ -m political -v` → 13 passed, 1.8s → [OK] Run integration for e2e
