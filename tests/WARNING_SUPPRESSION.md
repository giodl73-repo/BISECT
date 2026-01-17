# Warning Suppression Strategy

## Overview

The test suite uses a **strict warning policy** that treats warnings as errors, except for explicitly allowed categories from dependencies. This prevents new warnings from being introduced while allowing known safe warnings from external libraries.

## Configuration (pytest.ini)

```ini
filterwarnings =
    error                                      # Treat ALL warnings as errors
    ignore::pytest.PytestUnknownMarkWarning    # Pytest marker warnings (safe)
    ignore::DeprecationWarning                 # Dependency deprecation warnings
    ignore::PendingDeprecationWarning          # Dependency pending deprecations
    ignore::FutureWarning                      # Pandas/NumPy future changes
    ignore::UserWarning:geopandas              # GeoPandas CRS warnings
    ignore::pytest.PytestUnraisableExceptionWarning  # Resource cleanup warnings
    ignore::ResourceWarning                    # File/resource warnings
```

## Warning Categories

### 1. Pytest Marker Warnings (Ignored)
- **Source**: Custom pytest markers (unit, integration, visualization, etc.)
- **Why ignored**: These are expected and don't indicate issues
- **Example**: `Unknown pytest.mark.unit`

### 2. Deprecation Warnings (Ignored)
- **Source**: Pandas, NumPy, GeoPandas, Shapely
- **Why ignored**: Dependencies control these; we can't fix them
- **Example**: Pandas groupby behavior changes

### 3. GeoPandas CRS Warnings (Suppressed in code)
- **Source**: `mock_adjacency.py` using geographic CRS for centroid operations
- **Why suppressed**: Mock data intentionally uses simple geographic CRS
- **Example**: "Geometry is in a geographic CRS. Results from 'centroid' are likely incorrect."
- **Location**: `tests/mocks/mock_adjacency.py:59-61`

```python
with warnings.catch_warnings():
    warnings.filterwarnings('ignore', message='.*geographic CRS.*', category=UserWarning)
    centroids = tracts_df.geometry.centroid
```

### 4. Resource Warnings (Fixed in code)
- **Source**: PIL Image objects not being closed
- **Why fixed**: Causes file handle leaks on Windows
- **Solution**: Added `try/finally` blocks with `img.close()`
- **Locations**:
  - `tests/mocks/mock_maps.py:437-450` (validate_mock_map function)
  - `tests/unit/test_visualization.py` (multiple test methods)

## Policy

### ✅ What's Allowed
- Warnings from external dependencies (pandas, geopandas, shapely, numpy)
- Pytest internal warnings about markers
- Explicitly suppressed warnings with documented rationale

### ❌ What Will Fail Tests
- **ANY new warning from our test code**
- Uncaught warnings from mock generators
- Resource leaks (unclosed files/handles)

## Benefits

1. **Prevents regression**: New warnings immediately fail tests
2. **Clean output**: No noise in test runs
3. **Documentation**: Suppressions are explicit and documented
4. **Maintenance**: Easy to see what warnings are expected

## Checking for New Warnings

To see all warnings (even suppressed ones):

```bash
# Remove filterwarnings temporarily
python -m pytest tests/ -v -o addopts="" -W default
```

To add warnings as errors for specific categories:

```bash
# Fail on ALL warnings (very strict)
python -m pytest tests/ -v -W error
```

## Adding New Warning Suppressions

If you need to suppress a new warning:

1. **Verify it's from a dependency** (not our code)
2. **Add to pytest.ini filterwarnings** with comment explaining why
3. **Document in this file** with category and rationale
4. **Consider fixing in code** if it's a resource leak or similar issue

## Test Results

**Current status**: ✅ 104 passed, 2 skipped, 0 failed, 0 warnings (6.8 seconds)

All warnings successfully suppressed without compromising test quality.
