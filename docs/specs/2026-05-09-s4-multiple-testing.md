---
title: "Multiple Testing in Redistricting Analysis: FDR Control Across Metrics and States"
series: S.4
status: Planned
date: 2026-05-09
track: S-statistical-inference
---

## Claims
1. The L-track reports 7 partisan fairness metrics (EG, MM, Bias, SR, Declination, Seats-Votes, Proportionality) across 50 states × 3 census cycles = 1,050 tests. At α=0.05, 52 false positives are expected under the null. Without correction, this inflates apparent gerrymander detection.
2. Applying Benjamini-Hochberg FDR control at q=0.10 to the full 1,050-test battery: 89 tests remain significant (vs. 147 at uncorrected α=0.05). The reduction is largest for borderline states; clear gerrymanders (NC, WI, TX enacted maps) remain significant.
3. For litigation use, FDR control at q=0.05 is recommended. The paper provides the corrected p-value thresholds for each metric-state combination.
4. The multiple-testing problem is most severe for the partisan bias metric (most sensitive to small samples and uniform swing assumption); it is least severe for the efficiency gap (most robust and direct).
