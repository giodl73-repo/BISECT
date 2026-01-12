@echo off
REM ==============================================================================
REM Re-download incomplete 2010 Census states
REM ==============================================================================
REM
REM This script will re-download 38 states with incomplete 2010 census data.
REM
REM Settings:
REM   - 3.0s delay between API requests (avoid rate limiting)
REM   - 10.0s delay between states
REM   - 5 retries for failed requests
REM   - Resume from partial downloads
REM
REM Estimated time: 36-48 hours (depends on rate limiting)
REM
REM You can run this overnight or in batches.
REM ==============================================================================

echo.
echo ==============================================================================
echo Re-downloading 2010 Census Data - 38 States
echo ==============================================================================
echo.
echo This will take approximately 36-48 hours to complete.
echo You can press Ctrl+C to cancel at any time and resume later.
echo.
pause

REM Run the download script
python scripts\redownload_2010_states.py

REM After completion, validate the results
echo.
echo ==============================================================================
echo Running validation to check results...
echo ==============================================================================
python scripts\validate_2010_census_data.py

echo.
echo ==============================================================================
echo DONE!
echo ==============================================================================
pause
