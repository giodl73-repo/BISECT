@echo off
REM ============================================================================
REM Generate test data for Playwright dashboard tests
REM
REM Creates a complete test dataset including:
REM - Small states (VT, DE, WY) for quick testing
REM - Medium state (AL) for multi-district testing
REM - All analysis stages (political, demographic, compactness, urban)
REM - National maps and aggregations
REM - Generated dashboards
REM
REM Usage:
REM   create_test_data.bat              - Full test dataset (4 states + national)
REM   create_test_data.bat --quick      - Quick test dataset (VT, DE only)
REM   create_test_data.bat --full       - All 50 states (slow, for comprehensive testing)
REM ============================================================================

setlocal enabledelayedexpansion

REM Parse command-line arguments
set MODE=standard
if /i "%~1"=="--quick" set MODE=quick
if /i "%~1"=="--full" set MODE=full
if /i "%~1"=="--help" goto show_help

echo ============================================================================
echo Dashboard Test Data Generation
echo ============================================================================
echo.

REM Set states based on mode
if "%MODE%"=="quick" (
    set STATES=VT DE
    set DESCRIPTION=Quick mode: Vermont ^+ Delaware ^(~2 minutes^)
) else if "%MODE%"=="full" (
    set STATES=
    set DESCRIPTION=Full mode: All 50 states ^(~2-4 hours^)
) else (
    set STATES=VT DE WY AL
    set DESCRIPTION=Standard mode: VT ^+ DE ^+ WY ^+ AL ^(~5 minutes^)
)

echo Mode: %MODE%
echo States: %DESCRIPTION%
echo Year: 2020
echo Version: test
echo.
if not "%MODE%"=="full" (
    echo NOTE: National maps will NOT be generated ^(partial state run^)
    echo       National maps require all 50 states for meaningful aggregation
    echo       This is OK for dashboard testing - tests focus on state-level content
    echo.
)

REM Confirm if full mode
if "%MODE%"=="full" (
    echo WARNING: Full 50-state run will take 2-4 hours
    echo.
    set /p CONFIRM="Continue with full run? (y/n): "
    if /i not "!CONFIRM!"=="y" (
        echo Cancelled.
        exit /b 0
    )
    echo.
)

echo [STEP 1/3] Running redistricting pipeline...
echo.

if "%MODE%"=="full" (
    REM Full 50-state run with national maps and aggregation
    python scripts/pipeline/run_complete_redistricting.py --year 2020 --version test --reset --dpi 150
) else (
    REM Subset of states - use process_single_state.py to avoid national processing
    REM Process each state individually (no national maps/aggregates)
    for %%S in (%STATES%) do (
        echo.
        echo Processing %%S...
        python scripts/pipeline/process_single_state.py --state %%S --year 2020 --output-dir outputs\us_2020_test --dpi 150
        if errorlevel 1 (
            echo [ERROR] Failed to process %%S
            exit /b 1
        )
    )
)

if errorlevel 1 (
    echo.
    echo [ERROR] Redistricting pipeline failed
    exit /b 1
)

echo.
echo [STEP 2/3] Verifying outputs...
echo.

REM Check that key files exist
set OUTPUT_DIR=outputs\us_2020_test

if not exist "%OUTPUT_DIR%" (
    echo [ERROR] Output directory not found: %OUTPUT_DIR%
    exit /b 1
)

if not exist "%OUTPUT_DIR%\states\vermont" (
    echo [ERROR] Vermont data not found
    exit /b 1
)

echo [OK] Vermont data exists
if exist "%OUTPUT_DIR%\states\delaware" echo [OK] Delaware data exists
if exist "%OUTPUT_DIR%\states\wyoming" echo [OK] Wyoming data exists
if exist "%OUTPUT_DIR%\states\alabama" echo [OK] Alabama data exists

REM Check for national maps
if exist "%OUTPUT_DIR%\national" (
    echo [OK] National maps exist
) else (
    echo [WARN] National maps not found
)

echo.
echo [STEP 3/3] Generating dashboard...
echo.

REM Check if dashboard exists
if exist "%OUTPUT_DIR%\index.html" (
    echo [OK] Dashboard already generated: %OUTPUT_DIR%\index.html
) else (
    echo [INFO] Dashboard not found, may need to run deploy_web.bat
)

echo.
echo ============================================================================
echo [OK] Test Data Generation Complete
echo ============================================================================
echo.
echo Test data location: %OUTPUT_DIR%
echo.
echo Next steps:
echo   1. Run dashboard tests:
echo      run_dashboard_tests.bat
echo.
echo   2. Or run smoke tests only ^(30 seconds^):
echo      run_dashboard_tests.bat --smoke
echo.
echo   3. Or run with visible browser ^(debug^):
echo      run_dashboard_tests.bat --headed
echo.
echo   4. Generate dashboard ^(if needed^):
echo      deploy_web.bat --year 2020 --version test
echo.
echo ============================================================================

exit /b 0

:show_help
echo Usage: create_test_data.bat [MODE]
echo.
echo Modes:
echo   ^(default^)        Standard mode: VT + DE + WY + AL ^(~5 minutes^)
echo   --quick          Quick mode: VT + DE only ^(~2 minutes^)
echo   --full           Full mode: All 50 states ^(~2-4 hours^)
echo   --help           Show this help message
echo.
echo Examples:
echo   create_test_data.bat              Standard test dataset
echo   create_test_data.bat --quick      Minimal test dataset ^(fast^)
echo   create_test_data.bat --full       Complete 50-state dataset
echo.
echo What this generates:
echo   - District assignments and maps
echo   - Political analysis ^(2020 only^)
echo   - Demographic analysis
echo   - Compactness metrics
echo   - Urban metro area maps
echo   - Round progression maps
echo   - National aggregations
echo   - District summary CSVs
echo.
echo Output location: outputs\us_2020_test\
echo.
exit /b 0
