# Review 2 — Jonathan Rodden
**Paper**: F.5: Compactness at State Scale — Algorithmic State Legislative Districts Outperform Congressional
**Round**: R1
**Score**: 3/4

## Summary

F.5 provides the theoretical explanation for the compactness advantage documented in F.1. The headline finding — state house maps are 7.5% more compact than congressional maps at tract resolution, 11% more compact at block-group resolution — is well-supported theoretically and empirically. My review focuses on the political implications of this finding and on whether the comparison to enacted maps is appropriate.

## Strengths

The cross-census stability finding (mean advantage 0.026/0.027/0.028 PP units across 2000/2010/2020) is an important validation: it shows that the compactness advantage is structural, not an artifact of any particular census cycle's population distribution or district configuration. This makes the finding more useful as a baseline for redistricting reform arguments.

The state-specific variation is politically useful: the largest advantages occur in states with many house districts relative to congressional seats (TX at 150/38, PA at 203/17, NH at 400/2), which are precisely the states where the enacted maps have been most aggressively gerrymandered. This consistency — large algorithmic advantage in gerrymandering-prone states — is consistent with the hypothesis that enacted gerrymanders sacrifice compactness for partisan gain.

## Concerns

**C1 — Gerrymandering analysis in Section 5 needs stronger evidence.** The paper reports that algorithmic maps outperform enacted maps by a mean of 0.062 PP units (17% improvement), with the largest gains in "states where the enacted map is identified by courts or analysts as gerrymandered." This connection (large gains in gerrymandering states) is asserted in the abstract but not demonstrated in the text. Is the correlation between compactness advantage and identified-gerrymandering status statistically significant? Which states are classified as "identified as gerrymandered" and by what criterion? This is the paper's most politically important claim and it needs more rigorous support.

**C2 — The baseline comparison uses different-scale maps.** The fundamental comparison in F.5 — state house PP > congressional PP — compares maps with very different k values for the same state. As the paper correctly notes, this comparison is dominated by the O(1/√k) scaling law rather than algorithmic performance. A more meaningful comparison for partisan purposes would be: holding k fixed, do algorithmic maps outperform enacted maps? F.5 provides some of this in Section 5 (comparison to enacted state house maps), but the framing of the paper leads with the congressional-to-legislative comparison, which is less politically informative.

**C3 — Maryland as an outlier.** Table 1 shows Maryland has the lowest absolute PP in both categories (congressional: 0.302; house: 0.334). The paper explains this as reflecting "geographically constrained coastal and Chesapeake Bay boundary." This is correct but understates Maryland's redistricting history: Maryland's congressional map has been repeatedly identified as one of the most aggressive Democratic gerrymanders in the country (the 3rd Congressional District being the canonical example). The paper should note that Maryland's low congressional PP partly reflects the enacted gerrymander (not just natural geography) and that the algorithmic map's 0.334 house PP, while still low, likely substantially exceeds what the enacted Maryland congressional map would show.

**C4 — Enacted state legislative map comparison (Section 5) coverage.** The abstract says comparison covers "35 states where enacted maps are publicly available in GIS format" but F.1 says 38 states with available enacted maps. The discrepancy (35 vs. 38) is unexplained. F.5 may have different availability at the time of writing than F.1. The paper should specify which 35 states are included and when the enacted map data were collected.

## Recommendation

Accept with minor revisions. The mathematical analysis is strong. C1 (gerrymandering correlation evidence) and C3 (Maryland context) are important for the paper's political relevance.
