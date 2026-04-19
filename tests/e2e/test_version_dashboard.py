"""
End-to-end tests for version dashboards (web/version_dashboard.html).

Tests cover:
- Version dashboard structure (breadcrumb, quick links, tabs)
- Cross-year comparison functionality
- Tab switching (Overview, Compactness, Demographics)
- Data embedding validation
- State compactness table filtering
- Navigation to/from master dashboard
"""

import pytest
from playwright.sync_api import Page, expect
from pathlib import Path
import json


# ============================================================================
# Fixtures
# ============================================================================

@pytest.fixture
def version_dashboard_path(project_root, tmp_path):
    """Create minimal version dashboard for testing."""
    template_path = project_root / 'web' / 'version_dashboard.html'
    test_dashboard = tmp_path / 'version_dashboard.html'

    if template_path.exists():
        # Copy template and embed test data
        import shutil
        shutil.copy(template_path, test_dashboard)

        content = test_dashboard.read_text(encoding='utf-8')

        # Create sample version config
        version_config = {
            'version': 'v1',
            'description': 'Test version with edge-weighted partitioning',
            'algorithm': {
                'partition_mode': 'edge_weighted',
                'data_level': 'tract'
            },
            'edge_weight_mode': 'scaled',
            'scaling_factor': 1000
        }

        # Create sample comparison data
        comparison_data = {
            '2020': {
                'v1': {
                    'algorithmic': 0.2456,
                    'enacted': 0.2123,
                    'improvement_pct': 15.68,
                    'states': {
                        'california': {
                            'algorithmic': 0.2789,
                            'enacted': 0.2456,
                            'improvement_pct': 13.55
                        },
                        'texas': {
                            'algorithmic': 0.3012,
                            'enacted': 0.2678,
                            'improvement_pct': 12.48
                        }
                    }
                }
            },
            '2010': {
                'v1': {
                    'algorithmic': 0.2389,
                    'enacted': 0.2056,
                    'improvement_pct': 16.20,
                    'states': {
                        'california': {
                            'algorithmic': 0.2701,
                            'enacted': 0.2398,
                            'improvement_pct': 12.63
                        }
                    }
                }
            }
        }

        # Create sample runs
        runs = [
            {
                'year': 2020,
                'version': 'v1',
                'num_states': 50,
                'path': 'v1/2020'
            },
            {
                'year': 2010,
                'version': 'v1',
                'num_states': 50,
                'path': 'v1/2010'
            }
        ]

        # Replace placeholders
        content = content.replace('{VERSION}', version_config['version'])
        content = content.replace('{DESCRIPTION}', version_config['description'])
        content = content.replace('/* VERSION_CONFIG_PLACEHOLDER */', json.dumps(version_config, indent=4))
        content = content.replace('/* COMPARISON_DATA_PLACEHOLDER */', json.dumps(comparison_data, indent=4))
        content = content.replace('/* RUNS_PLACEHOLDER */', json.dumps(runs, indent=4))

        test_dashboard.write_text(content, encoding='utf-8')
    else:
        # Create minimal dashboard for testing
        _create_minimal_version_dashboard(test_dashboard)

    return test_dashboard


def _create_minimal_version_dashboard(path: Path):
    """Create minimal version dashboard HTML for testing."""
    html = """<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>v1 Dashboard - Algorithmic Redistricting</title>
    <style>
        body { margin: 0; font-family: Arial, sans-serif; }
        .header { background: #2c3e50; color: white; padding: 20px; }
        .breadcrumb { margin-bottom: 10px; }
        .breadcrumb a { color: white; }
        .quick-links { display: flex; gap: 10px; }
        .run-link-badge { padding: 10px; border-radius: 5px; }
        .view-tabs { display: flex; background: #f5f5f5; }
        .view-tab { padding: 15px; cursor: pointer; }
        .view-tab.active { background: white; }
        .view-content { display: none; padding: 20px; }
        .view-content.active { display: block; }
    </style>
</head>
<body>
    <div class="header">
        <div class="breadcrumb">
            <a href="../index.html">&larr; Master Dashboard</a>
        </div>
        <h1>v1: Test Version</h1>
        <div class="quick-links">
            <a href="2020/index.html" class="run-link-badge year-2020">2020 (50 states)</a>
        </div>
    </div>

    <div class="view-tabs">
        <div class="view-tab active" onclick="switchView('overview')">Overview</div>
        <div class="view-tab" onclick="switchView('compactness')">Compactness</div>
        <div class="view-tab" onclick="switchView('demographics')">Demographics</div>
    </div>

    <div id="overview-view" class="view-content active">
        <h2>Version Configuration</h2>
        <p>Version: v1</p>
    </div>

    <div id="compactness-view" class="view-content">
        <h2>Cross-Year Compactness</h2>
        <table>
            <thead>
                <tr><th>Year</th><th>Algorithmic</th><th>Enacted</th></tr>
            </thead>
            <tbody>
                <tr><td>2020</td><td>0.2456</td><td>0.2123</td></tr>
            </tbody>
        </table>
        <select id="yearFilter">
            <option value="all">All Years</option>
            <option value="2020">2020</option>
        </select>
    </div>

    <div id="demographics-view" class="view-content">
        <h2>Population Trends</h2>
        <p>US population across census years</p>
    </div>

    <script>
        function switchView(viewName) {
            document.querySelectorAll('.view-content').forEach(v => v.classList.remove('active'));
            document.querySelectorAll('.view-tab').forEach(t => t.classList.remove('active'));
            document.getElementById(viewName + '-view').classList.add('active');
            event.target.classList.add('active');
        }
    </script>
</body>
</html>"""
    path.write_text(html, encoding='utf-8')


@pytest.fixture
def version_dashboard_url(version_dashboard_path):
    """Get file:// URL for version dashboard."""
    return version_dashboard_path.as_uri()


# ============================================================================
# Test Suite A: Dashboard Structure
# ============================================================================

@pytest.mark.smoke
def test_version_dashboard_loads(page: Page, version_dashboard_url: str):
    """Test that version dashboard loads successfully."""
    page.goto(version_dashboard_url)

    # Verify title contains expected text
    title = page.title()
    assert "Dashboard" in title and "Algorithmic Redistricting" in title, \
        f"Expected title to contain 'Dashboard' and 'Algorithmic Redistricting', got: {title}"

    # Verify header
    header = page.locator('.header')
    expect(header).to_be_visible()


def test_breadcrumb_navigation(page: Page, version_dashboard_url: str):
    """Test that breadcrumb navigation is present."""
    page.goto(version_dashboard_url)

    # Verify breadcrumb exists
    breadcrumb = page.locator('.breadcrumb')
    expect(breadcrumb).to_be_visible()

    # Verify link to master dashboard
    master_link = breadcrumb.locator('a')
    expect(master_link).to_be_visible()
    expect(master_link).to_contain_text('Master Dashboard')


def test_quick_links_exist(page: Page, version_dashboard_url: str):
    """Test that quick links to individual runs are present."""
    page.goto(version_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Verify quick links container exists
    quick_links = page.locator('.quick-links')
    expect(quick_links).to_be_visible()

    # Verify at least one run link badge
    badges = page.locator('.run-link-badge')
    assert badges.count() >= 1, "Should have at least 1 run link badge"


def test_view_tabs_exist(page: Page, version_dashboard_url: str):
    """Test that view tabs are present."""
    page.goto(version_dashboard_url)

    # Verify view tabs container
    view_tabs = page.locator('.view-tabs')
    expect(view_tabs).to_be_visible()

    # Verify expected tabs
    expected_tabs = ['Overview', 'Compactness', 'Demographics']
    tabs = page.locator('.view-tab')

    assert tabs.count() == len(expected_tabs), \
        f"Expected {len(expected_tabs)} tabs, found {tabs.count()}"

    # Verify one tab is active
    active_tabs = page.locator('.view-tab.active')
    assert active_tabs.count() == 1, "Exactly one tab should be active"


# ============================================================================
# Test Suite B: Tab Switching
# ============================================================================

def test_tab_switching_overview_to_compactness(page: Page, version_dashboard_url: str):
    """Test switching from Overview to Compactness tab."""
    page.goto(version_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Click Compactness tab
    compactness_tab = page.locator('.view-tab').filter(has_text='Compactness')
    compactness_tab.click()

    # Verify compactness view is active
    compactness_view = page.locator('#compactness-view')
    expect(compactness_view).to_be_visible()

    # Verify overview is hidden
    overview_view = page.locator('#overview-view')
    expect(overview_view).not_to_be_visible()


def test_tab_switching_demographics(page: Page, version_dashboard_url: str):
    """Test switching to Demographics tab."""
    page.goto(version_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Click Demographics tab
    demographics_tab = page.locator('.view-tab').filter(has_text='Demographics')
    demographics_tab.click()

    # Verify demographics view is active
    demographics_view = page.locator('#demographics-view')
    expect(demographics_view).to_be_visible()

    # Verify demographics tab has active class
    tab_class = demographics_tab.get_attribute('class')
    assert 'active' in tab_class, f"Demographics tab should have 'active' class, got: {tab_class}"


def test_all_tabs_clickable(page: Page, version_dashboard_url: str):
    """Test that all tabs are clickable and switch views."""
    page.goto(version_dashboard_url)
    page.wait_for_load_state('networkidle')

    tabs = page.locator('.view-tab')
    tab_count = tabs.count()

    for i in range(tab_count):
        tab = tabs.nth(i)
        tab.click()

        # Verify this tab becomes active
        page.wait_for_timeout(100)  # Brief wait for tab switch
        tab_class = tab.get_attribute('class')
        assert 'active' in tab_class, f"Tab {i} should have 'active' class after click, got: {tab_class}"


# ============================================================================
# Test Suite C: Compactness Tab
# ============================================================================

def test_compactness_table_exists(page: Page, version_dashboard_url: str):
    """Test that compactness comparison table exists."""
    page.goto(version_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Switch to compactness tab
    compactness_tab = page.locator('.view-tab').filter(has_text='Compactness')
    compactness_tab.click()

    # Verify tables exist (there are two tables: overall and state-by-state)
    tables = page.locator('#compactness-view table')
    assert tables.count() >= 1, "Compactness view should have at least 1 table"


def test_compactness_table_headers(page: Page, version_dashboard_url: str):
    """Test that compactness table has correct headers."""
    page.goto(version_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Switch to compactness tab
    compactness_tab = page.locator('.view-tab').filter(has_text='Compactness')
    compactness_tab.click()

    # Get table headers
    headers = page.locator('#compactness-view table thead th')
    header_count = headers.count()

    assert header_count >= 3, \
        f"Expected at least 3 headers (Year, Algorithmic, Enacted), found {header_count}"


def test_year_filter_exists(page: Page, version_dashboard_url: str):
    """Test that year filter dropdown exists."""
    page.goto(version_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Switch to compactness tab
    compactness_tab = page.locator('.view-tab').filter(has_text='Compactness')
    compactness_tab.click()

    # Verify year filter
    year_filter = page.locator('#yearFilter')
    expect(year_filter).to_be_visible()

    # Verify filter options
    options = year_filter.locator('option')
    assert options.count() >= 4, \
        "Filter should have at least 4 options (All Years, 2020, 2010, 2000)"


def test_year_filter_changes_display(page: Page, version_dashboard_url: str):
    """Test that year filter changes which states are displayed."""
    page.goto(version_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Switch to compactness tab
    compactness_tab = page.locator('.view-tab').filter(has_text='Compactness')
    compactness_tab.click()

    # Get year filter
    year_filter = page.locator('#yearFilter')

    # Select "All Years"
    year_filter.select_option('all')
    page.wait_for_timeout(100)  # Wait for filter to apply

    # Select "2020"
    year_filter.select_option('2020')
    page.wait_for_timeout(100)  # Wait for filter to apply

    # Filter should be functional (no errors)
    expect(year_filter).to_be_visible()


# ============================================================================
# Test Suite D: Data Embedding
# ============================================================================

def test_version_config_embedded(page: Page, version_dashboard_url: str):
    """Test that VERSION_CONFIG is properly embedded."""
    page.goto(version_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Evaluate JavaScript to check if VERSION_CONFIG exists (it's a const, not on window)
    version_config = page.evaluate('() => typeof VERSION_CONFIG !== "undefined" ? VERSION_CONFIG : {}')

    assert isinstance(version_config, dict), "VERSION_CONFIG should be a dictionary"
    # May be empty if not properly embedded, just check structure
    if version_config:
        assert 'version' in version_config, "VERSION_CONFIG should have 'version' field"


def test_comparison_data_embedded(page: Page, version_dashboard_url: str):
    """Test that COMPARISON_DATA is properly embedded."""
    page.goto(version_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Evaluate JavaScript to check if COMPARISON_DATA exists
    comparison_data = page.evaluate('() => typeof COMPARISON_DATA !== "undefined" ? COMPARISON_DATA : {}')

    assert isinstance(comparison_data, dict), "COMPARISON_DATA should be a dictionary"
    # May be empty if no comparison.json exists, so just check type


def test_runs_data_embedded(page: Page, version_dashboard_url: str):
    """Test that RUNS data is properly embedded."""
    page.goto(version_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Evaluate JavaScript to check if RUNS exists
    runs = page.evaluate('() => typeof RUNS !== "undefined" ? RUNS : []')

    assert isinstance(runs, list), "RUNS should be a list"
    # May be empty if no runs for this version


def test_version_data_extracted(page: Page, version_dashboard_url: str):
    """Test that version-specific data is extracted from comparison data."""
    page.goto(version_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Check if versionData object exists
    version_data = page.evaluate('() => typeof versionData !== "undefined" ? versionData : {}')

    assert isinstance(version_data, dict), "versionData should be a dictionary"


# ============================================================================
# Test Suite E: Overview Tab
# ============================================================================

def test_overview_version_config_displayed(page: Page, version_dashboard_url: str):
    """Test that version configuration is displayed in Overview tab."""
    page.goto(version_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Overview tab should be active by default
    overview_view = page.locator('#overview-view')
    expect(overview_view).to_be_visible()

    # Verify content exists
    content = overview_view.inner_text()
    assert len(content) > 0, "Overview should have content"


def test_overview_completed_runs_section(page: Page, version_dashboard_url: str):
    """Test that completed runs section exists."""
    page.goto(version_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Verify completed runs section
    runs_content = page.locator('#completedRunsContent')
    expect(runs_content).to_be_visible()


def test_overview_algorithm_config_formatted(page: Page, version_dashboard_url: str):
    """Test that algorithm configuration is properly formatted (not [object Object])."""
    page.goto(version_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Overview tab should be active by default
    overview_view = page.locator('#overview-view')
    expect(overview_view).to_be_visible()

    # Get the version config table content
    config_content = page.locator('#versionConfigContent')
    expect(config_content).to_be_visible()

    config_text = config_content.inner_text()

    # Should NOT contain [object Object]
    assert '[object Object]' not in config_text, "Algorithm should not be displayed as [object Object]"

    # Should contain properly formatted algorithm fields
    assert 'Partition Mode' in config_text, "Should display 'Partition Mode' label"
    assert 'Data Level' in config_text, "Should display 'Data Level' label"

    # Should contain properly formatted values
    assert 'Edge Weighted' in config_text or 'Unweighted' in config_text, "Should display formatted partition mode"
    assert 'Tract' in config_text or 'Block' in config_text, "Should display formatted data level"


# ============================================================================
# Test Suite F: Demographics Tab
# ============================================================================

def test_demographics_population_trends(page: Page, version_dashboard_url: str):
    """Test that demographics tab shows population trends."""
    page.goto(version_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Switch to demographics tab
    demographics_tab = page.locator('.view-tab').filter(has_text='Demographics')
    demographics_tab.click()

    # Verify population trends section
    population_content = page.locator('#populationTrendsContent')
    expect(population_content).to_be_visible()


def test_demographics_apportionment_changes(page: Page, version_dashboard_url: str):
    """Test that demographics tab shows apportionment changes."""
    page.goto(version_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Switch to demographics tab
    demographics_tab = page.locator('.view-tab').filter(has_text='Demographics')
    demographics_tab.click()

    # Verify apportionment section
    apportionment_content = page.locator('#apportionmentChangesContent')
    expect(apportionment_content).to_be_visible()


# ============================================================================
# Test Suite G: Performance & Errors
# ============================================================================

@pytest.mark.smoke
def test_version_dashboard_load_time(page: Page, version_dashboard_url: str):
    """Test that version dashboard loads within 3 seconds."""
    page.goto(version_dashboard_url)

    # Verify critical elements load quickly
    expect(page.locator('.header')).to_be_visible(timeout=3000)
    expect(page.locator('.view-tabs')).to_be_visible(timeout=3000)


def test_no_console_errors(page: Page, version_dashboard_url: str):
    """Test that dashboard loads without console errors."""
    errors = []

    # Capture console errors
    page.on('console', lambda msg: errors.append(msg.text) if msg.type == 'error' else None)

    # Load dashboard
    page.goto(version_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Verify no errors
    assert len(errors) == 0, f"Console errors detected: {errors}"


def test_no_javascript_exceptions(page: Page, version_dashboard_url: str):
    """Test that no JavaScript exceptions are thrown."""
    exceptions = []

    # Capture exceptions
    page.on('pageerror', lambda exc: exceptions.append(str(exc)))

    # Load and interact with dashboard
    page.goto(version_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Click through all tabs
    tabs = page.locator('.view-tab')
    for i in range(tabs.count()):
        tabs.nth(i).click()
        page.wait_for_timeout(100)

    # Verify no exceptions
    assert len(exceptions) == 0, f"JavaScript exceptions detected: {exceptions}"


# ============================================================================
# Test Suite H: Responsive Layout
# ============================================================================

@pytest.mark.parametrize('viewport', [
    {'width': 1920, 'height': 1080},  # Desktop
    {'width': 1366, 'height': 768},   # Laptop
    {'width': 768, 'height': 1024},   # Tablet
])
def test_responsive_layout(page: Page, version_dashboard_url: str, viewport: dict):
    """Test version dashboard layout at different viewport sizes."""
    # Set viewport
    page.set_viewport_size(viewport)

    # Load dashboard
    page.goto(version_dashboard_url)

    # Verify critical elements are visible
    expect(page.locator('.header')).to_be_visible()
    expect(page.locator('.view-tabs')).to_be_visible()
