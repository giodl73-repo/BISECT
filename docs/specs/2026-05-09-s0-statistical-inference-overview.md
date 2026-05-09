---
title: "Statistical Inference in Redistricting: Challenges and Framework"
series: S.0
status: Planned
date: 2026-05-09
track: S-statistical-inference
---

## Claims
1. Four statistical challenges are unique to redistricting: (a) the test statistic is a function of the entire plan (not a scalar), (b) the null distribution requires simulating the plan space, (c) multiple metrics create multiple testing problems, and (d) the ensemble is often too small for precise tail estimation.
2. The existing program addresses challenge (b) via the GerryChain ensemble (G.1) and SMC (G.7), and challenge (a) via metric reduction (K, L tracks). Challenges (c) and (d) are unaddressed. Track S fills both.
3. A unified statistical framework for redistricting inference has three components: (i) a calibrated null distribution from the ensemble, (ii) a test statistic that combines multiple metrics via FDR control, (iii) a power analysis specifying the minimum ensemble size for reliable detection.
