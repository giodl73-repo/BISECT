main.pdf: main.tex sections/*.tex references.bib
	pdflatex main
	bibtex main
	pdflatex main
	pdflatex main

.PHONY: clean
clean:
	rm -f main.aux main.bbl main.blg main.log main.out main.pdf
