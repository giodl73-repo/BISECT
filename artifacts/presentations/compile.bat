@echo off
REM Compile All Presentations

cd /d %~dp0

REM Parse command-line arguments
set ARGS=%*

REM Extract year and version for display (optional, for clarity)
set YEAR=2020
set VERSION=v1
set "TEMP_ARGS=%*"
:parse_display
if "%~1"=="" goto end_parse_display
if /i "%~1"=="--year" set YEAR=%~2
if /i "%~1"=="--version" set VERSION=%~2
shift
goto parse_display
:end_parse_display

echo ======================================================================
echo Compiling All Presentation Materials
echo ======================================================================
echo This will compile all presentations and output to outputs\presentations\
echo Year: %YEAR%
echo Version: %VERSION%
echo.

REM ======================================================================
REM Edge-Weighted Recursive Bisection Presentation
REM ======================================================================
echo [1/1] Compiling Edge-Weighted Recursive Bisection Materials
echo ----------------------------------------------------------------------
cd edge_weighted_bisection
call compile.bat %ARGS%
if errorlevel 1 (
    echo [ERROR] Presentation compilation failed
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
echo All Presentations Compiled Successfully!
echo ======================================================================
echo Output files:
echo   - outputs\presentations\edge_weighted_bisection\presentation.pdf
echo   - outputs\presentations\edge_weighted_bisection\laymen_guide.pdf
echo ======================================================================
echo.

pause
