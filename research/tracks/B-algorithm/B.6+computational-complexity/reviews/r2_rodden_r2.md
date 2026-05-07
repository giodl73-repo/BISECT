---
reviewer: Jonathan Rodden
round: 2
score: 3
date: 2026-05-05
---

## Summary

The revision addresses all three of my P1 concerns from Round 1. The
approximation ratio is no longer stated as a formal guarantee for METIS;
the discussion now correctly distinguishes the theoretical landscape (ARV
for Sparsest Cut) from the empirical performance (within 3% of best of
10,000 seeds). The OLS regression now excludes k=1 states. The P=NP
framing is now qualified for non-technical audiences.

From a political science and policy perspective, the paper is now
substantially cleaner. The legal arguments are more defensible because
they no longer rest on a formal approximation claim that could not be
verified in deposition.

## Strengths

- **The approximation discussion is now legally defensible.** The revised
  Section 5.2 correctly separates the theoretical landscape (no known
  polynomial-time algorithm achieves better than O(log n) for balanced
  k-partition in general) from the empirical finding (METIS within 3% of
  best of 10,000 seeds). This is the right framing for expert witness
  testimony: it does not overclaim a formal 4.1× worst-case bound that
  has never been observed empirically.
- **The P=NP qualification is well-placed.** The addition in Section 5.1
  ("P = NP is universally believed to be false among theoretical computer
  scientists; making the NP-hardness result effectively unconditional for
  all practical purposes") is exactly what was needed for a legal audience.
- **The feasibility vs. optimisation distinction is clear.** The new remark
  after Theorem 1 explicitly distinguishes finding any valid partition
  (trivially feasible via spanning tree) from finding the optimal one
  (NP-hard). This is important for courts that might otherwise conflate
  "NP-hard redistricting" with "redistricting is impossible."

## Remaining Concerns (P2 — Suggestions)

- **The 2030 Census projection is still absent.** I raised this in Round 1
  as a P2 item and it remains unaddressed. The paper notes the expected
  78,000–82,000 tract count for 2030 and has a fitted scaling law — a
  concrete runtime projection for 2030 would be directly useful for DIA
  planning. This is a one-sentence calculation.
- **The B.7 cross-reference could be more prominent.** The revised text
  correctly references B.7 for the 3% empirical gap, but only in passing.
  A more explicit statement — "B.7 shows that the DIA's fixed seed (42)
  produces maps within 3% of the best among 10,000 seeds, providing
  empirical evidence that the heuristic quality is adequate for legal
  purposes" — would strengthen the argument for a policy audience.
- **The population heterogeneity robustness check was not addressed.**
  I asked in Round 1 whether population heterogeneity (variance of tract
  populations within a state) predicts runtime residuals. This is still
  missing. It is a P2 item but would make the OLS fit more credible.

## Score: 3 — Minor Revision

All P1 items are addressed. The remaining issues are P2 suggestions that
would strengthen the policy utility of the paper. The core theoretical
and empirical contributions are now sound.
