# Review 3 (Round 2): Moon Duchin (Metric Geometry, GerryChain)
**Paper**: H.1: BisectionEnsemble
**Round**: 2
**Recommendation**: Minor Revision

---

## Response to Round 1 Concerns

### TX Causal Account — Addressed (Satisfactory)

The revised Section 4.2 correctly distinguishes GerryChain without pair reselection from GerryChain with pair reselection, and correctly identifies the failure mode as geometric (boundary constraint from 36 surrounding districts) rather than a balance-ratio issue. The statement "Both GerryChain-with-pair-reselection and BisectionEnsemble succeed on TX by trying adjacent district pairs until a balanced cut exists" is accurate. The characterization of the advantage as throughput ("the Rust implementation processes ~1000 pair attempts per second vs. ~5 in Python") rather than a structural difference is exactly the correction I requested.

The Table 1 caption now correctly says "GerryChain (without pair reselection) stalls on Texas." This is the right qualification.

### Ergodicity — Addressed (Satisfactory)

The revised Section 5.1 replaces "By the ergodicity of the ReCom chain, the distribution converges to the stationary distribution" with "To the extent the local chain mixes over the feasible bisection space of each node — a property whose formal proof is left to future work — the distribution of accepted plans approximates the stationary distribution." This is the correct hedged formulation. The addition of empirical acceptance rates (NC: 61%, WI: 84%, TX bisection nodes: >50%) as informal evidence of mixing is appropriate.

### Locality Qualification — Addressed (Satisfactory)

The revised Key Advantages list now states: "At depth $\ell$, each node manages $\approx n/2^\ell$ tracts; the locality advantage grows with depth. The root node ($\ell=0$) manages half the state, while leaf nodes manage $\approx n/k$ tracts." This is the transparency I requested about the root node.

### GerryChain Citation — Not Addressed

I noted in Round 1 that the paper uses `duchin2019gerrychain` (an early arXiv preprint co-authored by Duchin and Walch) as the GerryChain citation, when the standard citation for the method is DeFord, Duchin, Solomon 2021 (HDSR), already in the bibliography as `deford2021recombination`. The paper still leads the Introduction with `\citep{duchin2019gerrychain}` alongside `\citep{deford2021recombination}`. The `duchin2019gerrychain` citation is not wrong, but it attributes GerryChain primarily to a preprint rather than the peer-reviewed publication. I would prefer `\citep{deford2021recombination}` as the primary GerryChain citation in the Introduction, with `duchin2019gerrychain` as a supplementary reference if needed. This is minor but affects how the community reads attribution.

### GerryChain Reproducibility — Not Addressed

The GerryChain comparison in Table 1 still does not specify version, tolerance $\varepsilon$, ReCom variant, initial plan, or random seed. The revision plan listed this as C-4 (P2). For a stochastic comparison that is the paper's central motivation, this remains a gap. I accept that providing 10 independent runs may be infeasible, but at minimum the configuration should be specified in a footnote or appendix so the result is in principle reproducible.

## Minor Points

- The paper uses "bipartition failure" and "bipartition imbalance" interchangeably in places (Introduction vs. Section 3 header). Standardizing on one term was a Round 1 minor request that remains open.
- The revised Table 1 caption reads "GerryChain-with-pair-reselection would also succeed but at lower throughput." This is correct but speculative — the paper has not run GerryChain with pair reselection on TX. The sentence should say "would be expected to succeed" rather than "would also succeed."

## Overall Assessment

The three P1 issues I flagged (causal account, ergodicity, locality) are all adequately addressed. The GerryChain citation preference and reproducibility gap are minor issues that could be addressed with a footnote and citation change. I support acceptance after these minor corrections.

**Score**: 3.0/4
