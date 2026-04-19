@echo off
REM Reorganize data directory for Enhancement 47
REM This script moves NHGIS data to Census 2000/ and creates consistent structure

echo ========================================
echo Data Directory Reorganization
echo Enhancement 47: Data Separation
echo ========================================
echo.

REM Create Census 2000 subdirectories
echo Creating Census 2000 subdirectories...
mkdir "data\Census 2000\tracts" 2>nul
mkdir "data\Census 2000\places" 2>nul
mkdir "data\Census 2000\population" 2>nul
mkdir "data\Census 2000\baseline" 2>nul
mkdir "data\Census 2000\demographics" 2>nul
mkdir "data\Census 2000\elections" 2>nul

REM Move NHGIS tract data
echo.
echo Moving NHGIS tract shapefiles to Census 2000\tracts...
move "data\NHGIS\nhgis0001_shapefile_tl2000_us_tract_2000\*" "data\Census 2000\tracts\" >nul 2>&1
if %errorlevel% equ 0 (
    echo [OK] Tract shapefiles moved
    rmdir "data\NHGIS\nhgis0001_shapefile_tl2000_us_tract_2000" 2>nul
) else (
    echo [WARN] Tract shapefiles already moved or not found
)

REM Move NHGIS place data
echo Moving NHGIS place shapefiles to Census 2000\places...
move "data\NHGIS\nhgis0004_shapefile_tl2000_us_place_2000\*" "data\Census 2000\places\" >nul 2>&1
if %errorlevel% equ 0 (
    echo [OK] Place shapefiles moved
    rmdir "data\NHGIS\nhgis0004_shapefile_tl2000_us_place_2000" 2>nul
) else (
    echo [WARN] Place shapefiles already moved or not found
)

REM Move NHGIS population CSVs
echo Moving NHGIS population CSVs to Census 2000\population...
move "data\NHGIS\nhgis0006_csv\*" "data\Census 2000\population\" >nul 2>&1
if %errorlevel% equ 0 (
    echo [OK] Population CSVs moved
    rmdir "data\NHGIS\nhgis0006_csv" 2>nul
) else (
    echo [WARN] Population CSVs already moved or not found
)

REM Move NHGIS baseline congressional districts
echo Moving NHGIS baseline congressional districts to Census 2000\baseline...
move "data\NHGIS\nhgis0003_shapefile_tl2010_us_cd108th_2000\*" "data\Census 2000\baseline\" >nul 2>&1
if %errorlevel% equ 0 (
    echo [OK] Baseline congressional districts moved
    rmdir "data\NHGIS\nhgis0003_shapefile_tl2010_us_cd108th_2000" 2>nul
) else (
    echo [WARN] Baseline districts already moved or not found
)

REM Move NHGIS README if it exists
if exist "data\NHGIS\README.md" (
    echo Moving NHGIS README.md...
    move "data\NHGIS\README.md" "data\Census 2000\README_NHGIS.md" >nul 2>&1
    echo [OK] README moved
)

REM Remove empty NHGIS directory
echo.
echo Removing empty NHGIS directory...
rmdir "data\NHGIS" 2>nul
if %errorlevel% equ 0 (
    echo [OK] NHGIS directory removed
) else (
    echo [INFO] NHGIS directory not empty or already removed
)

REM Create Census 2010 subdirectories
echo.
echo Creating Census 2010 subdirectories...
mkdir "data\Census 2010\demographics" 2>nul
mkdir "data\Census 2010\elections" 2>nul
echo [OK] Census 2010 subdirectories created

REM Create Census 2020 subdirectories
echo Creating Census 2020 subdirectories...
mkdir "data\Census 2020\demographics" 2>nul
mkdir "data\Census 2020\elections" 2>nul
echo [OK] Census 2020 subdirectories created

REM List final structure
echo.
echo ========================================
echo Final Structure:
echo ========================================
echo.
tree /F "data" /A
echo.
echo ========================================
echo Reorganization Complete!
echo ========================================
echo.
echo Next steps:
echo 1. Review the new structure above
echo 2. Check that all files moved correctly
echo 3. Run: py -3.13 scripts/data/build_all_processed_data.py --year 2020 --states VT
echo.

pause
