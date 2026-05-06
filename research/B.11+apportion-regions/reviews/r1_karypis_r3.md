> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Round 3 Review — George Karypis
**R3 Score: 3.3/4.0** (R2: not on panel, new reviewer this round)

## Summary Assessment

This paper makes a genuine contribution to the computational redistricting literature. The ApportionRegions framework is original: using prime factorization to drive hierarchical METIS partitioning is a structurally new idea that has not appeared in the graph partitioning literature to my knowledge. The zero-variance empirical result across 480 runs (all 50 states, 10–20 seeds each) is credible and significant — METIS's behavior on subgraph-decomposed problems being seed-invariant is a plausible and testable claim, and the authors test it convincingly.

The Round 3 revision addresses two previously open P1 items substantively.

## Response to P1-D (Population Balance)

The new paragraphs describing the boundary-swap rebalancing algorithm are an improvement over the earlier "future work" deflection. The algorithm is described clearly: iterate over district boundaries, find tract-swap moves that reduce the maximum population deviation without breaking contiguity, and terminate when all districts are within ±0.5%. The concrete numbers (NC: 23 iterations → 0.48% max deviation; GA: 31 iterations → 0.47%) are precisely the kind of specific empirical reporting that was missing.

One concern: the claim that "the partisan outcome is unchanged by rebalancing in both cases" is asserted without supporting evidence. In 23–31 swap iterations, it is possible — even likely — that some marginal tracts near competitive district boundaries are reassigned. The authors should either (a) verify formally that no district changes partisan control after swapping or (b) report the specific tracts swapped and show they are not in contested margins. The current claim reads as an assumption rather than a verified finding.

## Response to P1-E (GerryChain Comparison)

The new "Comparison with ensemble methods" subsection is substantively useful. The ReCom runtime comparison (10,000 plans in ~4 hours vs. AR in <3 seconds) correctly characterises the function-space difference between sampling and deterministic methods. The percentile estimates (AR at ~70th percentile on compactness, ~55th on minority representation, ~75th on partisan outcome within the NC-14 ReCom distribution) are plausible given published NC ensemble literature.

However, the subsection is explicit that these are "plausible estimates consistent with published GerryChain results" rather than actual runs against the AR plan. This is acceptable for a research paper that distinguishes itself from ensemble methods conceptually, but reviewers at venues like Political Analysis or JCSS will expect at minimum a citation to a specific published NC-14 ensemble study that supports the percentile estimates. The current citation to Duchin and Walch 2019 is a general GerryChain paper, not an NC-14 ensemble analysis. Herschlag et al. 2020 (already in the bibliography) would be the more appropriate citation for NC-specific ensemble percentile claims.

## Remaining Concerns

1. **METIS version not reported.** The zero-variance claim is made across 480 runs but the METIS version used is not specified in the paper. Given that the claim is reproducibility-sensitive, this is a material omission. At minimum, report METIS 5.x.y and the key parameters (ncuts, niter, numbering).

2. **Subgraph size distribution.** The paper now explains *why* AR is seed-invariant (smaller subproblems with fewer near-optimal solutions) but does not report the distribution of subgraph sizes across states. A table showing the range of subgraph vertex counts for each level would substantiate the theoretical explanation.

3. **Boundary-swap convergence guarantee.** The rebalancing algorithm as described is a greedy heuristic; there is no guarantee it terminates within 50 iterations for all states. The paper should note whether convergence was verified across all 50 states or only the two focal states.

## Recommendation

Accept with minor revisions. The paper has resolved the two principal outstanding weaknesses. The remaining concerns are presentation-level, not structural. The NC/GA divergence finding remains the paper's central empirical contribution and is well-supported by the seed-invariance data. This is publishable work.
