@echo off
REM Build METIS on Windows using Visual Studio and CMake

echo ============================================================
echo METIS Windows Build Script
echo ============================================================
echo.

set METIS_SRC=C:\Users\giodl\sources\repo\Apportionment\tools\METIS-5.2.1
set METIS_BUILD=%METIS_SRC%\build\windows
set METIS_INSTALL=C:\metis

if not exist "%METIS_SRC%" (
    echo ERROR: METIS source not found at %METIS_SRC%
    echo Please update the METIS_SRC variable in this script.
    exit /b 1
)

echo METIS Source: %METIS_SRC%
echo Build Directory: %METIS_BUILD%
echo Install Directory: %METIS_INSTALL%
echo.

REM Create build directory
if not exist "%METIS_BUILD%" mkdir "%METIS_BUILD%"

REM Configure with CMake
echo Configuring METIS with CMake...
cd "%METIS_BUILD%"
cmake ..\.. ^
    -G "Visual Studio 17 2022" ^
    -A x64 ^
    -DCMAKE_INSTALL_PREFIX="%METIS_INSTALL%" ^
    -DGKLIB_PATH="%METIS_SRC%\GKlib" ^
    -DSHARED=ON

if errorlevel 1 (
    echo ERROR: CMake configuration failed
    echo.
    echo Troubleshooting:
    echo   1. Make sure Visual Studio 2022 is installed
    echo   2. Try "Visual Studio 16 2019" if you have VS 2019
    echo   3. Make sure GKlib is extracted in the METIS directory
    exit /b 1
)

echo.
echo Building METIS (Release mode)...
cmake --build . --config Release --target ALL_BUILD

if errorlevel 1 (
    echo ERROR: Build failed
    exit /b 1
)

echo.
echo Installing METIS to %METIS_INSTALL%...
cmake --build . --config Release --target INSTALL

if errorlevel 1 (
    echo ERROR: Installation failed
    exit /b 1
)

echo.
echo ============================================================
echo METIS Build Complete!
echo ============================================================
echo.
echo Binaries installed to: %METIS_INSTALL%\bin
echo   - gpmetis.exe  (graph partitioning)
echo   - mpmetis.exe  (mesh partitioning)
echo   - m2gmetis.exe (mesh to graph conversion)
echo   - graphchk.exe (graph checker)
echo.
echo Library: %METIS_INSTALL%\lib\metis.lib
echo Headers: %METIS_INSTALL%\include\metis.h
echo.
echo Next steps:
echo   1. Add %METIS_INSTALL%\bin to your PATH
echo   2. Set environment variable: METIS_DLL=%METIS_INSTALL%\lib\metis.dll
echo   3. Try: py -3.13 -m pip install pymetis
echo.
pause
