@echo off
REM ==============================================================================
REM Fix Ohio 2020 Census Data
REM ==============================================================================
REM
REM Ohio 2020 is 99.35% complete (missing 77k out of 11.8M people)
REM This script re-downloads Ohio with proper rate limiting
REM
REM Time: ~30-45 minutes (88 counties)
REM ==============================================================================

echo.
echo ==============================================================================
echo Fixing Ohio 2020 Census Data
echo ==============================================================================
echo.
echo This will take approximately 30-45 minutes.
echo.
pause

python scripts\download_tracts_improved.py --state OH --year 2020

echo.
echo ==============================================================================
echo Validating Ohio 2020 data...
echo ==============================================================================
python scripts\validate_2020_census_data.py

echo.
echo ==============================================================================
echo DONE!
echo ==============================================================================
pause
