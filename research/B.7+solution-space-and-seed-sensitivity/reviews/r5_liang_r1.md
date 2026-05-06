---
reviewer: Percy Liang
round: 1
score: 3
date: 2026-05-05
---

## Summary

B.7 is the most data-intensive paper in the B-series, sweeping 10,000 seeds per state across 50 states to characterise the METIS solution space. The experimental design is thorough, the metrics are well-chosen, and the findings are clearly presented. From an empirical methodology perspective, the paper is mostly sound but has a significant factual error in the abstract (the METIS call count), underreports uncertainty in the key metrics, and does not fit a distributional model to the last-improvement seed data despite using this distribution as the statistical foundation for the T=600 threshold.

## Strengths

- **Experimental scale and reproducibility.** 10,000 seeds per state, SHA-256 chain seeding (fully specified), 72 CPU-hours on a 16-core workstation, parallel-by-state execution — this is a fully reproducible experimental setup. The seed chain formula (s_i = SHA256(s_{i-1} || BE32(i))) is concrete and independently verifiable.
- **The CV-based sensitivity metric is appropriate.** Using CV = sigma/mu for within-state seed sensitivity correctly normalises for differences in edge-cut magnitude across states. The four-category breakdown (single-district, small, medium, large) is sensible and the category-level aggregation shows the expected pattern.
- **The epsilon-ball analysis is novel.** The observation that 83% ± 12% of seeds fall within 5% of the minimum is a useful characterisation of the near-optimal density. The state-level variation (89% for large states, 61% for GA/NC) is correctly attributed to the graph structure differences.

## Weaknesses / P1 Items (Required Fixes)

- **The abstract contains a 1000x error.** The abstract states "500 million total METIS calls." Section 3.1 states 10,000 seeds × 50 states = 500,000 total METIS calls. This is a three-order-of-magnitude error (500,000,000 vs. 500,000). This must be corrected; it currently makes the computational scale of the paper appear far larger than it is.
- **The last-improvement seed distribution is not modelled.** Section 4.4 reports last-improvement seed indices for specific states (GA: 489, NC: 1023, WI: 1023, FL: 329, MI: 319, TX: 298) but does not report summary statistics (mean, median, 95th percentile) for the full 50-state distribution. The T=600 threshold is claimed to be "justified" by this distribution, but without distributional statistics, readers cannot assess whether T=600 represents the 90th, 95th, or 99th percentile of the last-improvement index distribution. Report the full distribution statistics across all 50 states, and state explicitly what percentile T=600 represents.
- **Confidence intervals are absent for the key claims.** Table 2 (approximation gap) reports mean and std dev of gap but no confidence intervals. For the claim that the DIA seed gap (2.9% ± 1.7%) is statistically indistinguishable from the median gap (3.1% ± 1.8%), the paired t-test p-value (p = 0.41) is reported but the 95% CI on the difference is not. The 95% CI on the difference would allow readers to rule out practically significant differences even if the t-test is non-significant.

## P2 Items (Suggestions)

- **Fit a parametric model to the last-improvement seed distribution.** A geometric distribution (constant probability of improvement at each seed) or an exponential fit would provide a principled basis for the T=600 threshold. Even a simple non-parametric CDF plot of last-improvement indices across the 50 states would be informative.
- **Report full partisan variance data for all 43 non-trivial states.** Table 4 covers only Wisconsin and North Carolina. Table 5 (seat-count SD across states) reports aggregated SD by category. A full table showing min/max Democratic seats across 10,000 seeds for each of the 43 non-trivial states would be a valuable appendix contribution and would support the "any state" claim in the abstract.

## Score: 3 — Minor Revision

The 500M/500K error is a mandatory fix. The distributional characterisation of the last-improvement seed (P1.2) and the confidence interval reporting (P1.3) are important for the paper's credibility as an empirical study. These are all fixable without new experiments.
