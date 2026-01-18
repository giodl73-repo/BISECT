@echo off
REM ============================================================================
REM Run Playwright test harness for dashboard testing
REM
REM Usage:
REM   run_dashboard_tests.bat                  - Run all tests (headless)
REM   run_dashboard_tests.bat --smoke          - Run smoke tests only (quick)
REM   run_dashboard_tests.bat --headed         - Run with visible browser
REM   run_dashboard_tests.bat --update-baselines - Update visual baselines
REM ============================================================================

REM Change to project root (one level up from batch/)
cd /d "%~dp0\.."

setlocal enabledelayedexpansion

REM Parse command-line arguments
set MODE=all
set HEADED=
set UPDATE_BASELINES=
set EXTRA_ARGS=

:parse_args
if "%~1"=="" goto end_parse
if /i "%~1"=="--smoke" set MODE=smoke
if /i "%~1"=="--headed" set HEADED=--headed
if /i "%~1"=="--update-baselines" set UPDATE_BASELINES=--update-baselines
if /i "%~1"=="--help" goto show_help
shift
goto parse_args
:end_parse

echo ============================================================================
echo Congressional Redistricting - Dashboard Test Suite
echo ============================================================================
echo.

REM Check if pytest is installed
python -c "import pytest" 2>nul
if errorlevel 1 (
    echo [ERROR] pytest not installed
    echo.
    echo Please install test dependencies:
    echo   pip install pytest pytest-playwright playwright
    echo   playwright install chromium
    exit /b 1
)

REM Check if playwright is installed
python -c "import playwright" 2>nul
if errorlevel 1 (
    echo [ERROR] playwright not installed
    echo.
    echo Please install test dependencies:
    echo   pip install pytest pytest-playwright playwright
    echo   playwright install chromium
    exit /b 1
)

REM Display test mode
if "%MODE%"=="smoke" (
    echo Mode: Smoke tests only ^(quick validation^)
    echo.
    set EXTRA_ARGS=-m smoke
) else (
    echo Mode: Full test suite
    echo.
)

if not "%HEADED%"=="" (
    echo Browser: Visible ^(headed mode^)
    echo.
) else (
    echo Browser: Headless
    echo.
)

REM Run tests
echo Running tests...
echo.

if not "%UPDATE_BASELINES%"=="" (
    echo [UPDATE] Regenerating visual regression baselines
    echo.
    python -m pytest tests/e2e/test_visual_regression.py %UPDATE_BASELINES% -v
) else (
    python -m pytest tests/e2e/ %EXTRA_ARGS% %HEADED% -v
)

set TEST_EXIT_CODE=%errorlevel%

echo.
echo ============================================================================

if %TEST_EXIT_CODE%==0 (
    echo [OK] All tests passed
) else (
    echo [FAIL] Some tests failed
    echo.
    echo Screenshots saved to: tests\screenshots\diff\
    echo Test report: tests\test-results\html-report\index.html
)

echo ============================================================================

exit /b %TEST_EXIT_CODE%

:show_help
echo Usage: run_dashboard_tests.bat [OPTIONS]
echo.
echo Options:
echo   --smoke              Run smoke tests only ^(30 seconds^)
echo   --headed             Run with visible browser ^(debug mode^)
echo   --update-baselines   Update visual regression baselines
echo   --help               Show this help message
echo.
echo Examples:
echo   run_dashboard_tests.bat                   Full test suite ^(headless^)
echo   run_dashboard_tests.bat --smoke           Quick smoke test
echo   run_dashboard_tests.bat --headed          Full suite with visible browser
echo   run_dashboard_tests.bat --update-baselines Update baselines after UI change
echo.
exit /b 0
