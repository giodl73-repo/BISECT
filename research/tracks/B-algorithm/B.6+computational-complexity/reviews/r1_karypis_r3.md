---
reviewer: George Karypis
round: 3
score: 4
date: 2026-05-05
---

## Summary

The revision resolves the one remaining P1 item I identified in Round 2.
The log k covariate test is now present as a dedicated paragraph in
Section 4.2, with a concrete two-variable OLS result: b = 1.05 ± 0.03,
c = 0.08 ± 0.15, p = 0.41, Delta-AIC < 2. The finding that c is not
significantly different from zero directly addresses my concern that the
residual super-linearity claim was asserted rather than tested. The seven
excluded k=1 states are now listed explicitly in the footnote, which
closes the Liang notation concern as well.

The paper is now ready for acceptance.

## Strengths

- **The log k covariate test is correctly executed.** The two-variable
  model log T = log a + b log n + c log(log k) is the appropriate
  functional form given the theoretical bound O(niter * n log k).
  Using log(log k) rather than log k as the covariate is correct: the
  theoretical term is log k, so the covariate for log T should be
  log(log k). The authors get this right.
- **The concrete numbers are appropriate.** b = 1.05 ± 0.03,
  c = 0.08 ± 0.15, 95% CI [-0.07, 0.23], p = 0.41 — these are all
  the numbers a reader needs to assess the statistical argument.
  The Delta-AIC < 2 criterion is a standard model-selection test and
  is correctly applied.
- **The power caveat is well-placed.** The paragraph acknowledges that
  the log k range is narrow (log k in [0.69, 3.95], k from 2 to 52),
  which limits detection power. This is honest: we cannot rule out a
  small log k effect, but the point estimate c = 0.08 is small relative
  to b = 1.05, and the CI includes zero. This is a statistically mature
  statement.
- **The excluded state list in the footnote is now explicit.** Alaska,
  Delaware, Montana, North Dakota, South Dakota, Vermont, and Wyoming
  are listed by name, with the count "43 states (k >= 2) form the OLS
  sample" confirming the arithmetic. This closes the Liang notation
  concern from Round 2.

## Remaining Concerns (P2 — Not Required for Acceptance)

- **Dyer and Frieze (1985) Theorem 2 pinning.** I noted in Round 2 that
  the proof sketch should explicitly reference Theorem 2 of Dyer and
  Frieze (1985) (the planar graph case). This is still a citation
  precision issue and remains a P2 item for a journal submission.
  For a research paper in this series, the current citation level is
  acceptable.
- **2030 Census projection.** A one-sentence runtime projection for 2030
  (n ~ 9,200 tracts for California using b = 1.07) remains absent.
  This is a minor P2 suggestion; the cross-census stability discussion
  already implies the projection.

## Score: 4 — Accept

The sole P1 item from Round 2 (log k covariate test) is now resolved
with appropriate statistical rigor. The footnote correction closes the
Liang notation concern. This paper is theoretically sound and empirically
careful. I recommend acceptance.
