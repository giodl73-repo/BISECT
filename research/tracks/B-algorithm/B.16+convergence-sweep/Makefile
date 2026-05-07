## B.16 — ConvergenceSweep: T=600 Statutory Convergence Criterion
## Build: make
## Clean: make clean
## Full rebuild: make clean && make

PAPER = main
LATEX = pdflatex
BIBTEX = bibtex

.PHONY: all clean view

all: $(PAPER).pdf

$(PAPER).pdf: $(PAPER).tex references.bib \
	sections/01-introduction.tex \
	sections/02-algorithm.tex \
	sections/03-t-sufficiency.tex \
	sections/04-sha256-seed.tex \
	sections/05-implementation.tex \
	sections/06-conclusion.tex
	$(LATEX) $(PAPER)
	$(BIBTEX) $(PAPER)
	$(LATEX) $(PAPER)
	$(LATEX) $(PAPER)

clean:
	rm -f $(PAPER).aux $(PAPER).bbl $(PAPER).blg $(PAPER).log \
	       $(PAPER).out $(PAPER).pdf $(PAPER).toc \
	       sections/*.aux

view: $(PAPER).pdf
	open $(PAPER).pdf 2>/dev/null || start $(PAPER).pdf 2>/dev/null || \
	xdg-open $(PAPER).pdf
