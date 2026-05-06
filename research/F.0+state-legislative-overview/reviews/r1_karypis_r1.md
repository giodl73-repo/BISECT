# Review 1 — George Karypis
**Paper**: F.0: Algorithmic State Legislative Redistricting — A Research Program
**Round**: R1
**Score**: 3/4

## Summary

F.0 frames a six-paper research program applying minimum-edge-cut recursive bisection to all 50 state legislatures. The overview introduces the k/n > 0.05 resolution selection rule, describes the --chamber and --resolution CLI extensions, and previews key empirical findings from F.1--F.6. The paper is clearly structured and makes a genuine contribution by systematically cataloguing the computational challenges that distinguish state legislative from congressional redistricting.

## Strengths

The resolution selection rule is sensibly motivated. The three-part justification — population balance feasibility, compactness optimisation richness, and empirical calibration from C.1 — is appropriate for a methods overview. The runtime table (Table 2) is useful and concrete: wall-clock figures for NH (420s), WA at block-group (180s), and TX at block-group (380s) anchor the discussion. The observation that runtime scales as O(k·n·log n) in the normal regime and degrades toward O(n^2·log n) when k/n approaches 1 is the kind of algorithmic precision that distinguishes this track from purely applied redistricting work.

The table of chamber scales (Table 1) effectively illustrates the problem: NH at k/n = 1.25 and WY at 0.43 immediately communicate why tract resolution is infeasible, and the values appear to be accurate counts based on 2020 TIGER data.

## Concerns

**C1 — Block-level deferral not quantified.** The paper correctly flags that NH, WY, VT, SD, and ND have k/n_bg > 0.20 and in principle require block-level resolution. However, the overview does not quantify what accuracy loss is accepted by using block-group resolution anyway. For NH with k/n_bg = 0.57, the 20-units-per-district heuristic is violated by a factor of approximately 11. The population balance figures cited (±0.5% achieved) are presented without caveating that block-group resolution is itself suboptimal for NH — a reader might incorrectly conclude the results are fully rigorous at all five states.

**C2 — Runtime scaling claim needs citation anchor.** The claim that runtime scales as O(n^2·log n) when k/n approaches 1 is stated but not derived or cited to B.5/B.6. Section 6 of F.0 references B.5 and B.6 for the normal-regime claim, but the degenerate-regime degradation is presented as a new finding without the supporting analysis that would belong in F.3.

**C3 — METIS ufactor interaction.** The --resolution flag changes the adjacency graph size by 3x but the paper does not discuss whether the METIS ufactor=5 (0.5% imbalance target) remains appropriate across resolutions. At block-group resolution with very small districts (NH: T* = 3,342 persons, block groups averaging ~150 persons), the ufactor constraint may interact with the imbalance budget in non-obvious ways.

**C4 — Vermont case study missing from Table 1.** The introduction states that VT requires block-level redistricting, yet Vermont does not appear in Table 1's chamber-scale examples. Given that Vermont is a prominent edge case (k/n = 0.81 at block-group resolution), its omission from the illustrative table leaves the discussion incomplete.

## Minor Issues

The footnote text in the F.2 section (Section 4.2) contains an in-text correction: "TN 99:33 is 3:1" — this parenthetical self-correction should be cleaned before publication. The paper is otherwise well-written and appropriate for the overview role in the track.

## Recommendation

Accept with minor revisions. The methodological framework is sound and the overview paper serves its purpose. Address C1 and C4 before final submission; C2 and C3 can be deferred to F.3 where the runtime analysis is developed in full.
