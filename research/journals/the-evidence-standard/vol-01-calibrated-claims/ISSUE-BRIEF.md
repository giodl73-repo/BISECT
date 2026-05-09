---
journal: The Evidence Standard
volume: 1
title: "Calibrated Claims"
status: seed
arc-position: 8 of 10 — establishes how to prove it statistically
reader-promise: "A GerryChain ensemble tells you a plan is at the 0.3rd percentile. A calibrated test tells you the p-value is 0.004. Courts need the second number, not the first — and this volume shows how to get it."
target-reader: Expert witnesses, litigators, federal judges and their clerks
excluded-claims: No adoption mechanics. No policy advocacy. This volume is methodological.
terminal-connection: A federal rule requires courts to evaluate compliance. Calibrated statistical evidence is how courts measure whether a plan deviates from the algorithmic standard. This volume gives courts the tools.
---

## Reader Promise (expanded)

The standard use of redistricting ensemble evidence presents a percentile: "the enacted plan is more compact than 99.7% of valid plans." Courts have started asking the obvious follow-up: "what's the p-value?" This volume answers that question and explains why the raw percentile is not the same as a p-value. The effective sample size of a 1,000-step GerryChain ensemble is approximately 70 — not 1,000. The ESS-corrected p-value for the NC 2022 enacted map is 0.041 (borderline significant), not the implied p < 0.001. At 10,000 steps, it becomes 0.004. Courts need to know the difference.

The volume also introduces the Bayesian Detection Score (BDS): the probability that the enacted plan is genuinely in the extreme tail of the plan space, accounting for ensemble uncertainty. NC 2022: BDS = 0.97. Bisect algorithmic plan: BDS = 0.03.

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

- HERALD: "ESS" needs a plain-language translation — "how many independent observations the chain actually gives you"
- LOKI: S.2 BDS uses a uniform prior — must note prior sensitivity
- SIGMA: S.1 abstract/table consistency on TX p-value (0.11 at 1K, 0.03 at 10K) — already fixed but verify
