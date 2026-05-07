---
reviewer: Percy Liang
round: 1
score: 3
date: 2026-05-05
---

## Summary

B.10 presents county-sticky edge weighting as a practical approach to subdivision-respecting redistricting. The formulation is clean, the experimental design is systematic, and the results are well-documented. From an empirical methodology perspective, the paper is mostly solid: the parameter sweep covers a reasonable range, the metrics are appropriate, and the case studies are well-chosen. My primary concerns are about the coarse sweep grid (only 6 values of alpha_c, with no sensitivity analysis around the recommended alpha_c = 3.0), the reproducibility of the county split counts, and the absence of uncertainty quantification in the aggregate results.

## Strengths

- **The experimental design is transparent and reproducible.** The 50-state × 6 alpha_c sweep (300 total runs) is clearly specified: DIA seed (s_0), TIGER/Line county FIPS mapping, fixed METIS configuration. The methodology section is specific enough that an independent researcher could replicate the sweep.
- **The case study selection is appropriate.** Iowa (low k, many small counties), Texas (high k, heterogeneous county sizes), and California (high k, many large counties) cover the three qualitatively different regimes for county preservation. The case studies correctly validate the model's predictions: Iowa benefits most, California least.
- **The three-metric outcome set is complete.** County splits, multi-county districts, and perfectly-preserved counties provide a consistent picture. The population deviation metric (0.41% to 0.44%) correctly verifies that the constitutional constraint is not violated.

## Weaknesses / P1 Items (Required Fixes)

- **The alpha_c sweep is too coarse to reliably locate the elbow.** Six values of alpha_c ({1.0, 1.5, 2.0, 3.0, 5.0, 10.0}) provide only one data point between 2.0 and 5.0 (at 3.0). The marginal return analysis claims a 6x drop at alpha_c = 3.0, but this comparison is between (1.0-3.0) and (3.0-5.0) — a factor-of-3 jump on the left and a factor-of-1.67 jump on the right. The apparent "elbow" at 3.0 may be an artifact of the coarser grid on the right side. At minimum, add alpha_c ∈ {2.5, 3.5} to the grid and recompute the marginal return. If the elbow stays at 3.0, the DIA default is confirmed. If it shifts to 2.5 or 3.5, the DIA default may need recalibration.
- **The county split counts are not reported with uncertainty.** The paper runs one seed per state per alpha_c value. County splits at a given alpha_c are therefore deterministic for a fixed seed, but seed-to-seed variation (from B.7's finding of CV ≤ 4% for edge cut) could propagate to county split variation. If different seeds produce materially different county split counts at alpha_c = 3.0, the 487→323 headline figure may not be stable. The paper should report whether the 5 seeds closest to the DIA seed (i=0, 1, 2, 3, 4) produce consistent county split counts for Iowa and Texas.
- **The aggregate county split count (487 nationally) is not decomposed by state.** The paper reports 487 total county splits at baseline and 323 at alpha_c = 3.0. But the distribution across states is not shown — only the Iowa and Texas case studies are provided. A full 50-state table showing baseline splits, sticky splits, and reduction by state would make the aggregate claim verifiable and would show which states benefit most from county-sticky weighting.

## P2 Items (Suggestions)

- **Add a Pareto frontier plot.** A 2D scatter plot of (county splits, mean PP) for the 6 alpha_c values with the Pareto frontier drawn would make the elbow location visually clear and would be more convincing than the tabular marginal return analysis.
- **Test at block-group resolution for Iowa.** The paper uses census-tract resolution for the entire sweep. Block-group resolution would capture county boundaries more precisely (county lines often follow road and creek boundaries that are sub-tract-level features). For Iowa, where county preservation is the primary goal, block-group resolution might show higher split reduction at the same alpha_c.
- **Report seed sensitivity of county splits at alpha_c = 3.0 for Georgia and North Carolina.** B.7 identified these as high-seed-variance states. If county-sticky weighting further increases seed sensitivity (by introducing additional near-optimal separator configurations at county boundaries), T=600 may be insufficient for these states under the county-sticky configuration.

## Score: 3 — Minor Revision

The coarse alpha_c grid (P1.1) is the primary methodological concern: the DIA default is specified as exactly 3.0, so demonstrating that the elbow is robustly at 3.0 (not 2.5 or 3.5) matters. The missing 50-state split table (P1.3) is a straightforward documentation gap. The seed sensitivity concern (P1.2) is important but can be deferred to future work with appropriate hedging.
