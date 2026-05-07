## G.11 -- Multi-scale MCMC: Hierarchical Mixing for Large-k Redistricting
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
	sections/02-background.tex \
	sections/03-algorithm.tex \
	sections/04-resolution-system.tex \
	sections/05-mixing.tex \
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
