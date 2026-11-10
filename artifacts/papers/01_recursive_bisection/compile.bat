@echo off
REM Compile Paper 1: Baseline Recursive Bisection

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
set OUTPUT_DIR=..\..\..\outputs\artifacts\papers\01_recursive_bisection

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

echo Compiling Paper 1: Baseline Recursive Bisection
echo ==================================================
echo Output: %OUTPUT_DIR%
echo.

REM First pass
echo LaTeX pass 1/3...
pdflatex -interaction=nonstopmode recursive_bisection.tex >nul 2>&1

REM BibTeX
echo BibTeX...
bibtex recursive_bisection >nul 2>&1

REM Second pass
echo LaTeX pass 2/3...
pdflatex -interaction=nonstopmode recursive_bisection.tex >nul 2>&1

REM Third pass
echo LaTeX pass 3/3...
pdflatex -interaction=nonstopmode recursive_bisection.tex >nul 2>&1

echo.

if exist recursive_bisection.pdf (
    echo [OK] recursive_bisection.pdf created
    copy /Y recursive_bisection.pdf "%OUTPUT_DIR%\recursive_bisection.pdf" >nul
    echo [OK] Copied to %OUTPUT_DIR%\recursive_bisection.pdf
) else (
    echo [ERROR] recursive_bisection.pdf not created
    exit /b 1
)

echo.

REM Clean up auxiliary files
echo Cleaning auxiliary files...
del /Q *.aux *.log *.bbl *.blg *.out *.toc 2>nul

echo Done!
echo.
