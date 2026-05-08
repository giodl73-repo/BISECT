# Mean-Median Difference in Congressional Redistricting
**Series**: L.2
**Status**: Accepted 3.5/4
**Target**: Journal of Politics

## Algorithm / Subject
Analysis of the Mean-Median Difference (MM) as a partisan fairness metric for congressional redistricting. The MM is defined as $\text{MM} = \bar{v}_D - \tilde{v}_D$, where $\bar{v}_D$ is the mean Democratic vote share across all districts and $\tilde{v}_D$ is the median Democratic vote share. The range is typically $[-0.10, +0.10]$; positive values indicate Democratic geographic disadvantage (Democratic votes concentrated in fewer high-margin districts, pulling mean above median), negative values indicate the reverse. The paper investigates whether MM captures genuine gerrymandering signal or merely reflects geographic sorting of partisan voters — and how bisect algorithmic redistricting interacts with both.

## Key Claims
1. MM captures geographic sorting effects that exist independently of intentional gerrymandering: in NC and WI, the natural geographic sorting of Democratic voters into urban cores generates a baseline MM of approximately 0.03–0.04 even in algorithmically neutral maps, establishing that MM > 0 alone is not evidence of manipulation.
2. Bisect maps reduce MM vs. enacted by 60–70% on NC ($k=14$) and WI ($k=8$): enacted NC MM $\approx 0.07$ vs. bisect NC MM $\approx 0.02$, and enacted WI MM $\approx 0.06$ vs. bisect WI MM $\approx 0.02$; the residual in bisect maps reflects geographic sorting, not algorithmic bias.
3. MM is less sensitive to election-specific partisan swings than EG: across four election cycles (2016–2020), the standard deviation of MM estimates for NC is approximately 0.015, roughly half the standard deviation of EG estimates (approximately 0.030), making MM more stable for single-election analysis.

## Layer
Legal

## Empirical Targets
- States: NC ($k=14$), WI ($k=8$), TX ($k=38$), plus FL ($k=28$) for partisan analysis
- Data: VEST 2020 presidential precinct returns mapped to 2020 census tracts
- Compare: bisect algorithmic maps vs. enacted 2022 congressional maps

## Legal / Practitioner Value
MM was introduced by Wang (2016) and has been cited in NC redistricting litigation (Harper v. Hall) alongside EG as a complementary metric less sensitive to landslide elections. Its main legal limitation is the geographic sorting confound: opponents of gerrymandering claims argue that any positive MM merely reflects where Democrats choose to live, not mapmaker choices. This paper resolves the confound empirically — by comparing enacted maps to bisect maps holding geography constant, the enacted-vs-bisect MM gap isolates the mapmaker contribution. Courts can interpret the gap (approximately 0.05 for NC) as the portion of MM attributable to redistricting choices rather than residential patterns, a distinction that strengthens gerrymandering claims under state "free and equal elections" standards.

## Section Structure
§1 Introduction, §2 Mathematical Definition and Properties, §3 Behavior Under Algorithmic Redistricting, §4 Empirical Results, §5 Legal Landscape, §6 Conclusion
