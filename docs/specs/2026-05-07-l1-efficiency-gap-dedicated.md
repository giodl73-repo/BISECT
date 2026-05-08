# Efficiency Gap: Dedicated Treatment
**Series**: L.1
**Status**: Accepted 3.5/4
**Target**: American Political Science Review

## Algorithm / Subject
Dedicated treatment of the Efficiency Gap (EG) as a standalone partisan fairness metric for congressional redistricting. The EG, introduced by Stephanopoulos and McGhee (2015), measures the asymmetry of wasted votes between parties: $\text{EG} = (\text{Wasted}_D - \text{Wasted}_R) / \text{Total Votes}$, where wasted votes for the winning party are votes above the 50%+1 threshold and for the losing party are all votes cast. Positive EG favors Republicans (more Democratic votes wasted); negative EG favors Democrats. The paper provides dedicated analysis of EG behavior under bisect algorithmic redistricting, extending the partial treatment in C.5 with multi-election sensitivity analysis and full legal history of the 8% threshold from Stephanopoulos and McGhee (2015) through Gill v. Whitford (2019) and Rucho (2019).

## Key Claims
1. Bisect NC maps produce $|\text{EG}| < 0.03$ compared to the enacted NC map's $|\text{EG}| = 0.09$, a threefold reduction attributable directly to compactness-driven avoidance of packing and cracking.
2. EG is sensitive to landslide elections — a uniform 5-point swing in statewide vote share shifts the NC EG by approximately 0.04, requiring multi-election averaging (2016, 2018, 2020 presidential; 2018 Senate) to produce a stable estimate; single-election EG estimates have standard deviation ~0.03 across election cycles.
3. The 8% EG threshold proposed by Stephanopoulos and McGhee (2015) as a presumptive unconstitutionality standard was explicitly not adopted by SCOTUS in Gill v. Whitford, 585 U.S. 48 (2018), and has no federal constitutional weight post-Rucho; its use in expert testimony requires disclosure that it is an academic proposal, not a legal standard.

## Layer
Legal

## Empirical Targets
- States: NC ($k=14$), WI ($k=8$), TX ($k=38$), plus FL ($k=28$) for partisan analysis
- Data: VEST 2020 presidential precinct returns mapped to 2020 census tracts
- Compare: bisect algorithmic maps vs. enacted 2022 congressional maps

## Legal / Practitioner Value
EG was the primary metric in Gill v. Whitford (2018) and Whitford v. Gill (7th Cir. 2016), which produced the most detailed federal judicial discussion of any partisan fairness metric before Rucho closed the federal door. State courts in NC (Harper v. Hall) and WI have cited EG as one of several indicators. Expert witnesses must navigate the Gill standing doctrine, the academic origin of the 8% threshold, and the sensitivity to election choice — the paper provides a rigorous multi-election averaging protocol and discloses where EG misleads (wave elections, uncontested races, states with few competitive districts). The bisect reference distribution shows courts that EG neutrality is achievable by design, not by luck.

## Section Structure
§1 Introduction, §2 Mathematical Definition and Properties, §3 Behavior Under Algorithmic Redistricting, §4 Empirical Results, §5 Legal Landscape, §6 Conclusion
