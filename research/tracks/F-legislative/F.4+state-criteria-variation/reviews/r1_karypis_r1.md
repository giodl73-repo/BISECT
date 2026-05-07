# Review 1 — George Karypis
**Paper**: F.4: Satisfying 50 Different Rule Sets — State Constitutional Redistricting Criteria and Algorithmic Adaptation
**Round**: R1
**Score**: 3/4

## Summary

F.4 develops a five-type taxonomy of state redistricting regimes and maps each category of legal criterion to an algorithmic parameter. The paper argues that a single METIS recursive bisection architecture, parameterized through YAML configurations, can satisfy the dominant legal criteria in all 50 states. This is primarily a policy/legal paper, but the algorithmic mapping sections have technical content that I review here.

## Strengths

The algorithm mapping in Section 3 is technically correct and well-specified. The compactness mapping (compactness parameter scaling edge weights by shared boundary length) is the right approach: edge-weight proportional to shared boundary length converts the minimum-cut objective into an approximation of perimeter minimisation, which is what compactness requirements call for. The county preservation mapping (county_weight multiplier on cross-county edges) is similarly well-designed and the effective range (1.5 to 3.0) is appropriate — values above 3.0 would create population balance conflicts as noted.

The partisan neutrality mapping — structural rather than parametric — is the paper's most important algorithmic insight: the algorithm cannot satisfy a partisan objective because it lacks the data to compute one. This is qualitatively different from "we promised not to use partisan data" and the paper correctly emphasises this distinction.

## Concerns

**C1 — Community-of-interest weight matrix specification.** Section 3.3 describes the community-of-interest mapping using a weight modifier δ_uv < 1 for tracts in different COI units. This inverts the usual logic: lower weight means the algorithm is less reluctant to cut the edge, which means it is more willing to put adjacent tracts in different districts. But if COI preservation means keeping communities together, the algorithm should be reluctant to cut COI-crossing edges, which requires higher weight (not lower). The paper's δ_uv < 1 for cross-COI edges appears to be backwards: it would make the algorithm prefer to separate communities rather than preserve them. This is a potential sign error in the COI mapping that should be verified.

**C2 — Population tolerance accumulation.** Section 3.5 states that the accumulated balance error is "bounded by O(log k) times the single-level error." For congressional redistricting with k=52, this gives approximately log₂(52) ≈ 5.7 levels, so the accumulated error is at most approximately 5.7× the single bisection error. At 0.5% tolerance per level, this could accumulate to 2.9% — outside the Wesberry standard. The paper claims 0.5% is consistently achievable, which must mean the per-level error is much less than 0.5%/log k. This reconciliation should be explained.

**C3 — YAML configuration examples not provided.** The abstract promises "one model configuration per state type" and Section 4 (model configs) is referenced throughout. The model YAML configurations should be included in the paper or as a reproducible appendix: a reader wanting to implement California's commission criteria needs to see what coi=true, compactness=0.85, vra=true, county_weight=1.5 actually looks like in the configuration file format.

**C4 — Florida as the only Type V state.** The paper classifies Florida as the sole unambiguous Type V (restrictive) state. This requires verifying whether Amendment 6 (2010) is truly stricter than Arizona's AIRC requirements (which include "promote competitive elections" — substantive outcome requirements). The paper places Arizona in Type III (commission) rather than Type V, but Arizona's "competitive elections" criterion is arguably more partisan-fairness-oriented than Florida's "no partisan intent" prohibition. The classification should be justified more explicitly.

## Recommendation

Accept with minor revisions. C1 (COI weight direction) is a potential technical error that must be verified and corrected if wrong.
