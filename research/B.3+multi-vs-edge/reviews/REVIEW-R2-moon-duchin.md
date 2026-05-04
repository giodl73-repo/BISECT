> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Review Round 2: Why Single-Objective Graph Partitioning Outperforms Multi-Constraint Optimization for Asymmetric Redistricting Goals

**Reviewer**: Moon Duchin (Rutgers University)
**Expertise**: Redistricting, gerrymandering detection, geometric probability, VRA compliance, GerryChain
**Round**: 2 (Revision Review)
**Date**: 2026-05-02

## Summary of Revisions Received

I gave this paper a 3/4 in Round 1 and asked for stronger engagement with VRA legal standards, legally accurate minority population definitions, geographic compactness analysis, and geographic visualizations. In Round 1, my concerns were classified as P2 (important but not blocking). The revision concentrated on the P1 items identified by Karypis and Phillips — implementation, balanced design, and statistical rigor — with only limited attention to my domain-specific concerns. I review the revision with this in mind.

## P1 Resolution Assessment

### P1-1, P1-2, P1-3, P1-4 — RESOLVED

The P1 items have been satisfactorily resolved by my co-reviewers' accounts. I particularly note that the balanced 140-vs-140 design reveals a result I find important from a redistricting standpoint: multi-constraint's complete failure in Alabama and Louisiana across all 28 parameter values confirms that the method is not merely sub-optimal in those states — it is categorically incapable of creating the required minority opportunity districts under any tested configuration. From a legal and practical standpoint, this is the most policy-relevant finding in the paper. The statistical framing in Section 5.6 (2.7% upper bound for zero-success states) provides the kind of probabilistic confidence that a redistricting commission or court expert would need to make actionable recommendations.

The implementation bug fix (30.0% corrected vs 35.0% buggy MC success) is scientifically appropriate and the paper is stronger for it.

## P2 Domain Concerns: Partial Progress

### P2-2: VRA Legal Standards — Partially Addressed

The revision adds citations to *Allen v. Milligan* (2023) and *Gingles* (1986), which I requested, and the language "VRA requires creation of MM districts" has been softened. This is progress. However:

- The paper still does not mention *Gingles* Prong 1's geographic compactness requirement, which is directly relevant to whether the edge-weighted districts would survive legal scrutiny beyond the numerical threshold.
- The discussion of Alabama's 53.6% minority best-case edge-weighted district (Table 2) does not address whether 53.6% is sufficient for functional electoral opportunity. The redistricting literature generally targets 55–60% to account for turnout differentials and age distributions (non-citizen and under-18 populations reduce effective voting share). A brief note on this limitation would strengthen the paper's applicability.

I accept that full legal analysis is outside the scope of this paper's contribution. A one-paragraph disclaimer acknowledging these limitations would be sufficient for the domain audience.

### P2-3: Minority Population Definition — Insufficiently Addressed

The "all non-white" aggregate definition remains unchanged. The paper now cites *Bartlett v. Strickland* in passing, but does not engage with the core concern: VRA Section 2 analysis should be group-specific, not aggregate. Alabama's 36.9% "minority" is primarily Black/African American (approximately 26.8%), not a legally recognized coalition.

I recognize that rerunning all experiments with group-specific populations is a major undertaking. For the revised version I would accept: (1) a clear statement that the analysis uses aggregate minority VAP as an approximation, (2) acknowledgment that Black VAP is the legally operative definition for the southern states in the study, and (3) a note that the numerical results are likely conservative (aggregate minority ≥ Black-only, so success rates with aggregate are upper bounds on what would be achieved with group-specific analysis). This framing is honest without requiring new computation.

### P2-4: Compactness and Maps — Still Absent

This is my most significant remaining concern. The paper still contains no geographic visualizations. For a redistricting paper submitted to any venue that reaches the redistricting community, the absence of maps is conspicuous. I cannot assess whether the districts produced by edge-weighted optimization are geographically realistic or whether the 48% edge-cut penalty in Louisiana translates to legally problematic shapes.

I recognize that Polsby-Popper computation requires geometric data that may not be easily accessible. However, a single Alabama map showing the edge-weighted best-configuration district assignment — even a rough schematic generated from tract centroids — would address the core concern. This is a visualization request, not a computation request.

## Score and Recommendation

**Score: 3.5/4 — Accept with Minor Revisions**

The paper has improved substantially. The P1 items are resolved and the statistical section is convincing. The state-dependency finding is a genuine contribution to both the algorithms literature and redistricting practice. The correction of the implementation bug and the expanded parameter sweep together provide a far more credible empirical basis for the paper's claims.

My remaining concerns are all in P2 territory and do not require new experiments:
1. Add a paragraph on *Gingles* Prong 1 geographic compactness and the 55–60% functional opportunity threshold (legal accuracy)
2. Add an explicit framing statement about aggregate vs group-specific minority VAP (legal accuracy)
3. Add at least one state map (Alabama edge-weighted best configuration) — this is achievable with existing data

These three additions would make the paper fully suitable for the redistricting domain audience as well as the algorithms audience. Without them, the paper serves the algorithms community well but provides limited direct guidance to redistricting practitioners and courts.

I support publication after a final revision pass. Given that my concerns are all minor additions rather than structural changes, I do not require re-review.

**Verdict**: Accept with minor revisions.
