@echo off
REM Master Compile Script for All Artifacts

cd /d %~dp0

REM Parse command-line arguments
set TARGET=all
set RESET_FLAG=0
set SKIP_FIGURES=0
set SKIP_IMAGES=0
set YEAR=2010
set VERSION=v1

:parse_args
if "%~1"=="" goto end_parse
if /i "%~1"=="--target" (
    set TARGET=%~2
    shift
    shift
    goto parse_args
)
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

echo ======================================================================
echo Master Artifact Compiler
echo ======================================================================
echo.
echo Target: %TARGET%
echo Year: %YEAR%
echo Version: %VERSION%
if %RESET_FLAG%==1 echo Reset: YES
if %SKIP_FIGURES%==1 echo Skip shared figures: YES
if %SKIP_IMAGES%==1 echo Skip local images: YES
echo.

REM Build arguments for child scripts
set CHILD_ARGS=--year %YEAR% --version %VERSION%
if %RESET_FLAG%==1 set CHILD_ARGS=%CHILD_ARGS% --reset
if %SKIP_FIGURES%==1 set CHILD_ARGS=%CHILD_ARGS% --skip-figures
if %SKIP_IMAGES%==1 set CHILD_ARGS=%CHILD_ARGS% --skip-images

REM ======================================================================
REM Generate Shared Figures
REM ======================================================================
if %SKIP_FIGURES%==1 goto skip_figures
if /i "%TARGET%"=="all" goto generate_figures
if /i "%TARGET%"=="figures" goto generate_figures
goto skip_figures

:generate_figures
echo ======================================================================
echo [1/4] Generating Shared Figures
echo ======================================================================
echo.

echo Generating: Shared figures for all artifacts...
python ..\scripts\figures\generate_all_figures.py --year %YEAR% --version %VERSION%
if errorlevel 1 (
    echo [ERROR] Figure generation failed
    pause
    exit /b 1
)
echo [OK] Figures generated successfully
echo.

REM Mark that figures have been generated so child scripts skip regeneration
set SKIP_FIGURES=1
set CHILD_ARGS=--year %YEAR% --version %VERSION%
if %RESET_FLAG%==1 set CHILD_ARGS=%CHILD_ARGS% --reset
set CHILD_ARGS=%CHILD_ARGS% --skip-figures
if %SKIP_IMAGES%==1 set CHILD_ARGS=%CHILD_ARGS% --skip-images

:skip_figures

REM ======================================================================
REM Compile Guides
REM ======================================================================
if /i "%TARGET%"=="all" goto compile_guides
if /i "%TARGET%"=="guides" goto compile_guides
if /i "%TARGET%"=="guide" goto compile_guides
goto skip_guides

:compile_guides
echo ======================================================================
echo [2/4] Compiling Guides
echo ======================================================================
echo.

echo Compiling: Edge-Weighted Bisection Guide...
cd guides\edge_weighted_bisection
call compile.bat %CHILD_ARGS%
if errorlevel 1 (
    echo [ERROR] Guide compilation failed
    cd ..\..
    pause
    exit /b 1
)
cd ..\..
echo [OK] Guide compiled successfully
echo.

:skip_guides

REM ======================================================================
REM Compile Presentations
REM ======================================================================
if /i "%TARGET%"=="all" goto compile_presentations
if /i "%TARGET%"=="presentations" goto compile_presentations
if /i "%TARGET%"=="presentation" goto compile_presentations
goto skip_presentations

:compile_presentations
echo ======================================================================
echo [3/4] Compiling Presentations
echo ======================================================================
echo.

echo Compiling: Edge-Weighted Bisection Presentation...
cd presentations\edge_weighted_bisection
call compile.bat %CHILD_ARGS%
if errorlevel 1 (
    echo [ERROR] Presentation compilation failed
    cd ..\..
    pause
    exit /b 1
)
cd ..\..
echo [OK] Presentation compiled successfully
echo.

:skip_presentations

REM ======================================================================
REM Compile Papers
REM ======================================================================
if /i "%TARGET%"=="all" goto compile_papers
if /i "%TARGET%"=="papers" goto compile_papers
if /i "%TARGET%"=="paper" goto compile_papers
goto skip_papers

:compile_papers
echo ======================================================================
echo [4/4] Compiling Papers
echo ======================================================================
echo.

echo Compiling: All Papers (via papers/compile.bat)...
cd papers
call compile.bat %CHILD_ARGS%
if errorlevel 1 (
    echo [ERROR] Papers compilation failed
    cd ..
    pause
    exit /b 1
)
cd ..
echo [OK] Papers compiled successfully
echo.

:skip_papers

echo.
echo ======================================================================
echo Master compilation complete!
echo ======================================================================
echo.
echo Outputs available in: outputs\artifacts\
echo.
echo   Shared Figures:
echo   - outputs\figures\schematic\*.png
echo   - outputs\figures\real_tracts_examples\*.png
echo   - outputs\figures\round_progression\*.png
echo.
echo   Guides:
echo   - ..\outputs\artifacts\guides\edge_weighted_bisection\laymen_guide.pdf
echo.
echo   Presentations:
echo   - ..\outputs\artifacts\presentations\edge_weighted_bisection\presentation.pdf
echo.
echo   Papers:
echo   - ..\outputs\artifacts\papers\01_recursive_bisection\recursive_bisection.pdf
echo   - ..\outputs\artifacts\papers\02_edge_weighted_bisection\edge_weighted_bisection.pdf
echo   - ..\outputs\artifacts\papers\03_combined_recursive_bisection\recursive_bisection_with_edge_weighted_cuts.pdf
echo.
echo ======================================================================
echo.
