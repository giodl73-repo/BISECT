PDF=main.pdf

.PHONY: all clean

all: $(PDF)

$(PDF): main.tex references.bib sections/*.tex
	pdflatex main
	bibtex main
	pdflatex main
	pdflatex main

clean:
	rm -f main.aux main.bbl main.blg main.log main.out main.pdf
