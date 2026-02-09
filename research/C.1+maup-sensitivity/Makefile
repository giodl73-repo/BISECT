# Makefile for 11+maup-sensitivity paper

.PHONY: pdf clean

pdf:
	pdflatex main.tex
	bibtex main
	pdflatex main.tex
	pdflatex main.tex

clean:
	rm -f *.aux *.log *.bbl *.blg *.out *.toc *.lot *.lof *.synctex.gz
	rm -f sections/*.aux
