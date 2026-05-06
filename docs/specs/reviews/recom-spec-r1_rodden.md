---
reviewer: Jonathan Rodden
spec: redist-ensemble ReCom crate
round: 1
score: 3
date: 2026-05-06
---

## Summary
The spec is technically motivated but does not adequately situate the proposed ensemble tool within the landscape of research questions the G-series papers are actually trying to answer. The 50th-percentile NC cut-fraction result is presented as a positive validation, but its interpretation depends heavily on which metric is being assessed — and the spec conflates compactness (cut-fraction) with partisan outcome, which are different distributions with different geographic drivers.

## Strengths
- The auditing framing ("does the enacted plan fall in the middle of the distribution?") is exactly the right research question for the G-series papers. Positioning the tool primarily as an auditor rather than a generator is correct and avoids the legal risk of appearing to generate candidate plans.
- The JSON output format includes both the pooled distribution statistics and the plan's percentile rank, which is the minimal sufficient output for an ensemble audit report. This is well-designed.
- The use of cut-fraction as the convergence diagnostic is appropriate: it is a continuous, well-behaved statistic that responds to the spanning tree structure and is the natural METIS analog.

## P1 — Required changes

- **The "compactness extremum" finding (0.1–0.7th percentile) is mentioned in the motivation but not explained in the spec.** If the METIS-generated plans fall at the 0.1th percentile of the ReCom cut-fraction distribution, this means the METIS plans are *far more compact* than typical random plans — they are outliers in the direction of over-compactness, not gerrymandering. This is a significant and non-obvious finding that requires an explicit section in the spec explaining: (a) whether "more compact than 99.9% of random plans" is a feature or a problem; (b) whether this result generalises across all 50 states or is specific to NC; and (c) how it affects the interpretation of the G-series papers' partisan outcome claims. A plan at the 0.1th percentile of compactness is not a "typical" plan — it is an extreme one, and the spec must address what this implies for the research agenda.

- **The spec does not distinguish between auditing the enacted plan vs. auditing the METIS-generated plan.** These are two different research questions. The enacted plan audit (is the legislature's map an outlier?) is the legally relevant question. The METIS plan audit (is our generated map a fair representative of the feasible space?) is the methodological validation question. The spec currently presents both as equivalent uses of the same tool, but they require different framing, different comparison baselines, and different interpretive standards. The spec must add a section that explicitly separates these two use cases.

- **The 50th-percentile NC result for cut-fraction does not validate that the METIS plan is "unbiased" on partisan outcomes.** Geographic sorting means that compactness-optimal plans in states like NC will tend to crack urban Democratic concentrations, producing Republican-leaning outcomes that are compact but partisan. The spec should not use the 50th-percentile cut-fraction result as evidence of partisan neutrality — it is evidence of compactness neutrality, which is a different claim. The spec must add a caveat distinguishing cut-fraction percentile from partisan-outcome percentile, and should note that the G-series papers require a separate ensemble analysis on partisan statistics (D_seats, efficiency gap) not covered by this spec.

## P2 — Suggested improvements

- Add a discussion of what "convergence" means for partisan statistics vs. compactness statistics. ESS and R-hat are designed for continuous, approximately Gaussian statistics. Partisan seat counts are discrete and bounded (0 to k). The spec should acknowledge that 10,000 steps may be sufficient for cut-fraction but insufficient for seat-count distributions in competitive states where one or two swing districts determine the outcome.

- Consider adding a geographic-sorting diagnostic to the output: the Moran's I of the residual partisan lean after accounting for the ensemble distribution. This would directly address the Rodden critique that compact plans in sorted states are not neutral plans.

## Score: 3/4
The spec delivers a sound implementation plan but leaves the most important interpretive question — what does the compactness extremum mean for the G-series research claims? — entirely unaddressed. One focused revision pass can fix this.
