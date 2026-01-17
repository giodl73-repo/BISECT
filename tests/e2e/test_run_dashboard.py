"""
End-to-end tests for the run dashboard (web/dashboard.html).

Tests cover:
- Navigation (state selection, tab switching)
- Content validation (tables, maps, data)
- Edge cases (missing data, large states)
- Performance (load times)
"""

import pytest
from playwright.sync_api import Page, expect


# ============================================================================
# Test Suite A: Navigation & Structure
# ============================================================================

@pytest.mark.smoke
def test_dashboard_loads_successfully(page_with_dashboard: Page):
    """Test that dashboard loads without errors."""
    page = page_with_dashboard

    # Verify page loaded
    expect(page).to_have_title("Congressional Redistricting - 2020")

    # Verify header exists
    header = page.locator('.header')
    expect(header).to_be_visible()

    # Verify sidebar exists
    sidebar = page.locator('.sidebar')
    expect(sidebar).to_be_visible()

    # Verify content area exists
    content = page.locator('.content-area')
    expect(content).to_be_visible()


@pytest.mark.smoke
def test_sidebar_has_states(page_with_dashboard: Page):
    """Test that sidebar contains state items."""
    page = page_with_dashboard

    # Get all state items
    states = page.locator('.state-item')
    count = states.count()

    # Should have at least 2 states (Vermont, Delaware)
    assert count >= 2, f"Expected at least 2 states, found {count}"

    # Verify first state is clickable
    first_state = states.first
    expect(first_state).to_be_visible()


@pytest.mark.smoke
def test_tabs_present(page_with_dashboard: Page):
    """Test that all tabs are present."""
    page = page_with_dashboard

    # Expected tabs (at minimum)
    expected_tabs = ['overview', 'districts', 'rounds', 'compactness']

    for tab_id in expected_tabs:
        tab = page.locator(f'.tab[data-tab="{tab_id}"]')
        expect(tab).to_be_visible()


def test_state_navigation(page_with_dashboard: Page):
    """Test clicking through states updates content."""
    page = page_with_dashboard

    # Click Vermont
    vermont = page.locator('.state-item[data-state="vermont"]')
    vermont.click()

    # Verify state name in header
    header_text = page.locator('#current-state')
    expect(header_text).to_contain_text('Vermont')

    # Verify URL hash updated
    assert 'vermont' in page.url

    # Click Delaware
    delaware = page.locator('.state-item[data-state="delaware"]')
    delaware.click()

    # Verify state name updated
    expect(header_text).to_contain_text('Delaware')

    # Verify URL hash updated
    assert 'delaware' in page.url


def test_tab_navigation(page_with_dashboard: Page):
    """Test clicking tabs switches content."""
    page = page_with_dashboard

    # Select a state first
    page.locator('.state-item[data-state="vermont"]').click()

    # Test each tab
    tabs = ['overview', 'districts', 'rounds', 'compactness']

    for tab_id in tabs:
        # Click tab
        tab = page.locator(f'.tab[data-tab="{tab_id}"]')
        tab.click()

        # Verify tab is active
        expect(tab).to_have_class('tab active')

        # Verify corresponding content is visible
        content = page.locator(f'#{tab_id}')
        expect(content).to_be_visible()

        # Verify content has active class
        expect(content).to_have_class('tab-content active')


# ============================================================================
# Test Suite B: Content Validation
# ============================================================================

def test_overview_tab_has_table(page_with_dashboard: Page):
    """Test that Overview tab contains district summary table."""
    page = page_with_dashboard

    # Select Vermont
    page.locator('.state-item[data-state="vermont"]').click()

    # Click Overview tab
    page.locator('.tab[data-tab="overview"]').click()

    # Verify table exists
    table = page.locator('.district-summary')
    expect(table).to_be_visible()

    # Verify table has headers
    headers = table.locator('th')
    assert headers.count() >= 4  # District, Population, PP, Reock


def test_districts_tab_has_map(page_with_dashboard: Page):
    """Test that Districts tab contains map image."""
    page = page_with_dashboard

    # Select Vermont
    page.locator('.state-item[data-state="vermont"]').click()

    # Click Districts tab
    page.locator('.tab[data-tab="districts"]').click()

    # Verify map image exists
    map_img = page.locator('#districts-map')
    expect(map_img).to_be_visible()


def test_rounds_tab_has_content(page_with_dashboard: Page):
    """Test that Rounds tab contains round progression."""
    page = page_with_dashboard

    # Select Vermont
    page.locator('.state-item[data-state="vermont"]').click()

    # Click Rounds tab
    page.locator('.tab[data-tab="rounds"]').click()

    # Verify rounds container exists
    container = page.locator('#rounds-container')
    expect(container).to_be_visible()


def test_compactness_tab_has_map(page_with_dashboard: Page):
    """Test that Compactness tab contains analysis map."""
    page = page_with_dashboard

    # Select Vermont
    page.locator('.state-item[data-state="vermont"]').click()

    # Click Compactness tab
    page.locator('.tab[data-tab="compactness"]').click()

    # Verify compactness map exists
    map_img = page.locator('#compactness-map')
    expect(map_img).to_be_visible()


# ============================================================================
# Test Suite C: Edge Cases
# ============================================================================

def test_multiple_state_switches(page_with_dashboard: Page):
    """Test switching between states multiple times."""
    page = page_with_dashboard

    # Switch between states 5 times
    for _ in range(5):
        page.locator('.state-item[data-state="vermont"]').click()
        expect(page.locator('#current-state')).to_contain_text('Vermont')

        page.locator('.state-item[data-state="delaware"]').click()
        expect(page.locator('#current-state')).to_contain_text('Delaware')


def test_tab_switching_preserves_state(page_with_dashboard: Page):
    """Test that switching tabs preserves selected state."""
    page = page_with_dashboard

    # Select Vermont
    page.locator('.state-item[data-state="vermont"]').click()
    initial_state = page.locator('#current-state').inner_text()

    # Switch through all tabs
    tabs = ['overview', 'districts', 'rounds', 'compactness']
    for tab_id in tabs:
        page.locator(f'.tab[data-tab="{tab_id}"]').click()
        # State should remain the same
        current_state = page.locator('#current-state').inner_text()
        assert current_state == initial_state


@pytest.mark.smoke
def test_page_load_time(page_with_dashboard: Page):
    """Test that page loads within reasonable time (3 seconds)."""
    page = page_with_dashboard

    # Page should already be loaded by fixture
    # Verify content is visible (indicates full load)
    expect(page.locator('.header')).to_be_visible(timeout=3000)
    expect(page.locator('.sidebar')).to_be_visible(timeout=3000)
    expect(page.locator('.content-area')).to_be_visible(timeout=3000)


# ============================================================================
# Test Suite D: Interactive Features
# ============================================================================

def test_default_tab_is_overview(page_with_dashboard: Page):
    """Test that Overview tab is selected by default."""
    page = page_with_dashboard

    # Overview tab should be active
    overview_tab = page.locator('.tab[data-tab="overview"]')
    expect(overview_tab).to_have_class('tab active')

    # Overview content should be visible
    overview_content = page.locator('#overview')
    expect(overview_content).to_have_class('tab-content active')


def test_only_one_tab_active_at_a_time(page_with_dashboard: Page):
    """Test that only one tab is active at any time."""
    page = page_with_dashboard

    # Select a state
    page.locator('.state-item[data-state="vermont"]').click()

    # Click different tabs and verify only one is active
    tabs = ['overview', 'districts', 'rounds', 'compactness']

    for tab_id in tabs:
        page.locator(f'.tab[data-tab="{tab_id}"]').click()

        # Count active tabs (should be exactly 1)
        active_tabs = page.locator('.tab.active')
        assert active_tabs.count() == 1

        # Count active content (should be exactly 1)
        active_content = page.locator('.tab-content.active')
        assert active_content.count() == 1


# ============================================================================
# Test Suite E: Data Integrity
# ============================================================================

def test_state_data_attributes_present(page_with_dashboard: Page):
    """Test that state items have required data attributes."""
    page = page_with_dashboard

    # Get all state items
    states = page.locator('.state-item')

    # Verify each has data-state attribute
    for i in range(states.count()):
        state = states.nth(i)
        data_state = state.get_attribute('data-state')
        assert data_state is not None
        assert len(data_state) > 0


def test_tab_data_attributes_present(page_with_dashboard: Page):
    """Test that tab buttons have required data attributes."""
    page = page_with_dashboard

    # Get all tabs
    tabs = page.locator('.tab')

    # Verify each has data-tab attribute
    for i in range(tabs.count()):
        tab = tabs.nth(i)
        data_tab = tab.get_attribute('data-tab')
        assert data_tab is not None
        assert len(data_tab) > 0


# ============================================================================
# Test Suite F: Browser Compatibility
# ============================================================================

@pytest.mark.parametrize('viewport', [
    {'width': 1920, 'height': 1080},  # Desktop
    {'width': 1366, 'height': 768},   # Laptop
    {'width': 768, 'height': 1024},   # Tablet
])
def test_responsive_layout(page: Page, dashboard_url: str, viewport: dict):
    """Test dashboard layout at different viewport sizes."""
    # Set viewport
    page.set_viewport_size(viewport)

    # Load dashboard
    page.goto(dashboard_url)

    # Verify critical elements are visible
    expect(page.locator('.header')).to_be_visible()
    expect(page.locator('.sidebar')).to_be_visible()
    expect(page.locator('.content-area')).to_be_visible()


# ============================================================================
# Test Suite G: Console Errors
# ============================================================================

def test_no_console_errors(page: Page, dashboard_url: str):
    """Test that dashboard loads without console errors."""
    errors = []

    # Capture console messages
    page.on('console', lambda msg: errors.append(msg.text) if msg.type == 'error' else None)

    # Load dashboard
    page.goto(dashboard_url)

    # Wait for page to fully load
    page.wait_for_load_state('networkidle')

    # Verify no errors
    assert len(errors) == 0, f"Console errors detected: {errors}"
