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

## Overview

Compile LaTeX documents (academic papers and presentations) using pdflatex and bibtex. Automatically handles multiple compilation passes for cross-references, bibliographies, and navigation.

## Prerequisites

**Required Software**:
- **pdflatex** - LaTeX compiler (part of MikTeX, TeX Live, etc.)
- **bibtex** - Bibliography processor (for papers with references)

**Check installation**:
```bash
pdflatex --version
bibtex --version
```

**Project Structure**:
```
papers/
├── 01_recursive_bisection/
│   ├── recursive_bisection.tex     # Main paper
│   ├── references.bib             # Bibliography
│   └── compile.bat                # Compilation script
├── 02_edge_weighted_bisection/
├── 03_combined_recursive_bisection/
└── compile.bat                    # Compile all papers

presentations/
├── edge_weighted_bisection/
│   ├── presentation.tex           # Beamer slides
│   ├── laymen_guide.tex          # Layman's guide
│   └── compile.bat               # Compilation script
└── compile.bat                   # Compile all presentations
```

## When to Use This Skill

- User says: "Compile the paper" or "Build the LaTeX document"
- User says: "Generate the PDF for the presentation"
- User wants to compile slides after adding figures
- User needs to rebuild document after content changes
- User asks to "compile all papers" or "compile all presentations"

## Compilation Types

### 1. Academic Papers (with BibTeX)

**Passes**:
1. pdflatex (1st pass) - Process document structure
2. bibtex - Generate bibliography
3. pdflatex (2nd pass) - Include bibliography
4. pdflatex (3rd pass) - Resolve cross-references

**Example documents**:
- `papers/01_recursive_bisection/recursive_bisection.tex`
- `papers/02_edge_weighted_bisection/edge_weighted_bisection.tex`
- `papers/03_combined_recursive_bisection/recursive_bisection_with_edge_weighted_cuts.tex`

### 2. Presentations (Beamer, no BibTeX)

**Passes**:
1. pdflatex (1st pass) - Process slides
2. pdflatex (2nd pass) - Generate navigation elements

**Example documents**:
- `presentations/edge_weighted_bisection/presentation.tex`
- `presentations/edge_weighted_bisection/laymen_guide.tex`

### 3. Batch Compilation

**Compile all papers**:
```bash
cd papers
compile.bat --year 2020 --version v1
```

**Compile all presentations**:
```bash
cd presentations
compile.bat --year 2020 --version v1
```

## Workflow

### Step 1: Identify Document

Ask user if not specified:
- **Document path**: Which .tex file to compile?
- **Type**: Paper (with BibTeX) or Presentation (no BibTeX)?
- **Year**: Census year for figure generation (2000, 2010, 2020)
- **Version**: Pipeline version for figures (v1, v2, test, etc.)

### Step 2: Navigate to Directory

```bash
cd papers/03_combined_recursive_bisection
# or
cd presentations/edge_weighted_bisection
```

### Step 3: Run Compilation

**Option A: Use existing compile.bat** (recommended)
```bash
# Single paper or presentation
compile.bat --year 2020 --version v1

# With reset (clear old outputs)
compile.bat --year 2020 --version v1 --reset
```

**Option B: Manual compilation**

**For papers with BibTeX**:
```bash
pdflatex -interaction=nonstopmode paper_name.tex
bibtex paper_name
pdflatex -interaction=nonstopmode paper_name.tex
pdflatex -interaction=nonstopmode paper_name.tex
```

**For presentations without BibTeX**:
```bash
pdflatex -interaction=nonstopmode presentation.tex
pdflatex -interaction=nonstopmode presentation.tex
```

### Step 4: Check Output

**Verify PDF created**:
```bash
ls *.pdf
```

**Check for errors in log**:
```bash
grep -i error *.log
```

### Step 5: Clean Up (Optional)

Remove auxiliary files:
```bash
del *.aux *.log *.bbl *.blg *.out *.toc *.nav *.snm *.vrb
```

## Compilation Flags

**`-interaction=nonstopmode`**:
- Don't stop for user input on errors
- Allows batch processing
- Errors written to .log file

**`>nul 2>&1`** (in batch files):
- Suppress console output (cleaner)
- Errors still logged to .log file

## Common Parameters

### --year
Census year for figure generation
- **Values**: 2000, 2010, 2020
- **Default**: 2020
- **Usage**: Passed to figure generation scripts

### --version
Pipeline version for figures
- **Values**: v1, v2, test, etc.
- **Default**: v1
- **Usage**: Determines which pipeline outputs to use

### --reset
Clear old output directory before compilation
- **Effect**: Deletes `outputs/papers/` or `outputs/presentations/` directory
- **Usage**: Fresh start, remove old PDFs

## Output Locations

### Papers
```
outputs/papers/{paper_name}/
└── {paper_name}.pdf
```

**Examples**:
- `outputs/papers/01_recursive_bisection/recursive_bisection.pdf`
- `outputs/papers/03_combined_recursive_bisection/recursive_bisection_with_edge_weighted_cuts.pdf`

### Presentations
```
outputs/presentations/{presentation_name}/
├── presentation.pdf
└── laymen_guide.pdf
```

**Example**:
- `outputs/presentations/edge_weighted_bisection/presentation.pdf`
- `outputs/presentations/edge_weighted_bisection/laymen_guide.pdf`

## Auxiliary Files Generated

**Common**:
- `.aux` - Auxiliary file (cross-references)
- `.log` - Compilation log (errors, warnings)
- `.out` - PDF bookmarks/hyperlinks

**Papers (with BibTeX)**:
- `.bbl` - Formatted bibliography
- `.blg` - BibTeX log
- `.toc` - Table of contents

**Presentations (Beamer)**:
- `.nav` - Navigation elements
- `.snm` - Slide notes
- `.vrb` - Verbatim content

**Note**: Auxiliary files are automatically cleaned up by compile.bat scripts

## Troubleshooting

### LaTeX Not Found

```
'pdflatex' is not recognized as an internal or external command
```

**Solution**: Install LaTeX distribution
- **Windows**: [MikTeX](https://miktex.org/) or [TeX Live](https://www.tug.org/texlive/)
- **Add to PATH**: Ensure LaTeX binaries in system PATH

### Compilation Errors

**Missing package**:
```
! LaTeX Error: File `beamer.cls' not found.
```

**Solution**: Install missing package
- MikTeX: Packages install automatically on first use
- Manual: `mpm --install=beamer` (MikTeX Package Manager)

**Undefined references**:
```
LaTeX Warning: There were undefined references.
```

**Solution**: Run additional pdflatex pass
- This is normal after BibTeX
- compile.bat scripts already handle this

**Bibliography errors**:
```
! I couldn't open file name `paper.bib'
```

**Solution**: Check bibliography file path
- Verify `\bibliography{references}` points to existing .bib file
- Ensure .bib file is in same directory

### Figures Not Found

```
! Package pdftex.def Error: File `figure.png' not found
```

**Solution**: Generate figures first
- Use `/create-presentation-figures` to generate figures
- Verify figure paths in LaTeX match actual file locations
- Check that pipeline has run to generate round progression maps

### Compilation Slow

**Cause**: Repeated failed compilations without clearing .aux files

**Solution**: Clean auxiliary files first
```bash
del *.aux *.log *.bbl *.blg *.out *.toc *.nav *.snm *.vrb
```

## Advanced Usage

### Compile Without Figures

If figure generation fails but you want to see document structure:

```bash
# Comment out \includegraphics commands temporarily
# Or use draft mode
\documentclass[draft]{beamer}  % Shows boxes instead of figures
```

### Check for Warnings

```bash
grep -i warning *.log | grep -v "Underfull\|Overfull"
```

Filters out minor box warnings, shows important warnings only.

### Generate Different Figure Years

Compile presentation with 2010 data instead of 2020:

```bash
compile.bat --year 2010 --version v1
```

Figures will use 2010 census data and pipeline outputs.

### Batch Compile All Documents

**All papers**:
```bash
cd papers
compile.bat --year 2020 --version v1
```

**All presentations**:
```bash
cd presentations
compile.bat --year 2020 --version v1
```

## Integration with Figure Generation

### Typical Workflow

1. **Generate figures**:
   ```bash
   /create-presentation-figures --year 2020 --version v1
   ```

2. **Compile document**:
   ```bash
   /compile-latex --document presentation.tex --type presentation
   ```

3. **Open PDF**:
   ```bash
   start outputs/presentations/edge_weighted_bisection/presentation.pdf
   ```

### Automatic Figure Generation

The `compile.bat` scripts automatically call figure generation:

**Papers**:
```batch
python create_figures.py --year %YEAR% --version %VERSION%
```

**Presentations**:
```batch
python ..\..\scripts\figures\generate_all_figures.py --year %YEAR% --version %VERSION%
```

If figure generation fails, compilation continues with warning.

## Related Skills

- `/create-presentation-figures` - Generate figures before compilation
- `/create-pedagogical-example` - Create algorithm examples for papers
- `/run-statistical-analysis` - Generate statistical tables for papers
- `/create-state-map` - Generate state visualizations for figures

## Performance

**Typical compilation times**:

| Document Type | Passes | Time | Notes |
|--------------|--------|------|-------|
| Short presentation (10 slides) | 2 | ~10 sec | No figures |
| Long presentation (50 slides) | 2 | ~30 sec | With figures |
| Short paper (5 pages) | 3 | ~15 sec | With BibTeX |
| Long paper (20 pages) | 3 | ~45 sec | With BibTeX, figures |

**Bottlenecks**:
- Figure rendering (high-res images)
- Complex mathematics (many equations)
- Large tables (statistical results)
- Bibliography (many citations)

## Best Practices

1. **Compile frequently**: Catch errors early
2. **Check logs**: Don't ignore warnings
3. **Use compile.bat**: Consistent, automated workflow
4. **Clean auxiliary files**: When switching between document versions
5. **Version control**: Commit .tex and .bib, not .pdf
6. **Generate figures first**: Ensure all figures available before compilation
7. **Use nonstopmode**: For automated workflows

## Common Workflow Examples

### Compile Single Paper

```bash
cd papers/03_combined_recursive_bisection
compile.bat --year 2020 --version v1
```

### Compile Presentation After Adding Figures

```bash
# 1. Generate new figures
cd presentations/edge_weighted_bisection
python create_figures.py --year 2020 --version v1

# 2. Compile presentation
compile.bat --year 2020 --version v1

# 3. Open result
start ../../outputs/presentations/edge_weighted_bisection/presentation.pdf
```

### Fresh Compilation (Clear Old Outputs)

```bash
compile.bat --year 2020 --version v1 --reset
```

### Compile All Papers for Publication

```bash
cd papers
compile.bat --year 2020 --version v1
# Generates all papers in outputs/papers/
```

## Next Steps

After compilation:
- Open PDF to review
- Check for missing figures or references
- Verify equations render correctly
- Share outputs/papers/ or outputs/presentations/ PDFs with collaborators
- Commit changes to .tex files (not PDFs)
