@echo off
REM Compile Paper 3: Recursive Bisection with Edge-Weighted Cuts

cd /d %~dp0

REM Parse command-line arguments
set RESET_FLAG=0
set YEAR=2020
set VERSION=v1

:parse_args
if "%~1"=="" goto end_parse
if /i "%~1"=="--reset" (
    set RESET_FLAG=1
    shift
    goto parse_args
)
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
REM Unknown argument, skip it
shift
goto parse_args
:end_parse

REM Create output directory
set OUTPUT_DIR=..\..\..\outputs\artifacts\papers\03_combined_recursive_bisection

REM Handle --reset flag
if %RESET_FLAG%==1 (
    if exist "%OUTPUT_DIR%" (
        echo [RESET] Clearing existing output directory: %OUTPUT_DIR%
        rmdir /s /q "%OUTPUT_DIR%"
        echo [OK] Output directory cleared
        echo.
    )
)

if not exist "%OUTPUT_DIR%" mkdir "%OUTPUT_DIR%"

echo Compiling Paper 3: Recursive Bisection with Edge-Weighted Cuts
echo ==================================================================
echo Output: %OUTPUT_DIR%
echo Year: %YEAR%
echo Version: %VERSION%
echo.

REM ======================================================================
REM Generate Figures
REM ======================================================================
echo [1/2] Generating paper figures...
echo ----------------------------------------------------------------------

python create_figures.py --year %YEAR% --version %VERSION%
if errorlevel 1 (
    echo [ERROR] Figure generation failed
    pause
    exit /b 1
)

echo [OK] Figures generated
echo.

REM ======================================================================
REM Compile Paper
REM ======================================================================
echo [2/2] Compiling paper (recursive_bisection_with_edge_weighted_cuts.tex)...
echo ----------------------------------------------------------------------

REM First pass
echo LaTeX pass 1/3...
pdflatex -interaction=nonstopmode recursive_bisection_with_edge_weighted_cuts.tex >nul 2>&1

REM BibTeX
echo BibTeX...
bibtex recursive_bisection_with_edge_weighted_cuts >nul 2>&1

REM Second pass
echo LaTeX pass 2/3...
pdflatex -interaction=nonstopmode recursive_bisection_with_edge_weighted_cuts.tex >nul 2>&1

REM Third pass
echo LaTeX pass 3/3...
pdflatex -interaction=nonstopmode recursive_bisection_with_edge_weighted_cuts.tex >nul 2>&1

echo.

if exist recursive_bisection_with_edge_weighted_cuts.pdf (
    echo [OK] recursive_bisection_with_edge_weighted_cuts.pdf created
    copy /Y recursive_bisection_with_edge_weighted_cuts.pdf "%OUTPUT_DIR%\recursive_bisection_with_edge_weighted_cuts.pdf" >nul
    echo [OK] Copied to %OUTPUT_DIR%\recursive_bisection_with_edge_weighted_cuts.pdf
) else (
    echo [ERROR] recursive_bisection_with_edge_weighted_cuts.pdf not created
    exit /b 1
)

echo.

REM Clean up auxiliary files
echo Cleaning auxiliary files...
del /Q *.aux *.log *.bbl *.blg *.out *.toc 2>nul

echo Done!
echo.
