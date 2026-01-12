@echo off
REM Quick parallel redistricting with default settings
REM Usage: run_parallel.bat [--workers N] [--version NAME] [additional args...]
REM Examples:
REM   run_parallel.bat
REM   run_parallel.bat --workers 8
REM   run_parallel.bat --version v2 --workers 6

REM NOTE: For best viewing, resize your terminal to at least 60 lines tall and 120 columns wide
REM This allows you to see all 50 state progress bars plus post-processing output

python scripts/pipeline/run_complete_redistricting.py --workers 4 %*

REM Cleanup on exit/cancel
if errorlevel 1 (
    echo.
    echo Cleaning up...
    timeout /t 2 /nobreak >nul
)
