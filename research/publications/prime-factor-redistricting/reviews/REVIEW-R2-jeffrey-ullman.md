# Round 2 Review — R-50 (Jeffrey D. Ullman)
**R2 Score: 3.0/4.0** (R1: 2.5, Δ = +0.5)

## Response to Revision

The revision makes meaningful progress on the formal weaknesses raised in Round 1. Definition 3.1 (Canonical p-partition as a function of (G, p, seed)) and the formal Reuse Theorem with proof sketch are substantive improvements. The authors correctly distinguish PFR-binary from direct-k-METIS partitioning and carry the Reuse Theorem through both cases.

The zero-variance finding across 480 runs is algorithmically interesting. However, the paper presents this as an empirical observation without a supporting formal argument. "METIS finds the same solution from any seed" is a statement about the algorithm's convergence behavior on specific graph instances — it is not implied by the Reuse Theorem, which assumes determinism. The authors should either (a) cite the METIS literature on why multilevel coarsening produces stable results on near-planar geographic graphs, or (b) label this explicitly as an empirical finding. Calling the method "functionally seedless" without this caveat overstates the formal guarantee.

The P1-D non-resolution is a continuing concern. Documenting balance violations as "future work" does not close the gap for a paper making production claims. The asymptotic complexity of the post-processing step is uncharacterized.

## Remaining Concerns

1. The Reuse Theorem proof sketch invokes the METIS determinism assumption without characterizing when it holds or fails. The assumption deserves at least a paragraph of justification citing METIS implementation behavior.
2. The "functionally seedless" claim requires formal or empirical backing for why graph instances arising in PFR exhibit near-unique minima. Without this, the claim is an inference from 480 data points.
3. Population balance post-processing is uncharacterized in complexity terms.
4. Time complexity of the full PFR pipeline is still not formally stated.

## New Concerns

The term "functionally seedless" will invite scrutiny. Readers will ask: is this a property of METIS on planar graphs, a property of the specific geographic instances tested, or a general property of PFR? The paper does not distinguish these, and a reviewer at an algorithms venue will read "functionally seedless" as a claim about algorithm behavior, not observed outputs on 50 states.
