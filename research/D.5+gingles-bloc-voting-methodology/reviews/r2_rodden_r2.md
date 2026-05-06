# Review 2 — Reviewer: Jonathan Rodden (Political Geography / Voting Behavior)
**Paper:** D.5 — Quantifying VRA Section 2 Evidence with Algorithmic Redistricting: A Gingles Prong-by-Prong Methodology
**Round:** 2
**Score:** 3/4

## Summary

The revision addresses my principal concerns: the WLS vs. King EI choice is now explicitly justified, the bootstrap uses county-clustered resampling, and the geographic description of the Black Belt district is corrected. I maintain my score at 3/4. The paper is now a methodologically defensible guide for VRA expert witnesses.

## Addressed Issues

The WLS vs. King EI justification (Section 4.3, responding to my M1 concern) is now present and well-reasoned. The three-part argument — (a) WLS is more computationally tractable for 50-state deployment, (b) HC3 provides valid inference under WLS even with heteroskedasticity, (c) King EI assumes a specific distributional model that may not hold for Alabama precinct data — is adequate for expert witness purposes. The additional argument that the LOO stability analysis provides the key robustness check (which is more directly interpretable than the distributional assumptions of King EI) is the strongest of the three. This is the right justification for a legal context.

The county-clustered bootstrap (Section 6.2) directly addresses my concern about spatial autocorrelation in precinct-level resampling. The finding that county-clustered CIs are approximately 40% wider than unclustered CIs for the 2020 AL-07 general election is the quantitative demonstration I requested. The wider CIs are now reported in the Alabama worked example, and the Prong 3 conclusion remains robust under the more conservative clustered intervals.

The geographic description of the Black Belt district is corrected (Section 5.2): the description no longer says "from the Georgia border toward Mobile" — it now correctly describes the district as running east-west through the Black Belt counties (Wilcox, Dallas, Lowndes, Marengo, Greene, Hale, Perry) and connecting southward toward Mobile through southwest Alabama. This is geographically accurate.

## Remaining Concerns

The partisan control absorption problem (that in high-racial-polarization states the partisan control may absorb racial signal) is addressed in Section 7.2 and the VIF analysis is now reported (VIF=3.2 for Alabama). The response is technically adequate: VIF=3.2 is below the standard threshold of 5, indicating that the collinearity is present but not severe enough to make the racial coefficient unreliable.

However, the identification claim — that the partial correlation interpretation requires the partisan control variable is not a mediator — is only noted in a footnote rather than addressed in the main text. For a high-racial-polarization state like Alabama, the mediation question is more than a footnote concern: if Black voters are Democrats because they are Black (not vice versa), then conditioning on partisan alignment partially conditions on race. The paper's response (VIF<5) addresses collinearity but not mediation. A brief paragraph in the methodology section acknowledging this identification challenge and the argument for why conditioning is appropriate in the *Callais* context would strengthen the paper.

## Minor Issues

- The 2018 Democratic Senate primary description is now corrected: the paper refers to the Jones vs. competition race in the correct election cycle.
- The Holm vs. B-H discussion has been moved to the methodology section (as Duchin's M2 requested), with the B-H p-values reported alongside Holm p-values for Alabama as a robustness check.

## Recommendation

Accept. The WLS justification, the clustered bootstrap, and the geographic correction are genuine improvements. The identification challenge is acknowledged. The paper is ready for publication.
