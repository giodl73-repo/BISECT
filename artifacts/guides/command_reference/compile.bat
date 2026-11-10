@echo off
REM Compile Command Reference Cheat Sheet

cd /d %~dp0

REM Parse command-line arguments
set RESET_FLAG=0

:parse_args
if "%~1"=="" goto end_parse
if /i "%~1"=="--reset" (
    set RESET_FLAG=1
    shift
    goto parse_args
)
REM Unknown argument, skip it
shift
goto parse_args
:end_parse

REM Create output directory
set OUTPUT_DIR=..\..\..\outputs\artifacts\guides\command_reference

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
echo Compiling Command Reference Cheat Sheets
echo ======================================================================
echo.
echo Output directory: %OUTPUT_DIR%
echo.

REM ======================================================================
REM Compile GitHub Reference
REM ======================================================================
echo [1/2] Compiling GitHub reference (github_reference.tex)...
echo ----------------------------------------------------------------------

pdflatex -interaction=nonstopmode github_reference.tex >nul 2>&1
if errorlevel 1 (
    echo [ERROR] LaTeX compilation failed for github_reference.tex
    echo Running again with output to see errors...
    pdflatex -interaction=nonstopmode github_reference.tex
    pause
    exit /b 1
)

if exist github_reference.pdf (
    echo [OK] github_reference.pdf created
    copy /Y github_reference.pdf "%OUTPUT_DIR%\github_reference.pdf" >nul
    echo [OK] Copied to %OUTPUT_DIR%\github_reference.pdf
) else (
    echo [ERROR] github_reference.pdf not created
    pause
    exit /b 1
)

echo.

REM ======================================================================
REM Compile Redistricting Reference
REM ======================================================================
echo [2/2] Compiling redistricting reference (redistricting_reference.tex)...
echo ----------------------------------------------------------------------

pdflatex -interaction=nonstopmode redistricting_reference.tex >nul 2>&1
if errorlevel 1 (
    echo [ERROR] LaTeX compilation failed for redistricting_reference.tex
    echo Running again with output to see errors...
    pdflatex -interaction=nonstopmode redistricting_reference.tex
    pause
    exit /b 1
)

if exist redistricting_reference.pdf (
    echo [OK] redistricting_reference.pdf created
    copy /Y redistricting_reference.pdf "%OUTPUT_DIR%\redistricting_reference.pdf" >nul
    echo [OK] Copied to %OUTPUT_DIR%\redistricting_reference.pdf
) else (
    echo [ERROR] redistricting_reference.pdf not created
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
echo   - %OUTPUT_DIR%\github_reference.pdf
echo   - %OUTPUT_DIR%\redistricting_reference.pdf
echo ======================================================================
echo.
