## G.8 -- Flip Proposals for Local Sensitivity Analysis
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
	sections/02-formal-properties.tex \
	sections/03-local-sensitivity.tex \
	sections/04-comparison.tex \
	sections/05-audit-chain.tex \
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
