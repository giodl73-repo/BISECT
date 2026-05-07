# Review 2 — Jonathan Rodden
**Paper**: F.3: Resolution Selection for State Legislative Redistricting: When Census Tracts Are Too Coarse
**Round**: R1
**Score**: 3/4

## Summary

F.3 is primarily a methods paper, but the MAUP analysis in Section 5 has direct political relevance: the finding that resolution choice can swing partisan outcomes by ±2 seats out of 98 in WA House is a politically significant result that connects to debates about who controls technical redistricting decisions. My review focuses on this MAUP finding and its implications.

## Strengths

The MAUP policy implication (Section 5.4) is well-stated: "For algorithmic redistricting to be credibly non-partisan, the resolution selection rule must be stated in advance and applied consistently." This is the correct framing. If resolution choice is a politically consequential decision (±2 seats in WA, +1 seat in TX), then the rule for making that choice must be pre-specified and justified independently of partisan outcomes. The k/n > 0.05 rule, stated in this paper, does exactly that.

The WA MAUP finding (+2 Democratic seats from tract to block-group resolution) is credible. The Puget Sound area has a sharp urban-rural boundary along the Cascade Range and the I-90 corridor, making boundary placement sensitive to the exact resolution of geographic units in competitive suburban areas. The 2-seat shift out of 98 is small in relative terms but could be decisive in a closely divided chamber.

## Concerns

**C1 — Political direction of MAUP not consistently noted.** The WA MAUP result is +2 Democratic seats (59D vs. 57D). The TX MAUP result is +1 Democratic seat (73D vs. 72D). Both MAUP effects favour Democrats, which is consistent with the hypothesis that block-group resolution captures denser urban geography more precisely. The paper presents these as individual results without noting the consistent directional pattern. If the MAUP effect systematically favours Democrats (because block groups capture urban density more precisely than tracts, and Democratic voters are more urban), this is a politically important finding that should be highlighted.

**C2 — MAUP effect size relative to election outcomes.** The paper reports the MAUP effect in seat count (±2 seats) without relating it to the margin of partisan control. In WA, the current House partisan composition is relevant context: if Democrats hold 57 seats and Republicans 41 seats (a 16-seat margin), a 2-seat MAUP effect is small. If the chamber is closely divided, a 2-seat MAUP could determine control. The paper should provide the enacted partisan composition of WA House as context for the 2-seat MAUP finding.

**C3 — MAUP analysis uses a single resolution pair.** The MAUP analysis compares tract to block-group resolution for three states. It does not analyse the MAUP from block-group to block resolution for the five states with k/n_bg > 0.20 (NH, WY, VT, SD, ND). If resolution choice is politically consequential at the tract-to-block-group level, it is presumably also consequential at the block-group-to-block level for these five states. The paper should either extend the MAUP analysis to the block-group-to-block comparison or explicitly note this as a gap.

**C4 — MAUP and enacted maps.** The MAUP analysis compares two algorithmic maps (tract vs. block-group resolution) but does not compare either to the enacted map. For political purposes, the relevant question is not only "does resolution choice shift outcomes by 2 seats" but also "is the algorithmic map at either resolution closer to or farther from the enacted map's partisan outcome?" Adding a column showing the enacted map's partisan outcome for WA, TX, and CA would contextualize the MAUP finding politically.

## Recommendation

Accept with minor revisions. The MAUP analysis is the most politically important finding in the paper. C1 (directional pattern of MAUP effect) and C2 (relationship to actual partisan control) would significantly strengthen the paper's political relevance.
