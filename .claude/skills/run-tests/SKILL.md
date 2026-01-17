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

## Overview

Execute the test suite with intelligent filtering, clear reporting, and actionable guidance. This skill provides a streamlined interface to the 151-test suite, supporting execution by category (unit/integration/E2E), by component (redistricting, political, demographic, etc.), with coverage reporting, and with failure analysis.

## Prerequisites

**Required**:
- Test suite installed (pytest, pytest-playwright)
- Tests located in `tests/` directory
- Mock data fixtures in `tests/fixtures/` and `tests/mocks/`

**Optional**:
- pytest-cov for coverage reporting
- Chromium browser for E2E tests (`playwright install chromium`)

## When to Use This Skill

- User says: "Run tests"
- User says: "Run all tests"
- User says: "Run unit tests"
- User says: "Run tests with coverage"
- User says: "Test the redistricting code"
- When validating code changes before commit
- When checking test suite health
- When generating coverage reports
- After implementing new features

## Workflow

### Step 1: Determine Test Scope

Ask user what tests to run using `AskUserQuestion`:

**Test Type Options**:
- **All tests** (151 tests, ~18 seconds) - Complete validation
- **Unit tests** (110 tests, ~7 seconds) - Fast component testing
- **Integration tests** (21 tests, ~3 seconds) - Multi-stage pipeline flows
- **E2E dashboard tests** (20 tests, ~8 seconds) - Full dashboard validation
- **Specific component** - Filter by marker (redistricting, political, demographic, etc.)

**Additional Options**:
- **With coverage** - Generate HTML coverage report
- **Verbose output** - Show detailed test information
- **Failed first** - Run previously failed tests first
- **Show markers** - Display available test markers

### Step 2: Build Pytest Command

Based on user selection, construct appropriate pytest command:

**All tests**:
```bash
pytest tests/ -v
```

**By test type**:
```bash
pytest tests/unit/ -v           # Unit tests only
pytest tests/integration/ -v    # Integration tests only
pytest tests/e2e/ -v           # E2E tests only
```

**By component marker**:
```bash
pytest tests/ -m redistricting -v    # Redistricting tests
pytest tests/ -m political -v        # Political analysis tests
pytest tests/ -m demographic -v      # Demographic tests
pytest tests/ -m compactness -v      # Compactness tests
pytest tests/ -m visualization -v    # Visualization tests
pytest tests/ -m dashboard -v        # Dashboard tests
```

**With coverage**:
```bash
pytest tests/ --cov=apportionment --cov-report=html -v
```

**Advanced options**:
```bash
pytest tests/ --lf -v              # Run last failed tests first
pytest tests/ --markers            # Show available markers
pytest tests/ -v --tb=long         # Verbose failure tracebacks
pytest tests/ -v -s               # Show print statements
```

### Step 3: Execute Tests

Use `Bash` tool to run pytest command:

```bash
cd C:\src\apportionment
pytest [command from Step 2]
```

**Monitor execution**:
- Track progress messages
- Note test counts (passed/failed/skipped)
- Capture any error output
- Record execution time

**Handle interruptions**:
- If user cancels: Report partial results
- If timeout: Suggest running smaller subset
- If crash: Offer to use `/debug-tests` skill

### Step 4: Parse Results

Extract key statistics from pytest output using `Grep` if needed:

**Look for**:
- Total tests run
- Number passed (green)
- Number failed (red)
- Number skipped (yellow)
- Execution time
- Coverage percentage (if --cov used)

**Example output patterns**:
```
====== 151 passed in 18.23s ======
====== 110 passed in 7.12s ======
====== 18 passed, 2 failed in 8.45s ======
====== 100 passed, 5 skipped in 15.32s ======
```

### Step 5: Report Summary

Provide clear summary with statistics:

**Success (all passing)**:
```
[OK] All tests passed!

Summary:
- Tests run: 151
- Passed: 151 (100%)
- Failed: 0
- Skipped: 0
- Time: 18.2 seconds

Coverage: 92% (if --cov used)
```

**Partial failures**:
```
[WARN] Some tests failed

Summary:
- Tests run: 151
- Passed: 148 (98%)
- Failed: 3 (2%)
- Skipped: 0
- Time: 18.5 seconds

Failed tests:
- tests/unit/test_political_analysis.py::test_seat_calculation
- tests/integration/test_single_state_flow.py::test_vermont_pipeline
- tests/e2e/test_run_dashboard.py::test_political_tab

Next step: Use /debug-tests to investigate failures
```

**All failed / Critical errors**:
```
[FAIL] Test suite failed

Summary:
- Tests run: 12
- Passed: 0
- Failed: 12 (100%)
- Time: 2.1 seconds

Critical issue detected (all tests failing)
Common causes:
- Import errors (check PYTHONPATH)
- Missing dependencies (check requirements.txt)
- Mock data not generated (check fixtures)

Next step: Use /debug-tests for guided troubleshooting
```

### Step 6: Suggest Next Steps

Based on results, recommend actions:

**If all passed**:
- Consider running with coverage to check code coverage
- Ready to commit changes
- Run full pipeline test if major changes made

**If some failed**:
- Use `/debug-tests` to investigate specific failures
- Re-run failed tests with `pytest --lf -v`
- Check if failures are related to recent changes

**If many failed**:
- Use `/debug-tests` for systematic troubleshooting
- Check common issues (imports, mocks, data)
- Verify test environment is set up correctly

**If coverage generated**:
- Open HTML report: `start htmlcov/index.html` (Windows)
- Review uncovered code sections
- Identify areas needing more tests

## Test Categories

### By Test Type

**Unit Tests** (110 tests, 7 seconds):
- Redistricting algorithm (10 tests)
- METIS integration (27 tests)
- Political analysis (13 tests)
- Demographic analysis (13 tests)
- Compactness metrics (15 tests)
- Visualization (18 tests)
- Aggregation (14 tests)

**Integration Tests** (21 tests, 3 seconds):
- Single-state pipeline flows
- National aggregation
- Multi-stage validation

**E2E Dashboard Tests** (20 tests, 8 seconds):
- Dashboard functionality (9 tests)
- Artifact validation (11 tests)

### By Component Marker

Available markers:
- `redistricting` - Redistricting algorithm tests
- `political` - Political analysis tests
- `demographic` - Demographic analysis tests
- `compactness` - Compactness metric tests
- `visualization` - Map generation tests
- `aggregation` - CSV aggregation tests
- `dashboard` - Dashboard E2E tests
- `unit` - All unit tests
- `integration` - All integration tests
- `slow` - Tests taking >10 seconds

## Coverage Reporting

When running with `--cov`:

**HTML Report Generated**:
- Location: `htmlcov/index.html`
- View: `start htmlcov/index.html` (Windows) or `open htmlcov/index.html` (Mac)

**Coverage Interpretation**:
- **90-100%**: Excellent coverage
- **70-90%**: Good coverage, some gaps
- **50-70%**: Moderate coverage, needs improvement
- **<50%**: Low coverage, significant testing needed

**Files to Check**:
- `apportionment/partition/` - Core algorithm
- `apportionment/data/` - Data processing
- `apportionment/visualization/` - Visualization
- `scripts/` - Pipeline scripts (not included in coverage)

## Quick Commands Reference

```bash
# All tests (~18 seconds)
pytest tests/ -v

# Fast unit tests only (~7 seconds)
pytest tests/unit/ -v

# Integration tests (~3 seconds)
pytest tests/integration/ -v

# E2E dashboard tests (~8 seconds)
pytest tests/e2e/ -v

# With coverage report
pytest tests/ --cov=apportionment --cov-report=html -v

# Run previously failed tests first
pytest tests/ --lf -v

# Show available markers
pytest tests/ --markers

# Verbose with full tracebacks
pytest tests/ -v --tb=long

# Run specific component
pytest tests/ -m redistricting -v
pytest tests/ -m political -v
pytest tests/ -m dashboard -v

# Run with debugger (stops on failure)
pytest tests/ --pdb -v

# Show print statements
pytest tests/ -v -s
```

## Troubleshooting

**Import Errors**:
```
Issue: ModuleNotFoundError: No module named 'apportionment'
Solution: Set PYTHONPATH: export PYTHONPATH="${PYTHONPATH}:$(pwd)"
```

**Playwright Not Installed**:
```
Issue: Playwright browser not found
Solution: Install browser: playwright install chromium
```

**Mock Data Missing**:
```
Issue: FileNotFoundError in test fixtures
Solution: Mock data generated automatically by conftest.py fixture
         If issues persist, use /debug-tests
```

**Coverage Not Generated**:
```
Issue: --cov flag not recognized
Solution: Install pytest-cov: pip install pytest-cov
```

**Tests Taking Too Long**:
```
Issue: E2E tests timing out
Solution: Use --headed --slowmo flags to debug
         Or run smaller subset: pytest tests/e2e/test_run_dashboard.py -v
```

## Performance Notes

**Execution Times** (typical):
- All tests: ~18 seconds
- Unit tests: ~7 seconds
- Integration tests: ~3 seconds
- E2E tests: ~8 seconds

**Parallelization**:
Currently tests run sequentially. For faster execution:
```bash
# Install pytest-xdist
pip install pytest-xdist

# Run with 4 workers
pytest tests/ -n 4 -v
```

**Test Optimization**:
- Mock data generated once per session (session-scoped fixture)
- E2E tests reuse same mock run
- Unit tests use lightweight mocks (no real data)

## What You'll Get

After successful execution:
- **Test results summary** with pass/fail statistics
- **Execution time** for selected test category
- **Failure details** if any tests failed
- **Coverage report** (if --cov used) in `htmlcov/`
- **Next step recommendations** based on results
- **Quick commands** for re-running or debugging

## Next Steps

After running tests:

**If all passed**:
- Commit changes with confidence
- Run coverage analysis if not done: `pytest tests/ --cov=apportionment --cov-report=html -v`
- Consider running full pipeline test for integration validation

**If some failed**:
- Use `/debug-tests` to investigate failures
- Re-run failed tests: `pytest --lf -v`
- Check recent code changes for issues

**If coverage low**:
- Identify uncovered code in `htmlcov/index.html`
- Write additional tests for uncovered functions
- Focus on critical paths (redistricting, analysis)

**For CI/CD integration**:
- Add pytest command to workflow
- Use exit codes for pass/fail detection
- Upload coverage reports to Codecov

## Related Skills

- `/debug-tests` - Systematically debug test failures
- `/enhancement-implement` - Includes testing as Phase 4
- `/pipeline-debug` - Debug pipeline failures (separate from tests)

## Examples

### Example 1: Run All Tests

**User**: "Run all tests"

**Skill actions**:
1. Ask: "Run all 151 tests?" → User confirms
2. Execute: `pytest tests/ -v`
3. Parse: 151 passed in 18.23s
4. Report: [OK] All tests passed! Time: 18.2s
5. Suggest: "Ready to commit"

### Example 2: Run Unit Tests with Coverage

**User**: "Run unit tests with coverage"

**Skill actions**:
1. Ask: "Run 110 unit tests with coverage?" → User confirms
2. Execute: `pytest tests/unit/ --cov=apportionment --cov-report=html -v`
3. Parse: 110 passed in 7.45s, Coverage: 92%
4. Report: [OK] All unit tests passed! Coverage: 92%
5. Suggest: "Open htmlcov/index.html to view coverage details"

### Example 3: Run Tests with Failures

**User**: "Run tests"

**Skill actions**:
1. Ask: "Run all tests?" → User confirms
2. Execute: `pytest tests/ -v`
3. Parse: 148 passed, 3 failed in 18.67s
4. Report: [WARN] 3 tests failed (see list)
5. Suggest: "Use /debug-tests to investigate failures"

### Example 4: Run Specific Component

**User**: "Test the political analysis code"

**Skill actions**:
1. Recognize "political analysis" → suggest `-m political`
2. Execute: `pytest tests/ -m political -v`
3. Parse: 13 passed in 1.82s
4. Report: [OK] All political analysis tests passed!
5. Suggest: "Run integration tests to validate end-to-end"
