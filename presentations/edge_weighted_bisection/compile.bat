@echo off
REM Compile Presentation Materials: Edge-Weighted Recursive Bisection

cd /d %~dp0

REM Create output directory
set OUTPUT_DIR=..\..\outputs\presentations\edge_weighted_bisection
if not exist "%OUTPUT_DIR%" mkdir "%OUTPUT_DIR%"

echo ======================================================================
echo Compiling Edge-Weighted Recursive Bisection Presentation Materials
echo ======================================================================
echo.
echo Output directory: %OUTPUT_DIR%
echo.

REM ======================================================================
REM Compile Presentation Slides
REM ======================================================================
echo [1/2] Compiling presentation slides (presentation.tex)...
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
REM Compile Layman's Guide
REM ======================================================================
echo [2/2] Compiling layman's guide (laymen_guide.tex)...
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
echo   - %OUTPUT_DIR%\presentation.pdf
echo   - %OUTPUT_DIR%\laymen_guide.pdf
echo ======================================================================
echo.

pause
