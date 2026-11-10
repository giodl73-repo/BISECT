@echo off
REM Compile Presentation Materials: Edge-Weighted Recursive Bisection

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
set OUTPUT_DIR=..\..\..\outputs\artifacts\presentations\edge_weighted_bisection

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
echo Compiling Edge-Weighted Recursive Bisection Presentation Materials
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

echo [1/6] Generating shared figures (main text)...
echo ----------------------------------------------------------------------

python ..\..\..\scripts\figures\generate_all_figures.py --year %YEAR% --version %VERSION%
if errorlevel 1 (
    echo [WARNING] Some shared figures could not be generated
    echo Continuing with presentation compilation...
)

echo [OK] Shared figures ready
echo.

:skip_shared_figures

REM ======================================================================
REM Generate Presentation-Specific Images
REM ======================================================================
if %SKIP_IMAGES%==1 goto skip_local_images

echo [2/6] Generating appendix examples (presentation-specific)...
echo ----------------------------------------------------------------------

python create_appendix_examples.py --year %YEAR%
if errorlevel 1 (
    echo [WARNING] Some appendix examples could not be generated
    echo Continuing with presentation compilation...
)

echo [OK] Appendix examples ready
echo.

REM ======================================================================
REM Copy Metro Maps from Pipeline Outputs
REM ======================================================================
echo [3/6] Copying metro maps (Minneapolis)...
echo ----------------------------------------------------------------------

REM Create metro_maps directory in output
set METRO_DIR=%OUTPUT_DIR%\metro_maps
if not exist "%METRO_DIR%" mkdir "%METRO_DIR%"

REM Copy Minneapolis metro map from pipeline outputs
set SOURCE_METRO=..\..\..\outputs\us_%YEAR%_%VERSION%\states\minnesota\maps\metros\minneapolis.png
if exist "%SOURCE_METRO%" (
    copy /Y "%SOURCE_METRO%" "%METRO_DIR%\minneapolis.png" >nul
    echo [OK] Minneapolis metro map copied
) else (
    echo [WARNING] Minneapolis metro map not found at %SOURCE_METRO%
)

REM Copy national map from pipeline outputs
set MAPS_DIR=%OUTPUT_DIR%\maps
if not exist "%MAPS_DIR%" mkdir "%MAPS_DIR%"

set SOURCE_NATIONAL=..\..\..\outputs\us_%YEAR%_%VERSION%\maps\us_all_districts.png
if exist "%SOURCE_NATIONAL%" (
    copy /Y "%SOURCE_NATIONAL%" "%MAPS_DIR%\us_all_districts.png" >nul
    echo [OK] National districts map copied
) else (
    echo [WARNING] National districts map not found at %SOURCE_NATIONAL%
)

echo.

:skip_local_images

:compile_docs
REM ======================================================================
REM Compile Presentation Slides
REM ======================================================================
echo [2/3] Compiling presentation slides (presentation.tex)...
echo ----------------------------------------------------------------------

echo LaTeX pass 1/2...
pdflatex -interaction=nonstopmode presentation.tex >nul 2>&1
if errorlevel 1 (
    echo [ERROR] First LaTeX pass failed for presentation.tex
    pause
    exit /b 1
)

echo LaTeX pass 2/2 (for navigation)...
pdflatex -interaction=nonstopmode presentation.tex >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Second LaTeX pass failed for presentation.tex
    pause
    exit /b 1
)

if exist presentation.pdf (
    echo [OK] presentation.pdf created
    copy /Y presentation.pdf "%OUTPUT_DIR%\presentation.pdf" >nul
    echo [OK] Copied to %OUTPUT_DIR%\presentation.pdf
) else (
    echo [ERROR] presentation.pdf not created
    pause
    exit /b 1
)

echo.

REM ======================================================================
REM Clean up auxiliary files
REM ======================================================================
echo [3/3] Cleaning auxiliary files...
del /Q *.aux *.log *.nav *.out *.snm *.toc *.vrb 2>nul

echo.
echo ======================================================================
echo Compilation complete!
echo ======================================================================
echo Created:
echo   - %OUTPUT_DIR%\presentation.pdf
echo ======================================================================
echo.
