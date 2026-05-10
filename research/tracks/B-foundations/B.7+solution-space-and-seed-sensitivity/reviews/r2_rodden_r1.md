---
reviewer: Jonathan Rodden
round: 1
score: 3
date: 2026-05-05
---

## Summary

B.7 characterises the solution space of METIS recursive bisection through a massive seed sweep and uses the results to defend the DIA's single-seed specification. From a political science perspective, this is one of the more important papers in the B-series because it directly addresses the partisan manipulation concern — if seed selection matters for partisan outcomes, then whoever controls the seed controls the map. The paper's finding that seed selection matters far less than algorithm selection is politically significant. However, the partisan analysis is limited to two states (Wisconsin and North Carolina) and the paper does not adequately engage with the geographic-sorting literature, which offers an alternative explanation for why seed sensitivity is low.

## Strengths

- **The partisan dominance finding is the paper's most important contribution.** The comparison of within-algorithm seed variance (≤2 seats) to between-algorithm partisan gap (B.0: 12.8pp ≈ 1-2 seats for these states) is the right comparison and is well-framed. This is exactly the result that would be presented in expert testimony to rebut a claim that the DIA seed was strategically chosen.
- **The outlier analysis is intellectually honest.** The paper does not claim low seed sensitivity universally — it correctly identifies Georgia and North Carolina as outliers and provides a structural (non-ad hoc) explanation for why. This kind of honest characterisation of exceptions strengthens the overall argument.
- **The SHA-256 derivation makes the legal argument.** The observation that the DIA seed is derived from the census release identifier via SHA-256, making it unknowable before the census is published, directly refutes the argument that the seed was chosen to favour a particular party. This is the right legal argument and it is clearly stated.

## Weaknesses / P1 Items (Required Fixes)

- **The partisan analysis covers only 2 states.** Tables 4 and 5 report partisan seat distributions across seeds for Wisconsin and North Carolina. But the paper's abstract claims "partisan seat-share variance across 10,000 seeds is at most 2 seats for any state." This claim requires data for all 43 non-trivial states, not just 2. Either provide a table reporting seed-driven partisan variance (min seats, max seats, SD) for all 43 states, or acknowledge that the "any state" claim is limited to the 2 states reported.
- **Geographic sorting is not tested as a mechanism.** The paper's discussion (Section 5.1) attributes low seed sensitivity to the planar separator structure of census tract graphs: "optimal separators are approximately equally distributed around the graph's 'waist,' so many seeds find similar separators." This is a graph-theoretic explanation. But the redistricting literature (Chen and Rodden 2013, your own work) suggests that geographic sorting of partisan preferences — Democrats concentrating in cities — produces similar maps regardless of seed because the urban/rural geographic structure constrains the separator location. These are not mutually exclusive explanations, but the paper should test which one is doing the work: is seed sensitivity correlated with geographic polarisation (measured, e.g., by urban-rural partisan split), or is it primarily driven by k and graph structure?
- **The cross-census partisan analysis is absent.** The paper uses 2020 presidential election data to measure partisan outcomes. But the DIA seed formula changes with the census release identifier, so a different seed is used for each census cycle. The paper should at minimum note that the partisan-sensitivity analysis is specific to the 2020 election data, and that different election data (or different district assignments) could produce different partisan outcomes for the same seed.

## P2 Items (Suggestions)

- **Compare seed variance to within-ensemble variance.** The paper compares seed variance to the B.0 algorithm family gap. A more complete comparison would also include the within-ensemble variance for a ReCom run on the same states (how much does a neutral ensemble vary in partisan outcomes?). If ReCom shows higher partisan variance than the 10,000-seed METIS sweep, that would further support the claim that METIS concentrates solutions near a small number of compact configurations.
- **Report the epsilon-ball size by state.** Table 3 reports the 83% ± 12% epsilon-ball figure aggregated nationally. State-level epsilon-ball sizes would help practitioners understand which states have tightly concentrated solution spaces (good for DIA defensibility) vs. loosely concentrated ones (requiring more caution with single-seed specification).

## Score: 3 — Minor Revision

The "any state" partisan claim (P1.1) is an overstatement that needs to be backed up with complete data. The geographic-sorting mechanism test (P1.2) would substantially strengthen the discussion but is not required if the authors prefer to hedge it as future work. The cross-census note (P1.3) is a framing fix.
