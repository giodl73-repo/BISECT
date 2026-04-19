---
name: compile-latex
description: Compile LaTeX documents (papers, presentations) using pdflatex and bibtex. Handles multiple compilation passes, bibliography generation, and cleanup. Use when you need to compile academic papers or presentation slides.
allowed-tools:
  - Read
  - Bash
  - Glob
  - Grep
user-invocable: true
---

# Compile LaTeX

Compile LaTeX documents (academic papers/presentations) using pdflatex + bibtex. Automatically handles multiple compilation passes for cross-refs, bibliographies, navigation.

## Prerequisites
**Required**: pdflatex + bibtex (MikTeX/TeX Live) → Check: `pdflatex --version && bibtex --version`

**Structure**:
```
papers/
├── 01_recursive_bisection/recursive_bisection.tex + references.bib + compile.bat
├── 02_edge_weighted_bisection/
├── 03_combined_recursive_bisection/
└── compile.bat  # Compile all
presentations/
├── edge_weighted_bisection/presentation.tex + laymen_guide.tex + compile.bat
└── compile.bat  # Compile all
```

## When to Use
User says "Compile the paper/Build LaTeX document/Generate PDF for presentation/Compile slides", after adding figures, after content changes, "compile all papers/presentations"

## Compilation Types

**1. Academic Papers** (with BibTeX): 4 passes (pdflatex → bibtex → pdflatex → pdflatex)
- Examples: `papers/01_recursive_bisection/recursive_bisection.tex`, `papers/02_edge_weighted_bisection/edge_weighted_bisection.tex`, `papers/03_combined_recursive_bisection/recursive_bisection_with_edge_weighted_cuts.tex`

**2. Presentations** (Beamer, no BibTeX): 2 passes (pdflatex → pdflatex for navigation)
- Examples: `presentations/edge_weighted_bisection/presentation.tex`, `presentations/edge_weighted_bisection/laymen_guide.tex`

**3. Batch**: `cd papers && compile.bat --year 2020 --version v1` or `cd presentations && compile.bat`

## Workflow

### Step 1: Identify Document
Ask if not specified: Document path (.tex file)?, Paper (with BibTeX) or Presentation?, Year (2000/2010/2020)?, Version (v1/v2/test)?

### Step 2: Navigate to Directory
```bash
cd papers/03_combined_recursive_bisection
# or
cd presentations/edge_weighted_bisection
```

### Step 3: Run Compilation

**Option A: Use compile.bat** (recommended):
```bash
compile.bat --year 2020 --version v1           # Standard
compile.bat --year 2020 --version v1 --reset   # Clear old outputs
```

**Option B: Manual**:
**Papers** (with BibTeX):
```bash
pdflatex -interaction=nonstopmode paper_name.tex
bibtex paper_name
pdflatex -interaction=nonstopmode paper_name.tex
pdflatex -interaction=nonstopmode paper_name.tex
```

**Presentations** (no BibTeX):
```bash
pdflatex -interaction=nonstopmode presentation.tex
pdflatex -interaction=nonstopmode presentation.tex
```

### Step 4: Check Output
```bash
ls *.pdf                  # Verify PDF created
grep -i error *.log       # Check errors
```

### Step 5: Clean Up (Optional)
```bash
del *.aux *.log *.bbl *.blg *.out *.toc *.nav *.snm *.vrb
```

## Flags and Parameters

**`-interaction=nonstopmode`**: Don't stop for user input on errors, batch processing, errors → .log file
**`>nul 2>&1`** (batch files): Suppress console output, errors still logged

**`--year`**: Census year for figure generation (2000/2010/2020, default 2020) → Passed to figure scripts
**`--version`**: Pipeline version for figures (v1/v2/test, default v1) → Determines which pipeline outputs to use
**`--reset`**: Clear old output directory before compilation → Fresh start

## Output Locations

**Papers**: `outputs/papers/{paper_name}/{paper_name}.pdf`
- Example: `outputs/papers/01_recursive_bisection/recursive_bisection.pdf`

**Presentations**: `outputs/presentations/{presentation_name}/presentation.pdf + laymen_guide.pdf`
- Example: `outputs/presentations/edge_weighted_bisection/presentation.pdf`

## Auxiliary Files Generated

**Common**: `.aux` (cross-refs), `.log` (errors/warnings), `.out` (PDF bookmarks)
**Papers**: `.bbl` (formatted bib), `.blg` (bibtex log), `.toc` (table of contents)
**Presentations**: `.nav` (navigation), `.snm` (slide notes), `.vrb` (verbatim content)
Note: compile.bat scripts auto-clean these

## Troubleshooting

**LaTeX not found**: `'pdflatex' is not recognized` → Install MikTeX/TeX Live, add to PATH
**Missing package**: `! LaTeX Error: File 'beamer.cls' not found` → MikTeX auto-installs on first use or `mpm --install=beamer`
**Undefined references**: `LaTeX Warning: There were undefined references` → Run additional pdflatex pass (compile.bat handles this)
**Bibliography errors**: `! I couldn't open file name 'paper.bib'` → Check `\bibliography{references}` points to existing .bib file in same directory
**Figures not found**: `! Package pdftex.def Error: File 'figure.png' not found` → Use `/create-presentation-figures`, verify paths, check pipeline ran
**Compilation slow**: Repeated failures without clearing .aux → `del *.aux *.log *.bbl *.blg *.out *.toc *.nav *.snm *.vrb`

## Advanced Usage

**Compile without figures**: Comment out `\includegraphics` or use draft mode `\documentclass[draft]{beamer}` (shows boxes)
**Check warnings**: `grep -i warning *.log | grep -v "Underfull\|Overfull"` (filters minor box warnings)
**Different figure years**: `compile.bat --year 2010 --version v1` (uses 2010 census data)
**Batch compile all**: `cd papers && compile.bat --year 2020 --version v1` or `cd presentations && compile.bat`

## Integration with Figure Generation

**Typical workflow**:
1. `/create-presentation-figures --year 2020 --version v1`
2. `/compile-latex --document presentation.tex --type presentation`
3. `start outputs/presentations/edge_weighted_bisection/presentation.pdf`

**Auto figure generation**: compile.bat scripts auto-call figure generation:
- Papers: `python create_figures.py --year %YEAR% --version %VERSION%`
- Presentations: `python ..\..\scripts\figures\generate_all_figures.py --year %YEAR% --version %VERSION%`
- If figure generation fails, compilation continues with warning

## Performance

| Document Type | Passes | Time | Notes |
|--------------|--------|------|-------|
| Short presentation (10 slides) | 2 | ~10s | No figures |
| Long presentation (50 slides) | 2 | ~30s | With figures |
| Short paper (5 pages) | 3 | ~15s | With BibTeX |
| Long paper (20 pages) | 3 | ~45s | With BibTeX, figures |

**Bottlenecks**: Figure rendering (high-res images), complex math (many equations), large tables (statistical results), bibliography (many citations)

## Best Practices
Compile frequently (catch errors early), check logs (don't ignore warnings), use compile.bat (consistent workflow), clean auxiliary files (when switching versions), version control (.tex + .bib, not .pdf), generate figures first (ensure all available), use nonstopmode (automated workflows)

## Common Workflows

**Single paper**: `cd papers/03_combined_recursive_bisection && compile.bat --year 2020 --version v1`
**Presentation after adding figures**: `cd presentations/edge_weighted_bisection && python create_figures.py --year 2020 --version v1 && compile.bat --year 2020 --version v1 && start ../../outputs/presentations/edge_weighted_bisection/presentation.pdf`
**Fresh compilation**: `compile.bat --year 2020 --version v1 --reset`
**All papers for publication**: `cd papers && compile.bat --year 2020 --version v1`

## Next Steps
Open PDF to review, check for missing figures/references, verify equations render correctly, share outputs/papers/ or outputs/presentations/ PDFs with collaborators, commit changes to .tex files (not PDFs)

## Related Skills
`/create-presentation-figures` (generate figures before compilation), `/create-pedagogical-example` (create algorithm examples), `/run-statistical-analysis` (generate statistical tables), `/create-state-map` (generate state visualizations)
