# Tests

Automated test suite for Congressional Redistricting project.

## Quick Start

```bash
# Install dependencies
pip install pytest pytest-playwright playwright
playwright install chromium

# Generate test data (2 minutes)
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version test --states "VT,DE"

# Run all tests (< 3 minutes)
pytest tests/e2e/ -v

# Run smoke tests only (30 seconds)
pytest tests/e2e/ -m smoke -v
```

## Test Structure

```
tests/
├── e2e/                          # End-to-end Playwright tests
│   ├── conftest.py               # Pytest fixtures and configuration
│   ├── test_run_dashboard.py     # Run dashboard tests (30+ tests)
│   ├── test_master_dashboard.py  # Master dashboard tests (20+ tests)
│   └── test_visual_regression.py # Visual regression tests (15+ tests)
├── fixtures/                     # Test data fixtures
├── screenshots/                  # Screenshot baselines and diffs
│   ├── baseline/                 # Expected screenshots
│   └── diff/                     # Diff images on test failure
├── playwright.config.json        # Playwright configuration
└── README.md                     # This file
```

## Test Coverage

- **~75 total tests**
- **Execution time: < 3 minutes**
- **Coverage: 90%+ of dashboard features**

### Test Categories

| Test File | Tests | Time | Description |
|-----------|-------|------|-------------|
| `test_run_dashboard.py` | ~30 | 60s | Navigation, content, edge cases |
| `test_master_dashboard.py` | ~20 | 30s | Cross-run comparison |
| `test_visual_regression.py` | ~15 | 45s | Visual regression detection |

## Running Tests

### All Tests
```bash
pytest tests/e2e/ -v
```

### Smoke Tests (Quick Validation)
```bash
pytest tests/e2e/ -m smoke -v
```

### Visual Regression Tests
```bash
# Generate baselines
pytest tests/e2e/test_visual_regression.py --update-baselines

# Run visual tests
pytest tests/e2e/test_visual_regression.py -v
```

### Specific Test File
```bash
pytest tests/e2e/test_run_dashboard.py -v
```

### Specific Test
```bash
pytest tests/e2e/test_run_dashboard.py::test_state_navigation -v
```

### With Visible Browser (Debug Mode)
```bash
pytest tests/e2e/ --headed -v
```

## Test Markers

Tests are organized by marker for easy filtering:

| Marker | Description | Usage |
|--------|-------------|-------|
| `smoke` | Critical path tests | `pytest -m smoke` |
| `visual` | Visual regression tests | `pytest -m visual` |
| `slow` | Tests > 10 seconds | `pytest -m "not slow"` |

## What Tests Validate

### Data Integrity
- ✅ Pipeline output files exist
- ✅ CSV files have correct structure
- ✅ Maps render correctly
- ✅ Embedded data is valid JSON

### Functionality
- ✅ State navigation works
- ✅ Tab switching works
- ✅ Sorting/filtering works
- ✅ Links are valid

### Visual Regression
- ✅ UI elements render correctly
- ✅ Layout is consistent
- ✅ No unintended visual changes

### Performance
- ✅ Page load < 3 seconds
- ✅ No console errors
- ✅ Responsive layout works

## Developer Workflow

### Before Committing Pipeline Changes

```bash
# 1. Generate test data
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version test --states "VT,DE" --reset

# 2. Run smoke tests
pytest tests/e2e/ -m smoke -v

# 3. Run full suite
pytest tests/e2e/ -v

# 4. Review failures (if any)
ls tests/screenshots/diff/

# 5. Commit when tests pass
git commit -m "Update pipeline"
```

### After UI Changes

```bash
# 1. Update baselines
pytest tests/e2e/test_visual_regression.py --update-baselines

# 2. Verify baselines
pytest tests/e2e/test_visual_regression.py -v

# 3. Commit code and baselines
git add web/ tests/screenshots/baseline/
git commit -m "Update dashboard UI"
```

## Test Reports

### HTML Report
```bash
pytest tests/e2e/ --html=tests/test-results/report.html --self-contained-html
start tests/test-results/report.html
```

### Artifacts on Failure
- Screenshots: `tests/screenshots/diff/`
- Traces: `tests/test-results/` (view with `playwright show-trace`)

## Troubleshooting

### "Element not found"
```bash
# Run with visible browser to debug
pytest tests/e2e/ --headed -v
```

### "Missing test data"
```bash
# Generate test data
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version test --states "VT,DE"
```

### Visual regression false positives
```bash
# Regenerate baselines
pytest tests/e2e/test_visual_regression.py --update-baselines
```

### Playwright not installed
```bash
pip install pytest-playwright playwright
playwright install chromium
```

## Configuration

### Playwright Settings

See `playwright.config.json` for:
- Browser configuration
- Screenshot settings
- Timeout values
- Visual diff thresholds

### Pytest Settings

Markers are defined in `conftest.py`:
```python
markers = [
    "smoke: Quick smoke tests",
    "visual: Visual regression tests",
    "slow: Tests > 10 seconds"
]
```

## Full Documentation

For complete testing guide, see [docs/TESTING.md](../docs/TESTING.md)

## CI/CD Integration

Tests can be integrated into CI pipelines. See example GitHub Actions workflow in [docs/TESTING.md](../docs/TESTING.md#cicd-integration).
