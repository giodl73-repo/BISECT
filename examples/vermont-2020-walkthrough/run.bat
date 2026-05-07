@echo off
REM Vermont 2020 canonical walkthrough — Onboarding plan Task 1 (Windows mirror)
REM See run.sh for full documentation. ASCII-only output (CP1252).

setlocal enabledelayedexpansion

if "%REPO_ROOT%"=="" (
    pushd "%~dp0..\.." & set "REPO_ROOT=!CD!" & popd
)
if "%VERSION%"=="" set "VERSION=tutorial"
if "%OUTPUT_BASE%"=="" set "OUTPUT_BASE=%REPO_ROOT%\outputs"
set "YEAR=2020"
set "STATE=VT"
set "LABEL=vt_2020_tutorial"

cd /d "%REPO_ROOT%"

echo.
echo [step 1] Verifying redist binary on PATH...
where redist >NUL 2>&1
if errorlevel 1 (
    echo [FAIL] redist not on PATH ^(run bootstrap.bat first^)
    exit /b 1
)
echo [OK] redist found

echo.
echo [step 2] Fetching Vermont 2020 Census TIGER tracts ^(FIPS 50^)...
redist fetch --type tiger --states %STATE% --year %YEAR% --output-base "%OUTPUT_BASE%"
if errorlevel 1 (echo [FAIL] TIGER fetch failed & exit /b 1)

echo.
echo [step 3] Building adjacency graph from tract geometries...
redist fetch --type adjacency --states %STATE% --year %YEAR% --output-base "%OUTPUT_BASE%"
echo [OK] Adjacency present

echo.
echo [step 4] Running bisection: redist state --state %STATE% --year %YEAR% --label %LABEL%
redist state --state %STATE% --year %YEAR% --label %LABEL% --version %VERSION% --output-base "%OUTPUT_BASE%"
if errorlevel 1 (echo [FAIL] bisection failed & exit /b 1)

set "ASSIGN_FILE=%OUTPUT_BASE%\%VERSION%\%YEAR%\plans\%LABEL%\final_assignments.json"
if not exist "%ASSIGN_FILE%" (echo [FAIL] missing %ASSIGN_FILE% & exit /b 1)

echo.
echo [step 5] Running analyses: redist analyze --label %LABEL% --types all
redist analyze --label %LABEL% --year %YEAR% --version %VERSION% --output-base "%OUTPUT_BASE%" --types all
if errorlevel 1 (echo [FAIL] analyze failed & exit /b 1)

echo.
echo [step 6] Generating report: redist report --label %LABEL% --format html
redist report --label %LABEL% --year %YEAR% --version %VERSION% --output-base "%OUTPUT_BASE%" --format html
if errorlevel 1 (echo [FAIL] report failed & exit /b 1)

echo.
echo [step 7] Verifying provenance: redist doctor --verify-manifest
set "MANIFEST=%OUTPUT_BASE%\%VERSION%\%YEAR%\plans\%LABEL%\manifest.json"
redist doctor --verify-manifest "%MANIFEST%"
if errorlevel 1 (echo [FAIL] manifest verification failed & exit /b 1)

echo.
echo ==================================================
echo Vermont 2020 walkthrough complete.
echo Plan dir: %OUTPUT_BASE%\%VERSION%\%YEAR%\plans\%LABEL%\
echo.
echo Validate against pinned checksums:
echo   redist doctor --check-tutorial-data --tutorial vermont-2020
echo ==================================================

endlocal
