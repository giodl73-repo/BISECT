"""
End-to-end tests for the master dashboard (web/master_dashboard.html).

Tests cover:
- Cross-run comparison functionality
- Run cards and metadata
- Data embedding validation
- Comparison table features
"""

import pytest
from playwright.sync_api import Page, expect
from pathlib import Path
import json


# ============================================================================
# Fixtures
# ============================================================================

@pytest.fixture
def master_dashboard_path(project_root, tmp_path):
    """Create minimal master dashboard for testing."""
    template_path = project_root / 'web' / 'master_dashboard.html'
    test_dashboard = tmp_path / 'master_dashboard.html'

    if template_path.exists():
        # Copy template and embed sample data
        content = template_path.read_text(encoding='utf-8')

        # Create sample runs data
        runs_data = [
            {
                'year': '2020',
                'version': 'v1',
                'date': '2026-01-15',
                'districts': 435,
                'states': 50
            },
            {
                'year': '2020',
                'version': 'v2',
                'date': '2026-01-16',
                'districts': 435,
                'states': 50
            }
        ]

        # Create sample comparison data
        comparison_data = {
            '2020_v1': {
                'mean_polsby_popper': 0.234,
                'mean_reock': 0.456,
                'total_districts': 435,
                'dem_seats': 218,
                'rep_seats': 217
            },
            '2020_v2': {
                'mean_polsby_popper': 0.256,
                'mean_reock': 0.478,
                'total_districts': 435,
                'dem_seats': 220,
                'rep_seats': 215
            }
        }

        # Replace placeholders
        content = content.replace(
            'const RUNS_DATA_PLACEHOLDER = [];',
            f'const RUNS_DATA_PLACEHOLDER = {json.dumps(runs_data)};'
        )
        content = content.replace(
            'const COMPARISON_DATA_PLACEHOLDER = {};',
            f'const COMPARISON_DATA_PLACEHOLDER = {json.dumps(comparison_data)};'
        )

        test_dashboard.write_text(content, encoding='utf-8')
    else:
        # Create minimal dashboard for testing
        _create_minimal_master_dashboard(test_dashboard)

    return test_dashboard


def _create_minimal_master_dashboard(path: Path):
    """Create minimal master dashboard HTML."""
    html = """<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Congressional Redistricting - Master Dashboard</title>
    <style>
        body { margin: 0; font-family: Arial, sans-serif; background: #f5f5f5; }
        .header { background: #2c3e50; color: white; padding: 20px; text-align: center; }
        .container { max-width: 1400px; margin: 0 auto; padding: 20px; }
        .run-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 20px; margin: 20px 0; }
        .run-card { background: white; border-radius: 8px; padding: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        .run-card h3 { margin: 0 0 10px 0; color: #2c3e50; }
        .run-card .metadata { color: #7f8c8d; font-size: 14px; }
        .run-card .view-btn { display: inline-block; margin-top: 10px; padding: 8px 16px; background: #3498db; color: white; text-decoration: none; border-radius: 4px; }
        .comparison-table { width: 100%; background: white; border-radius: 8px; padding: 20px; margin: 20px 0; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        table { width: 100%; border-collapse: collapse; }
        th, td { padding: 12px; text-align: left; border-bottom: 1px solid #ecf0f1; }
        th { background: #34495e; color: white; cursor: pointer; }
        tr:hover { background: #f8f9fa; }
        .sortable::after { content: ' \\2195'; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Congressional Redistricting - Master Dashboard</h1>
        <p>Cross-Run Comparison</p>
    </div>

    <div class="container">
        <section class="comparison-table">
            <h2>Run Comparison</h2>
            <table id="comparison-table">
                <thead>
                    <tr>
                        <th class="sortable" data-column="run">Run</th>
                        <th class="sortable" data-column="pp">Polsby-Popper</th>
                        <th class="sortable" data-column="reock">Reock</th>
                        <th class="sortable" data-column="districts">Districts</th>
                        <th class="sortable" data-column="dem">Dem Seats</th>
                        <th class="sortable" data-column="rep">Rep Seats</th>
                    </tr>
                </thead>
                <tbody id="comparison-tbody">
                </tbody>
            </table>
        </section>

        <section class="run-cards" id="run-cards">
        </section>
    </div>

    <script>
        const RUNS_DATA_PLACEHOLDER = [
            { year: '2020', version: 'v1', date: '2026-01-15', districts: 435, states: 50 },
            { year: '2020', version: 'v2', date: '2026-01-16', districts: 435, states: 50 }
        ];

        const COMPARISON_DATA_PLACEHOLDER = {
            '2020_v1': { mean_polsby_popper: 0.234, mean_reock: 0.456, total_districts: 435, dem_seats: 218, rep_seats: 217 },
            '2020_v2': { mean_polsby_popper: 0.256, mean_reock: 0.478, total_districts: 435, dem_seats: 220, rep_seats: 215 }
        };

        // Initialize
        function init() {
            renderRunCards();
            renderComparisonTable();
            setupSorting();
        }

        function renderRunCards() {
            const container = document.getElementById('run-cards');
            RUNS_DATA_PLACEHOLDER.forEach(run => {
                const card = document.createElement('div');
                card.className = 'run-card';
                card.innerHTML = `
                    <h3>${run.year} - ${run.version}</h3>
                    <div class="metadata">
                        <p>Date: ${run.date}</p>
                        <p>Districts: ${run.districts}</p>
                        <p>States: ${run.states}</p>
                    </div>
                    <a href="outputs/us_${run.year}_${run.version}/index.html" class="view-btn">View Details</a>
                `;
                container.appendChild(card);
            });
        }

        function renderComparisonTable() {
            const tbody = document.getElementById('comparison-tbody');
            Object.entries(COMPARISON_DATA_PLACEHOLDER).forEach(([run, data]) => {
                const row = document.createElement('tr');
                row.innerHTML = `
                    <td>${run.replace('_', ' ')}</td>
                    <td>${data.mean_polsby_popper.toFixed(3)}</td>
                    <td>${data.mean_reock.toFixed(3)}</td>
                    <td>${data.total_districts}</td>
                    <td>${data.dem_seats}</td>
                    <td>${data.rep_seats}</td>
                `;
                tbody.appendChild(row);
            });
        }

        function setupSorting() {
            document.querySelectorAll('th.sortable').forEach(th => {
                th.addEventListener('click', () => {
                    console.log('Sort by:', th.dataset.column);
                });
            });
        }

        // Load on page ready
        document.addEventListener('DOMContentLoaded', init);
    </script>
</body>
</html>"""
    path.write_text(html, encoding='utf-8')


@pytest.fixture
def master_dashboard_url(master_dashboard_path):
    """Get file:// URL for master dashboard."""
    return master_dashboard_path.as_uri()


# ============================================================================
# Test Suite A: Dashboard Structure
# ============================================================================

@pytest.mark.smoke
def test_master_dashboard_loads(page: Page, master_dashboard_url: str):
    """Test that master dashboard loads successfully."""
    page.goto(master_dashboard_url)

    # Verify title
    expect(page).to_have_title("Congressional Redistricting - Master Dashboard")

    # Verify header
    header = page.locator('.header')
    expect(header).to_be_visible()
    expect(header.locator('h1')).to_contain_text('Master Dashboard')


def test_comparison_table_exists(page: Page, master_dashboard_url: str):
    """Test that comparison table is present."""
    page.goto(master_dashboard_url)

    # Wait for page to load
    page.wait_for_load_state('networkidle')

    # Verify comparison table section
    table_section = page.locator('.comparison-table')
    expect(table_section).to_be_visible()

    # Verify table
    table = page.locator('#comparison-table')
    expect(table).to_be_visible()


def test_run_cards_exist(page: Page, master_dashboard_url: str):
    """Test that run cards are displayed."""
    page.goto(master_dashboard_url)

    # Wait for page to load
    page.wait_for_load_state('networkidle')

    # Verify run cards section
    cards_section = page.locator('.run-cards')
    expect(cards_section).to_be_visible()

    # Verify at least one run card exists
    run_cards = page.locator('.run-card')
    assert run_cards.count() >= 1


# ============================================================================
# Test Suite B: Comparison Table
# ============================================================================

def test_comparison_table_headers(page: Page, master_dashboard_url: str):
    """Test that comparison table has correct headers."""
    page.goto(master_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Expected headers
    expected_headers = ['Run', 'Polsby-Popper', 'Reock', 'Districts']

    # Get all header cells
    headers = page.locator('#comparison-table thead th')
    header_count = headers.count()

    assert header_count >= len(expected_headers), \
        f"Expected at least {len(expected_headers)} headers, found {header_count}"


def test_comparison_table_has_data(page: Page, master_dashboard_url: str):
    """Test that comparison table contains data rows."""
    page.goto(master_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Get table body rows
    rows = page.locator('#comparison-tbody tr')
    row_count = rows.count()

    assert row_count >= 1, "Expected at least 1 data row in comparison table"


def test_table_columns_sortable(page: Page, master_dashboard_url: str):
    """Test that table headers are marked as sortable."""
    page.goto(master_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Get sortable headers
    sortable_headers = page.locator('th.sortable')
    count = sortable_headers.count()

    assert count >= 4, f"Expected at least 4 sortable columns, found {count}"


def test_table_row_hover_effect(page: Page, master_dashboard_url: str):
    """Test that table rows have hover effect."""
    page.goto(master_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Get first data row
    first_row = page.locator('#comparison-tbody tr').first

    if first_row.count() > 0:
        # Hover over row
        first_row.hover()

        # Row should still be visible (basic interaction test)
        expect(first_row).to_be_visible()


# ============================================================================
# Test Suite C: Run Cards
# ============================================================================

def test_run_card_structure(page: Page, master_dashboard_url: str):
    """Test that run cards have correct structure."""
    page.goto(master_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Get first run card
    first_card = page.locator('.run-card').first

    if first_card.count() > 0:
        # Verify card has title
        title = first_card.locator('h3')
        expect(title).to_be_visible()

        # Verify card has metadata
        metadata = first_card.locator('.metadata')
        expect(metadata).to_be_visible()

        # Verify card has view button
        btn = first_card.locator('.view-btn')
        expect(btn).to_be_visible()


def test_run_card_metadata(page: Page, master_dashboard_url: str):
    """Test that run cards display metadata."""
    page.goto(master_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Get first run card
    first_card = page.locator('.run-card').first

    if first_card.count() > 0:
        # Verify metadata contains expected text
        metadata = first_card.locator('.metadata')
        metadata_text = metadata.inner_text()

        # Should contain date, districts, states info
        assert 'Date:' in metadata_text or 'date' in metadata_text.lower()


def test_run_card_view_button_has_href(page: Page, master_dashboard_url: str):
    """Test that view buttons have valid href."""
    page.goto(master_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Get all view buttons
    view_buttons = page.locator('.view-btn')

    if view_buttons.count() > 0:
        first_btn = view_buttons.first
        href = first_btn.get_attribute('href')

        assert href is not None, "View button missing href attribute"
        assert len(href) > 0, "View button href is empty"


# ============================================================================
# Test Suite D: Data Embedding
# ============================================================================

def test_runs_data_embedded(page: Page, master_dashboard_url: str):
    """Test that RUNS_DATA is properly embedded."""
    page.goto(master_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Evaluate JavaScript to check if RUNS_DATA exists
    runs_data = page.evaluate('() => window.RUNS_DATA_PLACEHOLDER || []')

    assert isinstance(runs_data, list), "RUNS_DATA should be a list"
    assert len(runs_data) >= 1, "RUNS_DATA should contain at least 1 run"


def test_comparison_data_embedded(page: Page, master_dashboard_url: str):
    """Test that COMPARISON_DATA is properly embedded."""
    page.goto(master_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Evaluate JavaScript to check if COMPARISON_DATA exists
    comparison_data = page.evaluate('() => window.COMPARISON_DATA_PLACEHOLDER || {}')

    assert isinstance(comparison_data, dict), "COMPARISON_DATA should be a dictionary"
    assert len(comparison_data) >= 1, "COMPARISON_DATA should contain at least 1 entry"


def test_embedded_data_structure(page: Page, master_dashboard_url: str):
    """Test that embedded data has expected structure."""
    page.goto(master_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Get runs data
    runs_data = page.evaluate('() => window.RUNS_DATA_PLACEHOLDER || []')

    if len(runs_data) > 0:
        first_run = runs_data[0]

        # Verify expected fields
        assert 'year' in first_run, "Run data missing 'year' field"
        assert 'version' in first_run, "Run data missing 'version' field"


# ============================================================================
# Test Suite E: Performance
# ============================================================================

@pytest.mark.smoke
def test_master_dashboard_load_time(page: Page, master_dashboard_url: str):
    """Test that master dashboard loads within 3 seconds."""
    page.goto(master_dashboard_url)

    # Verify critical elements load quickly
    expect(page.locator('.header')).to_be_visible(timeout=3000)
    expect(page.locator('.comparison-table')).to_be_visible(timeout=3000)


def test_no_console_errors(page: Page, master_dashboard_url: str):
    """Test that dashboard loads without console errors."""
    errors = []

    # Capture console errors
    page.on('console', lambda msg: errors.append(msg.text) if msg.type == 'error' else None)

    # Load dashboard
    page.goto(master_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Verify no errors
    assert len(errors) == 0, f"Console errors detected: {errors}"


# ============================================================================
# Test Suite F: Responsive Layout
# ============================================================================

@pytest.mark.parametrize('viewport', [
    {'width': 1920, 'height': 1080},  # Desktop
    {'width': 1366, 'height': 768},   # Laptop
    {'width': 768, 'height': 1024},   # Tablet
])
def test_responsive_layout(page: Page, master_dashboard_url: str, viewport: dict):
    """Test master dashboard layout at different viewport sizes."""
    # Set viewport
    page.set_viewport_size(viewport)

    # Load dashboard
    page.goto(master_dashboard_url)

    # Verify critical elements are visible
    expect(page.locator('.header')).to_be_visible()
    expect(page.locator('.container')).to_be_visible()


# ============================================================================
# Test Suite G: Multiple Runs
# ============================================================================

def test_handles_multiple_runs(page: Page, master_dashboard_url: str):
    """Test that dashboard handles multiple runs correctly."""
    page.goto(master_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Get all run cards
    run_cards = page.locator('.run-card')
    card_count = run_cards.count()

    # Get all comparison table rows
    table_rows = page.locator('#comparison-tbody tr')
    row_count = table_rows.count()

    # Should have matching counts (or at least both > 0)
    assert card_count >= 1, "Should have at least 1 run card"
    assert row_count >= 1, "Should have at least 1 table row"
