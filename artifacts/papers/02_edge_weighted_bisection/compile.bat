@echo off
REM Compile Paper 2: Edge-Weighted Bisection

cd /d %~dp0

REM Parse command-line arguments
set RESET_FLAG=0
:parse_args
if "%~1"=="" goto end_parse
if /i "%~1"=="--reset" set RESET_FLAG=1
shift
goto parse_args
:end_parse

REM Create output directory
set OUTPUT_DIR=..\..\..\outputs\artifacts\papers\02_edge_weighted_bisection

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

echo Compiling Paper 2: Edge-Weighted Bisection
echo ==================================================
echo Output: %OUTPUT_DIR%
echo.

REM First pass
echo LaTeX pass 1/3...
pdflatex -interaction=nonstopmode edge_weighted_bisection.tex >nul 2>&1

REM BibTeX
echo BibTeX...
bibtex edge_weighted_bisection >nul 2>&1

REM Second pass
echo LaTeX pass 2/3...
pdflatex -interaction=nonstopmode edge_weighted_bisection.tex >nul 2>&1

REM Third pass
echo LaTeX pass 3/3...
pdflatex -interaction=nonstopmode edge_weighted_bisection.tex >nul 2>&1

echo.

if exist edge_weighted_bisection.pdf (
    echo [OK] edge_weighted_bisection.pdf created
    copy /Y edge_weighted_bisection.pdf "%OUTPUT_DIR%\edge_weighted_bisection.pdf" >nul
    echo [OK] Copied to %OUTPUT_DIR%\edge_weighted_bisection.pdf
) else (
    echo [ERROR] edge_weighted_bisection.pdf not created
    exit /b 1
)

echo.

REM Clean up auxiliary files
echo Cleaning auxiliary files...
del /Q *.aux *.log *.bbl *.blg *.out *.toc 2>nul

echo Done!
echo.
