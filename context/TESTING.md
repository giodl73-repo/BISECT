# Testing Guide

**Updated**: 2026-01-17

## Overview

**Test Suite**: 187 tests (110 unit, 21 integration, 56 E2E) - ~23s total

**Types**:
1. **Unit** (110) - Functions/classes in isolation, mock I/O - ~7s
2. **Integration** (21) - Component interactions, real fixtures - ~5s
3. **E2E** (20) - Complete workflows, full pipeline - ~8s

## Quick Start

```bash
# All tests
pytest tests/ -v                 # 18s

# By type
pytest tests/unit/ -v            # 7s
pytest tests/integration/ -v     # 5s
pytest tests/e2e/ -v             # 8s

# Specific test
pytest tests/unit/test_file.py::test_func -v

# With coverage
pytest tests/ --cov=src/apportionment --cov-report=html
```

## Test Structure

```
tests/
├─ unit/                    # 110 tests, 7s
│  ├─ partition/            # Bisection, METIS wrapper
│  ├─ data/                 # Loading, adjacency, validation
│  └─ visualization/        # Map generation, coloring
├─ integration/             # 21 tests, 5s
│  ├─ pipeline/             # State processing, orchestration
│  └─ analysis/             # Political, demographic, compactness
└─ e2e/                     # 56 tests, 11s
   ├─ test_redistricting_pipeline.py  # Full VT/DE runs
   ├─ test_pipeline_scripts.py        # ALL 26 pipeline scripts (36 tests)
   ├─ test_dashboard.py               # HTML generation
   └─ test_visual_regression.py       # UI consistency
```

## Unit Tests

**Purpose**: Test individual functions in isolation

**Characteristics**:
- Fast (milliseconds per test)
- Mock external dependencies (file I/O, subprocess)
- Test logic, not integration

**Examples**:
```python
# tests/unit/partition/test_recursive_bisection.py
def test_calculate_population_balance():
    assert calculate_balance([100, 100], 100) == 0.0
    assert calculate_balance([90, 110], 100) == 10.0

def test_validate_contiguity(mock_graph):
    assert is_contiguous(mock_graph, [0, 1, 2]) == True
    assert is_contiguous(mock_graph, [0, 5, 9]) == False
```

**Run**:
```bash
pytest tests/unit/ -v
pytest tests/unit/partition/ -v -k "bisection"
```

## Integration Tests

**Purpose**: Test component interactions

**Characteristics**:
- Real file I/O with test fixtures
- Mock expensive operations (METIS calls)
- Test coordination, not individual logic

**Examples**:
```python
# tests/integration/test_state_processing.py
def test_process_state_with_real_config(tmp_path):
    result = process_state('vermont', 2020, tmp_path)
    assert result.districts == 1
    assert (tmp_path / 'vermont_districts.csv').exists()

def test_config_loading_multi_year():
    config_2020 = load_config(2020)
    config_2010 = load_config(2010)
    assert config_2020['california'] == 52
    assert config_2010['california'] == 53
```

**Run**:
```bash
pytest tests/integration/ -v
```

## E2E Tests

**Purpose**: Complete end-to-end workflows

**Characteristics**:
- Use Vermont test data (single-district, fast)
- No mocks, full pipeline
- Verify outputs and artifacts

**Examples**:
```python
# tests/e2e/test_redistricting_pipeline.py
def test_complete_redistricting_vermont_2020(tmp_path):
    result = subprocess.run([
        sys.executable, 'scripts/pipeline/run_state_redistricting.py',
        '--state', 'vermont', '--year', '2020', '--version', 'test'
    ])
    assert result.returncode == 0
    assert (output_dir / 'districts' / 'vermont_districts.csv').exists()

# tests/e2e/test_dashboard.py
def test_generate_dashboard_with_test_data(tmp_path):
    generate_dashboard(year=2020, version='test', output=tmp_path)
    html = (tmp_path / 'dashboard.html').read_text()
    assert 'const districtData =' in html
    assert '<div id="map-container">' in html
```

**Run**:
```bash
pytest tests/e2e/ -v
```

## Pipeline Script E2E Tests

**Coverage**: ✅ ALL 26 pipeline scripts (36 tests, 100% coverage)

**Purpose**: Catch bugs in command-line pipeline scripts

**Why Needed**: Previous tests used mocks instead of real scripts, missing:
- Syntax errors (modules never imported)
- Missing function arguments
- Wrong file paths/directory structure
- Command-line interface issues

**What We Test**:
1. **Syntax Validation**: Scripts compile without errors
2. **Argument Validation**: Function calls have correct parameters
3. **Directory Structure**: Outputs go to correct subdirectories
4. **CLI Interface**: Scripts have proper argparse setup

**Examples**:
```python
# tests/e2e/test_pipeline_scripts.py

def test_script_imports_successfully():
    """Catch syntax errors by compiling script."""
    script = project_root / 'scripts' / 'pipeline' / 'analyze_district_demographics.py'

    with open(script, 'r', encoding='utf-8') as f:
        code = f.read()

    try:
        compile(code, str(script), 'exec')
    except SyntaxError as e:
        pytest.fail(f"Script has syntax error at line {e.lineno}: {e.msg}")

def test_create_rounds_hierarchy_calls_correct():
    """Catch missing function arguments."""
    script_path = project_root / 'scripts' / 'pipeline' / 'create_final_district_summary.py'

    with open(script_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # Check function signature includes state_code and census_year
    assert 'def create_rounds_hierarchy(run_dir: Path, num_districts: int, state_code: str, census_year: str' in content

    # Check all calls include required arguments (count commas)
    import re
    calls = re.findall(r'create_rounds_hierarchy\([^)]+\)', content)
    for call in calls:
        comma_count = call.count(',')
        assert comma_count >= 3, f"Missing arguments: {call}"

def test_output_directory_structure():
    """Verify outputs go to correct subdirectories."""
    script_path = project_root / 'scripts' / 'pipeline' / 'analyze_district_demographics.py'

    with open(script_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # Should create demographic/ subdirectory like political analysis
    assert "/ 'demographic'" in content or "demographic_dir" in content
    assert "district_demographics.csv" in content
```

**Real Bugs Caught**:
1. **Syntax Error**: `pass / 'demographic'` → Invalid syntax
2. **Missing Args**: `create_rounds_hierarchy(run_dir, num_districts)` → Missing state_code, census_year
3. **Wrong Directory**: Writing to root instead of `demographic/` subdirectory

**Scripts Tested (26/26)**:

*Core Pipeline*:
- process_single_state.py
- process_nation.py ⚠️ (V6 failure point)
- run_complete_redistricting.py
- run_state_redistricting.py
- add_cities_to_districts.py

*Analysis*:
- analyze_districts.py (political)
- analyze_district_demographics.py
- analyze_district_compactness.py
- create_final_district_summary.py

*National Aggregation*:
- create_us_aggregate.py
- create_us_rounds_hierarchy.py
- visualize_national_districts.py
- visualize_national_rounds.py ⚠️ (V6 failing script)

*Visualization* (9):
- visualize_individual_districts.py
- visualize_all_rounds.py
- visualize_partisan_lean.py
- visualize_district_demographics.py
- visualize_compactness.py
- visualize_metro_areas.py
- visualize_districts.py
- visualize_split.py

*Utilities*:
- create_single_district_states.py
- export_rounds_to_csv.py
- cleanup_district_summary.py
- fill_missing_cities.py
- fix_2010_missing_outputs.py

**Run**:
```bash
# All pipeline script tests (36 tests, ~8s)
pytest tests/e2e/test_pipeline_scripts.py -v

# Just syntax checks (fast)
pytest tests/e2e/test_pipeline_scripts.py -k "imports" -v

# Test specific script
pytest tests/e2e/test_pipeline_scripts.py::TestAnalyzeDistrictDemographics -v

# High priority only (scripts involved in V6 failure)
pytest tests/e2e/test_pipeline_scripts.py::TestProcessNation -v
pytest tests/e2e/test_pipeline_scripts.py::TestVisualizeNationalRounds -v
```

## Test Requirements (When Adding Code)

**Decision Tree**:
```
Add function/class?          YES → Unit tests (tests/unit/)
                              NO  ↓
Component interact?          YES → Integration tests (tests/integration/)
                              NO  ↓
Complete workflow?           YES → E2E tests (tests/e2e/)
                              NO  ↓
Visualization/dashboard?     YES → Dashboard tests (tests/e2e/)
                              NO  → Done
```

**Coverage Goals**:
- Unit: >80% coverage for new code
- Integration: Main path + common failures
- E2E: ≥1 complete Vermont workflow
- All: Reliable (no flaky tests)

## Test Patterns

### Fixtures (conftest.py)

```python
import pytest
from pathlib import Path

@pytest.fixture
def sample_tracts():
    return gpd.read_parquet('tests/fixtures/vermont_tracts.parquet')

@pytest.fixture
def tmp_output(tmp_path):
    output = tmp_path / 'outputs'
    output.mkdir()
    return output
```

### Mocking External Calls

```python
from unittest.mock import Mock, patch

@patch('subprocess.run')
def test_pipeline_calls_metis(mock_run):
    mock_run.return_value = Mock(returncode=0)
    result = run_redistricting('vermont', 2020)
    assert mock_run.called
```

### Testing File Generation

```python
def test_generates_district_csv(tmp_path):
    output_file = tmp_path / 'districts.csv'
    create_district_summary(output_file)

    assert output_file.exists()
    df = pd.read_csv(output_file)
    assert 'district' in df.columns
    assert len(df) == 1  # Vermont has 1 district
```

### Testing Error Handling

```python
def test_missing_data_graceful_failure():
    with pytest.raises(FileNotFoundError):
        load_tracts('nonexistent_state', 2020)

def test_invalid_year_returns_error():
    result = process_state('vermont', 1990)  # Invalid year
    assert result.returncode == 1
```

## Test Markers

```python
@pytest.mark.unit
def test_fast_unit_test(): pass

@pytest.mark.integration
def test_component_interaction(): pass

@pytest.mark.e2e
def test_complete_workflow(): pass

@pytest.mark.slow
def test_expensive_operation(): pass
```

**Run by marker**:
```bash
pytest -m unit          # Unit tests only
pytest -m integration   # Integration tests only
pytest -m "not slow"    # Skip slow tests
```

## Developer Workflow

### Before Committing

```bash
# 1. Run tests
pytest tests/ -v

# 2. Check coverage
pytest tests/ --cov=src/apportionment --cov-report=term-missing

# 3. Fix failures
pytest tests/unit/test_file.py::test_func -vvv  # Debug specific test

# 4. Commit
git add tests/
git commit -m "Add tests for new feature"
```

### After Pipeline Changes

```bash
# 1. Generate test data (2 min)
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version test --states "VT,DE" --reset

# 2. Run E2E tests (8s)
pytest tests/e2e/ -v

# 3. Commit if passing
```

### TDD (Test-Driven Development)

```bash
# 1. Write failing test
def test_new_feature():
    result = new_function(input)
    assert result == expected

# 2. Run test (should fail)
pytest tests/unit/test_file.py::test_new_feature -v

# 3. Implement function
def new_function(input):
    return expected

# 4. Run test (should pass)
pytest tests/unit/test_file.py::test_new_feature -v

# 5. Refactor (keep test passing)
```

## Common Issues

### Import Errors

**Problem**: `ModuleNotFoundError: No module named 'apportionment'`
**Fix**: Run from project root, ensure `src/` in PYTHONPATH

### Mock Not Working

**Problem**: Mock not being used
**Fix**: Check import path matches mock path exactly
```python
# Wrong
@patch('metis.partition')  # Mock top-level

# Right
@patch('apportionment.partition.metis_wrapper.partition')  # Mock actual import
```

### Fixture Not Found

**Problem**: `fixture 'sample_tracts' not found`
**Fix**: Define in `conftest.py` at appropriate level (test file dir or tests/)

### Test Data Missing

**Problem**: `FileNotFoundError: vermont_tracts.parquet`
**Fix**: Generate test fixtures first
```bash
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version test --states "VT"
```

### Slow Tests

**Problem**: Tests take too long
**Fix**: Mark as slow, use smaller datasets (VT not CA)
```python
@pytest.mark.slow
def test_california_redistricting(): pass
```

## Best Practices

### Test Independence
- ✅ Each test sets up own state
- ✅ No shared mutable state
- ✅ Use fixtures for common setup
- ❌ Don't depend on test execution order

### Test Naming
```python
# Pattern: test_<feature>_<scenario>_<expected>
def test_bisection_unbalanced_population_raises_error(): pass
def test_dashboard_vermont_has_1_district(): pass
def test_metis_wrapper_invalid_graph_returns_error(): pass
```

### Test Coverage
- **Critical paths**: 100% coverage
- **New code**: >80% coverage
- **Edge cases**: At least 1 test per edge case
- **Error handling**: Test failure modes

### Test Data
- Use **small datasets** (VT: 1 district, 60 tracts)
- Store fixtures in `tests/fixtures/`
- Don't commit large test data (>10MB)

### Assertions
```python
# ✅ Clear, specific assertions
assert len(districts) == 1
assert districts[0].population == 643503

# ❌ Vague assertions
assert districts  # Just checks truthy
```

## Coverage Report

```bash
# Generate HTML report
pytest tests/ --cov=src/apportionment --cov-report=html

# Open report
start htmlcov/index.html
```

**Target**: >80% overall, 100% critical paths

## CI/CD Integration

**GitHub Actions**:
```yaml
- name: Run tests
  run: pytest tests/ -v --cov=src/apportionment

- name: Upload coverage
  uses: codecov/codecov-action@v3
```

## Related Docs

- [CODING_PATTERNS.md](CODING_PATTERNS.md) - Testing patterns
- [ENHANCEMENT_WORKFLOW.md](ENHANCEMENT_WORKFLOW.md) - Test requirements
- `/run-tests` - Skill for running tests
- `/debug-tests` - Skill for debugging failures
