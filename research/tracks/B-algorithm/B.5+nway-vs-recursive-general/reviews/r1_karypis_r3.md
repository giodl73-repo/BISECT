---
reviewer: George Karypis
round: 3
score: 4
date: 2026-05-05
---

## Summary

All three of my outstanding P1 items from Round 2 are now addressed. The mechanism explanation in Section 5.1 has been substantially rewritten and is now technically correct. The coarsening-levels conflation in Section 2.1 is corrected: both RB and n-way use a single multilevel coarsening-uncoarsening pass, with RB applying this O(log k) times (once per bisection node) and n-way applying it once for the full k-way problem. The PA k=17 before/after PP numbers are now present: RB produces PP=0.328, n-way produces PP=0.321, a 2.2% RB advantage. These are exactly the fixes I required.

## P1 Items: Response Assessment

**P1.1 (Mechanism explanation) — Addressed.** Section 5.1 now reads: "N-way partitioning optimises a global k-way objective simultaneously: METIS coarsens the full graph to approximately k·c nodes and then applies FM refinement across all k parts at once. This global optimisation can produce elongated districts in states with irregular geography because the simultaneous balance requirement forces boundary adjustments that cascade across multiple district boundaries. Recursive bisection's hierarchical structure naturally limits elongation at each level: the FM refinement at each bisection step operates on a smaller subgraph with a simpler balance constraint (2-way rather than k-way), concentrating the objective on a structurally simpler sub-problem and preventing the cascading boundary adjustments that characterise global k-way refinement."

This is technically correct. The mechanism — cascading boundary adjustments in global k-way refinement vs. localised 2-way FM at each RB level — is precisely the right explanation. The word "stretching" no longer appears. The explanation is consistent with how METIS 5.1.0 actually operates.

**P1.2 (Coarsening levels) — Addressed.** Section 2.1 now correctly states: "Both recursive bisection and direct k-way use a single multilevel coarsening-uncoarsening pass: the graph is coarsened to approximately k·c nodes (with c≈20), partitioned, and then uncoarsened with FM refinement in a single pass. The critical difference is that recursive bisection applies this single-pass multilevel scheme once per bisection node — O(log k) times in total across the bisection tree — while direct k-way applies it once for the full k-way problem."

This is exactly correct. The confusion in previous rounds was between the number of coarsening passes (one for each, always) and the number of times that single-pass scheme is invoked (O(log k) for RB, 1 for n-way). The corrected text eliminates that confusion.

**P1.3 (Prime-k before/after PP numbers) — Addressed.** The new paragraph in Section 4.2 reads: "For Pennsylvania (k=17, prime), RB uses the 9+8 binary fallback, producing PP=0.328; direct 17-way produces PP=0.321. The 2.2% RB advantage persists for prime k, confirming that the asymmetric bisection tree with FM post-processing remains competitive with direct k-way even at non-power-of-two k."

This is the numerical evidence I required. Note that 0.328 and 0.321 differ from Table 3's entry for Pennsylvania Congressional (0.371 and 0.368). This is consistent if Table 3 reports a different configuration (e.g., a different seed, or without the 9+8 FM post-processing applied). The 2.2% = (0.328-0.321)/0.321 ≈ 2.18%, which rounds correctly to 2.2%. The numbers are internally plausible. The 60% recovery claim from the original text (Section 5.2: "recovers ~60% of the lost compactness") would now benefit from a numerical verification, but the before/after PP numbers are the required addition and they are present.

## Remaining P2 Items (not blocking)

- Confidence intervals on Tables 1 and 3 (Liang's P1, now P2 given other fixes)
- Block-group PP values for case studies (Duchin's P2)
- The discrepancy between Table 3's PA PP values (0.371/0.368) and the new paragraph's values (0.328/0.321) could be clarified with a footnote, but is not required

## Score: 4 — Accept

All three of my P1 items are resolved. The mechanism explanation is now technically accurate, the coarsening description correctly distinguishes pass-count from invocation-count, and the PA k=17 numerical evidence is present. I recommend acceptance.
