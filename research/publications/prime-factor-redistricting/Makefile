SLUG = prime-factor-redistricting
DOCS = ../../docs

.PHONY: pdf dist clean

pdf:
	pdflatex -interaction=nonstopmode main.tex
	bibtex main || true
	pdflatex -interaction=nonstopmode main.tex
	pdflatex -interaction=nonstopmode main.tex

dist: pdf
	mkdir -p $(DOCS)
	cp main.pdf $(DOCS)/$(SLUG).pdf
	@echo "deployed: $(DOCS)/$(SLUG).pdf"

clean:
	rm -f main.aux main.bbl main.blg main.log main.out main.pdf
