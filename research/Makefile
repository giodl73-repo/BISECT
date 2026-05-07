# Research papers build system
#
# Compiles every paper directory that contains a main.tex.
# Output PDFs are copied to docs/papers/ (committed to git — gitignore exempts docs/).
#
# Usage:
#   make          — compile all papers with main.tex
#   make clean    — remove LaTeX intermediates (keeps PDFs)
#   make docs     — compile + copy PDFs to docs/papers/
#   make list     — print which papers have main.tex
#
# Single paper:
#   make B.1+recursive-bisection

DOCS_DIR := ../docs/papers
LATEXMK  := latexmk -pdf -interaction=nonstopmode -quiet

# All directories that have a main.tex
PAPER_DIRS := $(shell find . -maxdepth 1 -type d -name '[ABCDE]*' \
                | sort \
                | while read d; do test -f "$$d/main.tex" && echo $$d; done)

.PHONY: all clean docs list $(PAPER_DIRS)

all: $(PAPER_DIRS)

$(PAPER_DIRS):
	@echo "  [TEX] $@"
	@cd "$@" && $(LATEXMK) main.tex 2>&1 | tail -3 || \
	  (echo "  [FAIL] $@ — trying pdflatex fallback"; \
	   pdflatex -interaction=nonstopmode main.tex > /dev/null 2>&1 && \
	   pdflatex -interaction=nonstopmode main.tex > /dev/null 2>&1)
	@test -f "$@/main.pdf" && echo "  [OK]  $@/main.pdf" || echo "  [SKIP] $@/main.pdf not produced"

docs: all
	@mkdir -p $(DOCS_DIR)
	@for d in $(PAPER_DIRS); do \
	  name=$$(basename $$d); \
	  if [ -f "$$d/main.pdf" ]; then \
	    cp "$$d/main.pdf" "$(DOCS_DIR)/$$name.pdf"; \
	    echo "  [COPY] docs/papers/$$name.pdf"; \
	  fi; \
	done
	@echo "Done — PDFs in $(DOCS_DIR)"

clean:
	@for d in $(PAPER_DIRS); do \
	  echo "  [CLEAN] $$d"; \
	  cd "$$d" && $(LATEXMK) -C main.tex > /dev/null 2>&1; cd ..; \
	done

list:
	@echo "Papers with main.tex:"
	@for d in $(PAPER_DIRS); do echo "  $$d"; done
