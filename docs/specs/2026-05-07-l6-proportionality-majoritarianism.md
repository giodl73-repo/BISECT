# Proportionality vs. Majoritarianism in Congressional Redistricting
**Series**: L.6
**Status**: Accepted 3.5/4
**Target**: Election Law Journal

## Algorithm / Subject
Normative paper examining the foundational tension between two theories of representational fairness in single-member district systems: proportional representation (a party winning $v\%$ of votes should win approximately $v\%$ of seats) and majoritarianism (the party winning a plurality of votes should be rewarded with a seat-share bonus that magnifies its mandate). The paper applies the Gallagher Index ($\text{LSq} = \sqrt{\tfrac{1}{2}\sum_i (v_i - s_i)^2}$) as the standard measure of disproportionality, documents the majoritarian tradition in Anglo-American electoral law, and evaluates bisect algorithmic maps under both standards. Covers the legal argument that neither standard is constitutionally required post-Rucho but that state constitutions may impose proportionality obligations — citing PA Supreme Court (League of Women Voters v. Commonwealth, 2018), NC Supreme Court (Harper v. Hall, 2022), and their respective constitutional language.

## Key Claims
1. Bisect algorithmic maps achieve Gallagher Index scores in the range $[0.03, 0.08]$ across NC, WI, and TX — significantly below enacted maps ($[0.10, 0.18]$) but not zero, because single-member districts structurally preclude perfect proportionality: even a perfectly neutral map with $k$ equal-population districts will produce a majoritarian seat bonus when vote shares are not uniformly distributed across districts, an inherent feature of geographic representation rather than mapmaker bias.
2. Under the majoritarian standard, bisect maps produce a seat bonus of approximately 1.3–1.5 (the leading party wins 30–50% more seats than its vote share would indicate under strict proportionality) — comparable to the historical Anglo-American average and substantially below the enacted NC bonus of approximately 2.1 (more than double the proportional seat entitlement).
3. Post-Rucho, neither the proportionality nor the majoritarianism standard is federally enforceable (Davis v. Bandemer, Vieth v. Jubelirer, and Rucho together foreclose federal courts from imposing either standard), but PA's "free and equal elections" clause (Art. I, §5) has been interpreted by the PA Supreme Court to prohibit maps that deviate from proportionality "far beyond what any neutral criteria would produce," and NC's "elections shall be free" clause (Art. I, §10) was the basis for Harper v. Hall — making the bisect reference distribution (neutral-criteria maps) directly relevant to measuring that constitutional threshold in state court.

## Layer
Legal

## Empirical Targets
- States: NC ($k=14$), WI ($k=8$), TX ($k=38$), plus FL ($k=28$) for partisan analysis
- Data: VEST 2020 presidential precinct returns mapped to 2020 census tracts
- Compare: bisect algorithmic maps vs. enacted 2022 congressional maps

## Legal / Practitioner Value
This paper occupies a distinct niche from L.1–L.5: rather than measuring a map's partisan properties, it asks which normative standard courts should apply and whether that choice is constitutionally compelled. The practical payoff is a principled answer to the defendant's most common response to partisan gerrymandering claims — "proportionality is not required" — by showing that (a) the claim is correct post-Rucho at the federal level, (b) state constitutional text may impose a softer proportionality floor, and (c) regardless of which standard governs, bisect maps satisfy both more closely than enacted maps. Expert witnesses can use this paper to frame the normative choice as a legal question for the court (not an empirical question for the expert) while still providing empirical evidence that is relevant under either standard.

## Section Structure
§1 Introduction, §2 Mathematical Definition and Properties, §3 Behavior Under Algorithmic Redistricting, §4 Empirical Results, §5 Legal Landscape, §6 Conclusion
