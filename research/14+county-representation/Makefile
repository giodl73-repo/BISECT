# Makefile for 14+county-representation

.PHONY: pdf clean

pdf:
	pdflatex main.tex
	bibtex main
	pdflatex main.tex
	pdflatex main.tex

clean:
	rm -f *.aux *.log *.bbl *.blg *.out *.toc *.pdf
	rm -f sections/*.aux
