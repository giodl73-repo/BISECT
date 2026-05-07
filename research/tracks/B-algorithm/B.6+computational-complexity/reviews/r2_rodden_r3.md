---
reviewer: Jonathan Rodden
round: 3
score: 4
date: 2026-05-05
---

## Summary

The Round 3 revision makes targeted, appropriate changes. The log k
covariate test now appears in Section 4.2 as a dedicated paragraph with
concrete regression output. The footnote listing the seven excluded k=1
states by name is a clean improvement. Both of these address the final
outstanding P1 item that Liang and Karypis identified in Round 2. From
a political science and policy standpoint, the paper is now complete.

## Strengths

- **The n-dominance conclusion is now empirically supported.** The
  prior version stated that super-linearity was "attributable to the
  log k factor" as an assertion. The revised version tests this claim
  directly: c = 0.08 ± 0.15, p = 0.41, Delta-AIC < 2. The conclusion
  that n alone is the sufficient predictor is now supported, not just
  asserted. This is the appropriate standard for a paper that will be
  cited in legal proceedings.
- **The statistical presentation is accessible to policy readers.**
  The paragraph leads with the model, states the fitted numbers, and
  gives the policy-relevant conclusion ("n is the dominant scaling
  factor") without burying the reader in statistical notation. The
  Delta-AIC criterion is less familiar to political scientists than a
  t-test, but the 95% CI [-0.07, 0.23] crossing zero is a clear signal.
- **The footnote improvement is correct.** Listing all seven excluded
  states explicitly and confirming "43 states (k >= 2) form the OLS
  sample" makes the regression reproducible from the paper alone.

## Remaining Concerns (P2 — Not Required)

- **2030 Census projection.** I raised this in Rounds 1 and 2 as a P2
  item and it remains absent. For DIA planning purposes, a one-sentence
  projection — "Using b = 1.07, a California-scale state with n ~ 9,200
  tracts in 2030 would take approximately 215 ms (extrapolating from the
  current 198.4 ms at n = 8,057)" — would have direct policy value. I
  understand this may be addressed in a companion technical note.
- **B.7 cross-reference specificity.** Rodden R2 suggested a more
  explicit parenthetical describing what B.7 is for readers unfamiliar
  with the series. The reference remains a passing citation. This is
  acceptable for series-internal use.

## Score: 4 — Accept

All P1 items are resolved across three rounds. The empirical section
now correctly tests its own claims. The paper is ready for circulation
as a technical foundation for DIA legal proceedings.
