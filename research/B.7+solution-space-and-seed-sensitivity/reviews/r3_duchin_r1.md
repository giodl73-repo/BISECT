---
reviewer: Moon Duchin
round: 1
score: 3
date: 2026-05-05
---

## Summary

B.7 is a systematic empirical study of seed sensitivity in METIS redistricting, motivated by the legal-defensibility question of the DIA's single-seed specification. The paper sweeps 10,000 seeds per state across all 50 states and measures normalised edge cut and partisan outcomes. The experimental design is careful and the conclusions are appropriate in scope. From a redistricting mathematics perspective, the paper engages correctly with the ensemble literature (citing DeFord-Duchin-Solomon, Fifield et al., Autry et al.) and correctly distinguishes between METIS's compactness-optimising behaviour and the neutral-ensemble sampling approach. My main concerns are about the Gumbel-distribution claim in the discussion and the epsilon-ball interpretation.

## Strengths

- **The solution space characterisation is methodologically sound.** The choice of coefficient of variation (CV) as the seed sensitivity metric is appropriate for comparing sensitivity across states with different n and k values, since it normalises by the mean. The within-state 10,000-seed sweep captures the distribution of near-optimal solutions in a way that is directly comparable to ensemble methods.
- **The epsilon-ball analysis is a useful conceptual contribution.** Reporting that 83% ± 12% of seeds fall within 5% of the minimum edge cut is a useful characterisation of the density of near-optimal configurations. This is analogous to the "number of plans within X% of optimal" metric used in the ensemble redistricting literature.
- **The contrast with unrestricted ensemble methods.** Section 5.1 correctly distinguishes METIS's objective-driven convergence from DeFord et al.'s uniform sampling: "METIS optimises a specific objective rather than sampling uniformly from the feasible set; the objective concentrates solutions near a small number of near-optimal configurations." This is an important and correct observation.

## Weaknesses / P1 Items (Required Fixes)

- **The Gumbel model for seed distribution is claimed but not used.** The abstract mentions that "seed sensitivity is low" and the introduction references "the per-state 'last improvement seed' distribution provides the statistical foundation for the T=600 threshold." But the paper never fits a distributional model to the last-improvement seed data. Section 4.4 reports specific seed indices (Georgia: 489, North Carolina: 1023, etc.) without fitting a distribution. If the T=600 threshold is based on a distributional model (e.g., Gumbel for extreme values), that model should be stated and fitted. If T=600 was chosen empirically without a distributional model, the paper should say so. The task prompt identifies this as a key question: "Is the Gumbel model for seed distribution justified?" — the paper does not address this at all, neither justifying nor rejecting a Gumbel model.
- **The 2.9% DIA seed gap statistic lacks a proper distributional context.** Section 4.2 reports the DIA seed gap as 2.9% ± 1.7% and compares it to the median seed gap (3.1% ± 1.8%), concluding they are statistically indistinguishable (p = 0.41). But this paired t-test treats the 50 states as 50 paired observations of (DIA gap, median gap). For a fixed seed formula (the SHA-256 chain), the DIA seed for each state is a pseudo-random but deterministic value — it is not drawn from the same distribution as the median gap. The correct comparison is: how often does the DIA seed outperform the median seed, and by how much? A sign test or Wilcoxon signed-rank test on the differences would be more appropriate than a t-test.
- **The 500M claim in the abstract is wrong.** The abstract states "500 million total METIS calls." Section 3.1 states "10,000 seeds per state × 50 states = 500,000 total METIS calls." 10,000 × 50 = 500,000, not 500,000,000. This is a three-order-of-magnitude error in the abstract. Fix it.

## P2 Items (Suggestions)

- **Fit a distributional model to the last-improvement seed index.** A Gumbel or exponential fit to the last-improvement seed distribution across states would provide a principled basis for the T=600 threshold. Even a simple moment comparison (mean and variance of the last-improvement index) would be informative. This is the paper's main unfinished business.
- **Report the full distribution of approximation gaps, not just the mean and SD.** The 83% epsilon-ball result is useful, but a histogram or CDF of seed-level approximation gaps for at least one state (e.g., Georgia, the highest-variance state) would be more informative for redistricting practitioners.

## Score: 3 — Minor Revision

The 500M/500K error (P1.3) must be fixed. The Gumbel model gap (P1.1) and the distributional context for the 2.9% figure (P1.2) are substantive but fixable. The paper is otherwise well-executed and appropriate in scope.
