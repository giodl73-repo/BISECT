# Review 3 — Moon Duchin
**Paper**: F.6: Voting Rights Act Compliance for State Legislative Redistricting — The 42% Threshold at Chamber Scale
**Round**: R1
**Score**: 3/4

## Summary

F.6 examines whether the 42% threshold for majority-minority district formation transfers from congressional to state house scale. The paper finds it does. My review focuses on whether the theoretical argument for threshold stability (Section 3.1) is correct, whether the Callais regression is properly specified, and whether the ensemble context affects the interpretation.

## Strengths

The theoretical argument for threshold stability (Section 3.1) is the paper's most important mathematical contribution. The argument is: "the 42% threshold represents the minimum initial minority share from which the algorithm can aggregate to 50% within the geographic constraint of a single contiguous district." The geometric argument is that below 42%, aggregating to 50% requires extending the district geographically until the compactness constraint becomes infeasible. This argument is scale-invariant "provided that the spatial distribution of minority populations is not dramatically different at different geographic resolutions." This is a non-trivial assumption: minority populations may be more or less spatially concentrated at the block-group level than at the tract level (the ecological inference problem). The paper should discuss whether there is evidence that the spatial distribution is comparable at tract and block-group resolution for the five covered states.

The Callais regression specification (Eq. 1) is correctly formulated: boundary_ij regressed on MinPop_ij (difference in minority population share between adjacent tracts), PartyShare_ij (difference in Democratic vote share), and geographic controls (distance, shared boundary length, county membership). The use of HC3 robust standard errors is appropriate for a boundary-pair regression with potential spatial correlation.

## Concerns

**C1 — Scale-invariance assumption for 42% threshold.** The theoretical argument says the threshold is "scale-invariant: it does not depend on the absolute size of the district (in population or area), only on the geographic distribution of the minority population relative to the district's scale." But "relative to the district's scale" is doing a lot of work here. At state house scale (47,000 persons per district), the relevant geographic unit for aggregation is smaller than at congressional scale (764,000 persons per district). The 42% threshold is derived from the geometry at tract resolution: how many adjacent tracts with minority share below 42% must be added before the district-level minority share falls below 50%? At block-group resolution, the same geometric argument applies but with block groups, which are smaller and more numerous. If minority concentration patterns are more (or less) granular at block-group resolution, the effective threshold may shift. The paper asserts scale-invariance but this requires the minority concentration geometry to be self-similar across resolutions, which is an empirical question.

**C2 — β₁/β₂ ratio interpretation.** The paper reports β₁/β₂ ratios ranging from 23 (Georgia) to 157 (Mississippi) and treats high ratios as evidence of race-neutrality. But the ratio β₁/β₂ is meaningful only when β₂ is non-zero. For Louisiana, β₂ = -0.002 (0.024) is not statistically different from zero, and the "effectively infinite" ratio correctly captures that partisan composition plays no role. For Georgia, β₂ = 0.006 (0.017) is also not statistically different from zero (t = 0.35), yet the paper reports a finite ratio of 23. The ratio should only be interpreted when β₂ is statistically significant; otherwise the paper should simply report that β₂ ≈ 0 and the Callais requirement is satisfied trivially. Reporting a finite ratio of 23 for Georgia implies that partisan composition explains 1/23 of what minority population explains, which is not meaningful when the partisan composition coefficient is statistically indistinguishable from zero.

**C3 — Ensemble context for VRASection.** VRASection adjusts the METIS partition post-hoc to create majority-minority districts. The paper evaluates this for a single-seed (seed 42) partition. Different seeds could produce different initial METIS partitions, and VRASection might not succeed in creating majority-minority districts in all seeds. The sensitivity of the VRASection results to seed choice should be tested, particularly for the borderline cases (South Carolina) where success at state house scale depends on the ability to aggregate minority tracts within a compact district.

**C4 — "4 of 5 states" — what is the failure case?** Table 1 shows South Carolina meeting the threshold ("Threshold met: Yes") at state house scale. But Table 1's note says South Carolina fails at congressional scale "because the Black population in South Carolina is geographically dispersed." At state house scale, South Carolina succeeds (28 majority-minority districts). So VRASection succeeds in all 5 covered states at state house scale. Yet the abstract says "4 of 5 covered states" and the introduction says "4 of 5 states." This is the same "4 of 5" claim as at congressional scale, but the table appears to show 5 of 5 at state house scale. This inconsistency must be resolved.

## Recommendation

Revise and resubmit. C4 (4/5 vs. 5/5 states at legislative scale) is a substantive inconsistency. C2 (β₁/β₂ ratio when β₂ is not significant) is a methodological concern.
