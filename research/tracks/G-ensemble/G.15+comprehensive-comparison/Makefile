## G.15 -- A Comprehensive Comparison of Redistricting Structure and Search Algorithms
## Build: make
## Clean: make clean

PAPER = main
LATEX = pdflatex
BIBTEX = bibtex
LATEXMK = latexmk

.PHONY: all clean view latexmk

all: $(PAPER).pdf

$(PAPER).pdf: $(PAPER).tex references.bib \
	sections/01-introduction.tex \
	sections/02-setup.tex \
	sections/03-structure-comparison.tex \
	sections/04-search-comparison.tex \
	sections/05-distributional.tex \
	sections/06-decision-framework.tex \
	sections/07-compute-budget.tex \
	sections/08-conclusion.tex
	$(LATEX) -interaction=nonstopmode $(PAPER)
	$(BIBTEX) $(PAPER)
	$(LATEX) -interaction=nonstopmode $(PAPER)
	$(LATEX) -interaction=nonstopmode $(PAPER)

latexmk:
	$(LATEXMK) -pdf -interaction=nonstopmode $(PAPER)

clean:
	rm -f $(PAPER).aux $(PAPER).bbl $(PAPER).blg $(PAPER).log \
	       $(PAPER).out $(PAPER).pdf $(PAPER).toc \
	       $(PAPER).fls $(PAPER).fdb_latexmk \
	       sections/*.aux

view: $(PAPER).pdf
	start $(PAPER).pdf 2>/dev/null || open $(PAPER).pdf 2>/dev/null || \
	xdg-open $(PAPER).pdf
