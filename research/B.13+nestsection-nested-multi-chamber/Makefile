# NestSection paper Makefile

PAPER    = main
SECTIONS = sections/01-introduction.tex \
           sections/02-related-work.tex \
           sections/03-algorithm.tex \
           sections/04-evaluation.tex \
           sections/05-discussion.tex \
           sections/06-conclusion.tex

.PHONY: all pdf clean watch

all: pdf

pdf: $(PAPER).pdf

$(PAPER).pdf: $(PAPER).tex $(SECTIONS) references.bib
	pdflatex -interaction=nonstopmode $(PAPER).tex
	bibtex $(PAPER)
	pdflatex -interaction=nonstopmode $(PAPER).tex
	pdflatex -interaction=nonstopmode $(PAPER).tex

clean:
	rm -f $(PAPER).pdf $(PAPER).aux $(PAPER).bbl $(PAPER).blg \
	      $(PAPER).log $(PAPER).out $(PAPER).toc \
	      $(PAPER).lof $(PAPER).lot

watch:
	while true; do \
	  inotifywait -qe close_write $(PAPER).tex $(SECTIONS) references.bib && \
	  $(MAKE) pdf; \
	done
