.PHONY: pdf clean

pdf:
	pdflatex main.tex
	bibtex main
	pdflatex main.tex
	pdflatex main.tex

clean:
	rm -f *.aux *.log *.bbl *.blg *.out *.toc *.lof *.lot
	rm -f sections/*.aux
