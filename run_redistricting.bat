@echo off
REM Wrapper for run_complete_redistricting.py
REM Supports Ctrl+C cancellation

REM NOTE: For best viewing, resize your terminal to at least 60 lines tall and 120 columns wide
REM This allows you to see all 50 state progress bars plus post-processing output

REM Pass all arguments to the Python script
python scripts/pipeline/run_complete_redistricting.py %*

REM If Ctrl+C was pressed, ensure Python processes are killed
if errorlevel 1 (
    echo.
    echo Cleaning up any remaining Python processes...
    taskkill /F /FI "WINDOWTITLE eq python*" /T >nul 2>&1
)
