@echo off
REM Batch file to compile the LaTeX paper
REM Usage: compile.bat [latex_bin_directory]
REM Example: compile.bat "C:\Program Files\MiKTeX\miktex\bin\x64"

REM Set LaTeX path: use argument if provided, otherwise use default
if "%~1"=="" (
    set LATEX_PATH=C:\Users\giodl\AppData\Local\Programs\MiKTeX\miktex\bin\x64
) else (
    set LATEX_PATH=%~1
)

set PAPER_DIR=%~dp0

echo Using LaTeX from: %LATEX_PATH%
echo Working in: %PAPER_DIR%
echo.

cd /d "%PAPER_DIR%"

echo Compiling recursive_bisection.tex (first pass)...
"%LATEX_PATH%\pdflatex.exe" -interaction=nonstopmode recursive_bisection.tex >nul 2>&1

echo Running bibtex for references...
"%LATEX_PATH%\bibtex.exe" recursive_bisection >nul 2>&1

echo Compiling recursive_bisection.tex (second pass)...
"%LATEX_PATH%\pdflatex.exe" -interaction=nonstopmode recursive_bisection.tex >nul 2>&1

echo Compiling recursive_bisection.tex (third pass for cross-refs)...
"%LATEX_PATH%\pdflatex.exe" -interaction=nonstopmode recursive_bisection.tex

echo.
echo Compilation complete! Check recursive_bisection.pdf
echo.
echo Page count:
"%LATEX_PATH%\pdfinfo.exe" recursive_bisection.pdf 2>nul | findstr "Pages:" || echo Could not determine page count
