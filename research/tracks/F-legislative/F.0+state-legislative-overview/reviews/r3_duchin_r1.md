# Review 3 — Moon Duchin
**Paper**: F.0: Algorithmic State Legislative Redistricting — A Research Program
**Round**: R1
**Score**: 3/4

## Summary

F.0 is an overview paper introducing a six-paper research program. My review focuses on the mathematical claims about the NestSection spine, the PP scaling argument, and the ensemble context — with particular attention to what the paper asserts versus what it can defensibly claim at this stage.

## Strengths

The k/n > 0.05 threshold is introduced carefully: the paper distinguishes between the infeasibility case (k > n) and the degradation case (k/n approaching 0.05), and correctly frames the latter as a quality threshold rather than a hard feasibility constraint. The three-part motivation — population balance feasibility, optimisation richness, and empirical calibration — is coherent, even if the empirical calibration claim ultimately rests on companion paper F.3 rather than on analysis in F.0 itself.

The nesting success rate claim (42/50 states) is specific and will be testable against the F.2 data. The previewed compactness penalty (2--3% mean edge-cut increase from nesting) is appropriately small and directionally plausible.

## Concerns

**C1 — The PP scale argument is incomplete in F.0.** Section 5.1 states that "smaller districts have relatively shorter perimeters because they can be carved from compact sub-regions of the state rather than spanning the entire state geography." This is qualitatively correct but the mechanism described is confused: PP's scale invariance means that scaling a fixed shape does not change PP at all (as F.0 correctly notes, A → s^2A, P → sP). The compactness advantage must therefore come from the boundary effect (districts avoid the irregular state boundary as k grows), which is the O(1/√k) argument developed rigorously only in F.5. F.0 should either defer this explanation entirely to F.5 or give the correct O(1/√k) framing explicitly.

**C2 — "42/50 states" counting inconsistency.** The paper claims in both the abstract and Section 5.4 that NestSection finds a compatible spine for "42 states." However, Table 4 in F.2's 50-state analysis (which I have reviewed) lists 9 states with gcd=1, not 7 as F.0's Section 5.4 states. The discrepancy is explicitly flagged in F.2 with "we recount compatible states at 40 after careful tally" — but this contradiction with F.0's "42" claim is unresolved across the two papers and represents an internal inconsistency that must be reconciled before publication.

**C3 — Ensemble context is entirely absent.** F.0 is an overview of algorithmic redistricting that produces single deterministic maps (minimum edge-cut, seed 42). The paper does not situate these single-map results in the ensemble context that has become standard in modern redistricting analysis (Markov chain methods, sequential Monte Carlo). A reader familiar with Duchin, DeFord, or Chikina's work will immediately ask: how does the F-track's single-map approach compare to drawing from the plan space? The overview should acknowledge this distinction and explain why a single-map approach is appropriate for this research program.

**C4 — "43% nesting success" claim mathematically stated incorrectly.** Section 6.3 states that "for non-power-of-2 values of k, the algorithm uses the nearest-power-of-2 strategy." This is a description of the congressional bisection, but for NestSection with spine g and per-spine counts H/g and S/g, neither may be powers of 2. The CLI description needs to specify how non-power-of-2 within-spine partitions are handled.

## Recommendation

Revise and resubmit. The core overview is sound but C1 (PP mechanism confusion) and C2 (nesting count inconsistency across F.0 and F.2) are substantive errors that require correction.
