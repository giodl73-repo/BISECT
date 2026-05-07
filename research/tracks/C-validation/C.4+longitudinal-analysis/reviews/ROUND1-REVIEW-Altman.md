# Round 1 Review: Twenty Years of Congressional Redistricting

**Reviewer**: Micah Altman (MIT Libraries)
**Date**: 2026-02-08
**Round**: 1
**Previous Score**: 3.0/4.0
**Current Score**: 3.5/4.0

---

## Summary Assessment

The authors have made commendable progress on reproducibility infrastructure. The addition of the Data and Code Availability section with GitHub and Zenodo commitments, combined with strengthened statistical documentation, substantially improves the paper's transparency. I raise my score from 3.0 to 3.5. The remaining reproducibility gaps are procedural (actual repository setup) rather than conceptual.

## Assessment of P1 Revisions

### P1.4: Data and Code Availability (Addressed ✓)

**Original concern**: No data/code access plan undermined reproducibility.

**Author response**: Added "Data and Code Availability" section with GitHub repository, Zenodo DOI, and Methods Supplement commitments.

**Assessment**: **Conceptually addressed, procedurally incomplete**. The new section (between Acknowledgments and Bibliography) commits to:
- GitHub repository: `https://github.com/username/congressional-redistricting` (placeholder URL)
- Zenodo archive: DOI `10.5281/zenodo.XXXXXXX` (to be assigned)
- Methods Supplement: "computational details, graph construction algorithms, METIS parameters, validation procedures"
- Data sources: Census TIGER/Line files, PL 94-171, Dave's Redistricting App

**Strengths**:
- Comprehensive scope: code, data, methods supplement
- Appropriate venues: GitHub (code), Zenodo (data archive with DOI)
- Source transparency: Census sources documented

**Gaps**:
- URLs are placeholders ("username" and "XXXXXXX")
- Says "to be published upon acceptance"—consider pre-publication access via anonymous repositories for review
- No mention of software environment (Python version, dependencies, METIS version availability)

**Recommendation**: Create repositories now, share anonymized links with reviewers. This allows verification during review, not post-acceptance.

### P1.2: Statistical Documentation (Addressed ✓)

**Original concern**: Robustness testing needed to validate commission effectiveness claim.

**Author response**: Added comprehensive robustness checks subsection with alternative metrics, outlier sensitivity, confound controls, temporal analysis.

**Assessment**: **Excellent**. The robustness subsection (5.4.1) provides reproducible statistical evidence:
- Effect size: Cohen's d = 1.64 (large effect)
- Precision: 95% CI [2.1pp, 12.5pp]
- Alternative metric: Reock compactness (+6.8pp, t=2.54, p=0.014)
- Outlier sensitivity: CA exclusion (+2.7pp, p=0.041), NY exclusion (+4.1pp, p=0.003)
- Confound controls: Multivariate regression (β=6.9pp, p=0.009) controlling for state size, prior compactness, swing state status
- Pre-trend check: 2000-2010 change for commission states was -0.8%

This level of statistical transparency enables reproducibility. A reader could replicate the analysis from this description.

**Minor note**: The regression specification isn't fully detailed (linear? logistic? interaction terms?). The Methods Supplement should include the full regression equation.

### P1.3: Methodological Transparency (Partially Addressed via VRA Discussion)

**Original concern**: Algorithm presented as objective without acknowledging parameter choices affect outcomes.

**Author response**: VRA discussion frames algorithmic approach as "pre-VRA baseline" requiring post-processing, acknowledging algorithms embed choices.

**Assessment**: **Indirectly addressed**. The VRA subsection (3.2.1) acknowledges that purely geometric optimization ignores legally mandated considerations, implying the algorithm isn't fully objective—it optimizes one objective (compactness) while excluding others (representation, communities). This is progress.

**Remaining gap**: The methodology section doesn't discuss parameter sensitivity. How does α=5.0 for minority-minority edges affect outcomes? What happens with α=10 or α=2? A 2-3 sentence acknowledgment that parameter choices embed assumptions would complete this.

## Reproducibility Assessment

### What Can Be Reproduced Now

From the paper alone, a reader could:
- Obtain census tract shapefiles (TIGER/Line files documented)
- Download population data (PL 94-171 files documented)
- Download enacted districts (Dave's Redistricting App documented)
- Understand METIS parameters (Algorithm section: recursive bisection, ±0.5% population balance, α=5.0 edge weights)
- Replicate statistical tests (formulas, test statistics, p-values provided)

### What Cannot Be Reproduced Yet

Without code/data repository:
- Graph construction from census tracts (adjacency detection algorithm not detailed)
- METIS invocation (command-line flags, edge weight encoding, random seed)
- Compactness calculations (Polsby-Popper formula given, but perimeter calculation method unspecified—geodesic? planar?)
- IoU calculations (intersection area computation—which GIS library? projection?)

The Methods Supplement will address these, but reviewers can't verify now.

## Minor Issues

### Data Transparency

1. **Census tract counts**: The paper reports 66,304 (2000), 74,002 (2010), 85,331 (2020) tracts. Where do these numbers come from? A citation to Census Bureau documentation would clarify.

2. **Enacted district source**: "Dave's Redistricting App" is informal. The formal source is the All About Redistricting project (Justin Levitt). Cite properly.

3. **Missing data**: Does every census tract have valid geometry and population? Any tracts excluded (e.g., water-only tracts)? Document preprocessing.

### Methods Supplement Contents

The promised Methods Supplement should include:
- Complete software environment (Python version, library versions, OS)
- Full graph construction pseudocode
- METIS command-line invocation
- Validation procedures (contiguity checking, population balance verification)
- Regression specifications (full equations)
- Figure generation scripts

Consider making this an appendix rather than separate supplement—keeps everything in one document.

### Software Preservation

GitHub + Zenodo is good for code/data, but METIS 5.1.0 is external software. Consider:
- Documenting METIS availability: `http://glaros.dtc.umn.edu/gkhome/metis/metis/download`
- Archiving METIS binary in Zenodo alongside your code (permissible under Apache License 2.0)
- Providing Docker container with entire software environment (ultimate reproducibility)

## Verdict

**Accept with minor revisions**. The P1 improvements establish reproducibility infrastructure. The remaining gaps (repository creation, Methods Supplement details) are procedural and don't block publication conceptually. Once repositories are live, this will be a model of computational transparency in redistricting research.

**Recommendation**: Before resubmission, create the repositories and update placeholder URLs. Share anonymized links with reviewers so they can verify reproducibility during review.

**Publication venue**: This is ready for **Political Analysis** or **Journal of Computational Social Science**. Both expect computational reproducibility, which this paper now provides.

Well done on taking reproducibility seriously.
