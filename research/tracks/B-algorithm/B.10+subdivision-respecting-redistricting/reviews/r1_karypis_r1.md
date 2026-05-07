---
reviewer: George Karypis
round: 1
score: 3
date: 2026-05-05
---

## Summary

B.10 introduces county-sticky edge weighting for subdivision-respecting redistricting, sweeps alpha_c in {1.0, 1.5, 2.0, 3.0, 5.0, 10.0} across 50 states, and identifies alpha_c = 3.0 as the Pareto-optimal choice. The paper is well-motivated, the formulation is correct, and the case studies are concrete and informative. From a graph-partitioning perspective, the county-sticky approach is a natural extension of weighted METIS and is technically sound. The Pareto-elbow identification is reasonable, though the claim that alpha_c = 3.0 is "the clear Pareto-optimal choice" would benefit from a more formal Pareto analysis and sensitivity testing.

## Strengths

- **The formulation is technically correct.** The county-sticky weight w_E'(u,v) = w_E(u,v) * alpha_c (intra-county) / 1.0 (inter-county) preserves planarity, connectivity, and positive edge weights, so METIS's correctness guarantees hold without modification. The proof that planarity and connectivity are preserved follows trivially from the fact that no edges are added or removed.
- **The case studies are calibrated correctly.** Iowa (k=4, 99 counties) and Texas (k=38, 254 counties) are the right extremes to test: Iowa has a low district-to-county ratio where county preservation is most achievable, while Texas has many unavoidable splits. California correctly identifies that most splits are unavoidable due to large county populations, and the algorithm correctly avoids only the 3 unnecessary splits.
- **The marginal return analysis is clear.** The Pareto-frontier description (11.2% split reduction per 1% PP cost at alpha_c = 1.0-3.0, dropping to 1.9% at 3.0-5.0 and 0.7% at 5.0-10.0) is a clean presentation of the elbow location. The 6x drop in marginal return at alpha_c = 3.0 is a defensible criterion for elbow identification.

## Weaknesses / P1 Items (Required Fixes)

- **The alpha_c = 3.0 Pareto elbow is identified qualitatively, not formally.** Section 4.1 argues that the elbow is at alpha_c = 3.0 based on the ratio of split reduction per unit PP cost. But the Pareto frontier is defined over two objectives (county splits, PP), and the "elbow" is a concept from multi-objective optimisation that can be formalised (e.g., using the angle between adjacent frontier segments, or the normalised distance from the ideal point). The paper should either (a) use a formal elbow-detection criterion and report the criterion value, or (b) show a plot of the Pareto frontier with the elbow visually identified. A table of marginal returns is not a Pareto frontier plot.
- **The county split count metric is inconsistently defined.** The paper defines "county splits" as county-district pairs (c, d) where district d intersects county c but is not fully contained. But the reduction from 487 to 323 is in "county splits" while the reduction from 312 to 198 is in "multi-county districts." These are different metrics (one counts county-district pairs, the other counts districts). The abstract says "county splits fall by 34% (487 → 323) and multi-county districts fall by 37% (312 → 198)." It is not immediately clear why there are 487 county splits but only 312 multi-county districts: this would imply that some counties are split into 3+ districts (adding extra pairs). The relationship between these two metrics should be explained explicitly, and the paper should state the baseline number of counties that are split into exactly 2 districts vs. 3+ districts.
- **Seed sensitivity at alpha_c = 3.0 is uncharacterised.** The methodology uses the DIA seed (s_0) for all 50 states. But B.7 shows that GA and NC have CV ≈ 4% for seed sensitivity at alpha_c = 1.0. If county-sticky weighting changes the landscape of near-optimal separators (by making county boundaries harder to cross), the seed sensitivity at alpha_c = 3.0 may differ from B.7's results. The paper should either report seed sensitivity at alpha_c = 3.0 for the high-variance states (GA, NC), or explicitly defer this to future work and hedge the results accordingly.

## P2 Items (Suggestions)

- **Plot the Pareto frontier (county splits vs. mean PP) for all six alpha_c values.** A 2D scatter plot of (county splits, mean PP) at each alpha_c, with the Pareto frontier drawn and the elbow marked, would make the alpha_c = 3.0 recommendation visually evident. The current presentation (Table 1 with marginal return ratios) is correct but less intuitive than a plot.
- **Evaluate the two-level hierarchy (county + municipal weights) on one state.** The discussion mentions alpha_m ≈ 1.5 for municipal boundaries as future work. A preliminary result for Iowa (which has clear municipal preservation requirements) would make the extension concrete.

## Score: 3 — Minor Revision

The qualitative Pareto elbow identification (P1.1), the metric relationship explanation (P1.2), and the seed sensitivity caveat (P1.3) require attention. None of these require new experiments. The core contribution is sound.
