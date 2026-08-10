---
pulse: 17
title: National bakeoff paper integration
status: complete
wave: nrs-2010-national-baseline
validation_level: L1 paper and evidence-routing alignment
---

# Pulse 17 - National Bakeoff Paper Integration

## Purpose

Route the completed NRS v0.3 national Tier 1 and Tier 2 bakeoff evidence into
the research portfolio without treating unavailable matrix cells or legacy
tract-profile comparisons as governed NRS results.

## Deliverables

- [x] Publish one paper-facing evidence matrix with completed and gated cells.
- [x] Integrate Tier 1 assignment/subdivision and Tier 2 geometry evidence into
  U.21, A.0, and A.5.
- [x] Separate K-series tract/algorithm fixtures from the governed common-block
  national geometry contract.
- [x] Reclassify B.0's four-State, eight-configuration matrix as exploratory
  where it uses estimated or pending cells.
- [x] Remove B.0 section terminators that truncated the indexed PDF after its
  introduction.
- [x] Rebuild and publish all five affected PDFs.
- [x] Update the paper index and VTRACE evidence inventory.

## Result

The paper-facing matrix now records Tier 1 and Tier 2 as complete and routes
elections, demographics, converged block-level ensembles, alternative BISECT
structures, and non-enacted comparators to separate future gates.

U.21, A.0, and A.5 report the governed 2020 common-block comparison. K.0
explicitly distinguishes its three-State, four-algorithm, single-seed fixtures
from the national block-projected geometry contract. B.0 now identifies its
mixed confirmed/estimated/pending matrix as exploratory and no longer converts
those cells into causal, superiority, or legal-compliance conclusions.

The rebuilt B.0 PDF contains the full 33-page paper; the prior indexed PDF was
a 5-page truncation caused by stray `\end{document}` commands in included
section files.

## Validation

```powershell
git diff --check
pdflatex -interaction=nonstopmode main.tex
bibtex main
pdflatex -interaction=nonstopmode main.tex
pdflatex -interaction=nonstopmode main.tex
pdfinfo docs/papers/<paper>.pdf
pdftotext docs/papers/<paper>.pdf -
```

All five LaTeX logs contain no errors, undefined references, or undefined
citations. Extracted PDF text contains the expected matrix claims and claim
boundaries.

## Remaining Gates

- independent external NRS v0.3 replication;
- frozen election and demographic inputs/protocols;
- converged block-level ensemble execution;
- preregistered national alternative-BISECT-structure comparison; and
- source-bound non-enacted national comparator families.

## Claim Boundary

The completed paper work documents a descriptive NRS v0.3 versus official
CD118 comparison on a common 2020 Census-block universe. It does not establish
original enacted linework, compactness superiority, fairness, intent, VRA
compliance, community preservation, robustness, optimality, causation, legal
validity, or adoption suitability.
