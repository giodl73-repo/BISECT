"""
Shared configuration for integration tests.

All integration tests read pipeline outputs from a configurable root directory.
By default they run against V3 across all three census decades.

Usage
-----
# Default — V3, all decades
pytest tests/integration

# Acceptance run for a specific version/year (env vars work at collection time)
PIPELINE_VERSION=V4 PIPELINE_YEAR=2020 pytest tests/integration

# Or via CLI flags (equivalent)
pytest tests/integration --pipeline-version V4 --pipeline-year 2020

# Run the full acceptance suite against any completed version
pytest tests/integration --pipeline-version V1 --pipeline-year 2020 -v
"""

import os
import pytest
from pathlib import Path


# ---------------------------------------------------------------------------
# CLI option registration (also readable at module import via env vars)
# ---------------------------------------------------------------------------

def pytest_addoption(parser):
    parser.addoption(
        '--pipeline-version', default=None,
        help='Pipeline version to validate (e.g. V3, V4). '
             'Also read from PIPELINE_VERSION env var. Default: V3'
    )
    parser.addoption(
        '--pipeline-year', default=None,
        help='Single census year to validate (2020, 2010, 2000). '
             'Also read from PIPELINE_YEAR env var. Default: all available years'
    )


def pytest_configure(config):
    """Resolve version/year from CLI or env and store on config for use everywhere."""
    version = (
        config.getoption('--pipeline-version', default=None)
        or os.environ.get('PIPELINE_VERSION', 'V3')
    )
    year = (
        config.getoption('--pipeline-year', default=None)
        or os.environ.get('PIPELINE_YEAR', None)
    )
    config._pipeline_version = version
    config._pipeline_year = year


# ---------------------------------------------------------------------------
# Module-level helpers (importable by test files for parametrize)
# ---------------------------------------------------------------------------

def _get_version() -> str:
    """Get configured version (CLI > env > default V3)."""
    # Try pytest config first (available after pytest_configure)
    try:
        import _pytest.config
        cfg = _pytest.config._pytest_current_test  # noqa — private API
    except Exception:
        pass
    return os.environ.get('PIPELINE_VERSION', 'V3')


def get_outputs_root() -> Path:
    """Return the pipeline outputs root for the configured version."""
    version = os.environ.get('PIPELINE_VERSION', 'V3')
    return Path('outputs') / version


def get_years(root: Path | None = None) -> list[str]:
    """Return census years to test for the configured version."""
    if root is None:
        root = get_outputs_root()
    requested = os.environ.get('PIPELINE_YEAR')
    all_years = ['2020', '2010', '2000']
    if requested:
        return [requested] if requested in all_years else []
    return [y for y in all_years if (root / y / 'states').exists()]


# ---------------------------------------------------------------------------
# Fixtures (for tests that prefer fixture injection over module-level globals)
# ---------------------------------------------------------------------------

@pytest.fixture(scope='session')
def pipeline_version(request) -> str:
    return getattr(request.config, '_pipeline_version', 'V3')


@pytest.fixture(scope='session')
def pipeline_root(pipeline_version) -> Path:
    root = Path('outputs') / pipeline_version
    if not root.exists():
        pytest.skip(f'Pipeline outputs not found: {root}')
    return root


@pytest.fixture(scope='session')
def pipeline_years(request, pipeline_root) -> list[str]:
    year = getattr(request.config, '_pipeline_year', None)
    all_years = ['2020', '2010', '2000']
    if year:
        return [year] if year in all_years else []
    return [y for y in all_years if (pipeline_root / y / 'states').exists()]
