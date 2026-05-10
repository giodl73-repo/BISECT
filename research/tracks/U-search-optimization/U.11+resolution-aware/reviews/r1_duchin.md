# R1 Review — Moon Duchin
**Paper**: U.11 Resolution-Aware Redistricting: Geographic Granularity as a First-Class Parameter
**Score**: 3/4
**Verdict**: Minor Revision

## Summary
This paper formalizes geographic resolution as a first-class parameter for redistricting pipelines, providing correctness proofs for GEOID-prefix partition derivation and county adjacency construction, plus an audit manifest extension. The contributions are technically sound and practically motivated. The main weaknesses are the underpowered empirical section — the 27% autocorrelation claim is from a single run — and a question about whether Option B's county-level Markov chain belongs to the same distributional class as standard ReCom, which the paper does not address.

## Strengths
- The formalization of GEOID prefix structure (Definition 1 and Theorem 1) gives a rigorous foundation for what practitioners do informally. The orphan-detection guarantee is valuable and not available in existing tools.
- Theorem 2 (county adjacency correctness) provides a proof that the induced criterion captures geographic adjacency, with appropriate caveats about water boundaries in Remarks 1 and 2.
- The manifest extensions in Section 4 are well-designed: the `fine_to_coarse_formula` field enables independent partition reconstruction, which is the right invariant for an auditable redistricting tool.
- The three multi-scale options (A/B/C) are cleanly organized with explicit data requirements, implementation status, and CLI reference — the sort of practical completeness that makes a paper useful.

## Concerns
- **County-level MH approximation (same class as standard ReCom?)**: Section 3.3 states that the coarse-level accept/reject decision "is not an exact Metropolis-Hastings ratio — it is the same approximation as standard ReCom applied to the county-level graph." This is technically correct but may be read as more reassuring than it should be. Standard ReCom's stationary distribution is well-characterized (weight proportional to number of balanced bipartitions). Does Option B's multi-scale chain have the same stationary distribution as single-scale ReCom at tract level? Or does the county coarsening change the stationary measure? The paper should state clearly whether the two chains (single-scale tract, Option B multi-scale) are claimed to sample the same distribution or only approximately the same distribution. If the latter, what is the known gap?
- **Autocorrelation claim from single run**: The 27% lag-100 autocorrelation reduction is footnoted as "Estimated value from a single 2000-step run, seed $s=42$." This is the headline empirical result of the paper. A single 2000-step run is not sufficient to estimate autocorrelation with any meaningful confidence interval — especially for lag-100, which requires substantially more steps to estimate precisely. The paper should either (a) replace this with multi-run estimates, (b) state explicit uncertainty bounds on the single-run estimate, or (c) more strongly caveat this as preliminary and remove the percentage from the abstract.
- **No comparison of plan-space distributions**: The paper compares autocorrelation but does not compare the plan-space distributions produced by Option B and single-scale tract ReCom. If the distributions differ (which they may, given the different proposal mechanism), practitioners need to know this before choosing Option B for litigation. A Wasserstein distance or partisan-distribution comparison would strengthen the empirical section.
- **Coarse tolerance heuristic lacks formal guarantee**: Section 3.3 provides a heuristic argument for the 3× coarse tolerance factor, but acknowledges it is "a practical heuristic, not a formal guarantee." For states with unequal county populations (California example), the heuristic may break down. The paper should state the conditions under which the fine-level perturbation is guaranteed to recover population balance, or bound the failure probability.

## Required Changes (P1/P2)
- **P1**: Clarify whether Option B multi-scale chains are claimed to sample the same stationary distribution as single-scale tract ReCom. If not, characterize the known difference or state explicitly that this is an open question.
- **P1**: Either provide multi-run autocorrelation estimates with confidence intervals, or move the 27% figure out of the abstract and into the empirical section as a preliminary observation. A single 2000-step run does not support a headline percentage claim.
- **P2**: Add at minimum a brief comparison of partisan-distribution summaries (e.g., mean D-seat share, ensemble spread) between Option B and single-scale runs to verify that the two chains explore similar regions of plan space.
- **P2**: State formal conditions on county population equality under which the 3× coarse tolerance guarantee holds, or acknowledge that the heuristic may require tuning for highly unequal county-population states.
