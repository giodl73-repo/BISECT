# Review 4 — Nicholas Stephanopoulos
**Paper**: F.5: Compactness at State Scale — Algorithmic State Legislative Districts Outperform Congressional
**Round**: R1
**Score**: 3/4

## Summary

F.5 provides the theoretical grounding for the compactness claims in F.0 and F.1. From a legal perspective, the paper's value lies in its quantification of a systematic compactness advantage — a quantification that could be used in litigation to establish algorithmic redistricting as a baseline against which enacted maps are measured. My review focuses on the legal usability of the paper's findings.

## Strengths

The finding that algorithmic state house maps outperform enacted state house maps by a mean of 0.062 PP units (17% improvement) in 35 states, with the largest advantages in states with identified gerrymanders, is the most legally significant finding in the F track. Courts in Pennsylvania (LWV v. Commonwealth), Ohio, and North Carolina have used compactness metrics as evidence of gerrymandering. The F.5 results provide a systematic baseline: enacted maps in these states are significantly less compact than an algorithmic baseline, and the magnitude of the compactness deficit is predictably correlated with other evidence of gerrymandering.

The cross-census stability finding (consistent advantage across 2000/2010/2020) is important for litigation purposes: it means the algorithmic baseline can be applied to all three redistricting cycles and is not specific to any one population distribution.

## Concerns

**C1 — Polsby-Popper as the operative legal standard.** Courts use various compactness measures: Polsby-Popper, Reock score, convex hull ratio, and others. Different courts have preferred different measures. Pennsylvania's Supreme Court used Polsby-Popper in its remedial analysis; Arizona's AIRC has used multiple measures. The paper uses only Polsby-Popper. For maximum legal utility, the paper should compute at least one additional compactness measure (Reock score is the most commonly used alternative) to show that the advantage is robust to metric choice.

**C2 — Gerrymandering classification not sourced.** Section 5 (Improvement over Enacted) asserts that the largest gains occur "in states where the enacted map is identified by courts or analysts as gerrymandered." The states with the largest gains (PA, WI, OH, NC) are indeed widely identified as having gerrymandered state legislative maps, but the paper does not provide a citation or list of which states are in the "identified as gerrymandered" category. For a paper that will be cited in litigation, this classification must be sourced to specific court decisions or academic analyses.

**C3 — The enacted map comparison conflates different redistricting cycles.** F.5's comparison to enacted state house maps uses "enacted 2020-cycle state house maps." But the algorithmic maps are generated using 2020 census data, 2000, 2010, and 2020. The comparison baseline (enacted maps) is only from the 2020 cycle. This means the 2000 and 2010 algorithmic maps are compared against 2020 enacted maps — a cross-cycle comparison that does not isolate algorithmic quality from population redistribution effects.

**C4 — VRA-mode status of comparison maps.** F.5 states that "VRA mode is disabled for this comparison to isolate the compactness effect." This is methodologically appropriate for isolating the compactness effect. However, in states with VRA requirements (the covered states in F.6), the legally operative algorithmic map would have VRA mode enabled, which reduces compactness in minority districts. The paper should note that the compactness advantage reported in F.5 is an upper bound on the advantage for states where VRA mode would be enabled.

## Recommendation

Accept with minor revisions. C1 (additional compactness measure for legal robustness) and C2 (gerrymandering classification sourcing) would significantly strengthen the paper's litigation utility.
