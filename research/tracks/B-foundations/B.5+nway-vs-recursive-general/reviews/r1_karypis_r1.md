---
reviewer: George Karypis
round: 1
score: 3
date: 2026-05-05
---

## Summary

This paper extends the B.3/B.4 comparison of recursive bisection (RB) vs. direct k-way partitioning from congressional chambers to a broader set of 727 chamber configurations spanning congressional, state senate, and state house maps. The central claim — that RB outperforms n-way by +0.003–+0.004 mean Polsby–Popper across all chamber types with negligible runtime cost — is plausible and consistent with what we know about METIS on planar graphs. The paper is well-structured and the experimental setup is careful. However, from a graph-partitioning perspective, several technical claims require tightening before this can be accepted at a methods venue.

## Strengths

- **Comprehensive experimental scope.** Testing 727 configurations across two geographic resolutions (tract and block-group) is the most systematic RB vs. n-way comparison in the redistricting literature. The paired t-test design (pairing by state) correctly accounts for between-state variation.
- **Prime-k analysis is novel and practically important.** Section 4.2's identification of the prime-k edge case — where asymmetric bisection trees narrow the RB advantage — is a genuine contribution not previously documented. The proposed FM post-processing mitigation is practical and recovers ~60% of lost compactness.
- **Runtime characterisation at large k.** Quantifying the crossover point (k > 80, state house chambers) and establishing the absolute magnitude of the n-way runtime advantage (at most 178 ms) settles an open practical question for DIA implementers.

## Weaknesses / P1 Items (Required Fixes)

- **The n-way advantage explanation is incomplete.** Section 5.1 attributes RB's compactness advantage to "hierarchical constraint structure" and separator exploitation, but the mechanism is stated without formal support. In METIS kmetis (direct k-way), FM refinement operates on the full k-way partition — it does not "stretch districts across geographic features" in the way implied. The actual mechanism is that RB's per-bisection FM refinement operates on smaller subgraphs with tighter balance constraints, producing lower boundary-length cuts per level. This should be stated correctly, as the current explanation could mislead practitioners about when to prefer each strategy.
- **Coarsening levels are mischaracterised.** The related-work section states "Karypis and Kumar showed that n-way METIS (kmetis) scales to O(m) per level and O(log k) levels for RB." This conflates RB and n-way. N-way (kmetis) in METIS 5 uses a single coarsening phase to ~20k nodes, then k-way initial partitioning — it does not use O(log k) coarsening levels. RB uses O(log k) bisection levels. This distinction matters because it affects the runtime comparison: at large k, kmetis's single coarsening pass is cheaper than RB's repeated bisections, which is exactly why n-way is faster at k > 80.
- **The prime-k mitigation is underspecified.** Section 5.2 proposes "rounding to the nearest power of two at each level and post-processing with FM refinement." It is unclear what "rounding to the nearest power of two" means when k is fixed by apportionment law. If k = 17 is fixed (Pennsylvania), you cannot change k. The mitigation should be clarified: we suspect the authors mean applying one extra FM sweep on the asymmetric-split boundaries after the bisection tree is built. Please restate this precisely.

## P2 Items (Suggestions)

- **Consider showing partition quality variance, not just mean.** The paper reports mean PP, but the distribution matters: a strategy with lower mean but lower variance may be preferable to one with higher mean but occasional very bad districts. A box plot of per-district PP by strategy and chamber type would strengthen the results section.
- **Evaluate SCOTCH or KaHyPar on a subset.** The limitations section acknowledges that the comparison is limited to METIS 5.1.0. Testing even a 5-state subset with SCOTCH would provide evidence that the RB advantage is a property of the problem class (planar graphs) rather than METIS's specific implementation, which is the claim made in the discussion.

## Score: 3 — Minor Revision

The paper makes a solid empirical contribution but requires correction of the mechanism explanation (P1.1) and the coarsening-levels description (P1.2), and clarification of the prime-k mitigation (P1.3). These are fixes within the authors' existing data and analysis. I expect a revised version to be acceptable.
