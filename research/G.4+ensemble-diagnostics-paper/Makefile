## G.4 — Ensemble Diagnostics Paper
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
	sections/02-rhat.tex \
	sections/03-ess.tex \
	sections/04-hamming.tex \
	sections/05-application.tex \
	sections/06-mixing-time.tex \
	sections/07-statutory.tex \
	sections/08-implementation.tex
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
