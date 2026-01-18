@echo off
REM Master Compilation Script for All Research Artifacts
REM
REM This script compiles figures, presentations, and papers in the correct order,
REM ensuring all dependencies are met.

REM Change to project root (one level up from batch/)
cd /d "%~dp0\.."

REM Parse command-line arguments
set YEAR=2020
set VERSION=v1
set SKIP_FIGURES=0
set SKIP_PRESENTATIONS=0
set SKIP_PAPERS=0

:parse_args
if "%~1"=="" goto end_parse
if /i "%~1"=="--year" (
    set YEAR=%~2
    shift
    shift
    goto parse_args
)
if /i "%~1"=="--version" (
    set VERSION=%~2
    shift
    shift
    goto parse_args
)
if /i "%~1"=="--skip-figures" (
    set SKIP_FIGURES=1
    shift
    goto parse_args
)
if /i "%~1"=="--skip-presentations" (
    set SKIP_PRESENTATIONS=1
    shift
    goto parse_args
)
if /i "%~1"=="--skip-papers" (
    set SKIP_PAPERS=1
    shift
    goto parse_args
)
REM Unknown argument, skip it
shift
goto parse_args
:end_parse

echo ======================================================================
echo Master Research Artifacts Compilation
echo ======================================================================
echo This script compiles all research outputs in dependency order:
echo   1. Shared Figures
echo   2. Presentations
echo   3. Papers
echo.
echo Parameters:
echo   Year: %YEAR%
echo   Version: %VERSION%
echo.
echo Options:
echo   Skip Figures: %SKIP_FIGURES%
echo   Skip Presentations: %SKIP_PRESENTATIONS%
echo   Skip Papers: %SKIP_PAPERS%
echo.
pause
echo.

REM ======================================================================
REM Step 1: Generate All Shared Figures
REM ======================================================================
if %SKIP_FIGURES%==1 (
    echo [1/3] Shared Figures - SKIPPED
    echo.
) else (
    echo [1/3] Generating All Shared Figures
    echo ======================================================================
    echo.

    python scripts\figures\generate_all_figures.py --year %YEAR% --version %VERSION%
    if errorlevel 1 (
        echo.
        echo [WARNING] Some figures could not be generated
        echo Continuing with remaining compilation steps...
        pause
    )

    echo.
    echo [OK] Shared figures ready
    echo.
)

REM ======================================================================
REM Step 2: Compile Presentations
REM ======================================================================
if %SKIP_PRESENTATIONS%==1 (
    echo [2/3] Presentations - SKIPPED
    echo.
) else (
    echo [2/3] Compiling Presentations
    echo ======================================================================
    echo.

    REM Edge-Weighted Bisection Presentation
    echo [2a/3] Edge-Weighted Bisection Presentation...
    cd presentations\edge_weighted_bisection
    call compile.bat --year %YEAR% --version %VERSION%
    if errorlevel 1 (
        echo [ERROR] Presentation compilation failed
        cd ..\..
        pause
        exit /b 1
    )
    cd ..\..

    echo.
    echo [OK] Presentations compiled
    echo.
)

REM ======================================================================
REM Step 3: Compile Papers
REM ======================================================================
if %SKIP_PAPERS%==1 (
    echo [3/3] Papers - SKIPPED
    echo.
) else (
    echo [3/3] Compiling Papers
    echo ======================================================================
    echo.

    cd papers
    call compile.bat --year %YEAR% --version %VERSION%
    if errorlevel 1 (
        echo [ERROR] Paper compilation failed
        cd ..
        pause
        exit /b 1
    )
    cd ..

    echo.
    echo [OK] Papers compiled
    echo.
)

REM ======================================================================
REM Summary
REM ======================================================================
echo ======================================================================
echo Master Compilation Complete!
echo ======================================================================
echo.
echo Generated Artifacts:
if %SKIP_FIGURES%==0 (
    echo   Figures:
    echo     - outputs\figures\schematic\*.png
    echo     - outputs\figures\real_tracts_examples\*.png
    echo     - outputs\figures\round_progression\*.png
    echo.
)
if %SKIP_PRESENTATIONS%==0 (
    echo   Presentations:
    echo     - outputs\presentations\edge_weighted_bisection\presentation.pdf
    echo     - outputs\presentations\edge_weighted_bisection\laymen_guide.pdf
    echo.
)
if %SKIP_PAPERS%==0 (
    echo   Papers:
    echo     - outputs\papers\01_recursive_bisection\recursive_bisection.pdf
    echo     - outputs\papers\02_edge_weighted_bisection\edge_weighted_bisection.pdf
    echo     - outputs\papers\03_combined_recursive_bisection\recursive_bisection_with_edge_weighted_cuts.pdf
    echo.
)
echo ======================================================================
echo.

pause
