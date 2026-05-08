# Declination: Geometric Partisan Fairness and Its Legal Status
**Series**: L.4
**Status**: Accepted 3.5/4
**Target**: Journal of Politics

## Algorithm / Subject
Analysis of Declination ($\delta$) as a partisan fairness metric for congressional redistricting, with special attention to its legal status. Declination was proposed by Warrington (2018) as a geometric measure of packing and cracking that operates directly on the vote-share diagram. Given the sorted Democratic vote shares $v_1 \leq v_2 \leq \cdots \leq v_k$, let $\bar{v}_D$ be the mean of Democratic-won districts (those above 0.50) and $\bar{v}_R$ be the mean of Republican-won districts (those below 0.50). The angles $\theta_D = \arctan(2\bar{v}_D - 1)$ and $\theta_R = \arctan(1 - 2\bar{v}_R)$ measure how far each party's wins are from 0.50 (cracking pushes opponents' wins far below 0.50, increasing $\theta_R$; packing concentrates opponents' wins far above 0.50, increasing $\theta_D$). Declination is $\delta = 2(\theta_D - \theta_R)/\pi$; positive values indicate Republican advantage. The paper provides the first systematic analysis of declination under algorithmic redistricting and a comprehensive account of why courts have not adopted it.

## Key Claims
1. Declination captures packing and cracking patterns more directly than EG because it operates on the geometric structure of the vote-share distribution rather than on aggregate wasted-vote counts: a map that cracks Democratic voters into many districts just below 50% produces a large positive $\delta$ even when EG may be modest due to offsetting patterns in Republican-won districts.
2. Bisect algorithmic maps produce $|\delta| < 0.10$ across NC ($k=14$), WI ($k=8$), and TX ($k=38$) — compared to enacted NC declination of approximately $+0.35$ — because compactness-driven bisection does not systematically place Democratic voters just below 50%, the spatial signature of cracking.
3. Declination has not been accepted as evidence by any federal or state court as of 2026; it was introduced as an exhibit in Common Cause v. Rucho (M.D.N.C. 2018) but the court did not rely on it, and subsequent state court redistricting opinions in NC, PA, and OH have cited EG, MM, and Bias without mentioning declination — the paper documents this legal status explicitly and discusses the methodological critiques (sensitivity to the number of districts won by each party, undefined when one party wins all seats) that may explain judicial non-adoption.

## Layer
Legal

## Empirical Targets
- States: NC ($k=14$), WI ($k=8$), TX ($k=38$), plus FL ($k=28$) for partisan analysis
- Data: VEST 2020 presidential precinct returns mapped to 2020 census tracts
- Compare: bisect algorithmic maps vs. enacted 2022 congressional maps

## Legal / Practitioner Value
Declination's legal status is explicitly undeveloped: no court has adopted it, and it was not among the metrics endorsed by any of the expert witnesses whose testimony shaped the Rucho opinion. Expert witnesses considering declination should be prepared to defend it against three methodological critiques documented in the paper: (1) undefined when one party sweeps all seats; (2) sensitive to the precise seat threshold (50% or plurality); (3) no well-established threshold for presumptive unconstitutionality. The paper's contribution is not to rehabilitate declination for court use but to show empirically that bisect maps produce near-zero declination as a byproduct of compactness, providing a reference point for future expert witnesses if the metric gains legal traction.

## Section Structure
§1 Introduction, §2 Mathematical Definition and Properties, §3 Behavior Under Algorithmic Redistricting, §4 Empirical Results, §5 Legal Landscape, §6 Conclusion
