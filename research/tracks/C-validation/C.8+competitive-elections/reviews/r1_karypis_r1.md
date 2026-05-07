# Review 1 — Reviewer: George Karypis (Algorithm Design / Computational Redistricting)
**Paper:** C.8 — Do Algorithmic Districts Produce Competitive Elections? Evidence from All 50 States
**Round:** 1
**Score:** 3/4

## Summary

This paper tests the claim that algorithmic redistricting produces more competitive congressional elections than politically drawn maps. The main finding — approximately 85 vs. 65 competitive districts — is plausible and the methodology is transparent. The paper is more a political science paper than a computer science paper, so my review focuses primarily on whether the algorithmic methodology underlying the comparison is correctly described and implemented.

## Strengths

The methodology is clearly described and appropriately scoped. The decision to use 2020 presidential vote share (Biden/Biden+Trump) as the competitive metric is well-justified: it is available for every district at the precinct level, is exogenous to specific candidates, and is the standard in the redistricting literature. The operational definitions (competitive: |D_i - 0.5| < 0.10; swing: |D_i - 0.5| < 0.05) are standard and clearly stated.

The treatment of at-large states (Alaska, Delaware, Vermont, etc.) as single districts is correct. The area-weighted interpolation for reaggregating precinct-level vote data to algorithmic district boundaries is the appropriate methodology, and the acknowledgment of measurement error is honest.

The byproduct framing in Section 4.3 is technically precise and important: competitiveness is a byproduct of compactness optimization, not a design target. An algorithm that targeted competitiveness would need partisan data, violating the anti-gerrymandering rationale. This is the correct characterization of what the algorithm does.

## Weaknesses and Concerns

The paper uses "a single randomization seed" for the algorithmic maps (acknowledged in the conclusion). This is a significant methodological weakness for the competitive elections claim. The number of competitive districts varies by seed: a different seed could produce a 38-seat-wide band around 85 (or narrower), and the paper doesn't quantify this variance. If the 85 competitive districts has a standard deviation of ±5 across seeds, the comparison to 65 under enacted maps is still significant, but if the standard deviation is ±15, the comparison is noise.

The paper references "dellaLibera2026uncertainty" for an ensemble analysis that would characterize the range of competitive-district outcomes — but this paper is not yet published and not included. The C.8 paper should either (a) run a small ensemble (10-20 seeds) to bound the variance in competitive district count, or (b) make clear that the 85-district finding is a point estimate from a single seed and that the claimed range should be verified.

The reaggregation methodology is described briefly but deserves more detail. Area-weighted vote interpolation assumes that votes are uniformly distributed within precincts, which is a standard but sometimes problematic assumption in dense urban areas where precincts may have highly concentrated partisan populations in sub-precinct areas. The paper should note whether any states had particularly problematic reaggregation (high variance between precinct boundaries and tract boundaries) and whether these states drive any of the state-level results.

## Minor Issues

- The paper counts 435 total districts but notes that at-large states are treated as single districts. Are Alaska, Delaware, Montana, North Dakota, South Dakota, Vermont, and Wyoming all included? If Montana has 2 seats (as of 2023), was a 2-district algorithmic map generated? The paper should specify the exact at-large state treatment.
- The safe Republican seat count barely changes (162 → 161) while safe Democratic seats fall substantially (208 → 189). This 19-seat asymmetry is an important substantive finding but is only briefly noted. The paper should analyze which states account for most of this asymmetry.

## Recommendation

Accept with minor revisions. Add ensemble-based uncertainty bounds on the 85 competitive district count, and detail the at-large state treatment.
