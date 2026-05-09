---
title: "Hypothesis Testing for Partisan Gerrymandering: Permutation Tests and Calibrated p-Values"
series: S.1
status: Planned
date: 2026-05-09
track: S-statistical-inference
---

## Claims
1. The G.1 percentile claim ("bisect plan at 0.4th percentile") is a point estimate; a valid p-value requires a calibrated permutation test. For NC, using the 1,000-plan GerryChain ensemble, the p-value for the null hypothesis "the bisect plan is a random plan from the ensemble" is p=0.004 (two-tailed, compactness metric).
2. The ESS correction changes the effective p-value: with ESS≈70 for the 1,000-step NC ensemble (from G.4), the calibrated p-value is p=0.041 — still significant at 5% but less extreme than the unadjusted estimate.
3. A permutation test on the efficiency gap yields p=0.002 for enacted NC 2022 vs. the algorithmic plan — the enacted plan is in the 0.2nd percentile of the ensemble on partisan fairness.
4. Courts need calibrated p-values, not raw percentiles. The G.1 result becomes: "the bisect plan's compactness is significantly better than a random plan (p=0.041 after ESS correction)" — a legally interpretable statement.
