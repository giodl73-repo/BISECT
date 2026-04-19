"""
Visual regression tests for dashboards.

These tests capture screenshots and compare them to baseline images
to detect unintended visual changes.

Usage:
    # Generate/update baselines
    pytest tests/e2e/test_visual_regression.py --update-baselines

    # Run visual regression tests
    pytest tests/e2e/test_visual_regression.py -v
"""

import pytest
from playwright.sync_api import Page, expect
from pathlib import Path


# ============================================================================
# Fixtures
# ============================================================================

@pytest.fixture
def baseline_dir(screenshots_dir):
    """Get baseline screenshots directory."""
    baseline = screenshots_dir / 'baseline'
    baseline.mkdir(parents=True, exist_ok=True)
    return baseline


@pytest.fixture
def diff_dir(screenshots_dir):
    """Get diff screenshots directory."""
    diff = screenshots_dir / 'diff'
    diff.mkdir(parents=True, exist_ok=True)
    return diff


# ============================================================================
# Test Suite A: Run Dashboard Visual Regression
# ============================================================================

@pytest.mark.visual
def test_run_dashboard_full_page_desktop(page: Page, dashboard_url: str, baseline_dir: Path):
    """Visual regression test for full dashboard at desktop resolution."""
    # Set desktop viewport
    page.set_viewport_size({'width': 1920, 'height': 1080})

    # Load dashboard
    page.goto(dashboard_url)
    page.wait_for_load_state('networkidle')

    # Take screenshot
    screenshot_path = baseline_dir / 'run_dashboard_desktop_full.png'
    page.screenshot(path=str(screenshot_path), full_page=True)


@pytest.mark.visual
def test_run_dashboard_overview_tab(page: Page, dashboard_url: str, baseline_dir: Path):
    """Visual regression test for Overview tab."""
    page.goto(dashboard_url)
    page.wait_for_load_state('networkidle')

    # Select Vermont
    page.locator('.state-item[data-state="vermont"]').click()

    # Click Overview tab (should be default)
    page.locator('.dimension-tab[data-dimension="overview"]').click()

    # Wait for content
    page.wait_for_timeout(500)  # Brief pause for rendering

    # Take screenshot of content area
    content_area = page.locator('.content-area')
    screenshot_path = baseline_dir / 'run_dashboard_overview_tab.png'
    content_area.screenshot(path=str(screenshot_path))


@pytest.mark.visual
def test_run_dashboard_districts_tab(page: Page, dashboard_url: str, baseline_dir: Path):
    """Visual regression test for Districts tab."""
    page.goto(dashboard_url)
    page.wait_for_load_state('networkidle')

    # Select Vermont
    page.locator('.state-item[data-state="vermont"]').click()

    # Click Districts tab
    page.locator('.dimension-tab[data-dimension="districts"]').click()

    # Wait for content
    page.wait_for_timeout(500)

    # Take screenshot of content area
    content_area = page.locator('.content-area')
    screenshot_path = baseline_dir / 'run_dashboard_districts_tab.png'
    content_area.screenshot(path=str(screenshot_path))


@pytest.mark.visual
def test_run_dashboard_rounds_tab(page: Page, dashboard_url: str, baseline_dir: Path):
    """Visual regression test for Rounds tab."""
    page.goto(dashboard_url)
    page.wait_for_load_state('networkidle')

    # Select Vermont
    page.locator('.state-item[data-state="vermont"]').click()

    # Click Rounds tab
    page.locator('.dimension-tab[data-dimension="rounds"]').click()

    # Wait for content
    page.wait_for_timeout(500)

    # Take screenshot of content area
    content_area = page.locator('.content-area')
    screenshot_path = baseline_dir / 'run_dashboard_rounds_tab.png'
    content_area.screenshot(path=str(screenshot_path))


@pytest.mark.visual
def test_run_dashboard_compactness_tab(page: Page, dashboard_url: str, baseline_dir: Path):
    """Visual regression test for Compactness tab."""
    page.goto(dashboard_url)
    page.wait_for_load_state('networkidle')

    # Select Vermont
    page.locator('.state-item[data-state="vermont"]').click()

    # Click Compactness tab
    page.locator('.dimension-tab[data-dimension="compactness"]').click()

    # Wait for content
    page.wait_for_timeout(500)

    # Take screenshot of content area
    content_area = page.locator('.content-area')
    screenshot_path = baseline_dir / 'run_dashboard_compactness_tab.png'
    content_area.screenshot(path=str(screenshot_path))


@pytest.mark.visual
def test_run_dashboard_sidebar(page: Page, dashboard_url: str, baseline_dir: Path):
    """Visual regression test for sidebar."""
    page.goto(dashboard_url)
    page.wait_for_load_state('networkidle')

    # Take screenshot of sidebar
    sidebar = page.locator('.sidebar')
    screenshot_path = baseline_dir / 'run_dashboard_sidebar.png'
    sidebar.screenshot(path=str(screenshot_path))


@pytest.mark.visual
def test_run_dashboard_header(page: Page, dashboard_url: str, baseline_dir: Path):
    """Visual regression test for header."""
    page.goto(dashboard_url)
    page.wait_for_load_state('networkidle')

    # Take screenshot of header
    header = page.locator('.header')
    screenshot_path = baseline_dir / 'run_dashboard_header.png'
    header.screenshot(path=str(screenshot_path))


# ============================================================================
# Test Suite B: Master Dashboard Visual Regression
# ============================================================================

@pytest.mark.visual
def test_master_dashboard_full_page(page: Page, master_dashboard_url: str, baseline_dir: Path):
    """Visual regression test for full master dashboard."""
    page.set_viewport_size({'width': 1920, 'height': 1080})

    page.goto(master_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Take screenshot
    screenshot_path = baseline_dir / 'master_dashboard_full.png'
    page.screenshot(path=str(screenshot_path), full_page=True)


@pytest.mark.visual
def test_master_dashboard_comparison_table(page: Page, master_dashboard_url: str, baseline_dir: Path):
    """Visual regression test for comparison table."""
    page.goto(master_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Take screenshot of main container (new structure doesn't use .comparison-table)
    main_container = page.locator('.main-container')
    screenshot_path = baseline_dir / 'master_dashboard_comparison_table.png'
    main_container.screenshot(path=str(screenshot_path))


@pytest.mark.visual
def test_master_dashboard_run_cards(page: Page, master_dashboard_url: str, baseline_dir: Path):
    """Visual regression test for run cards."""
    page.goto(master_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Take screenshot of view tabs section (new structure uses .view-tabs, not .run-cards)
    view_tabs = page.locator('.view-tabs')
    screenshot_path = baseline_dir / 'master_dashboard_run_cards.png'
    view_tabs.screenshot(path=str(screenshot_path))


# ============================================================================
# Test Suite C: Responsive Design Regression
# ============================================================================

@pytest.mark.visual
@pytest.mark.parametrize('viewport_name,viewport', [
    ('desktop', {'width': 1920, 'height': 1080}),
    ('laptop', {'width': 1366, 'height': 768}),
    ('tablet', {'width': 768, 'height': 1024}),
])
def test_run_dashboard_responsive(page: Page, dashboard_url: str, baseline_dir: Path,
                                  viewport_name: str, viewport: dict):
    """Visual regression test for responsive layouts."""
    # Set viewport
    page.set_viewport_size(viewport)

    # Load dashboard
    page.goto(dashboard_url)
    page.wait_for_load_state('networkidle')

    # Take screenshot
    screenshot_path = baseline_dir / f'run_dashboard_responsive_{viewport_name}.png'
    page.screenshot(path=str(screenshot_path), full_page=False)


@pytest.mark.visual
@pytest.mark.parametrize('viewport_name,viewport', [
    ('desktop', {'width': 1920, 'height': 1080}),
    ('laptop', {'width': 1366, 'height': 768}),
    ('tablet', {'width': 768, 'height': 1024}),
])
def test_master_dashboard_responsive(page: Page, master_dashboard_url: str, baseline_dir: Path,
                                     viewport_name: str, viewport: dict):
    """Visual regression test for master dashboard responsive layouts."""
    # Set viewport
    page.set_viewport_size(viewport)

    # Load dashboard
    page.goto(master_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Take screenshot
    screenshot_path = baseline_dir / f'master_dashboard_responsive_{viewport_name}.png'
    page.screenshot(path=str(screenshot_path), full_page=False)


# ============================================================================
# Test Suite D: State-Specific Visual Tests
# ============================================================================

@pytest.mark.visual
@pytest.mark.parametrize('state', ['vermont', 'delaware'])
def test_state_overview_visual(page: Page, dashboard_url: str, baseline_dir: Path, state: str):
    """Visual regression test for state overview pages."""
    page.goto(dashboard_url)
    page.wait_for_load_state('networkidle')

    # Select state
    page.locator(f'.state-item[data-state="{state}"]').click()

    # Click Overview tab
    page.locator('.tab[data-tab="overview"]').click()
    page.wait_for_timeout(500)

    # Take screenshot
    content_area = page.locator('.content-area')
    screenshot_path = baseline_dir / f'state_{state}_overview.png'
    content_area.screenshot(path=str(screenshot_path))


# ============================================================================
# Test Suite E: Screenshot Comparison (Actual Regression Tests)
# ============================================================================
# Note: These tests will only run if baselines exist

@pytest.mark.visual
def test_visual_regression_overview_tab(page: Page, dashboard_url: str, baseline_dir: Path, diff_dir: Path):
    """Compare Overview tab screenshot to baseline."""
    baseline_path = baseline_dir / 'run_dashboard_overview_tab.png'

    if not baseline_path.exists():
        pytest.skip(f"Baseline not found: {baseline_path}. Run with --update-baselines first.")

    page.goto(dashboard_url)
    page.wait_for_load_state('networkidle')

    # Select Vermont
    page.locator('.state-item[data-state="vermont"]').click()

    # Click Overview tab
    page.locator('.tab[data-tab="overview"]').click()
    page.wait_for_timeout(500)

    # Compare screenshot
    content_area = page.locator('.content-area')

    # Use Playwright's built-in screenshot comparison
    try:
        expect(content_area).to_have_screenshot(
            'run_dashboard_overview_tab.png',
            max_diff_pixels=100
        )
    except AssertionError as e:
        # Save diff screenshot
        diff_path = diff_dir / 'run_dashboard_overview_tab_diff.png'
        content_area.screenshot(path=str(diff_path))
        raise AssertionError(f"Visual regression detected. Diff saved to: {diff_path}") from e


@pytest.mark.visual
def test_visual_regression_comparison_table(page: Page, master_dashboard_url: str, baseline_dir: Path, diff_dir: Path):
    """Compare comparison table screenshot to baseline."""
    baseline_path = baseline_dir / 'master_dashboard_comparison_table.png'

    if not baseline_path.exists():
        pytest.skip(f"Baseline not found: {baseline_path}. Run with --update-baselines first.")

    page.goto(master_dashboard_url)
    page.wait_for_load_state('networkidle')

    # Compare screenshot (new structure uses .main-container, not .comparison-table)
    main_container = page.locator('.main-container')

    try:
        expect(main_container).to_have_screenshot(
            'master_dashboard_comparison_table.png',
            max_diff_pixels=100
        )
    except AssertionError as e:
        # Save diff screenshot
        diff_path = diff_dir / 'master_dashboard_comparison_table_diff.png'
        main_container.screenshot(path=str(diff_path))
        raise AssertionError(f"Visual regression detected. Diff saved to: {diff_path}") from e


# ============================================================================
# Utility Functions
# ============================================================================

def get_baseline_paths(baseline_dir: Path) -> list[Path]:
    """Get all baseline screenshot paths."""
    if not baseline_dir.exists():
        return []
    return list(baseline_dir.glob('*.png'))


def baseline_exists(baseline_dir: Path, name: str) -> bool:
    """Check if baseline screenshot exists."""
    return (baseline_dir / name).exists()
