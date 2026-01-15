@echo off
REM Compile All Papers

cd /d %~dp0

echo ======================================================================
echo Compiling All Papers
echo ======================================================================
echo This will compile all three papers and output to outputs\papers\
echo.

REM ======================================================================
REM Paper 1: Baseline Recursive Bisection
REM ======================================================================
echo [1/3] Compiling Paper 1: Baseline Recursive Bisection
echo ----------------------------------------------------------------------
cd 01_recursive_bisection
call compile.bat
if errorlevel 1 (
    echo [ERROR] Paper 1 compilation failed
    cd ..
    pause
    exit /b 1
)
cd ..
echo.

REM ======================================================================
REM Paper 2: Edge-Weighted Bisection
REM ======================================================================
echo [2/3] Compiling Paper 2: Edge-Weighted Bisection
echo ----------------------------------------------------------------------
cd 02_edge_weighted_bisection
call compile.bat
if errorlevel 1 (
    echo [ERROR] Paper 2 compilation failed
    cd ..
    pause
    exit /b 1
)
cd ..
echo.

REM ======================================================================
REM Paper 3: Combined Recursive Bisection with Edge-Weighted Cuts
REM ======================================================================
echo [3/3] Compiling Paper 3: Combined Recursive Bisection
echo ----------------------------------------------------------------------
cd 03_combined_recursive_bisection
call compile.bat
if errorlevel 1 (
    echo [ERROR] Paper 3 compilation failed
    cd ..
    pause
    exit /b 1
)
cd ..
echo.

REM ======================================================================
REM Summary
REM ======================================================================
echo ======================================================================
echo All Papers Compiled Successfully!
echo ======================================================================
echo Output files:
echo   - outputs\papers\01_recursive_bisection\recursive_bisection.pdf
echo   - outputs\papers\02_edge_weighted_bisection\edge_weighted_bisection.pdf
echo   - outputs\papers\03_combined_recursive_bisection\recursive_bisection_with_edge_weighted_cuts.pdf
echo ======================================================================
echo.

pause
