---
reviewer: Moon Duchin
round: 3
score: 4
date: 2026-05-05
---

## Summary

The targeted revision adds the log k covariate test and tightens the
footnote, which resolves the statistical integrity concern that Liang
and Karypis flagged as P1 in Round 2. My own Round 2 concerns were all
P2 items (Dyer and Frieze Theorem 2 pinning, approximation framing, and
connected bisection literature). These remain unaddressed, as is
appropriate given their secondary status. The paper is theoretically
sound and empirically careful.

## Strengths

- **The functional form of the two-variable model is correct.**
  The covariate log(log k) is the right choice for the regression log T
  against log n and log(log k), because the theoretical model predicts
  T proportional to n * log k, meaning log T = log a + log n + log(log k)
  up to constants. Testing this functional form is the right test. The
  authors execute it correctly.
- **The null result is stated conservatively.** The paragraph does not
  claim to have ruled out a log k effect — it notes the narrow log k
  range limits power, but that the point estimate c = 0.08 is small
  relative to b = 1.05 and the CI includes zero. This is the appropriate
  epistemic posture: we fail to reject the null, but we cannot confirm
  that log k contributes nothing.
- **The n-dominance conclusion is now defensible.** "n alone is the
  sufficient predictor of runtime at census-tract scale" is a conditional
  claim (at this scale, with these data) rather than an absolute claim.
  That is the right framing. A skeptical expert witness challenging this
  would need to argue that the narrow log k range is itself an artifact —
  which is implausible given the data include states from k=2 (Rhode
  Island) to k=52 (California).

## Remaining Concerns (P2 — Not Required for Acceptance)

- **Dyer and Frieze (1985) Theorem 2 pinning.** I raised this in Round 2
  and it remains unaddressed. For a paper that will appear in legal
  proceedings, citing "(Theorem 2 of Dyer and Frieze 1985, which covers
  planar graphs explicitly)" would prevent a credentialed opponent from
  claiming that the planar-graph NP-hardness is not established.
  This is precision, not a correctness issue.
- **Approximation gap clarification.** Section 5.2 still does not have
  the sentence I requested: "The true optimum is unknown; 10,000 seeds
  is not a lower bound, and the actual gap to the optimum may be smaller
  or larger than 3%." The current text implies the 3% figure is a bound
  on the gap to some external reference, which could be misread in
  cross-examination.
- **Connected bisection literature.** Section 2 still does not cite
  Chataigner et al. 2007 or Hager et al. 2013. This is a literature
  completeness issue, not a correctness issue.

## Score: 4 — Accept

The P1 log k covariate test is executed correctly and resolves the
core statistical integrity concern. The P2 items are genuine improvements
but not blocking. I recommend acceptance.
