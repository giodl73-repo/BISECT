.PHONY: pdf clean

pdf:
	pdflatex main.tex
	bibtex main
	pdflatex main.tex
	pdflatex main.tex

clean:
	rm -f *.aux *.bbl *.blg *.log *.out *.toc *.pdf
	rm -f sections/*.aux
