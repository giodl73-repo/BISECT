# Pipeline Test System

Comprehensive testing framework for redistricting pipeline unit and integration tests.

## Overview

This test system complements the E2E dashboard tests with comprehensive coverage for all pipeline stages:

- **Unit Tests**: Test individual components in isolation (~40 tests)
- **Integration Tests**: Test complete pipeline flows (~15 tests)
- **Mock Data Generators**: Realistic test data for all stages
- **Test Utilities**: Reusable assertions, validators, cleanup helpers

For E2E dashboard tests, see `tests/README.md`.

## Quick Start

### Run All Pipeline Tests

```bash
# Unit + integration tests
run_tests.bat all

# Or use pytest directly
python -m pytest tests/unit tests/integration -v
```

### Run Specific Test Types

```bash
run_tests.bat unit           # Unit tests only
run_tests.bat integration    # Integration tests only
run_tests.bat quick          # Quick suite (fail fast)
run_tests.bat coverage       # With coverage report
```

### Run Tests by Category

```bash
pytest tests/unit -m redistricting    # Redistricting algorithm
pytest tests/unit -m political        # Political analysis
pytest tests/unit -m demographic      # Demographics
pytest tests/unit -m compactness      # Compactness metrics
pytest tests/unit -m visualization    # Map generation
pytest tests/unit -m aggregation      # CSV aggregation
```

## Directory Structure

```
tests/
├── unit/                          # Unit tests (110 tests)
│   ├── test_redistricting.py     # Core algorithm (10 tests)
│   ├── test_metis_integration.py # METIS wrapper (27 tests)
│   ├── test_political_analysis.py # Political (13 tests)
│   ├── test_demographic_analysis.py # Demographics (13 tests)
│   ├── test_compactness_analysis.py # Compactness (15 tests)
│   ├── test_visualization.py     # Maps (18 tests)
│   └── test_aggregation.py       # Aggregation (14 tests)
│
├── integration/                   # Integration tests (21 tests)
│   ├── test_single_state_flow.py # Single-state flow (13 tests)
│   └── test_national_aggregation.py # National (8 tests)
│
├── mocks/                         # Mock data generators
│   ├── mock_tracts.py            # Census tracts
│   ├── mock_adjacency.py         # Adjacency graphs
│   ├── mock_districts.py         # District assignments
│   ├── mock_analysis.py          # Analysis results
│   └── mock_maps.py              # Map placeholders
│
├── utils/                         # Test utilities
│   ├── assertions.py             # Assertion helpers
│   ├── validators.py             # Data validators
│   └── cleanup.py                # Cleanup utilities
│
├── conftest.py                    # Shared pytest fixtures
└── pytest.ini                     # Pytest configuration
```

## Mock Data Generators

### Generate Test Data

```python
from tests.mocks.mock_tracts import generate_mock_tracts
from tests.mocks.mock_adjacency import generate_mock_adjacency
from tests.mocks.mock_districts import generate_mock_districts

# Generate 200 tracts for Alabama
tracts = generate_mock_tracts(num_tracts=200, state='alabama', year='2020')

# Generate adjacency graph
graph = generate_mock_adjacency(tracts, connectivity=0.2, edge_weighted=True)

# Generate district assignments (7 districts, balanced)
districts = generate_mock_districts(tracts, num_districts=7, tolerance=0.005)
```

### Generate Analysis Results

```python
from tests.mocks.mock_analysis import (
    generate_mock_political_analysis,
    generate_mock_demographic_analysis,
    generate_mock_compactness_analysis
)

# Political analysis (D/R vote shares, winners)
political = generate_mock_political_analysis(districts, state='alabama')

# Demographics (race/ethnicity, diversity index)
demographics = generate_mock_demographic_analysis(districts)

# Compactness (Polsby-Popper, Reock)
compactness = generate_mock_compactness_analysis(districts)
```

### Generate Maps

```python
from tests.mocks.mock_maps import generate_mock_state_map, generate_mock_national_map

# State map (800x600 PNG)
generate_mock_state_map(output_file, state='alabama', map_type='districts')

# National map with AK/HI insets (1600x1000 PNG)
generate_mock_national_map(output_file, map_type='political')
```

## Test Utilities

### Assertions

```python
from tests.utils import (
    assert_valid_csv,
    assert_valid_png,
    assert_population_balanced,
    assert_graph_connected,
    assert_shares_sum_to_one
)

# Validate CSV structure
assert_valid_csv('output.csv', required_columns=['district', 'population'], min_rows=7)

# Validate PNG image
assert_valid_png('map.png', min_width=800, min_height=600)

# Validate population balance (±1%)
assert_population_balanced(districts, num_districts=7, tolerance=0.01)

# Validate graph connectivity
assert_graph_connected(graph)

# Validate shares sum to 1.0
assert_shares_sum_to_one(df, ['white', 'black', 'hispanic', 'asian', 'other'])
```

### Validators

```python
from tests.utils import (
    validate_tract_data,
    validate_adjacency_graph,
    validate_district_assignments
)

# Validate tract data quality
result = validate_tract_data(tracts)
if result['valid']:
    print("[OK] Tract data valid")
else:
    print(f"[ERROR] {result['errors']}")

# Validate graph
result = validate_adjacency_graph(graph, num_tracts=200)
print(f"Graph: {result['node_count']} nodes, {result['edge_count']} edges")

# Validate districts
result = validate_district_assignments(districts, num_districts=7, tolerance=0.01)
print(f"Max deviation: {result['max_deviation']:.2%}")
```

## Pytest Markers

```bash
# By test type
pytest -m unit               # Unit tests
pytest -m integration        # Integration tests
pytest -m slow               # Slow tests (large datasets)

# By category
pytest -m redistricting      # Redistricting algorithm
pytest -m political          # Political analysis
pytest -m demographic        # Demographic analysis
pytest -m compactness        # Compactness metrics
pytest -m visualization      # Map generation
pytest -m aggregation        # CSV aggregation
```

## Coverage Reports

```bash
# Generate HTML coverage report
run_tests.bat coverage
# Output: htmlcov/index.html

# Or with pytest
pytest tests/unit tests/integration --cov=apportionment --cov-report=html
```

**Coverage Targets:**
- Overall: 90%+
- Unit tests: 95%+
- Integration tests: 85%+

## CI/CD Integration

Tests run automatically on push/PR via GitHub Actions:

- **Matrix**: Ubuntu + Windows, Python 3.11-3.13
- **Unit Tests**: ~30 seconds
- **Integration Tests**: ~1 minute
- **Total**: ~2 minutes per matrix cell

See `.github/workflows/test_pipeline.yml`.

## Writing New Tests

### Unit Test Template

```python
import pytest
from pathlib import Path
import sys

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

class TestMyComponent:
    """Test my component."""

    def test_basic_functionality(self):
        """Test basic case."""
        # Arrange
        input_data = ...

        # Act
        result = my_function(input_data)

        # Assert
        assert result == expected

# Pytest markers
pytestmark = [
    pytest.mark.unit,
    pytest.mark.redistricting,  # or political, demographic, etc.
]
```

### Integration Test Template

```python
import pytest

class TestPipelineFlow:
    """Test complete pipeline flow."""

    def test_full_pipeline(self, tmp_output_dir):
        """Test full pipeline."""
        from tests.mocks.mock_tracts import generate_mock_tracts
        from tests.mocks.mock_districts import generate_mock_districts

        # Generate and validate
        tracts = generate_mock_tracts(num_tracts=200, state='alabama')
        districts = generate_mock_districts(tracts, num_districts=7)

        assert len(districts) == 200
        assert len(districts['district'].unique()) == 7

# Pytest markers
pytestmark = [pytest.mark.integration]
```

## Performance

- **Quick suite**: ~5 seconds
- **Unit tests**: ~30 seconds
- **Integration tests**: ~1 minute
- **Full suite**: ~2 minutes

## Troubleshooting

### Import Errors

```bash
# Ensure project root in Python path
export PYTHONPATH="${PYTHONPATH}:$(pwd)"  # Unix
set PYTHONPATH=%PYTHONPATH%;%CD%          # Windows
```

### Coverage Not Generated

```bash
pip install pytest-cov
```

### Tests Fail on Windows

- Check for Unicode characters in console output (use ASCII)
- Use Path objects (not string concatenation)
- Watch for Windows path separators (backslashes)

## Contributing

When adding tests:

1. Follow naming conventions: `test_*.py`, `test_*()`
2. Add appropriate markers (unit/integration + category)
3. Include docstrings
4. Use fixtures for common data (conftest.py)
5. Keep unit tests fast (small datasets)
6. Test edge cases, not just happy paths

## References

- [pytest Documentation](https://docs.pytest.org/)
- [Enhancement 31 Specification](../../context/enhancements/active/31_pipeline_test_system.md)
- [E2E Dashboard Tests](README.md)
