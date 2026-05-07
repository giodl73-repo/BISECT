# Review 2 — Reviewer: Jonathan Rodden (Political Geography / Voting Behavior)
**Paper:** D.5 — Quantifying VRA Section 2 Evidence with Algorithmic Redistricting: A Gingles Prong-by-Prong Methodology
**Round:** 1
**Score:** 3/4

## Summary

This paper develops a systematic methodology for Gingles analysis that I find well-motivated and largely well-executed from a political geography standpoint. The Alabama worked example is convincing, and the post-*Callais* Prong 3 methodology is the most important methodological contribution. My concerns are about the ecological inference model choice and the geographic independence assumption in the bootstrap.

## Strengths

The use of contested primary elections for Prong 2 analysis (Section 3.3) is the right methodological choice. Primary elections in which candidates of the same party compete are the cleanest test of racial cohesion independent of party. The paper correctly notes that general election results conflate racial and partisan cohesion, and that *Callais* requires disentangling them.

The WLS ecological regression is standard and appropriate for precinct-level data. The Alabama Prong 2 estimates (β̂_min = 0.84-0.91 across five elections) are consistent with well-established findings from the literature on Black political cohesion in Alabama. These estimates are credible.

The geographic sorting discussion in Section 5.3 (the note that the new Black Belt district "follows county lines through the Black Belt region, with the irregular geometry attributable to population-balance requirements rather than racial gerrymandering") is an important characterization that expert witnesses will need to defend. The reference to county lines as the explanatory framework is geographically accurate for the Black Belt.

## Weaknesses and Concerns

The ecological inference model choice (Goodman ecological regression in Prong 2, WLS with partisan control in Prong 3) is adequate but not state-of-the-art. Gary King's (1997) EI model — which the paper cites but does not use — provides more accurate estimates in the presence of the ecological inference bias problem that WLS is subject to. For legal contexts where the accuracy of racial voting behavior estimates is contested, using the best available estimator matters.

The paper should justify the WLS choice over the King EI model explicitly. The likely justification is computational tractability (King's EI requires Markov chain Monte Carlo and can be slow for large precinct datasets) or software availability. But in an expert witness context, opposing counsel will note that the WLS estimator is less accurate than EI, and the expert should have a prepared response.

The bootstrap confidence intervals (Section 6.2) use independent precinct-level resampling ("draws precincts with replacement"). But precincts within a county or district are spatially autocorrelated — voting behavior in adjacent precincts is correlated due to neighborhood sorting. Ignoring spatial autocorrelation in the bootstrap produces confidence intervals that are too narrow, understating the uncertainty in the ecological regression estimates. The paper should use clustered bootstrap (resampling clusters at the county or district level) or at minimum acknowledge that the reported CIs may be anti-conservative.

The Alabama example shows a Prong 3 racial coefficient dropping from 0.71 (no partisan control) to 0.43 (with partisan control). The paper presents this as evidence that 0.43 is the "true" racial bloc voting component. But the partial correlation interpretation requires that the partisan control variable (presidential vote share) is not a mediator — that is, that the relationship between precinct racial composition and majority bloc voting does not partly operate through partisan alignment. In highly racially polarized states like Alabama, racial composition and partisan alignment are so highly correlated that the "partisan control" may be absorbing substantial racial signal. The paper's response to the "partisan control absorbs true racial effects" challenge in Section 7.2 is reasonable but does not fully address this identification issue.

## Minor Issues

- The paper's description of the Black Belt district "traversing the Black Belt counties (Wilcox, Dallas, Lowndes, Marengo, Greene, Hale, Perry) from the Georgia border toward Mobile" appears geographically imprecise. The Black Belt counties generally run east-west across south-central Alabama, and a district connecting them to the Georgia border would need to extend significantly northeast. Is this description accurate? A map or geographic verification would help.
- The Prong 2 analysis uses "the 2018 Democratic Senate primary (Jones vs. competition)." Doug Jones's 2017 Senate victory in the special election (not 2018 general) is the relevant Doug Jones election. The 2018 primary may be the right election for Prong 2 cohesion analysis, but the characterization should be precise.

## Recommendation

Accept with minor revisions. Justify the WLS choice over the King EI model explicitly, use clustered bootstrap for confidence intervals, and verify the geographic description of the Black Belt district.
