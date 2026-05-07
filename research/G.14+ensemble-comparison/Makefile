## G.14 -- A Practitioner's Comparison of Redistricting Ensemble Algorithms
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
	sections/03-compactness.tex \
	sections/04-distributional.tex \
	sections/05-decision-framework.tex \
	sections/06-conclusion.tex
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
