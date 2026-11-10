#!/bin/bash
# Script to compile the LaTeX paper
# Usage: ./compile.sh [latex_bin_directory]

# Set LaTeX path: use argument if provided, otherwise use default
if [ -z "$1" ]; then
    LATEX_PATH="/c/Users/giodl/AppData/Local/Programs/MiKTeX/miktex/bin/x64"
else
    LATEX_PATH="$1"
fi

# Get script directory and change to it
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Using LaTeX from: $LATEX_PATH"
echo "Working in: $SCRIPT_DIR"
echo ""

echo "Compiling recursive_bisection.tex (first pass)..."
"$LATEX_PATH/pdflatex.exe" -interaction=nonstopmode recursive_bisection.tex > /dev/null 2>&1

echo "Running bibtex for references..."
"$LATEX_PATH/bibtex.exe" recursive_bisection > /dev/null 2>&1

echo "Compiling recursive_bisection.tex (second pass)..."
"$LATEX_PATH/pdflatex.exe" -interaction=nonstopmode recursive_bisection.tex > /dev/null 2>&1

echo "Compiling recursive_bisection.tex (third pass for cross-refs)..."
"$LATEX_PATH/pdflatex.exe" -interaction=nonstopmode recursive_bisection.tex 2>&1 | tail -5

echo ""
echo "Compilation complete! Check recursive_bisection.pdf"
echo ""

# Count pages
if [ -f recursive_bisection.pdf ]; then
    echo "Checking page count..."
    "$LATEX_PATH/pdfinfo.exe" recursive_bisection.pdf 2>/dev/null | grep "Pages:" || echo "pdfinfo not available"
fi
