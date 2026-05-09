---
title: "Power Analysis for Gerrymandering Detection: How Many Plans Do You Need?"
series: S.3
status: Planned
date: 2026-05-09
track: S-statistical-inference
---

## Claims
1. The minimum GerryChain ensemble size for 80% power to detect a 10pp partisan advantage at 5% significance (the typical litigation standard) is approximately 5,000 steps for NC (k=14) and 30,000 steps for CA (k=52).
2. Current practice in redistricting litigation uses 10,000-step ensembles. This provides 80% power for NC but only 45% power for CA — meaning CA litigation frequently uses underpowered ensembles.
3. The power function depends on: (a) the true partisan advantage (effect size), (b) the ensemble's ESS (from G.4), and (c) the test statistic's variance under the null. S.3 provides power tables for all 50 states.
4. For litigation purposes, S.3 recommends a "litigation-grade" ensemble minimum: 20,000 steps for states with k≤17, 50,000 steps for states with k>17 (TX, CA, NY, FL). The bisect pipeline's H.2 implementation makes these run times feasible.
