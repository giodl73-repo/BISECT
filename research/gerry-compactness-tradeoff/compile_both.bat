@echo off
REM Compile both main paper and supplementary materials
REM Usage: compile_both.bat

echo ============================================================
echo Compiling VRA-Compactness Tradeoff Paper (Main + Supplement)
echo ============================================================
echo.

REM Main Paper
echo [1/2] Compiling main paper (main.tex)...
echo ------------------------------------------------------------
pdflatex -interaction=nonstopmode main.tex >nul 2>&1
if errorlevel 1 (
    echo [ERROR] First pdflatex pass failed for main paper
    type main.log | findstr /C:"!" | findstr /V "Underfull\|Overfull"
    exit /b 1
)

bibtex main >nul 2>&1
if errorlevel 1 (
    echo [WARNING] BibTeX reported issues for main paper
    type main.blg | findstr /C:"Warning\|Error"
)

pdflatex -interaction=nonstopmode main.tex >nul 2>&1
pdflatex -interaction=nonstopmode main.tex >nul 2>&1

if exist main.pdf (
    echo [OK] Main paper compiled successfully: main.pdf
) else (
    echo [FAIL] Main paper compilation failed
    exit /b 1
)

echo.

REM Supplementary Materials
echo [2/2] Compiling supplementary materials (main_supplement.tex)...
echo ------------------------------------------------------------
pdflatex -interaction=nonstopmode main_supplement.tex >nul 2>&1
if errorlevel 1 (
    echo [ERROR] First pdflatex pass failed for supplement
    type main_supplement.log | findstr /C:"!" | findstr /V "Underfull\|Overfull"
    exit /b 1
)

bibtex main_supplement >nul 2>&1
if errorlevel 1 (
    echo [WARNING] BibTeX reported issues for supplement
    type main_supplement.blg | findstr /C:"Warning\|Error"
)

pdflatex -interaction=nonstopmode main_supplement.tex >nul 2>&1
pdflatex -interaction=nonstopmode main_supplement.tex >nul 2>&1

if exist main_supplement.pdf (
    echo [OK] Supplementary materials compiled successfully: main_supplement.pdf
) else (
    echo [FAIL] Supplement compilation failed
    exit /b 1
)

echo.
echo ============================================================
echo Compilation Summary
echo ============================================================

REM Get file sizes
for %%F in (main.pdf) do set MAIN_SIZE=%%~zF
for %%F in (main_supplement.pdf) do set SUPP_SIZE=%%~zF

REM Convert bytes to MB
set /a MAIN_MB=MAIN_SIZE/1048576
set /a SUPP_MB=SUPP_SIZE/1048576

echo Main Paper:        main.pdf (%MAIN_MB% MB)
echo Supplementary:     main_supplement.pdf (%SUPP_MB% MB)
echo.

REM Get page counts (requires pdfinfo or similar - skip if not available)
echo To view page counts, use: pdfinfo main.pdf ^| findstr Pages
echo.

echo Compilation complete!
echo ============================================================
