@echo off
REM Sequential redistricting (one state at a time)
REM Usage: run_sequential.bat [--version NAME] [additional args...]
REM Examples:
REM   run_sequential.bat
REM   run_sequential.bat --version v2

REM NOTE: For best viewing, resize your terminal to at least 60 lines tall and 120 columns wide
REM This allows you to see all 50 state progress bars plus post-processing output

python scripts/pipeline/run_complete_redistricting.py --workers 1 %*
