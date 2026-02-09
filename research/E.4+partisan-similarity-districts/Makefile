.PHONY: pdf clean

pdf: main.pdf

main.pdf: main.tex sections/*.tex
	pdflatex main.tex
	bibtex main || true
	pdflatex main.tex
	pdflatex main.tex

clean:
	rm -f *.aux *.log *.out *.bbl *.blg *.toc *.lot *.lof
	rm -f sections/*.aux
