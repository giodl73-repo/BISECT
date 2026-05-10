---
journal: The Evidence Standard
volume: 1
title: "Calibrated Claims"
status: seed
arc-position: 8 of 10 — establishes how to prove it statistically
reader-promise: "Raw ensemble percentiles are not automatically court-ready evidence. This volume tests how to calibrate percentiles into statistical claims with uncertainty."
target-reader: Expert witnesses, litigators, federal judges and their clerks
excluded-claims: No adoption mechanics. No policy advocacy. This volume is methodological.
terminal-connection: Any enforceable algorithmic standard needs a defensible evidence method. This volume auditions the statistical toolkit.
---

> Seed status: p-values, BDS scores, power claims, and multiple-testing claims require SCALE review before publication.

## Reader Promise (expanded)

The standard use of redistricting ensemble evidence presents a percentile: "the enacted plan is more compact than 99.7% of valid plans." Courts may ask the follow-up: "what's the p-value?" This volume auditions a method for answering that question and explains why the raw percentile is not the same as a p-value. Effective sample size, convergence, chain design, and multiple testing all need to be visible before any claim becomes public.

The volume also auditions the Bayesian Detection Score (BDS): a candidate way to express the probability that a plan is genuinely in the extreme tail of the plan space, accounting for ensemble uncertainty.

## Slots (6-8 candidates, 2-3x pool)

| Slot | Role | Claim class | Candidate papers |
|------|------|-------------|-----------------|
| Editorial | The p-value question — what courts need and what the literature currently delivers | interpretive | — |
| Formula | The ESS correction — from percentile to calibrated p-value | formal | S.1 (hypothesis testing) |
| Bayesian | The BDS — posterior probability the plan is genuinely extreme | formal | S.2 (Bayesian redistricting) |
| Metrics background | What we're testing: PP, efficiency gap, partisan bias — the vocabulary | operational | K.7 §overview, L.0 §framework |
| Power | Minimum ensemble size for reliable significance — 5K steps NC, 30K steps CA | empirical | S.3 (power analysis) |
| Multiple testing | 1,050 L-track tests, FDR correction — which findings survive | empirical | S.4 (multiple testing) |
| Legal bridge | How to present calibrated p-values in expert witness testimony — the Daubert gate | operational | R.4 §daubert, S.1 §conclusion |

## Review lenses

- COMMONS: "ESS" needs a plain-language translation — "how many independent observations the chain actually gives you"
- SCALE: S.2 BDS uses a uniform prior — must note prior sensitivity
- DATUM: S.1 abstract/table consistency on TX p-value (0.11 at 1K, 0.03 at 10K) — already fixed but verify
