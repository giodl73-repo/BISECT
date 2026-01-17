"""
Pytest fixtures and configuration for Playwright end-to-end tests.

This module provides fixtures for:
- Test data generation (sample run with Vermont + Delaware)
- Dashboard URL construction
- Browser page fixtures
- Screenshot comparison utilities
"""

import pytest
from pathlib import Path
from playwright.sync_api import Page, expect
import json
import pandas as pd
import shutil


# ============================================================================
# Session-scoped fixtures (setup once for all tests)
# ============================================================================

@pytest.fixture(scope='session')
def project_root():
    """Get project root directory."""
    return Path(__file__).parent.parent.parent


@pytest.fixture(scope='session')
def sample_run_dir(project_root, tmp_path_factory):
    """
    Generate sample dashboard with minimal test data.

    Creates outputs/us_2020_test/ with:
    - Vermont (1 district)
    - Delaware (1 district)
    - All required CSVs and maps
    - Generated dashboard HTML

    Returns path to generated dashboard index.html
    """
    # Use temporary directory for test outputs
    temp_outputs = tmp_path_factory.mktemp('test_outputs')
    run_dir = temp_outputs / 'us_2020_test'
    run_dir.mkdir()

    # Create data directory structure
    data_dir = run_dir / 'data'
    data_dir.mkdir()

    # Create states directory
    states_dir = run_dir / 'states'
    states_dir.mkdir()

    # Generate Vermont data (1 district)
    vt_dir = states_dir / 'vermont'
    vt_dir.mkdir()
    _generate_state_data(vt_dir, 'vermont', 1, 643077)

    # Generate Delaware data (1 district)
    de_dir = states_dir / 'delaware'
    de_dir.mkdir()
    _generate_state_data(de_dir, 'delaware', 1, 990837)

    # Copy dashboard template and generate HTML
    template_path = project_root / 'web' / 'dashboard.html'
    output_path = run_dir / 'index.html'

    # For now, just copy the template (will enhance with data embedding later)
    if template_path.exists():
        shutil.copy(template_path, output_path)
    else:
        # Create minimal dashboard HTML for testing
        _create_minimal_dashboard(output_path)

    return output_path


def _generate_state_data(state_dir: Path, state_name: str, num_districts: int, total_population: int):
    """Generate minimal valid data for a single state."""

    # District summary CSV
    districts = []
    for i in range(1, num_districts + 1):
        districts.append({
            'district': i,
            'population': total_population // num_districts,
            'polsby_popper': 0.234 + (i * 0.01),
            'reock': 0.456 + (i * 0.01),
            'area_sq_km': 24000.0 + (i * 100),
            'perimeter_km': 800.0 + (i * 10)
        })

    df = pd.DataFrame(districts)
    df.to_csv(state_dir / 'district_summary.csv', index=False)

    # Create maps directory with placeholder images
    maps_dir = state_dir / 'maps'
    maps_dir.mkdir()

    # Create placeholder map images (1x1 pixel PNGs)
    _create_placeholder_png(maps_dir / 'districts.png')
    _create_placeholder_png(maps_dir / 'compactness_polsby_popper.png')
    _create_placeholder_png(maps_dir / 'compactness_reock.png')

    # Create rounds directory
    rounds_dir = maps_dir / 'rounds'
    rounds_dir.mkdir()
    for round_num in range(1, 7):
        _create_placeholder_png(rounds_dir / f'round_{round_num:02d}.png')

    # Create individual districts directory
    districts_dir = maps_dir / 'districts'
    districts_dir.mkdir()
    for i in range(1, num_districts + 1):
        _create_placeholder_png(districts_dir / f'district_{i:02d}.png')


def _create_placeholder_png(path: Path):
    """Create a minimal 1x1 pixel PNG for testing."""
    # Minimal PNG file (1x1 red pixel)
    png_data = bytes([
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,  # PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,  # IHDR chunk
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,  # 1x1 dimensions
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
        0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41,  # IDAT chunk
        0x54, 0x08, 0x99, 0x63, 0xF8, 0xCF, 0xC0, 0x00,
        0x00, 0x00, 0x03, 0x00, 0x01, 0x92, 0x7C, 0x8B,
        0xD7, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,  # IEND chunk
        0x44, 0xAE, 0x42, 0x60, 0x82
    ])
    path.write_bytes(png_data)


def _create_minimal_dashboard(path: Path):
    """Create minimal HTML dashboard for testing."""
    html = """<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Congressional Redistricting - 2020</title>
    <style>
        body { margin: 0; font-family: Arial, sans-serif; }
        .header { background: #2c3e50; color: white; padding: 20px; }
        .sidebar { width: 250px; background: #34495e; color: white; position: fixed; height: 100%; overflow-y: auto; }
        .state-item { padding: 10px; cursor: pointer; border-bottom: 1px solid #2c3e50; }
        .state-item:hover { background: #2c3e50; }
        .content-area { margin-left: 250px; padding: 20px; }
        .tabs { display: flex; border-bottom: 2px solid #2c3e50; margin-bottom: 20px; }
        .tab { padding: 10px 20px; cursor: pointer; border: none; background: none; }
        .tab.active { border-bottom: 3px solid #3498db; font-weight: bold; }
        .tab-content { display: none; }
        .tab-content.active { display: block; }
        table { border-collapse: collapse; width: 100%; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #34495e; color: white; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Congressional Redistricting - <span id="current-state">Select a State</span></h1>
    </div>

    <div class="sidebar">
        <div class="state-item" data-state="vermont">Vermont (1)</div>
        <div class="state-item" data-state="delaware">Delaware (1)</div>
    </div>

    <div class="content-area">
        <div class="tabs">
            <button class="tab active" data-tab="overview">Overview</button>
            <button class="tab" data-tab="districts">Districts</button>
            <button class="tab" data-tab="rounds">Rounds</button>
            <button class="tab" data-tab="compactness">Compactness</button>
        </div>

        <div id="overview" class="tab-content active">
            <h2>Overview</h2>
            <table class="district-summary">
                <thead>
                    <tr>
                        <th>District</th>
                        <th>Population</th>
                        <th>Polsby-Popper</th>
                        <th>Reock</th>
                    </tr>
                </thead>
                <tbody id="summary-table-body">
                </tbody>
            </table>
        </div>

        <div id="districts" class="tab-content">
            <h2>Districts</h2>
            <img id="districts-map" src="" alt="Districts Map" style="max-width: 100%;">
        </div>

        <div id="rounds" class="tab-content">
            <h2>Round Progression</h2>
            <div id="rounds-container"></div>
        </div>

        <div id="compactness" class="tab-content">
            <h2>Compactness Analysis</h2>
            <img id="compactness-map" src="" alt="Compactness Map" style="max-width: 100%;">
        </div>
    </div>

    <script>
        let currentState = null;

        // Tab switching
        document.querySelectorAll('.tab').forEach(tab => {
            tab.addEventListener('click', () => {
                document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
                document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
                tab.classList.add('active');
                document.getElementById(tab.dataset.tab).classList.add('active');
            });
        });

        // State selection
        document.querySelectorAll('.state-item').forEach(item => {
            item.addEventListener('click', () => {
                currentState = item.dataset.state;
                document.getElementById('current-state').textContent =
                    item.textContent.split(' (')[0];
                window.location.hash = currentState;
                loadStateData(currentState);
            });
        });

        function loadStateData(state) {
            // In real implementation, this would load CSV data
            // For testing, we just update the display
            console.log('Loading state:', state);
        }
    </script>
</body>
</html>"""
    path.write_text(html, encoding='utf-8')


# ============================================================================
# Function-scoped fixtures (setup for each test)
# ============================================================================

@pytest.fixture
def dashboard_url(sample_run_dir):
    """Get file:// URL for the generated dashboard."""
    return sample_run_dir.as_uri()


@pytest.fixture
def page_with_dashboard(page: Page, dashboard_url: str):
    """Load dashboard and return page object."""
    page.goto(dashboard_url)
    return page


# ============================================================================
# Utility fixtures
# ============================================================================

@pytest.fixture
def screenshots_dir(project_root):
    """Get screenshots directory for baseline/diff storage."""
    return project_root / 'tests' / 'screenshots'


# ============================================================================
# Markers for test categorization
# ============================================================================

def pytest_configure(config):
    """Register custom markers."""
    config.addinivalue_line(
        "markers", "smoke: Quick smoke tests for critical functionality"
    )
    config.addinivalue_line(
        "markers", "visual: Visual regression tests"
    )
    config.addinivalue_line(
        "markers", "slow: Tests that take longer than 10 seconds"
    )
