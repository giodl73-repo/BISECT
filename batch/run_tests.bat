@echo off
REM Test execution script for pipeline tests
REM Usage: run_tests.bat [test_type]

REM Change to project root (one level up from batch/)
cd /d "%~dp0\.."

setlocal enabledelayedexpansion

REM Parse arguments
set TEST_TYPE=%1
if "%TEST_TYPE%"=="" set TEST_TYPE=all

echo [INFO] Running pipeline tests: %TEST_TYPE%
echo.

if "%TEST_TYPE%"=="unit" goto run_unit
if "%TEST_TYPE%"=="integration" goto run_integration
if "%TEST_TYPE%"=="e2e" goto run_e2e
if "%TEST_TYPE%"=="quick" goto run_quick
if "%TEST_TYPE%"=="all" goto run_all
if "%TEST_TYPE%"=="coverage" goto run_coverage
if "%TEST_TYPE%"=="markers" goto run_markers
goto unknown_type

:run_unit
echo [INFO] Running unit tests...
python -m pytest tests/unit -v --tb=short --cov=apportionment --cov-report=term --cov-report=html
goto end

:run_integration
echo [INFO] Running integration tests...
python -m pytest tests/integration -v --tb=short
goto end

:run_e2e
echo [INFO] Running E2E tests...
echo [INFO] Note: Run create_test_data.bat first to generate test data
python -m pytest tests/e2e -v --tb=short
goto end

:run_quick
echo [INFO] Running quick test suite...
python -m pytest tests/unit -v --tb=short -x
goto end

:run_all
echo [INFO] Running all pipeline tests...
python -m pytest tests/unit tests/integration -v --tb=short --cov=apportionment --cov-report=term --cov-report=html
goto end

:run_coverage
echo [INFO] Running tests with detailed coverage report...
python -m pytest tests/unit tests/integration -v --tb=short --cov=apportionment --cov-report=term --cov-report=html --cov-report=xml
echo [INFO] Coverage report generated:
echo   - HTML: htmlcov/index.html
echo   - XML: coverage.xml
goto end

:run_markers
echo [INFO] Available test markers:
python -m pytest --markers
goto end

:unknown_type
echo [ERROR] Unknown test type: %TEST_TYPE%
echo.
echo Usage: run_tests.bat [test_type]
echo.
echo Test types:
echo   unit          - Run unit tests only
echo   integration   - Run integration tests only
echo   e2e           - Run end-to-end tests
echo   quick         - Run quick test suite
echo   all           - Run all pipeline tests (default)
echo   coverage      - Run tests with detailed coverage report
echo   markers       - Show available pytest markers
echo.
echo Examples:
echo   run_tests.bat unit
echo   run_tests.bat all
echo   run_tests.bat coverage
exit /b 1

:end
echo.
if %ERRORLEVEL% EQU 0 (
    echo [OK] Tests completed successfully
) else (
    echo [FAIL] Tests failed with exit code %ERRORLEVEL%
)
exit /b %ERRORLEVEL%
