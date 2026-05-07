---
reviewer: George Karypis
round: 2
score: 3
date: 2026-05-05
---

## Summary

The revision substantially improves the theoretical foundations of B.6. The
NP-hardness reduction is now correctly based on Connected Planar Graph Bisection
(Dyer and Frieze 1985), properly accounting for the contiguity requirement.
The O(sqrt(log n)) approximation claim has been removed and replaced with an
honest statement that no formal approximation ratio is known for METIS's
heuristic. The OLS regression now correctly excludes k=1 states. The runtime
theorem correctly uses niter = 100 as a fixed constant rather than O(log n).
These are the right fixes. The paper is substantially stronger.

My remaining concerns are secondary — the paper has addressed all three major
errors I identified in Round 1.

## Strengths

- **The NP-hardness reduction is now correct.** Reducing from Connected Planar
  Graph Bisection (Dyer and Frieze 1985) properly handles the contiguity
  requirement. The reduction is clean: unit weights, k=2, ε=0, and a
  bijection between valid solutions. The remark on feasibility vs.
  optimisation is an important addition that strengthens the legal framing.
- **The approximation section is now honest.** Proposition 1 no longer
  overclaims a formal guarantee for METIS. The "Context" paragraph correctly
  explains that ARV applies to Sparsest Cut, not balanced connected
  k-partition, and that FM local search has no formal connection to ARV.
  This is a significant improvement.
- **The runtime theorem is now self-consistent with the experiments.**
  Stating O(niter · n log k) with niter = 100 fixed is the correct bound.
  The acknowledgment that niter = 100 > log(8,057) ≈ 13 for California is
  honest and important.
- **The OLS regression fix is correct.** Excluding k=1 states and reporting
  b = 1.07 ± 0.03 for k>1 states (43 states) is the right approach. Adding
  runtime standard deviations to Table 1 is a meaningful improvement.

## Remaining Concerns (P2 — Suggestions)

- **The proof sketch of Theorem 1 could be tightened.** The sketch correctly
  identifies the source (Dyer and Frieze 1985) and the reduction, but does
  not explicitly state that Dyer and Frieze's NP-completeness proof covers
  planar graphs specifically. Their 1985 paper proves NP-completeness for
  general graphs and for planar graphs via separate reductions. The sketch
  should cite Theorem 2 of Dyer and Frieze (1985) explicitly (which covers
  planar graphs). This is a citation precision issue, not a correctness issue.
- **The log k covariate test is still missing.** The paper claims the residual
  super-linearity (b = 1.07 vs. 1.00) is attributable to the log k factor, but
  this is not tested. A two-variable OLS fit of log T against log n and
  log(log k) would directly test this claim. This is a P2 item but would
  significantly strengthen the empirical section.
- **The 2030 Census projection is mentioned but not computed.** The discussion
  notes the expected 78,000–82,000 tract count for 2030 but does not compute
  the projected runtime. Adding "using b = 1.07, expected runtime for California
  in 2030 (n ≈ 9,200 tracts) is approximately X ms" would be a useful addition.

## Score: 3 — Minor Revision

The three major errors from Round 1 are corrected. The remaining concerns
are P2 items that would strengthen the paper but are not required for
correctness. This is now in minor revision range.
