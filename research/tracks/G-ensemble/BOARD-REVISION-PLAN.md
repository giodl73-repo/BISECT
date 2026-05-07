# BOARD-REVISION-PLAN — Track G (Ensemble)
**Source**: REVIEW_BOARD.md Round 1 (2026-05-07)
**Track score**: 6.5/10 → target 7.5/10

## B1 Items (Blocking)

**B1.1 [ensemble-search: G + H]** — G.1 ESS error propagates to H.0 adversarial bar
G.4's ESS table has a calculation error (NC: should be 695, not 769). G.1's "negligible sampling error" is incorrect — actual 90% CI is 3.6× wider than reported. H.0 cites G.1 ensemble percentiles as an adversarial bar without qualifying they are point estimates. Complete G.4-C (ESS correction) and G.1-A (ESS-based uncertainty correction). Until these are done, H.0 cannot be submitted.

**B1.2 [ConvergenceSweep: G cites unvalidated B.16]** — G.0 and G.14 cite B.16
G.0 Section 6 and G.14 decision matrix both cite B.16's T=600 formula. B.16 has zero panel reviews. Until B.16 is reviewed, G.0 and G.14 must add footnotes: "The T=600 citation is contingent on B.16's panel review completion (scheduled; see Track B revision plan)."

## B2 Items (Important)

**B2.2 [VRA methodology: G.13 ↔ D.5]** — G.13 doesn't cite D.5
G.13's protected_districts list has no documented legal basis without a reference to D.5 (Gingles bloc-voting methodology). Add one sentence to G.13 Section 2.1: "The Gingles bloc-voting analysis required to identify protected communities is described in D.5 [cite]."

**B2.3 [synthesis-completeness: A ↔ G]** — Track A ignores Track G
G.1's 0.1–0.7th compactness percentile finding (the bisection plan is near the compactness extremum, not the partisan extremum) is the program's most important ensemble result and is absent from Track A. Complete A PP1.3 (add ensemble context paragraph to A.0 §4).

## B3 Items (Nice-to-have)

**B3.G** —
- G.1: ESS correction + TX/CA data completion (Tier 1 — already in G revision plan)
- G.4: ESS table fix + statutory formula reconciliation (Tier 1)
- G.7: Phase 2 completion (SmcPercentile, WI/TX results, R validation)
- G.5: correct abstract O(n² log n) → O(n³ log n)
- G.12: clarify use case around "ensemble coverage near compact plans"
- G.14: three targeted additions (VraRecom prerequisite, VRA × large-k, G.7 contingency)
- G.13: add D.5 cross-reference

## Critical Path to 7.5/10
G.4-C (ESS fix, 1 week) + G.1-A (ESS uncertainty, 2–3 weeks) → G.1 TX/CA data → G.14 targeted additions → submit G.6, G.8, G.9, G.10, G.11, G.13, G.14 → 7.5+
Time estimate: 4–6 weeks for submitted papers; 16–24 weeks for full G.0–G.7 foundation completion.
