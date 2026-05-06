---
reviewer: George Karypis
round: 1
score: 3
date: 2026-05-05
---

## Summary

B.7 characterises the solution space of METIS recursive bisection by sweeping 10,000 seeds per state across all 50 states and measuring normalised edge cut and partisan outcomes. The central empirical findings — CV < 2% for 48 states, DIA seed within 2.9% of minimum, partisan variance ≤ 2 seats — are plausible and well-presented. The paper is the most data-intensive in the B-series, and the 500,000-call sweep is a serious empirical undertaking. However, from a graph-partitioning perspective, the paper has technical gaps in how it characterises the solution space (the normalised edge cut metric is non-standard and underexplained), how it connects seed variance to the T=600 threshold, and whether the 10,000-seed sample is actually sufficient to characterise the tail of the distribution for the high-variance states.

## Strengths

- **Scale of the seed sweep.** 10,000 seeds per state × 50 states = 500,000 METIS calls is the largest seed sensitivity study I am aware of for redistricting graph partitioning. The systematic nature of the sweep (SHA-256 chain seeding, parallel-by-state execution) is methodologically sound.
- **The high-variance state identification.** Identifying Georgia and North Carolina as outliers (CV ≈ 4%) and providing a structural explanation (k=14 is neither a power of two nor highly composite, forcing asymmetric splits at every level) is a genuinely useful finding. The bisection tree analysis (7+7, then 4+3, etc.) is concrete and verifiable.
- **The partisan dominance finding.** Demonstrating that algorithm selection drives more partisan variance than seed selection (the 12.8 pp gap from B.0 vs. the ≤2-seat range across 10,000 seeds) is a clean and important result. The framing — "algorithm selection dominates seed selection as a determinant of partisan outcomes" — is precise and legally relevant.

## Weaknesses / P1 Items (Required Fixes)

- **The normalised edge cut metric is non-standard and needs more justification.** The paper defines EC_norm = EC / sqrt(min(|P1|, |P2|)) "following the GeoSection normalisation (B.8)." But for the seed sensitivity analysis, the relevant quantity is the edge cut itself (or the edge cut relative to the optimal for the given graph), not the GeoSection normalisation. The GeoSection normalisation divides by the square root of the minimum partition size to adjust for district count — a normalisation designed for cross-state comparison in B.8. For within-state seed sensitivity (where k and the graph are fixed across seeds), dividing by sqrt(min partition size) introduces noise unless the partition sizes are identical across seeds (which they won't be when k is non-power-of-two). The paper should either (a) use raw edge cut for within-state seed sensitivity, or (b) justify why the GeoSection normalisation is appropriate here.
- **The T=600 threshold justification is not statistically rigorous.** Section 4.4 reports "for 47 of 50 states, the last improvement occurs before seed index 600." But the "last improvement seed" is a random variable that depends on which of the 10,000 seeds happened to be tested. With 10,000 seeds, the last improvement for Georgia (last improvement at seed 489) means that at least one seed in {490, ..., 10,000} improved on the previous minimum — we don't know if seed 491, 600, or 2000 would produce an improvement with a fresh set of seeds. A frequentist argument would need to show: with probability ≥ 1-delta, no seed beyond index T produces an improvement, given the CV of the distribution. The current statement ("last improvement seed index 489") is a data point, not a guarantee. The T=600 claim needs a probabilistic bound.
- **The DIA seed's rank is reported as approximation gap, not rank percentile.** Section 4.2 reports the DIA seed gap as 2.9% ± 1.7%. But for the legal argument (the DIA seed is no better or worse than a random seed), the relevant statistic is the rank percentile of the DIA seed among 10,000 seeds per state. If the DIA seed consistently lands in the top 10% (or bottom 10%), that would be suspicious even if the absolute gap is small. Report the rank percentile distribution of the DIA seed across states.

## P2 Items (Suggestions)

- **Model the last-improvement seed distribution.** The convergence tail length is reported for individual states but not modelled. A geometric distribution or exponential fit to the "last improvement seed" distribution would provide a probabilistic basis for the T=600 threshold. If the tail is heavy (Pareto or log-normal), T=600 may be insufficient even for most states.
- **Report seed sensitivity for the alternative weight modes.** The limitations section acknowledges that county-sticky (B.10) and VRA-constrained (B.14) configurations may show different sensitivity profiles. Preliminary data on at least one alternative mode would strengthen the generalisability of the T=600 threshold across DIA configurations.

## Score: 3 — Minor Revision

The normalised edge cut justification (P1.1) and the T=600 probabilistic basis (P1.2) require additional work. The DIA seed rank percentile (P1.3) is a straightforward computation that would strengthen the legal argument. These fixes are feasible within the existing data.
