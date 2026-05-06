---
reviewer: Moon Duchin
round: 2
score: 3
date: 2026-05-05
---

## Summary

This is a substantial revision. All three major errors I identified in
Round 1 are addressed. The NP-hardness reduction now correctly reduces
from Connected Planar Graph Bisection, accounting for the contiguity
requirement. The ARV approximation claim is removed; Proposition 1 now
honestly states that no formal approximation ratio is known for METIS's
heuristic. The niter issue is resolved with an explicit acknowledgment
that niter = 100 is a constant, not O(log n). The paper has moved from
"major revision required" to "minor revision needed."

My remaining concerns are methodological refinements, not substantive
errors.

## Strengths

- **The core error is corrected.** The reduction from Connected Planar
  Graph Bisection (Dyer and Frieze 1985) is the right source. The proof
  sketch now correctly argues that validity of a redistricting solution
  implies a connected bisection of G' and vice versa. The contiguity
  requirement is no longer silently ignored.
- **Dropping the O(sqrt(log n)) claim is the right call.** The revised
  Proposition 1 is honest: no formal guarantee is asserted for METIS.
  The "Context" paragraph correctly explains why ARV does not apply
  (ARV is for Sparsest Cut via SDP; METIS is a local search heuristic
  with no SDP connection). This is intellectually rigorous.
- **The feasibility clarification strengthens the paper.** The remark
  after Theorem 1 on feasibility vs. optimisation is a useful addition
  that prevents a common misreading of complexity results for legal
  audiences. Spanning-tree-based feasibility is correctly invoked.

## Remaining Concerns

- **Dyer and Frieze (1985) citation needs pinning to the planar case.**
  Dyer and Frieze prove NP-completeness for connected k-partition in
  general and for planar graphs. The proof sketch should explicitly
  reference the planar graph case of their result (their Theorem 2 or
  equivalent). Without this precision, a skeptical reader might argue
  that only the general-graph hardness is established, not the planar
  case. A single parenthetical — "(Theorem 2 of Dyer and Frieze 1985,
  which covers planar graphs explicitly)" — would close this gap.
- **The empirical approximation framing needs one more sentence.**
  Section 5.2 says METIS is "within 3% of the best of 10,000 seeds"
  but does not note that this is an upper bound on the gap to the best
  *found* solution, not the true optimum. A sentence clarifying this
  — "The true optimum is unknown; 10,000 seeds is not a lower bound,
  and the actual gap to the optimum may be smaller or larger" — would
  be more accurate. This is important for a paper that will be cited in
  legal contexts.
- **The connected graph partition literature survey is still thin.**
  Section 2 cites Dyer and Frieze (1985) but does not mention the line
  of work on connected graph bisection complexity (Chataigner et al. 2007,
  Hager et al. 2013) that would provide fuller context. A one-paragraph
  survey of this literature would strengthen the related work section.

## Score: 3 — Minor Revision

The three major errors from Round 1 are all corrected. The paper is now
theoretically sound. The remaining issues are precision and context
improvements that can be addressed in a final revision.
