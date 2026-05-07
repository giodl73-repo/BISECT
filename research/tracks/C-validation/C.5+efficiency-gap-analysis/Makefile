.PHONY: all pdf clean

all: pdf

pdf: main.pdf

main.pdf: main.tex sections/*.tex
	pdflatex main.tex
	bibtex main
	pdflatex main.tex
	pdflatex main.tex

clean:
	rm -f *.aux *.log *.out *.bbl *.blg *.toc *.lof *.lot
	rm -f sections/*.aux

distclean: clean
	rm -f main.pdf
