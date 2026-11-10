#!/bin/bash
# Compile Paper 3: Recursive Bisection with Edge-Weighted Cuts

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Compiling Paper 3: Recursive Bisection with Edge-Weighted Cuts..."
echo "=================================================="

# First pass
echo "LaTeX pass 1/3..."
pdflatex -interaction=nonstopmode recursive_bisection_with_edge_weighted_cuts.tex > /dev/null

# BibTeX
echo "BibTeX..."
bibtex recursive_bisection_with_edge_weighted_cuts > /dev/null

# Second pass
echo "LaTeX pass 2/3..."
pdflatex -interaction=nonstopmode recursive_bisection_with_edge_weighted_cuts.tex > /dev/null

# Third pass
echo "LaTeX pass 3/3..."
pdflatex -interaction=nonstopmode recursive_bisection_with_edge_weighted_cuts.tex > /dev/null

echo ""
echo "Compilation complete!"
echo "Output: recursive_bisection_with_edge_weighted_cuts.pdf"
echo ""

# Clean up auxiliary files
echo "Cleaning auxiliary files..."
rm -f *.aux *.log *.bbl *.blg *.out *.toc

echo "Done!"
