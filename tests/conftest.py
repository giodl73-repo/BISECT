"""
Pytest configuration and shared fixtures for pipeline testing.

This module provides:
- Mock data fixtures (tracts, adjacency, districts, analysis, maps)
- Temporary directory fixtures
- Common test utilities
- Pytest hooks and configuration
- make_disconnected_plan() fixture (board amendment BENCHMARK)
"""

import json
import pytest
import sys
from pathlib import Path
import tempfile
import shutil

# Add project root and src directory to Python path
project_root = Path(__file__).parent.parent
src_dir = project_root / 'src'
sys.path.insert(0, str(project_root))
sys.path.insert(0, str(src_dir))


# ============================================================================
# Session-scoped Fixtures
# ============================================================================

@pytest.fixture(scope='session')
def project_root_dir():
    """Get project root directory."""
    return project_root


@pytest.fixture(scope='session')
def scripts_dir(project_root_dir):
    """Get scripts directory."""
    return project_root_dir / 'scripts'


@pytest.fixture(scope='session')
def data_dir(project_root_dir):
    """Get data directory."""
    return project_root_dir / 'data'


# ============================================================================
# Function-scoped Fixtures (Fresh for Each Test)
# ============================================================================

@pytest.fixture
def tmp_output_dir(tmp_path):
    """Create temporary output directory for test."""
    output_dir = tmp_path / 'output'
    output_dir.mkdir()
    return output_dir


@pytest.fixture
def tmp_data_dir(tmp_path):
    """Create temporary data directory for test."""
    data_dir = tmp_path / 'data'
    data_dir.mkdir()
    return data_dir


# ============================================================================
# Mock Data Fixtures (Lazy-loaded from mocks module)
# ============================================================================

@pytest.fixture
def mock_tracts_small():
    """Generate small mock tract dataset (50 tracts, 1 district)."""
    from tests.mocks.mock_tracts import generate_mock_tracts
    return generate_mock_tracts(num_tracts=50, state='vermont', year='2020')


@pytest.fixture
def mock_tracts_medium():
    """Generate medium mock tract dataset (200 tracts, 7 districts)."""
    from tests.mocks.mock_tracts import generate_mock_tracts
    return generate_mock_tracts(num_tracts=200, state='alabama', year='2020')


@pytest.fixture
def mock_tracts_large():
    """Generate large mock tract dataset (500 tracts, 52 districts)."""
    from tests.mocks.mock_tracts import generate_mock_tracts
    return generate_mock_tracts(num_tracts=500, state='california', year='2020')


@pytest.fixture
def mock_adjacency_small(mock_tracts_small):
    """Generate small mock adjacency graph."""
    from tests.mocks.mock_adjacency import generate_mock_adjacency
    return generate_mock_adjacency(mock_tracts_small, connectivity=0.2)


@pytest.fixture
def mock_adjacency_medium(mock_tracts_medium):
    """Generate medium mock adjacency graph."""
    from tests.mocks.mock_adjacency import generate_mock_adjacency
    return generate_mock_adjacency(mock_tracts_medium, connectivity=0.2)


@pytest.fixture
def mock_districts_small(mock_tracts_small):
    """Generate small mock district assignments (1 district)."""
    from tests.mocks.mock_districts import generate_mock_districts
    return generate_mock_districts(mock_tracts_small, num_districts=1)


@pytest.fixture
def mock_districts_medium(mock_tracts_medium):
    """Generate medium mock district assignments (7 districts)."""
    from tests.mocks.mock_districts import generate_mock_districts
    return generate_mock_districts(mock_tracts_medium, num_districts=7)


# ============================================================================
# Pytest Hooks
# ============================================================================

@pytest.fixture
def make_disconnected_plan():
    """Board amendment BENCHMARK fixture.

    Returns a callable that writes a known-bad final_assignments.json with one
    district split across non-adjacent tracts (disconnected plan). This allows
    the contiguity exit-code test to run without skipping.

    Usage in test:
        state, year, version, label = make_disconnected_plan(tmp_redist_output)

    The returned tuple (state, year, version, label) can be passed directly to
    `redist analyze --state STATE --year YEAR --version VERSION --label LABEL`.

    The fixture creates a synthetic VT-like plan where district 1 has two tracts
    in geographically non-adjacent positions (tracts share no adjacency edges),
    causing the BFS contiguity check to find 2 components in district 1.
    """
    def _make(output_dir: Path) -> tuple:
        state = "VT"
        year = "2020"
        version = "spec3_disconnected"
        label = "vt_disconnected_test"

        # Create the plan directory structure matching what redist analyze expects.
        # Path: {output_dir}/{year}/plans/{label}/
        plan_dir = output_dir / year / "plans" / label
        plan_dir.mkdir(parents=True, exist_ok=True)

        # Write manifest.json (required by the analyzer for metadata lookup).
        manifest = {
            "label": label,
            "state_code": state,
            "year": year,
            "chamber": "congressional",
            "num_districts": 1,
            "population_source": "total",
            "balance_tolerance_pct": 0.5,
            "population_balance_valid": True,
            "created_at": "2026-04-26T00:00:00Z",
            "created_by": "test_fixture",
        }
        (plan_dir / "manifest.json").write_text(json.dumps(manifest, indent=2))

        # Write final_assignments.json — a disconnected plan.
        #
        # Strategy: assign ALL 193 VT tracts to district 1, EXCEPT place two
        # geographically remote tracts in their own "district 1 island" by
        # simply not including middle tracts — but this is complex without real
        # adjacency data.
        #
        # Simpler guaranteed approach: use only 2 tracts from opposite corners of
        # Vermont that are NOT directly adjacent. The BFS contiguity check will
        # return 2 components.
        #
        # Tract GEOIDs chosen from vt_adjacency_2020_geoids.json:
        #   index 0  -> "50005957100" (Caledonia County, NE Vermont)
        #   index 175 -> "50025967300" (Windsor County, SE Vermont)
        # These are in different counties separated by multiple counties —
        # they cannot share a direct boundary edge. With only 2 tracts in
        # district 1 and no intermediate tracts, BFS finds 2 components.
        assignments = {
            "50005957100": 1,   # Caledonia County (NE Vermont, index 0)
            "50025967300": 1,   # Windsor County (SE Vermont, index 175)
        }
        (plan_dir / "final_assignments.json").write_text(json.dumps(assignments))

        return state, year, version, label

    return _make


# ---------------------------------------------------------------------------
# Files to exclude from collection entirely.
#
# These test files import from apportionment.partition.* (Python algorithm
# library archived 2026-04-29 → archive/python-pipeline-final/).  Listing
# them here prevents pytest from even trying to open/parse them, which means
# ImportError and SyntaxError inside those files cannot surface as collection
# errors.  They remain in the repo as forensic reference only.
# ---------------------------------------------------------------------------
collect_ignore = [
    "unit/test_ablation_analysis.py",
    "unit/test_vra_recursive_bisection.py",
    "unit/test_vra_targets.py",
]


def pytest_configure(config):
    """Pytest configuration hook."""
    # Ensure tests directory is in Python path
    tests_dir = Path(__file__).parent
    if str(tests_dir) not in sys.path:
        sys.path.insert(0, str(tests_dir))


def pytest_collection_modifyitems(config, items):
    """Modify test collection - add markers automatically."""
    for item in items:
        # Add markers based on test file location
        test_path = Path(item.fspath)

        if 'unit' in test_path.parts:
            item.add_marker(pytest.mark.unit)

        if 'integration' in test_path.parts:
            item.add_marker(pytest.mark.integration)

        # Add category markers based on test file name
        test_name = test_path.stem
        if 'redistricting' in test_name:
            item.add_marker(pytest.mark.redistricting)
        elif 'political' in test_name:
            item.add_marker(pytest.mark.political)
        elif 'demographic' in test_name:
            item.add_marker(pytest.mark.demographic)
        elif 'compactness' in test_name:
            item.add_marker(pytest.mark.compactness)
        elif 'visualization' in test_name:
            item.add_marker(pytest.mark.visualization)
        elif 'aggregation' in test_name:
            item.add_marker(pytest.mark.aggregation)


# ============================================================================
# Test Utilities
# ============================================================================

@pytest.fixture
def assert_csv_structure():
    """Fixture that returns CSV structure assertion helper."""
    from tests.utils.assertions import assert_valid_csv
    return assert_valid_csv


@pytest.fixture
def assert_png_valid():
    """Fixture that returns PNG validation helper."""
    from tests.utils.assertions import assert_valid_png
    return assert_valid_png


@pytest.fixture
def assert_population_balanced():
    """Fixture that returns population balance assertion helper."""
    from tests.utils.assertions import assert_population_balanced
    return assert_population_balanced
