PAPERS = $(wildcard gerry-*/)
DIST_DIR = docs

.PHONY: all clean dist

all:
	@for dir in $(PAPERS); do \
		echo "Building $$dir..."; \
		cd $$dir && pdflatex -interaction=nonstopmode main.tex && cd ..; \
	done

dist: all
	@mkdir -p $(DIST_DIR)
	@for dir in $(PAPERS); do \
		name=$$(basename $$dir); \
		cp $$dir/main.pdf $(DIST_DIR)/$$name.pdf 2>/dev/null || true; \
	done
	@echo "PDFs copied to $(DIST_DIR)/"

clean:
	@for dir in $(PAPERS); do \
		cd $$dir && rm -f *.aux *.log *.bbl *.blg *.out *.toc *.nav *.snm *.vrb && cd ..; \
	done
