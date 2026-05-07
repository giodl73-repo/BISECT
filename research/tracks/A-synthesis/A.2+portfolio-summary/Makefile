# Makefile for portfolio guide and summary

.PHONY: all guide summary clean

all: guide summary

guide:
	pdflatex guide.tex
	pdflatex guide.tex

summary:
	pdflatex summary.tex
	pdflatex summary.tex

clean:
	rm -f *.aux *.log *.out *.toc *.bbl *.blg

distclean: clean
	rm -f guide.pdf summary.pdf
