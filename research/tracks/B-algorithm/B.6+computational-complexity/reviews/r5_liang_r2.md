---
reviewer: Percy Liang
round: 2
score: 3
date: 2026-05-05
---

## Summary

The revision addresses the major empirical methodology issues I raised in
Round 1. The k=1 states are now excluded from the OLS regression with an
appropriate footnote explaining their trivial nature. Runtime standard
deviations are added to Table 1. The runtime theorem now correctly states
niter = 100 as a fixed constant rather than absorbing it into O(log n).

The remaining concerns are the log k covariate test (still absent) and
the lack of supplementary data for the full 50-state sample.

## Strengths

- **The k=1 exclusion is correct and well-documented.** The footnote
  explains the mechanism (k=1 requires no bisection; runtime is O(n)
  graph-building with negligible constant) and the justification
  (including k=1 states would bias the exponent by anchoring low-n
  observations at near-zero runtimes). The OLS result (b = 1.07 ± 0.03,
  43 states) is now the right number.
- **Standard deviations are now reported.** Table 1 now shows "mean ± std.
  dev." for each state. The variability is small (±5–6 ms for California)
  relative to the mean (198.4 ms), giving a coefficient of variation of
  about 3% — low enough to confirm that 10-run averaging is adequate.
- **The niter correction is honest.** Acknowledging that niter = 100 exceeds
  log(8,057) ≈ 13 for California, and that the theoretical bound is
  O(niter · n log k) with niter fixed, is the correct statement. The prior
  O(n log n log k) bound was misleading because it implied niter = O(log n)
  was satisfied experimentally.

## Remaining Concerns

- **The log k covariate test is still absent.** I raised this in Round 1 as
  a P1 item: the theoretical model is O(niter · n log k), but the empirical
  fit uses only log T ~ log n. The residual super-linearity (b = 1.07) is
  attributed to the log k factor without testing this. A two-variable
  regression of log T against log n and log(log k) is a straightforward
  analysis that would directly test whether the log k term accounts for
  the super-linearity. This remains an unfixed P1 item from Round 1.
  The paper now says (in Section 4.2) "the residual super-linearity is
  attributable to the log k factor: larger states tend to have more
  districts" — but this is still assertion, not test.
- **The supplementary 50-state table is still absent.** Table 1 shows 11
  representative states. The OLS fit uses 43 states. Readers cannot verify
  the regression without the full data. Even a compact appendix table (state,
  n, k, mean, std dev, ms/tract) would suffice. This was a P2 item in Round
  1 and remains unaddressed.
- **The regression notation needs clarification.** The caption of Table 2
  now says "k>1 states only; single-district states excluded" but the main
  text in Section 4.2 says "43 states" without listing which states are
  excluded. Alaska, Montana, North Dakota, South Dakota, Vermont, Delaware,
  Wyoming — that is 7 states, leaving 43. The text should confirm the
  excluded state list explicitly.

## Score: 3 — Minor Revision

The k=1 exclusion and variance reporting are meaningful improvements.
The remaining issue is the untested log k prediction (still P1 in my view)
and the missing supplementary data. The paper is substantially stronger
than Round 1 and is in minor revision range.
