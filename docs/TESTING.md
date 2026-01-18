# Testing Guide

Comprehensive testing documentation for the Congressional Redistricting project.

**Last Updated**: January 17, 2026

## Overview

The project includes two types of tests:

1. **Unit Tests** - Python unit tests for METIS integration and core algorithms
2. **End-to-End Tests** - Playwright browser tests for dashboard validation

## Quick Start

### Run All Tests

```bash
# Run dashboard tests (Playwright)
run_dashboard_tests.bat

# Run smoke tests only (30 seconds)
run_dashboard_tests.bat --smoke

# Run with visible browser (debug mode)
run_dashboard_tests.bat --headed
```

### Generate Test Data

Dashboard tests require sample output data. Generate using:

```bash
# Generate test data (Vermont + Delaware, ~2 minutes)
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version test --states "VT,DE"
```

## Playwright Dashboard Tests

### Purpose

Automated end-to-end tests that validate:
- Dashboard navigation and interactivity
- Data integrity from pipeline outputs
- Visual regression (UI consistency)
- Cross-browser compatibility

**Primary Use Case**: Catch breaking changes from pipeline modifications before they reach production.

### Test Structure

```
tests/
├── e2e/                          # End-to-end tests
│   ├── conftest.py               # Pytest fixtures
│   ├── test_run_dashboard.py     # Run dashboard tests (30+ tests)
│   ├── test_master_dashboard.py  # Master dashboard tests (20+ tests)
│   └── test_visual_regression.py # Visual regression tests (15+ tests)
├── fixtures/                     # Test data
├── screenshots/                  # Screenshot baselines and diffs
│   ├── baseline/                 # Expected screenshots
│   └── diff/                     # Diff images on failure
└── playwright.config.json        # Playwright configuration
```

### Installation

```bash
# Install test dependencies
pip install pytest pytest-playwright playwright

# Install Chromium browser
playwright install chromium
```

### Running Tests

#### All Tests (Full Suite)

```bash
# Headless mode (default)
pytest tests/e2e/ -v

# With visible browser (debug)
pytest tests/e2e/ --headed -v

# Specific test file
pytest tests/e2e/test_run_dashboard.py -v

# Specific test
pytest tests/e2e/test_run_dashboard.py::test_state_navigation -v
```

#### Smoke Tests (Quick Validation)

```bash
# Critical path tests only (~30 seconds)
pytest tests/e2e/ -m smoke -v
```

Smoke tests validate:
- Dashboard loads successfully
- State navigation works
- Tabs switch correctly
- Critical maps render
- Page load time < 3 seconds

#### Visual Regression Tests

```bash
# Generate/update baselines (after intentional UI changes)
pytest tests/e2e/test_visual_regression.py --update-baselines

# Run visual regression tests
pytest tests/e2e/test_visual_regression.py -v
```

### Test Categories

Tests are organized by marker:

| Marker | Description | Count | Execution Time |
|--------|-------------|-------|----------------|
| `@pytest.mark.smoke` | Critical path tests | ~10 | 30 seconds |
| `@pytest.mark.visual` | Visual regression tests | ~15 | 45 seconds |
| `@pytest.mark.slow` | Tests > 10 seconds | ~5 | 60 seconds |

Run specific category:
```bash
pytest tests/e2e/ -m smoke -v     # Smoke tests only
pytest tests/e2e/ -m visual -v    # Visual tests only
pytest tests/e2e/ -m "not slow"   # Skip slow tests
```

### Test Coverage

#### Run Dashboard Tests (test_run_dashboard.py)

**Suite A: Navigation & Structure**
- ✅ Dashboard loads successfully
- ✅ Sidebar has 50 states
- ✅ All tabs present (Overview, Districts, Rounds, etc.)
- ✅ State navigation updates content
- ✅ Tab switching works correctly

**Suite B: Content Validation**
- ✅ Overview tab has district summary table
- ✅ Districts tab has map image
- ✅ Rounds tab has progression maps
- ✅ Compactness tab has analysis maps

**Suite C: Edge Cases**
- ✅ Multiple state switches
- ✅ Tab switching preserves state
- ✅ Page load time < 3 seconds

**Suite D: Interactive Features**
- ✅ Default tab is Overview
- ✅ Only one tab active at a time

**Suite E: Data Integrity**
- ✅ State items have data attributes
- ✅ Tab buttons have data attributes

**Suite F: Browser Compatibility**
- ✅ Responsive layout (desktop, laptop, tablet)

**Suite G: Console Errors**
- ✅ No console errors on load

#### Master Dashboard Tests (test_master_dashboard.py)

**Suite A: Dashboard Structure**
- ✅ Master dashboard loads
- ✅ Comparison table exists
- ✅ Run cards exist

**Suite B: Comparison Table**
- ✅ Correct headers (Run, PP, Reock, Districts, etc.)
- ✅ Contains data rows
- ✅ Columns are sortable
- ✅ Row hover effect

**Suite C: Run Cards**
- ✅ Card structure (title, metadata, button)
- ✅ Metadata display
- ✅ View button has valid href

**Suite D: Data Embedding**
- ✅ RUNS_DATA properly embedded
- ✅ COMPARISON_DATA properly embedded
- ✅ Embedded data has expected structure

**Suite E: Performance**
- ✅ Load time < 3 seconds
- ✅ No console errors

**Suite F: Responsive Layout**
- ✅ Layout at different viewport sizes

**Suite G: Multiple Runs**
- ✅ Handles multiple runs correctly

#### Visual Regression Tests (test_visual_regression.py)

**Suite A: Run Dashboard Visual Regression**
- ✅ Full page (desktop)
- ✅ Overview tab
- ✅ Districts tab
- ✅ Rounds tab
- ✅ Compactness tab
- ✅ Sidebar
- ✅ Header

**Suite B: Master Dashboard Visual Regression**
- ✅ Full page
- ✅ Comparison table
- ✅ Run cards

**Suite C: Responsive Design Regression**
- ✅ Run dashboard (desktop, laptop, tablet)
- ✅ Master dashboard (desktop, laptop, tablet)

**Suite D: State-Specific Visual Tests**
- ✅ Vermont overview
- ✅ Delaware overview

**Suite E: Screenshot Comparison**
- ✅ Overview tab comparison
- ✅ Comparison table comparison

### Test Execution Time

| Test Suite | Test Count | Execution Time |
|------------|------------|----------------|
| Smoke tests | ~10 | 30 seconds |
| Run dashboard tests | ~30 | 60 seconds |
| Master dashboard tests | ~20 | 30 seconds |
| Visual regression tests | ~15 | 45 seconds |
| **Total** | **~75** | **< 3 minutes** |

### Developer Workflow

#### Before Committing Pipeline Changes

```bash
# 1. Make pipeline change
edit src/apportionment/partition/recursive_bisection.py

# 2. Generate fresh test data (2 minutes)
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version test --states "VT,DE" --reset

# 3. Run smoke tests (30 seconds)
pytest tests/e2e/ -m smoke -v

# 4. If smoke tests pass, run full suite (< 3 minutes)
pytest tests/e2e/ -v

# 5. If tests fail, review screenshots
ls tests/screenshots/diff/

# 6. Fix issues and re-run
pytest tests/e2e/ -v

# 7. Commit when tests pass
git add .
git commit -m "Update recursive bisection algorithm"
git push
```

#### After Intentional UI Changes

```bash
# 1. Make UI change
edit web/dashboard.html

# 2. Update visual regression baselines
pytest tests/e2e/test_visual_regression.py --update-baselines

# 3. Run visual regression tests to verify
pytest tests/e2e/test_visual_regression.py -v

# 4. Commit both code and baseline updates
git add web/dashboard.html tests/screenshots/baseline/
git commit -m "Update dashboard header styling"
```

### What Tests Catch

#### Data Format Changes
- ✅ New/removed columns in `district_summary.csv`
- ✅ Changed field names (`GEOID` → `geoid`)
- ✅ Wrong data types (string instead of int)
- ✅ Missing required fields

#### File Structure Changes
- ✅ Renamed directories (`political/` → `political_analysis/`)
- ✅ Moved files (`maps/rounds/` → `rounds/`)
- ✅ Missing output files (forgot to generate maps)

#### Code Logic Errors
- ✅ Null/NaN values in critical fields
- ✅ Empty CSV files (0 rows)
- ✅ Broken image paths (404 errors)
- ✅ Invalid JSON in embedded data

#### UI/UX Regressions
- ✅ Tabs don't switch
- ✅ Navigation broken
- ✅ Maps don't render
- ✅ Tables empty when data exists
- ✅ Unintended CSS changes

### Troubleshooting

#### Tests Fail with "Element not found"

**Problem**: Element selector not found on page

**Solution**:
```bash
# Run with headed mode to see what's happening
pytest tests/e2e/ --headed -v

# Check if test data was generated correctly
ls outputs/us_2020_test/states/vermont/
```

#### Visual Regression False Positives

**Problem**: Screenshots differ due to font rendering, anti-aliasing, etc.

**Solution**:
```bash
# Option 1: Regenerate baselines (if change was intentional)
pytest tests/e2e/test_visual_regression.py --update-baselines

# Option 2: Adjust threshold in playwright.config.json
{
  "expect": {
    "toHaveScreenshot": {
      "maxDiffPixels": 200  // Increase tolerance
    }
  }
}
```

#### Tests Timeout on Large States

**Problem**: Tests timeout waiting for content to load

**Solution**: Tests use small states (VT, DE) by default. If testing with larger states:
```python
# In test file: Increase timeout for specific tests
page.set_default_timeout(60000)  # 60 seconds
```

#### Playwright Not Installed

**Problem**: `playwright` module not found

**Solution**:
```bash
pip install pytest-playwright playwright
playwright install chromium
```

#### Missing Test Data

**Problem**: Tests fail because sample data doesn't exist

**Solution**:
```bash
# Generate test data first
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version test --states "VT,DE"
```

### Test Reports

#### HTML Report

```bash
# Generate HTML report
pytest tests/e2e/ --html=tests/test-results/report.html --self-contained-html

# Open report
start tests/test-results/report.html
```

#### Screenshot Artifacts

When tests fail:
- **Diff screenshots**: `tests/screenshots/diff/`
- **Baseline screenshots**: `tests/screenshots/baseline/`
- **Test traces**: `tests/test-results/` (use `playwright show-trace` to view)

### Configuration

#### Playwright Configuration (playwright.config.json)

```json
{
  "use": {
    "headless": true,
    "viewport": { "width": 1920, "height": 1080 },
    "screenshot": "only-on-failure",
    "trace": "retain-on-failure",
    "video": "retain-on-failure"
  },
  "expect": {
    "toHaveScreenshot": {
      "maxDiffPixels": 100,
      "maxDiffPixelRatio": 0.001
    }
  },
  "timeout": 30000
}
```

#### Pytest Configuration

Add to `pytest.ini` (if needed):
```ini
[pytest]
testpaths = tests/e2e
python_files = test_*.py
python_classes = Test*
python_functions = test_*
markers =
    smoke: Quick smoke tests for critical functionality
    visual: Visual regression tests
    slow: Tests that take longer than 10 seconds
```

### CI/CD Integration

Tests can be integrated into GitHub Actions, GitLab CI, or other CI systems.

#### Example GitHub Actions Workflow

```yaml
name: Dashboard Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: 3.13

      - name: Install dependencies
        run: |
          pip install -r requirements.txt
          playwright install chromium

      - name: Generate test fixtures
        run: |
          python scripts/pipeline/run_complete_redistricting.py --year 2020 --version test --states "VT,DE"

      - name: Run tests
        run: pytest tests/e2e/ --browser chromium --headed=false --tracing on -v

      - name: Upload test artifacts
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: test-results
          path: |
            tests/screenshots/diff/
            tests/test-results/
```

## Unit Tests

### METIS Integration Tests

Located in `scripts/tests/`:
- `test_metis_integration.py` - Tests for METIS wrapper

Run unit tests:
```bash
pytest scripts/tests/ -v
```

## Best Practices

### Test Independence

- Tests should not depend on each other
- Each test should set up its own state
- Use fixtures for common setup

### Test Naming

Follow naming convention:
```python
def test_<feature>_<scenario>_<expected_result>():
    """Test that <feature> <does what> when <scenario>."""
    pass
```

Examples:
- `test_state_navigation_updates_header()`
- `test_comparison_table_has_correct_headers()`
- `test_visual_regression_overview_tab()`

### Screenshot Stability

For visual regression tests:
- Disable animations/transitions in test mode
- Use fixed viewport sizes
- Wait for `networkidle` state before screenshots
- Use reasonable pixel diff thresholds (0.1%)

### Performance Testing

- Page load should be < 3 seconds
- Tab switching should be < 1 second
- State navigation should be < 1 second

Mark slow tests:
```python
@pytest.mark.slow
def test_large_state_performance():
    pass
```

## Coverage Goals

Target test coverage:
- **90%+** of dashboard features
- **95%+** of critical user paths (smoke tests)
- **100%** of data format validations
- **Zero** false positives on main branch

## Related Documentation

- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System architecture
- **[CODING_PATTERNS.md](CODING_PATTERNS.md)** - Coding conventions
- **[Enhancement 30](enhancements/active/30_playwright_testing.md)** - Test harness implementation details

## Questions?

For issues or questions about testing:
- Review test output and screenshots
- Check this documentation
- Review Enhancement 30 specification
- Run tests with `--headed` flag to debug
