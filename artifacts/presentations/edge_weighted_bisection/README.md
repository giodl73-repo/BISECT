# Edge-Weighted Recursive Bisection Presentation Materials

This directory contains presentation materials explaining our edge-weighted recursive bisection algorithm for congressional redistricting.

## Contents

1. **`presentation.tex`** - Beamer slides (PowerPoint-style presentation)
   - Navigable slides introducing graph concepts
   - Alabama and Minnesota examples
   - Visual explanations of the algorithm

2. **`laymen_guide.tex`** - Approachable document for general audiences
   - Written for non-technical readers (family, map enthusiasts)
   - Less formal than academic papers
   - Clear examples and visual explanations

3. **`figures/`** - Directory for diagrams and images

## Compiling

Both documents use LaTeX. To compile:

```bash
# Presentation slides
pdflatex presentation.tex
pdflatex presentation.tex  # Run twice for navigation

# Layman's guide
pdflatex laymen_guide.tex
pdflatex laymen_guide.tex  # Run twice for references
```

Or use your preferred LaTeX editor (TeXworks, Overleaf, etc.).

## Converting to PowerPoint

If you prefer PPTX format, you can:
1. Convert PDF to PPTX using Adobe Acrobat or online converters
2. Use the PDF directly (most presentation software supports PDF)
3. Recreate slides in PowerPoint using the content as a guide
