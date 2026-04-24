@echo off
REM Run acceptance tests against a specific pipeline version and year.
REM Usage:
REM   run_acceptance_tests.bat V3 2020     Test V3/2020 only
REM   run_acceptance_tests.bat V4 2020     Test V4/2020 (VRA run)
REM   run_acceptance_tests.bat V3          Test V3 all decades

set VERSION=%1
set YEAR=%2

if "%VERSION%"=="" (
    echo Usage: run_acceptance_tests.bat ^<version^> [year]
    echo   e.g. run_acceptance_tests.bat V3 2020
    exit /b 1
)

echo =============================================
echo  Acceptance Tests: %VERSION% %YEAR%
echo =============================================
echo.

if "%YEAR%"=="" (
    set PIPELINE_VERSION=%VERSION%
    py -3.13 -m pytest tests/integration -v --tb=short
) else (
    set PIPELINE_VERSION=%VERSION%
    set PIPELINE_YEAR=%YEAR%
    py -3.13 -m pytest tests/integration -v --tb=short
)
