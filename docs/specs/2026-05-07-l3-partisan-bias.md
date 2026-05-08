# Partisan Bias in Congressional Maps
**Series**: L.3
**Status**: Accepted 3.5/4
**Target**: American Political Science Review

## Algorithm / Subject
Analysis of Partisan Bias as a metric for evaluating congressional redistricting plans. Partisan Bias is defined as $\text{Bias} = S(0.50) - 0.50$, where $S(v)$ is the seat share won by Democrats when they receive exactly $v = 0.50$ of the statewide two-party vote. It is estimated from the observed district vote shares by applying a uniform swing: each district's Democratic vote share is shifted by $\delta = 0.50 - \bar{v}_D$ (the gap between statewide actual and 50%), and the resulting seat share is computed. Bias = 0 means the party winning exactly half the votes wins exactly half the seats at the 50% vote-share pivot; positive Bias favors Democrats, negative favors Republicans. The paper documents Partisan Bias behavior under bisect algorithmic redistricting and provides a rigorous treatment of the uniform swing assumption and its limitations.

## Key Claims
1. Partisan bias is zero by construction for any map that is symmetric under a uniform swap of Democratic and Republican vote shares — the key insight is that symmetry in the seats-votes function implies zero bias at all vote-share levels, not just at 50%, making Partisan Bias a special case of the broader symmetry standard.
2. Bisect algorithmic maps produce near-zero bias ($|\text{Bias}| < 0.02$, i.e., less than one seat in a 14-seat delegation) on NC ($k=14$), WI ($k=8$), and TX ($k=38$), while enacted NC maps produce $\text{Bias} \approx -0.07$ (favoring Republicans by approximately one full seat).
3. Partisan Bias requires a swing model — the uniform swing assumption (shifting all districts by the same $\delta$) is a first-order approximation that fails when there is substantial heterogeneity in district-level responsiveness; the paper documents that uniform swing overestimates Bias magnitude by approximately 15% for TX due to the high proportion of safe seats, and proposes a heteroskedastic-swing correction.

## Layer
Legal

## Empirical Targets
- States: NC ($k=14$), WI ($k=8$), TX ($k=38$), plus FL ($k=28$) for partisan analysis
- Data: VEST 2020 presidential precinct returns mapped to 2020 census tracts
- Compare: bisect algorithmic maps vs. enacted 2022 congressional maps

## Legal / Practitioner Value
Partisan Bias is part of the "symmetry standard" framework (Grofman and King, 2007) that preceded and informed the EG debate. It has been cited in expert testimony in NC, PA, and OH redistricting cases. Its primary legal strength is intuitive clarity: a jury-friendly statement that "if the parties swapped vote totals, the seat outcome would flip by X" directly communicates asymmetry without requiring jurors to understand wasted-vote algebra. Its primary legal weakness is the swing model dependency — courts and opposing experts can challenge the uniform swing assumption, and the paper provides a formal response by documenting the heteroskedastic correction and showing the qualitative conclusion (near-zero for bisect, substantial for enacted) is robust across swing model choices.

## Section Structure
§1 Introduction, §2 Mathematical Definition and Properties, §3 Behavior Under Algorithmic Redistricting, §4 Empirical Results, §5 Legal Landscape, §6 Conclusion
