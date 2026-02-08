# LaTeX Compilation Guide

## Quick Compilation

From the `gerry-compactness-tradeoff` directory:

```bash
# Full compilation (pdflatex + bibtex + pdflatex x2)
pdflatex main.tex
bibtex main
pdflatex main.tex
pdflatex main.tex
```

## Why Multiple Passes?

1. **First pdflatex**: Generates `.aux` file with citation keys
2. **bibtex**: Processes references and generates `.bbl` file
3. **Second pdflatex**: Incorporates bibliography into document
4. **Third pdflatex**: Resolves all cross-references and page numbers

## Alternative: Using latexmk

If you have `latexmk` installed (automates multiple passes):

```bash
latexmk -pdf main.tex
```

## File Structure

```
gerry-compactness-tradeoff/
├── main.tex                  # Main document with preamble
├── references.bib            # Bibliography entries
├── sections/
│   ├── 01_introduction.tex   # Section 1 (4-5 pages)
│   ├── 02_background.tex     # Section 2 (3-4 pages)
│   ├── 03_methodology.tex    # Section 3 (8-9 pages)
│   ├── 04_results.tex        # Section 4 (7-8 pages)
│   ├── 05_discussion.tex     # Section 5 (10-11 pages)
│   ├── 06_limitations.tex    # Section 6 (3 pages)
│   ├── 07_related_work.tex   # Section 7 (2.5 pages)
│   └── 08_conclusion.tex     # Section 8 (2 pages)
├── results/
│   ├── *.png                 # Visualization figures
│   └── *.csv                 # Data tables
└── scripts/
    └── *.py                  # Experiment scripts
```

## Expected Output

**Total Pages**: ~40-48 pages (double-spaced)

**Sections**:
- Abstract: 1 page
- Introduction: 4-5 pages
- Background: 3-4 pages
- Methodology: 8-9 pages
- Results: 7-8 pages
- Discussion: 10-11 pages
- Limitations: 3 pages
- Related Work: 2.5 pages
- Conclusion: 2 pages
- References: 2-3 pages

## Common Issues

### Missing Citations

If you see `[?]` in the compiled PDF:
1. Check that the citation key exists in `references.bib`
2. Run `bibtex main` to regenerate bibliography
3. Run `pdflatex main.tex` twice more

### Missing Figures

If figures are referenced but not displayed:
1. Ensure PNG files are in `results/` directory
2. Check that `\includegraphics` paths are correct
3. Use `\graphicspath{{results/}}` in preamble if needed

### Package Errors

If LaTeX complains about missing packages:
- Install TeX Live (full) or MikTeX
- Most required packages: `amsmath`, `graphicx`, `natbib`, `hyperref`, `booktabs`

## Output Files

After successful compilation:
- `main.pdf` - Final compiled paper
- `main.aux` - Auxiliary file (cross-references)
- `main.bbl` - Processed bibliography
- `main.blg` - Bibliography log
- `main.log` - Compilation log
- `main.out` - Hyperref outlines

## Cleanup

To remove auxiliary files:
```bash
rm main.aux main.bbl main.blg main.log main.out
```

Or with latexmk:
```bash
latexmk -c
```

## Next Steps

1. **Add figures**: Create LaTeX figure environments for PNG visualizations
2. **Add tables**: Convert CSV data to LaTeX tables
3. **Add authors**: Fill in author names and affiliations in `main.tex`
4. **Proofread**: Check for typos, formatting, and consistency
5. **Submit**: Export to journal-specific format (if needed)
