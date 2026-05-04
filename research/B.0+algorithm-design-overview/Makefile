## B.0 Synthesis Paper — Makefile
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
	sections/02-legal-requirements.tex \
	sections/03-algorithm-components.tex \
	sections/04-requirements-matrix.tex \
	sections/05-bakeoff.tex \
	sections/06-callais-compliance.tex \
	sections/07-discussion.tex \
	sections/08-conclusion.tex
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
