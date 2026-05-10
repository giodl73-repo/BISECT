---
reviewer: Percy Liang
round: 3
score: 4
date: 2026-05-05
---

## Summary

The Round 3 revision addresses both of my outstanding concerns from
Round 2. The log k covariate test — which I maintained as an unresolved
P1 item through two rounds — is now present in Section 4.2 with
concrete regression output: b = 1.05 ± 0.03, c = 0.08 ± 0.15,
95% CI [-0.07, 0.23], p = 0.41, Delta-AIC < 2. The footnote now
explicitly lists all seven excluded k=1 states and confirms the 43-state
OLS sample. These are the correct fixes. I recommend acceptance.

## Strengths

- **The two-variable regression is correctly specified.** The covariate
  is log(log k), not log k, because the theoretical prediction is
  log T = log a + b log n + c log(log k) (since T ~ n^b * (log k)^c
  implies log T = b log n + c log(log k) + const). The authors get
  this specification right. The alternative — using log k as the
  covariate — would have been a specification error.
- **The result is consistent across the reported numbers.** b = 1.05
  in the two-variable model vs. b = 1.07 in the univariate model
  (the 0.02 reduction when adding the second covariate is small and
  in the expected direction). R^2 = 0.985 vs. 0.984 (negligible
  improvement). The p = 0.41 for c is consistent with a 95% CI
  [-0.07, 0.23] centered near zero. These numbers are internally
  coherent, which supports their credibility.
- **The power caveat is appropriately placed.** The log k range
  (k from 2 to 52, log k from 0.69 to 3.95) is the right thing to
  report. Given this range, the data do not have strong power to detect
  a log k effect even if one exists at a magnitude c ~ 0.1. The
  conclusion is therefore correctly stated as "fail to detect" rather
  than "rule out."
- **The footnote is now complete.** The seven excluded states are listed
  alphabetically (Alaska, Delaware, Montana, North Dakota, South Dakota,
  Vermont, Wyoming) and the OLS sample size (43 states, k >= 2) is
  confirmed. A reader can now reproduce the OLS fit from information
  in the paper alone.

## Remaining Concerns (P2 — Not Required)

- **Full 50-state supplementary table.** I requested this in both
  Round 1 and Round 2 as a P2 item. The paper still reports only
  11 representative states in Table 1, while the OLS fit uses 43.
  A compact appendix table (state, n, k, mean, std dev, ms/tract)
  would allow full reproducibility. I understand the constraint for
  a research paper in this series; I note it for the journal
  submission version.
- **The univariate b = 1.07 in Section 4.2 opening is slightly
  inconsistent with b = 1.05 from the two-variable model.** The
  paragraph introducing the univariate fit still reports b = 1.07,
  while the covariate test gives b = 1.05 in the extended model.
  This is expected (adding a covariate changes the coefficient) and
  is internally consistent, but a one-sentence clarification — "the
  univariate and bivariate estimates differ by 0.02, within the
  margin of error of either" — would prevent a reader from reading
  it as a contradiction.

## Score: 4 — Accept

The P1 log k covariate test is resolved with appropriate statistical
rigor. The footnote improvement closes the notation concern I raised
in Round 2. Both of my outstanding items are addressed. I recommend
acceptance.
