> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied.

---

# Review Round 3: Why Single-Objective Graph Partitioning Outperforms Multi-Constraint Optimization for Asymmetric Redistricting Goals

**Reviewer**: Jonathan Rodden (Stanford University)
**Expertise**: Political geography, geographic sorting, partisan representation, redistricting
**Round**: 3 (First review)
**Date**: 2026-05-05

## Summary

First review of this paper. I approach it from a political geography and redistricting perspective, focusing on the paper's empirical claims about VRA compliance in Southern states with dispersed minority populations.

## Strengths

**1. The state-dependency finding is the most politically important result in recent redistricting algorithms literature.**

Multi-constraint optimization completely failing in Alabama and Louisiana across 28 parameter configurations is not a technical curiosity — it is a policy-critical finding. These are precisely the states where VRA litigation has been most active (*Allen v. Milligan* challenged Alabama's congressional map; *Robinson v. Ardoin* challenged Louisiana's). The finding that multi-constraint formulations cannot achieve VRA compliance in these states regardless of parameter tuning provides algorithmic evidence for a political geography explanation: the geographic dispersion of Black voters in the Black Belt region and urban cores, separated by suburban non-Black populations, creates a structural constraint conflict that no parameter choice can resolve.

From a political geography perspective, this result aligns with the geographic sorting literature. In states like Alabama and Louisiana, the minority population distribution is not compact enough (in the Gingles Prong 1 sense) to be captured by a single constraint operating on an equal-population partition. Edge-weighted optimization succeeds where multi-constraint fails because it treats geographic proximity as the primary optimization criterion, allowing the algorithm to draw around natural demographic concentrations rather than imposing an algebraic target that the geography cannot support.

**2. The balanced experimental design eliminates the key confound.**

The 140-vs-140 comparison is important for political credibility as well as methodological fairness. In redistricting litigation, parties challenging an algorithmic result will scrutinize whether the method was cherry-picked. Showing that 28 different parameter configurations of multi-constraint all fail in Alabama — not just the one the authors happened to test — provides the kind of exhaustive evidence that withstands challenge.

**3. The Gingles Prong 1 and functional opportunity threshold additions are legally appropriate.**

The acknowledgment that 53.6% majority does not guarantee functional electoral opportunity, and that Southern states' minority population distributions often satisfy geographic compactness conditions for Black Belt clustering but not for statewide minority population, correctly situates the algorithmic findings in the legal context.

## Weaknesses

**1. The paper does not analyze partisan outcomes of the algorithmic plans.**

This is a significant gap from a political science perspective. The paper is entirely about VRA compliance (majority-minority district counts), but redistricting has a second major dimension: partisan outcomes. In Alabama, Louisiana, and South Carolina — all states where multi-constraint fails to achieve VRA compliance — the algorithmic plans that succeed (edge-weighted) should be analyzed for their partisan implications. Are these plans more or less favorable to Republicans or Democrats than enacted plans? This analysis is not required for the VRA compliance argument, but it is essential for understanding the full political consequences of the method.

**2. The aggregate minority VAP definition overstates minority opportunity in some states.**

The paper now includes a paragraph acknowledging that aggregate minority VAP is an upper bound on group-specific (Black-only) analysis. However, this acknowledgment does not go far enough for South Carolina. South Carolina's "minority" population includes substantial Hispanic and Asian populations in the Upstate region (Greenville, Spartanburg) that have very different geographic distributions from the Black population in the Lowcountry and Pee Dee regions. Treating these as a single "minority" aggregate may make the 3 MM target appear demographically feasible when 3 majority-Black districts may not be. The paper should note this specifically for South Carolina.

**3. No discussion of whether the failure in Alabama is an artifact of the state's post-Milligan district configuration.**

Alabama's required MM count (2 districts) was established by *Allen v. Milligan* (2023), which found that Alabama's prior congressional map violated the VRA by creating only 1 majority-Black district when 2 were feasible. The paper uses this post-*Milligan* requirement but does not note that the feasibility of 2 MM districts was contested before the Supreme Court ruled. If the paper's algorithm had been run as evidence in *Milligan*, would it have supported or undermined the Court's conclusion? This historical connection would significantly increase the paper's legal relevance.

## P1 Items

None blocking.

## P2 Items

- **Partisan outcome analysis**: Compute efficiency gap and mean-median difference for the successful edge-weighted configurations in Alabama, Georgia, Louisiana, and Mississippi. Show whether algorithmic VRA-compliant plans have different partisan implications than enacted plans.

- **South Carolina minority disaggregation**: Note that SC's aggregate minority target may be misleading because Black and Hispanic/Asian populations are geographically separated. Recommend group-specific analysis for SC.

- **Allen v. Milligan connection**: Add one paragraph noting the paper's relevance to the question litigated in *Milligan*: can 2 majority-Black congressional districts be drawn in Alabama that are reasonably compact? The edge-weighted result (yes, 2 MM at 53.6%) directly answers this question.

## Score

**Score: 3.5/4 — Accept with Minor Revisions**

The paper's core contribution — demonstrating that multi-constraint formulations categorically fail in states with dispersed minority populations while edge-weighted formulations succeed — is important and well-supported. The partisan outcome analysis is the most significant missing element from a political science perspective. The *Allen v. Milligan* connection is a high-value addition that requires no new computation, only one paragraph of legal context. With these additions, I would move to 4/4.

**Recommendation**: Accept with minor revisions. *Allen v. Milligan* connection and partisan outcome analysis are the priority additions.
