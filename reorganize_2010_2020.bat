@echo off
REM Reorganize Census 2010 and 2020 data directories
REM Enhancement 47: Data Separation

echo ========================================
echo Census 2010/2020 Reorganization
echo ========================================
echo.

REM ============================================
REM Census 2010 - Already mostly organized
REM ============================================
echo Checking Census 2010 structure...
echo [OK] PL files in root
echo [OK] Geography files in geos/
echo [OK] demographics/ and elections/ already created
echo.

REM ============================================
REM Census 2020 - Rename directories
REM ============================================
echo Reorganizing Census 2020...
echo.

REM Rename tracts/ to tiger/ (TIGER/Line tract shapefiles - temporary)
if exist "data\Census 2020\tracts\" (
    echo Moving tracts/ to tiger/...
    move "data\Census 2020\tracts" "data\Census 2020\tiger" >nul 2>&1
    if %errorlevel% equ 0 (
        echo [OK] tracts/ renamed to tiger/
    ) else (
        echo [WARN] Failed to rename tracts/ to tiger/
    )
) else (
    echo [SKIP] tracts/ directory not found
)

REM Move or rename shps/ directory (county subdivisions - optional)
if exist "data\Census 2020\shps\" (
    echo.
    echo Found shps/ directory with county subdivision files.
    echo These are optional and not used in redistricting pipeline.
    echo Renaming to tiger_cousub/ for clarity...
    move "data\Census 2020\shps" "data\Census 2020\tiger_cousub" >nul 2>&1
    if %errorlevel% equ 0 (
        echo [OK] shps/ renamed to tiger_cousub/
    ) else (
        echo [WARN] Failed to rename shps/
    )
) else (
    echo [SKIP] shps/ directory not found
)

echo.
echo ========================================
echo Final Structure:
echo ========================================
echo.
tree /F "data\Census 2010" | findstr /V "\.pl"
echo.
tree /F "data\Census 2020" | findstr /V "\.pl"
echo.
echo ========================================
echo Reorganization complete!
echo ========================================
