@echo off
REM Wrapper for test/debug redistricting runs
REM Automatically uses --run-type test (outputs to dev/)

REM Change to project root (one level up from batch/)
cd /d "%~dp0\.."

REM Pass all arguments plus --run-type test to the Python script
python scripts/pipeline/run_complete_redistricting.py --run-type test %*

REM If Ctrl+C was pressed, ensure Python processes are killed
if errorlevel 1 (
    echo.
    echo Cleaning up any remaining Python processes...
    taskkill /F /FI "WINDOWTITLE eq python*" /T >nul 2>&1
)
