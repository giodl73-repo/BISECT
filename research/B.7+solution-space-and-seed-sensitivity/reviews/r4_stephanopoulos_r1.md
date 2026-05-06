---
reviewer: Nicholas Stephanopoulos
round: 1
score: 4
date: 2026-05-05
---

## Summary

B.7 is the most legally consequential paper in the B-series: it directly addresses whether the DIA's single-seed specification could constitute a partisan manipulation of the redistricting process. The paper's answer — that seed selection drives at most 2 seats of partisan variance, while algorithm selection drives the overwhelming majority of partisan outcomes — is exactly the finding needed to defend the DIA against seed-based challenges. The legal framing is tight and the evidence is directly on point. This paper is close to acceptance; the P1 issues are matters of precision rather than substance.

## Strengths

- **The legal argument is airtight.** Section 5.2 develops the two-part defence: (1) the DIA seed is unpredictable before the census release (SHA-256 derivation from census release identifier), and (2) even if a party could predict the seed, the partisan effect of seed choice is at most 2 seats — smaller than the effect of algorithm choice. This two-part argument anticipates and defeats both possible legal challenges.
- **The comparison to the B.0 algorithm-family gap.** The paper's most effective result is the comparison of within-algorithm seed variance (≤2 seats) to the between-algorithm partisan gap (12.8pp from B.0). This contextualises the seed choice as a second-order effect, dominated by the first-order choice of algorithm family. This framing is exactly right for expert testimony.
- **The T=600 ConvergenceSweep connection.** The last-improvement seed distribution data directly supports the B.16 threshold. The paper correctly presents this as providing the "statistical foundation" for T=600, though the formal connection needs strengthening (see P1.2).

## Weaknesses / P1 Items (Required Fixes)

- **The "any state" claim in the abstract and conclusion overstates the results.** The abstract states "partisan seat-share variance across 10,000 seeds is at most 2 seats for any state." The partisan analysis in Section 4.5 covers only Wisconsin and North Carolina. For the "any state" claim to hold, data must be provided for all 43 non-trivial states. If space constraints prevent full reporting, the abstract should be hedged to "for the two highest-variance states (GA, NC), seat-share range is at most 2 seats — the broadest range observed."
- **The T=600 threshold connection needs a formal statement.** Section 4.4 shows that 47 of 50 states have last improvement before seed 600, and the three exceptions (GA, NC, WI) use T=1,200. But the paper does not state what confidence level or error rate this implies for the ConvergenceSweep procedure. A Bayesian or frequentist statement — "with the T=600 threshold, the probability of missing a significant improvement (>X% below current best) is less than Y%" — would make the legal certification of the ConvergenceSweep defensible.
- **The partisan analysis uses presidential election data that may not align with redistricting criteria.** Section 3.4 assigns districts based on 2020 presidential election majority vote share. But redistricting law uses election data from multiple races (Secretary of State, Attorney General, state legislative) to avoid confounding from candidate-specific effects. The paper should note this limitation explicitly and acknowledge that the 2-seat variance bound may differ under multi-race partisan assignment methods.

## P2 Items (Suggestions)

- **Add a timeline showing seed unpredictability.** A brief narrative — "the census release date is public; the release identifier is published on that date; SHA-256 produces the seed within seconds; no party can compute the seed before the release identifier is published" — would make the unpredictability argument concrete for a legal audience.
- **Consider reporting seed sensitivity separately for the GeoSection (B.8) and county-sticky (B.10) configurations.** Courts might ask whether seed sensitivity changes under DIA's non-default configurations. The limitation section acknowledges this is an open question; even preliminary data on one configuration would strengthen the paper's generalisability claim.

## Score: 4 — Accept

This paper makes a direct and compelling legal argument about the DIA's seed specification. The P1 issues are fixes to precision and scope, not to the core finding. I would be comfortable recommending acceptance with minor revisions.
