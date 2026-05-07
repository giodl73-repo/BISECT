# Review 2 — Jonathan Rodden
**Paper**: F.1: Algorithmic State House Redistricting — A 50-State Empirical Study
**Round**: R1
**Score**: 3/4

## Summary

F.1 presents the most politically relevant results in the F track: a 50-state comparison of algorithmically generated state house maps against enacted maps. The county-split reduction findings (18% mean reduction, 30 of 38 states) and the compactness advantage (34 of 38 states, mean +0.081 PP points) are the empirical core. This is a paper I expect will be cited in redistricting litigation, which means it needs to be precise about what it can and cannot claim.

## Strengths

The county-split analysis is politically well-designed. The observation that states with aggressive gerrymanders (PA: -41%, WI: -36%, OH: -37%) show the largest reductions while commission states (CA: -9%, CO, MI) show smaller reductions is exactly the kind of analysis that connects algorithmic redistricting to the documented gerrymandering literature. The pattern is consistent with the hypothesis that enacted gerrymanders deliberately crack communities, and the algorithmic baseline reveals this cracking.

The four case studies are well-chosen. WA, TX, NH, and NE span the range from typical (WA at the threshold, TX below it) to extreme (NH as the hardest case) to unusual (NE unicameral). The case studies provide qualitative depth that the 50-state table cannot.

## Concerns

**C1 — No partisan outcome analysis despite 50-state scope.** The paper's limitations explicitly disclaim partisan outcome analysis ("outside the scope of this paper"), deferring to precinct-level data requirements. However, F.2 reports partisan outcomes for 30 states using Kuriwaki precinct data. Given that the paper's political motivation is gerrymandering, the absence of any partisan seat-share analysis from the primary empirical paper in the track is a significant gap. At minimum, F.1 should report whether the algorithmic state house maps would be Democratic or Republican leaning in the states where enacted maps are identified as gerrymanders.

**C2 — The comparison group (enacted maps) is not systematically described.** Section 6 compares algorithmic maps to "enacted 2020 state house maps" for 38 states. The 12 states without comparison maps are listed as a limitation but not identified. Were the 12 states with missing data non-randomly selected? If the 12 missing states are disproportionately commission states (which produce more comparable maps) or small states (which produce trivially compact maps), the 38-state sample may be biased toward showing a larger algorithmic advantage.

**C3 — Urban-rural geography not controlled in compactness comparison.** The paper reports that algorithmic maps achieve higher compactness than enacted maps in 34 of 38 states. However, the paper does not control for the fact that states with aggressive gerrymanders tend to have more politically polarised geographies (urban-rural divide), which independently reduces the achievable compactness of any map. A regression of compactness advantage on the Efficiency Gap or DRA partisan score of the enacted map would strengthen the causal interpretation.

**C4 — Wisconsin seat-share figure in F.2 partisan table.** F.2's partisan outcomes table (which I reviewed) shows Wisconsin House: 43D/56R under NestSection. F.1 reports Wisconsin county splits of 18 (vs. 28 enacted), a 36% reduction. These two papers should be cross-referenced for consistency: the Wisconsin algorithmic map that achieves 43D/56R (minority status) with 36% fewer county splits is specifically relevant for the gerrymandering literature (the REDMAP Wisconsin case).

## Recommendation

Accept with minor revisions. The empirical contribution is strong. C1 (partisan outcome gap) is the most important concern for the paper's political relevance. The authors should consider adding even a limited partisan analysis (e.g., using the Kuriwaki data already used in F.2) to the primary empirical paper.
