# Review 1 — George Karypis
**Paper**: F.3: Resolution Selection for State Legislative Redistricting: When Census Tracts Are Too Coarse
**Round**: R1
**Score**: 3/4

## Summary

F.3 is the methods paper for resolution selection. It derives and validates the k/n > 0.05 threshold, presents empirical comparisons for WA, TX, and CA, and analyzes MAUP effects of resolution choice. The paper makes a genuine methodological contribution: a principled, pre-specified rule for an important algorithmic choice that has previously been made ad hoc.

## Strengths

The three-part derivation of the 0.05 threshold is well-structured. The population balance feasibility argument (Section 3.1) correctly identifies that the 0.5% constraint is achievable when each district contains ≥20 units (with the CLT argument), even though the "too pessimistic" calculation based on max tract population would suggest a much tighter restriction. The compactness optimisation richness argument (Section 3.1) cites the exponential growth in configuration count with m ≥ 20 tracts, which is the correct framing for why METIS edge-cut minimisation becomes meaningful at that scale.

The empirical comparison for WA (Table 1) is the strongest validation: k/n = 0.067 just above the threshold, block-group giving +0.030 PP and -0.40pp deviation versus tract. The CA comparison (Table 3) correctly shows negligible improvement at block-group for k/n = 0.009. The TX comparison (Table 2) at k/n = 0.028 shows +0.008 PP improvement — small enough to justify the rule's classification of TX as not requiring block-group resolution.

## Concerns

**C1 — The 0.05 threshold is empirically calibrated, not theoretically derived.** Section 3.1 presents three motivations but the third ("empirical calibration") is the effective operative one: "PP scores begin to diverge between tract and block-group resolution when k/n > 0.05, with the block-group score being higher." The CLT argument for 20-units-per-district (Section 3.1, population balance feasibility) derives a threshold for achievability, not for optimality: with 20 units per district, the CLT ensures that sum variance is small relative to mean, but this does not uniquely determine the 0.05 threshold. A derivation that k/n ≤ 0.05 is the critical point where CLT guarantees kick in — not just where they "kick in enough" — would be more rigorous. As stated, the threshold is an empirical heuristic dressed as a derivation.

**C2 — Washington tract count.** F.3 states k/n = 0.067 for Washington House (k=98, n_tracts=1,458). This is confirmed: 98/1,458 = 0.0672. Block-group count: 4,874 (stated in F.3 and F.1). Block-group resolution ratio: 98/4,874 = 0.0201. These are internally consistent.

**C3 — MAUP results for high-k chambers (±2--4 seats).** Section 5.4 states that for high-k/n chambers (k/n > 0.05), the MAUP effect is ±2--4 seats. The WA specific result is +2 seats (57D → 59D from tract to block-group resolution). The claim "±2--4 seats" generalises from one data point (WA). The other high-k/n states in the sample (TX, CA) have k/n below 0.05 by the rule, so WA is the only data point in the paper for the "high-k/n" MAUP category. The generalisation to "±2--4 seats" needs either more states or explicit qualification that it is a one-state estimate.

**C4 — Resolution auto mode description.** Section 3.2 describes --resolution auto as automating the k/n > 0.05 rule. But Section 3.2 also states the rule requires looking up n_tracts "from the 2020 Census TIGER/Line tract shapefile for the state." If --resolution auto reads this from a configuration file, does the pipeline require the tract shapefile to be downloaded even for block-group-resolution runs? The data dependency between the auto-selection and the tract data should be clarified.

## Recommendation

Accept with minor revisions. The methodological contribution is genuine. C1 (threshold derivation rigor) and C3 (MAUP generalisation from single data point) are the most important concerns.
