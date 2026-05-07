.PHONY: pdf clean

pdf: main.pdf

main.pdf: main.tex sections/*.tex references.bib
	pdflatex main
	bibtex main
	pdflatex main
	pdflatex main

clean:
	rm -f *.aux *.bbl *.blg *.log *.out *.toc *.lof *.lot *.fls *.fdb_latexmk *.synctex.gz
	rm -f sections/*.aux

.PHONY: pdf clean
