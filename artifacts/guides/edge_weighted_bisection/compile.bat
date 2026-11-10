@echo off
REM Compile Layman's Guide: Edge-Weighted Recursive Bisection

cd /d %~dp0

REM Parse command-line arguments
set RESET_FLAG=0
set SKIP_FIGURES=0
set SKIP_IMAGES=0
set YEAR=2010
set VERSION=v1

:parse_args
if "%~1"=="" goto end_parse
if /i "%~1"=="--reset" (
    set RESET_FLAG=1
    shift
    goto parse_args
)
if /i "%~1"=="--skip-figures" (
    set SKIP_FIGURES=1
    shift
    goto parse_args
)
if /i "%~1"=="--skip-images" (
    set SKIP_IMAGES=1
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
set OUTPUT_DIR=..\..\..\outputs\artifacts\guides\edge_weighted_bisection

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

echo ======================================================================
echo Compiling Layman's Guide: Edge-Weighted Recursive Bisection
echo ======================================================================
echo.
echo Output directory: %OUTPUT_DIR%
echo Year: %YEAR%
echo Version: %VERSION%
if %SKIP_FIGURES%==1 echo Skip shared figures: YES
if %SKIP_IMAGES%==1 echo Skip local images: YES
echo.

REM ======================================================================
REM Generate Shared Figures
REM ======================================================================
if %SKIP_FIGURES%==1 goto skip_shared_figures

echo [1/3] Generating shared figures (main text)...
echo ----------------------------------------------------------------------

python ..\..\..\scripts\figures\generate_all_figures.py --year %YEAR% --version %VERSION%
if errorlevel 1 (
    echo [WARNING] Some shared figures could not be generated
    echo Continuing with guide compilation...
)

echo [OK] Shared figures ready
echo.

:skip_shared_figures

REM ======================================================================
REM Generate Guide-Specific Images
REM ======================================================================
if %SKIP_IMAGES%==1 goto skip_local_images

echo [2/3] Generating appendix examples (guide-specific)...
echo ----------------------------------------------------------------------

python create_appendix_examples.py --year %YEAR%
if errorlevel 1 (
    echo [WARNING] Some appendix examples could not be generated
    echo Continuing with guide compilation...
)

echo [OK] Appendix examples ready
echo.

:skip_local_images

:compile_docs
REM ======================================================================
REM Compile Layman's Guide
REM ======================================================================
echo [3/3] Compiling layman's guide (laymen_guide.tex)...
echo ----------------------------------------------------------------------

echo LaTeX pass 1/2...
pdflatex -interaction=nonstopmode laymen_guide.tex >nul 2>&1
if errorlevel 1 (
    echo [ERROR] First LaTeX pass failed for laymen_guide.tex
    pause
    exit /b 1
)

echo LaTeX pass 2/2 (for TOC)...
pdflatex -interaction=nonstopmode laymen_guide.tex >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Second LaTeX pass failed for laymen_guide.tex
    pause
    exit /b 1
)

if exist laymen_guide.pdf (
    echo [OK] laymen_guide.pdf created
    copy /Y laymen_guide.pdf "%OUTPUT_DIR%\laymen_guide.pdf" >nul
    echo [OK] Copied to %OUTPUT_DIR%\laymen_guide.pdf
) else (
    echo [ERROR] laymen_guide.pdf not created
    pause
    exit /b 1
)

echo.

REM ======================================================================
REM Clean up auxiliary files
REM ======================================================================
echo Cleaning auxiliary files...
del /Q *.aux *.log *.nav *.out *.snm *.toc *.vrb 2>nul

echo.
echo ======================================================================
echo Compilation complete!
echo ======================================================================
echo Created:
echo   - %OUTPUT_DIR%\laymen_guide.pdf
echo ======================================================================
echo.
