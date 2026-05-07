@echo off
REM Reorganize census data structure
REM Moves PL 94-171 files into redistricting/ subdirectory for consistency

echo ======================================================================
echo REORGANIZING CENSUS DATA STRUCTURE
echo ======================================================================
echo.
echo This script will move PL 94-171 files into redistricting/ subdirectories:
echo   - 2020: Move {state}2020.pl/ to redistricting/{state}2020.pl/
echo   - 2010: Move {state}2010.pl/ to redistricting/{state}2010.pl/
echo   - 2000: Rename geos/ to redistricting/
echo.
echo Press Ctrl+C to cancel, or
pause

echo.
echo [1/3] Organizing 2020...
if not exist "data\2020\redistricting" mkdir "data\2020\redistricting"
for /d %%d in ("data\2020\*2020.pl") do (
    echo   Moving %%~nxd...
    move "%%d" "data\2020\redistricting\" >nul 2>&1
)

echo.
echo [2/3] Organizing 2010...
if not exist "data\2010\redistricting" mkdir "data\2010\redistricting"
for /d %%d in ("data\2010\*2010.pl") do (
    echo   Moving %%~nxd...
    move "%%d" "data\2010\redistricting\" >nul 2>&1
)

echo.
echo [3/3] Organizing 2000...
if exist "data\2000\geos" (
    echo   Renaming geos/ to redistricting/...
    if exist "data\2000\redistricting" (
        echo   WARNING: redistricting/ already exists, skipping
    ) else (
        move "data\2000\geos" "data\2000\redistricting" >nul 2>&1
    )
)

echo.
echo ======================================================================
echo DONE!
echo ======================================================================
echo.
echo New structure:
echo   data/2020/redistricting/{state}2020.pl/
echo   data/2010/redistricting/{state}2010.pl/
echo   data/2000/redistricting/{state}geo.upl
echo.
echo You can now run census data processing with the updated scripts.
echo.
pause
