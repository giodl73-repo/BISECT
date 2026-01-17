# Tests

Comprehensive automated test suite for Congressional Redistricting project.

## Quick Start

```bash
# Install dependencies
pip install pytest pytest-playwright playwright pandas geopandas
playwright install chromium

# Run all tests (< 20 seconds)
pytest tests/ -v

# Run only E2E dashboard tests (8 seconds)
pytest tests/e2e/ -v

# Run only unit tests (7 seconds)
pytest tests/unit/ -v
```

## Test Structure

```
tests/
├── unit/                          # Unit tests (299 tests, 9s)
│   ├── test_syntax_validation.py  # **NEW** Python syntax validation (189 tests)
│   ├── test_redistricting.py      # Core algorithm tests
│   ├── test_metis_integration.py  # METIS wrapper tests
│   ├── test_political_analysis.py # Political analysis tests
│   ├── test_demographic_analysis.py # Demographics tests
│   ├── test_compactness_analysis.py # Compactness metrics tests
│   ├── test_visualization.py      # Map generation tests
│   ├── test_run_config.py         # Config system tests
│   └── test_aggregation.py        # CSV aggregation tests
│
├── integration/                   # Integration tests (21 tests, 3s)
│   ├── test_single_state_flow.py  # Single-state pipeline tests
│   └── test_national_aggregation.py # National rollup tests
│
├── e2e/                           # E2E Playwright tests (20 tests, 8s)
│   ├── conftest.py                # Pytest fixtures (mock_run fixture)
│   └── test_run_dashboard.py      # Dashboard tests with mock data
│
├── mocks/                         # Mock data generators
│   ├── mock_tracts.py             # Mock census tracts
│   ├── mock_adjacency.py          # Mock adjacency graphs
│   ├── mock_districts.py          # Mock district assignments
│   ├── mock_analysis.py           # Mock analysis results
│   └── mock_maps.py               # Mock map generation
│
├── fixtures/                      # Test fixtures
│   └── generate_mock_run.py       # Complete mock run generator
│
├── utils/                         # Test utilities
│   ├── assertions.py              # Custom assertions
│   ├── validators.py              # Data validators
│   └── cleanup.py                 # Cleanup helpers
│
├── conftest.py                    # Shared pytest fixtures
├── pytest.ini                     # Pytest configuration
├── README.md                      # This file
├── PIPELINE_TESTS.md              # Unit/integration test guide
└── TEST_SUMMARY.md                # Complete test results
```

## Test Coverage

**Total: 340 tests, ~20 seconds execution time**

### Test Breakdown

| Category | Tests | Time | Description |
|----------|-------|------|-------------|
| **Syntax Validation** | 189 | 2s | All Python files compile without errors |
| **Unit Tests** | 110 | 7s | Individual component tests with mocks |
| **Integration Tests** | 21 | 3s | Multi-stage pipeline flow tests |
| **E2E Dashboard Tests** | 20 | 8s | Full dashboard with mock data |
| **Total** | **340** | **~20s** | **Complete test coverage** |

### Coverage by Component

| Component | Tests | Coverage |
|-----------|-------|----------|
| **Python Syntax** | **189** | **100%** |
| Redistricting Algorithm | 10 | 95%+ |
| METIS Integration | 27 | 95%+ |
| Political Analysis | 13 | 90%+ |
| Demographic Analysis | 13 | 90%+ |
| Compactness Metrics | 15 | 90%+ |
| Visualization | 18 | 85%+ |
| Aggregation | 14 | 90%+ |
| Dashboard E2E | 20 | 90%+ |
| Pipeline Integration | 21 | 85%+ |

## Running Tests

### With Claude Code Skills (Recommended)

Claude Code provides intelligent test execution and debugging skills:

**`/run-tests`** - Execute tests with filtering and reporting
- Asks what to run (all/unit/integration/E2E/component)
- Supports coverage reporting
- Clear summaries and statistics
- Suggests next steps based on results

**`/debug-tests`** - Systematically debug test failures
- Automatic failure pattern detection (6+ common patterns)
- Guided troubleshooting steps
- Common issue checks
- Specific fix suggestions

**Example workflows**:
- Say: "Run all tests" → Claude offers `/run-tests`
- Say: "Run unit tests with coverage" → Claude offers `/run-tests` with options
- Say: "Why are my tests failing?" → Claude offers `/debug-tests`

See [docs/SKILLS.md](../docs/SKILLS.md#testing--validation-skills) for detailed documentation.

### Direct pytest Commands

### All Tests
```bash
# All 151 tests (~18 seconds)
pytest tests/ -v

# With coverage report
pytest tests/ --cov=apportionment --cov-report=html
```

### By Test Type
```bash
# Syntax validation only (189 tests, 2s) **NEW**
pytest tests/unit/test_syntax_validation.py -v

# Unit tests only (299 tests, 9s)
pytest tests/unit/ -v

# Integration tests only (21 tests, 3s)
pytest tests/integration/ -v

# E2E dashboard tests only (20 tests, 8s)
pytest tests/e2e/ -v
```

### By Component
```bash
# METIS integration tests
pytest tests/unit/test_metis_integration.py -v

# Political analysis tests
pytest tests/unit/test_political_analysis.py -v

# Dashboard artifact validation
pytest tests/e2e/test_run_dashboard.py -v
```

### By Marker
```bash
# Quick tests only
pytest tests/ -m "not slow" -v

# Unit tests for redistricting
pytest tests/unit/ -m redistricting -v

# Integration tests
pytest tests/integration/ -m integration -v
```

### Specific Test
```bash
# Single test
pytest tests/unit/test_metis_integration.py::test_metis_basic_partition -v

# Test class
pytest tests/unit/test_metis_integration.py::TestMETISOddDistrictCounts -v
```

### Debug Mode (E2E Tests)
```bash
# With visible browser
pytest tests/e2e/ --headed -v

# With slow motion (500ms between actions)
pytest tests/e2e/ --headed --slowmo=500 -v
```

## Test Markers

Tests are organized by marker for easy filtering:

| Marker | Description | Usage |
|--------|-------------|-------|
| `unit` | Unit tests (isolated components) | `pytest -m unit` |
| `integration` | Integration tests (multi-stage) | `pytest -m integration` |
| `redistricting` | Redistricting algorithm tests | `pytest -m redistricting` |
| `political` | Political analysis tests | `pytest -m political` |
| `demographic` | Demographic analysis tests | `pytest -m demographic` |
| `compactness` | Compactness metric tests | `pytest -m compactness` |
| `visualization` | Map generation tests | `pytest -m visualization` |
| `aggregation` | CSV aggregation tests | `pytest -m aggregation` |
| `slow` | Tests > 10 seconds | `pytest -m "not slow"` |

## What Tests Validate

### Syntax Validation Tests (189 tests) **NEW**
- ✅ **All Python Files**: Every .py file compiles without syntax errors (189 files checked)
- ✅ **Critical Scripts**: Pipeline, visualization, analysis scripts explicitly validated
- ✅ **Common Issues**: Unterminated f-strings, broken multi-line strings, import errors
- ✅ **Coverage**: 100% of Python codebase (src/, scripts/, tests/, tools/)
- ⚡ **Fast**: 2 seconds to check entire codebase
- 🛡️ **Guardian**: Catches syntax errors before they reach production

**Why This Matters:**
The Enhancement 29 incident demonstrated that syntax errors in visualization scripts weren't caught by existing tests because those scripts were never imported during test execution. This new test closes that gap by validating all Python files compile correctly.

### Unit Tests (110 tests)
- ✅ **Redistricting Algorithm**: Recursive bisection, split logic, subgraph extraction
- ✅ **METIS Integration**: Graph partitioning, weighted/unweighted, odd districts, population balance
- ✅ **Political Analysis**: Vote shares, seat counts, partisan lean, winner calculation
- ✅ **Demographic Analysis**: Race/ethnicity shares, diversity index, representation metrics
- ✅ **Compactness Metrics**: Polsby-Popper calculation, geometric properties
- ✅ **Visualization**: State maps, national maps, color schemes, PNG validation
- ✅ **Aggregation**: CSV merging, ranking, statistics, data quality checks
- ✅ **Configuration**: RunConfig system, JSON serialization, validation

### Integration Tests (21 tests)
- ✅ **Single-State Flow**: Complete pipeline (VT/AL), multi-year, error handling
- ✅ **National Aggregation**: Cross-state rollup, 435 districts, summary generation
- ✅ **Multi-Stage Validation**: Data flows correctly through all pipeline stages

### E2E Dashboard Tests (20 tests)
- ✅ **Dashboard Functionality**: All 6 tabs, state switching, table data, maps
- ✅ **Artifact Validation**: All CSVs exist, all maps exist, correct structure
- ✅ **Pipeline Guardian**: Tests fail if pipeline produces incorrect/missing files
- ✅ **Data Integrity**: CSV columns correct, row counts match, no null values
- ✅ **Mock Data**: Fast test execution without 4-hour real runs

## Developer Workflow

### Before Committing Code Changes

```bash
# 1. Quick syntax check (< 2 seconds) **NEW**
pytest tests/unit/test_syntax_validation.py -v

# 2. Run all tests (< 20 seconds)
pytest tests/ -v

# 3. Check coverage (optional)
pytest tests/ --cov=apportionment --cov-report=html

# 4. Commit when all tests pass
git commit -m "Update code"
```

**Pro Tip:** The syntax validation test is so fast (2 seconds) that you can run it before every commit to catch typos and syntax errors immediately.

### After Algorithm Changes

```bash
# 1. Run redistricting tests
pytest tests/unit/ -m redistricting -v

# 2. Run METIS integration tests
pytest tests/unit/test_metis_integration.py -v

# 3. Run integration tests
pytest tests/integration/ -v

# 4. Commit when all pass
git commit -m "Update redistricting algorithm"
```

### After Analysis Script Changes

```bash
# 1. Run component tests
pytest tests/unit/test_political_analysis.py -v
pytest tests/unit/test_demographic_analysis.py -v
pytest tests/unit/test_compactness_analysis.py -v

# 2. Run integration tests
pytest tests/integration/ -v

# 3. Commit when all pass
git commit -m "Update analysis scripts"
```

### After Dashboard Changes

```bash
# 1. Run E2E tests with mock data
pytest tests/e2e/ -v

# 2. Verify artifact validation passes
pytest tests/e2e/test_run_dashboard.py -k "artifact" -v

# 3. Commit when all pass
git commit -m "Update dashboard"
```

## Test Reports

### HTML Coverage Report
```bash
# Generate coverage report
pytest tests/ --cov=apportionment --cov-report=html

# Open in browser
start htmlcov/index.html  # Windows
open htmlcov/index.html   # Mac
```

### Verbose Test Report
```bash
# Detailed test output
pytest tests/ -v --tb=long

# Show stdout even for passing tests
pytest tests/ -v -s
```

### Artifacts on E2E Test Failure
- Screenshots: `tests/screenshots/` (for debugging visual issues)
- Playwright traces: `tests/test-results/` (view with `playwright show-trace`)

## Troubleshooting

### Import Errors
```bash
# Ensure project root in Python path
export PYTHONPATH="${PYTHONPATH}:$(pwd)"  # Unix
set PYTHONPATH=%PYTHONPATH%;%CD%          # Windows
```

### Playwright Not Installed
```bash
pip install pytest-playwright playwright
playwright install chromium
```

### E2E Tests - Element Not Found
```bash
# Run with visible browser to debug
pytest tests/e2e/ --headed -v

# Run with slow motion
pytest tests/e2e/ --headed --slowmo=500 -v
```

### Tests Run But Mock Data Not Generated
```bash
# Mock data is generated automatically by pytest fixture
# If issues persist, generate manually:
python tests/fixtures/generate_mock_run.py --states "vermont,alabama" --year 2020 --version test
```

### Coverage Not Generated
```bash
pip install pytest-cov
```

## Configuration

### Pytest Configuration

See `pytest.ini` for:
- Test markers (unit, integration, redistricting, political, etc.)
- Warning filters
- Test discovery patterns
- Python path settings

### E2E Test Configuration

E2E tests use mock data fixtures defined in:
- `tests/e2e/conftest.py` - Mock run fixture
- `tests/fixtures/generate_mock_run.py` - Complete mock run generator
- `tests/mocks/` - Individual mock data generators

## Mock Data System

The test suite uses a comprehensive mock data system:

**Mock Generators:**
- `mock_tracts.py` - Generates realistic census tract GeoDataFrames
- `mock_adjacency.py` - Creates NetworkX adjacency graphs
- `mock_districts.py` - Produces population-balanced district assignments
- `mock_analysis.py` - Generates political, demographic, compactness data
- `mock_maps.py` - Creates PNG placeholder maps

**Complete Run Generator:**
- `generate_mock_run.py` - Generates full pipeline output with all CSVs and maps
- Used by E2E tests to create complete dashboard test environment
- Generates data for Vermont (1 district) and Alabama (7 districts)
- Includes all analysis types: political, demographic, compactness, urban, national

## Documentation

### Test Documentation
- **`PIPELINE_TESTS.md`** - Unit and integration test guide
- **`TEST_SUMMARY.md`** - Complete test results and statistics
- **`README.md`** (this file) - Overview and quick start

### Enhancement Documentation
- **Enhancement 30** - Playwright Test Harness (`docs/enhancements/active/30_playwright_testing.md`)
- **Enhancement 31** - Pipeline Test System (`docs/enhancements/active/31_pipeline_test_system.md`)
- **Enhancement 33** - Dashboard Mock Data Integration (`docs/enhancements/active/33_dashboard_mock_data.md`)

## CI/CD Integration

All tests are designed for CI/CD:
- **Fast execution**: < 20 seconds for all 151 tests
- **No external dependencies**: Mock data generated automatically
- **Comprehensive coverage**: 90%+ code coverage
- **Pipeline guardian**: Tests fail if pipeline output is incorrect

### Example GitHub Actions Workflow

```yaml
name: Test Suite
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions/setup-python@v2
        with:
          python-version: 3.13
      - name: Install dependencies
        run: |
          pip install pytest pytest-playwright pytest-cov pandas geopandas
          playwright install chromium
      - name: Run all tests
        run: pytest tests/ -v --cov=apportionment
      - name: Upload coverage
        uses: codecov/codecov-action@v2
```
