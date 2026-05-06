---
reviewer: Moon Duchin
spec: redist-ensemble ReCom crate
round: 1
score: 2
date: 2026-05-06
---

## Summary
The spec describes a plausible Rust ReCom implementation but contains a fundamental mischaracterisation of how GerryChain's balance check works and an incorrect claim about what causes bipartition failures in TX. These are not implementation details — they affect the statistical validity of the sampler and the scientific claims the G-series papers can make.

## Strengths
- The choice of Wilson's loop-erased random walk over Aldous-Broder is correct. For redistricting graphs (sparse, planar-like, ~1,000–10,000 nodes), Wilson's algorithm consistently runs faster in practice because the cover time of planar graphs is lower than the O(n^2) worst case.
- The Hamming autocorrelation diagnostic is well-chosen for redistricting Markov chains. Because ReCom makes large moves (entire pairs of districts are reassigned), the autocorrelation structure is different from single-flip chains, and Hamming distance between successive assignments is the right statistic to track.
- The integration with the existing `redist-analysis::ensemble_diagnostics` R-hat and ESS implementation avoids re-implementing diagnostics and ensures the G.4 numbers remain reproducible from the same code.

## P1 — Required changes

- **The balance check retry logic does not match GerryChain's ReCom implementation.** GerryChain does not sample 50 random edges from the same tree. GerryChain's `recom` proposal either (a) enumerates *all* tree edges and picks uniformly among those that yield a balanced cut, or in the "bipartite" variant (b) resamples the spanning tree entirely if no balanced cut exists. The spec's 50-random-draws-from-fixed-tree approach is a novel algorithm, not a port of GerryChain. If this is intentional, the spec must justify the statistical equivalence (or acknowledge the difference in the target distribution). If it is a misunderstanding, it must be corrected. The consequences are non-trivial: the 50-retry approach over-samples tree edges near balanced cuts and under-samples those near extremes, subtly deforming the stationary distribution away from the uniform-over-valid-plans target.

- **The claim that TX bipartition failures are caused by "implementation" and would be fixed in Rust is incorrect.** Bipartition failures in TX (k=38) occur because, for large k, the merge region of a randomly selected adjacent pair frequently has no balanced cut at any spanning tree edge. This is a combinatorial property of the graph and the population distribution — it is independent of implementation language. A Rust implementation will fail to find balanced cuts for TX at exactly the same rate as GerryChain, unless the retry strategy is changed (e.g., by selecting pairs that are more likely to admit balanced cuts, or by using a different proposal). The spec must remove the implication that Rust speed resolves the bipartition failure problem and instead specify the algorithmic strategy for handling TX (k=38): likely a targeted pair-selection heuristic or a higher retry budget.

- **R-hat (Gelman-Rubin) is not well-defined for discrete redistricting statistics without modification.** The standard R-hat diagnostic assumes the chains are sampling a continuous distribution and uses variance estimates that are biased for discrete, bounded statistics like seat counts or contiguity indicators. For cut-fraction (a continuous statistic), R-hat at 1.003 for NC is plausible and interpretable. For integer seat counts or binary district labels, the R-hat formula requires the rank-normalisation correction described in Vehtari et al. (2021). The spec must either (a) restrict R-hat computation to continuous statistics (cut-fraction, pop-deviation) and use a different convergence diagnostic for discrete statistics, or (b) specify that the rank-normalised R-hat from Vehtari et al. (2021) is implemented.

## P2 — Suggested improvements

- Add a comparison section that specifies exactly which variant of ReCom this implements: the original DeFord-Duchin-Solomon (2021) "recom" (resample tree on failure) or the "bipartite ReCom" variant. These have different stationary distributions and the choice matters for the G-series papers' claims about the feasible space.

- The spec should note that the stationary distribution of ReCom is not the uniform distribution over all valid plans — it is a specific distribution that depends on the proposal acceptance probability and the adjacency graph structure. The ensemble therefore characterises "the ReCom-reachable space" not "all compact balanced plans." This is a known and accepted limitation of MCMC redistricting methods, but it should be stated explicitly in the research significance section.

## Score: 2/4
The balance check implementation and the TX bipartition claim are substantive algorithmic errors that would produce a statistically invalid sampler and a false claim in the research papers. These require a full revision before the spec can be approved.
