# Review 4 — Reviewer: Nicholas Stephanopoulos (Election Law / VRA Jurisprudence)
**Paper:** D.5 — Quantifying VRA Section 2 Evidence with Algorithmic Redistricting: A Gingles Prong-by-Prong Methodology
**Round:** 2
**Score:** 3/4

## Summary

My Round 1 score of 2/4 was driven by three specific legal errors: (1) the mischaracterization of *Callais* as requiring WLS+HC3+Holm rather than requiring disentanglement generally; (2) the Prong 1 threshold resting on an unavailable citation; and (3) the Alabama worked example using the pre-litigation enacted map rather than the post-*Allen* remedial context. The revision corrects all three. I am upgrading to 3/4.

The paper is not yet at 4/4 because the Prong 1 threshold calibration, while improved, still rests on a summary statistic rather than a distributional analysis with case-level citations. But the legal errors that blocked publication in Round 1 are corrected, and the paper can now serve as a credible methodology guide for VRA expert witnesses.

## Addressed Issues

**Error 1 — *Callais* mischaracterization (C1): Corrected.**
The paper now clearly distinguishes between what *Callais* requires (disentanglement of racial from partisan bloc voting) and what this paper proposes (a specific implementation using WLS+HC3+Holm). Section 4.3 now states: "*Callais* (2026) requires statistical disentanglement of race and partisan affiliation in Prong 3 analysis; WLS+HC3+Holm is one method satisfying this requirement, used here following Footnote 36 of the *Callais* opinion." This is an important correction: the paper no longer implies that other methodologies satisfying the disentanglement requirement are impermissible. The reference to Footnote 36 of *Callais* as the source of the specific regression methodology is the right way to anchor the approach without overclaiming what *Callais* mandated.

This is a substantial legal improvement. An expert witness using this methodology can now correctly say: "*Callais* requires disentanglement, and the WLS+HC3+Holm approach in Footnote 36 is one method satisfying that requirement." An opposing expert using King EI with partisan controls cannot use this paper against the expert witness by claiming that WLS+HC3+Holm is the exclusive *Callais* method.

**Error 2 — Prong 1 threshold calibration (C2): Substantially addressed.**
The threshold calibration analysis is now included directly in Section 2.3. The paper reports: (a) the distribution of alignment scores for all VRASection-generated majority-minority districts nationally (2020 census): minimum 0.56, median 0.74, maximum 0.97; (b) districts from states where Prong 1 has been litigated and courts accepted the district: average 0.71 (reported with specific state-level examples); and (c) the paper acknowledges that no cases in its dataset have a court-rejected Prong 1 district, and that the 0.5 threshold is derived as conservative relative to the empirical minimum (0.56) rather than from a rejected-district lower bound.

This is better than the original citation to an unavailable paper. The threshold now has an empirical basis within this paper. The acknowledgment that the dataset does not include rejected-district cases (and therefore the threshold is a floor derived from the accepted-district minimum, not from the gap between accepted and rejected districts) is honest and appropriate.

**Error 3 — *Allen v. Milligan* post-remand context (C3): Corrected.**
The Alabama worked example is now explicitly framed in the post-*Allen* remedial context (Section 5.1). The paper correctly describes the algorithmic output as demonstrating how the VRASection remedy would satisfy all three Gingles prongs in the *Allen* post-remand setting, which is the actual use case for expert witnesses in remedial map proceedings. The claim that "Alabama provides an ideal worked example because the three prongs have been litigated exhaustively in the district court record" is now accompanied by a citation to the remedial proceedings and the 2024 approved map.

**Error 4 — VIF reporting (C4): Added.**
The VIF diagnostic is now reported in Section 4.5 (collinearity diagnostics): VIF=3.2 for Alabama, VIF=1.8 for Wisconsin, VIF=2.1 for North Carolina. The Alabama VIF=3.2 is within the acceptable range (threshold VIF<5). The paper correctly notes that VIF values in the 2-4 range indicate that the partisan control is adding information while not making the racial coefficient unreliable. This resolves my VIF concern.

## Remaining Concerns

The Prong 1 threshold calibration, while improved, has a gap I want to flag. The paper shows the distribution of alignment scores for accepted districts (average 0.71, minimum 0.56), which derives the 0.5 threshold as a conservative floor below the observed minimum. But this calibration has a selection bias: only districts that were challenged are in the litigation dataset. Districts with alignment scores in the 0.4-0.55 range may not have been litigated (because they were never drawn, or were not challenged). The 0.5 threshold might be too conservative (accepting districts that would not meet the legal standard) or not conservative enough (some districts below 0.5 might satisfy Prong 1 under different geographic configurations). The paper acknowledges this limitation in a footnote, which is appropriate.

The primary Prong 2 alternative when primary election data is unavailable (general elections with a partisan control) is now more prominently described as a primary alternative rather than a secondary option (Section 3.3). This addresses my Error 4 (post-*Callais* Prong 2) concern.

The "biracial elections" terminology is replaced with "cross-racial elections" throughout, as I requested in my minor issues.

The deposition readiness discussion now notes that the `redist depo` module is one component of deposition readiness, not a complete solution — and lists the additional disclosure requirements (data sources, software versions, parameter choices, workflow description). This is the expansion I requested.

## Minor Issues

- The *Callais* citation now includes the 2026 decision date and a note that no subsequent circuit-level implementation guidance has been identified as of the paper's writing date (May 2026). This is appropriate given the recency of the decision.
- Citations for pre-*Callais* practice of reporting Prong 3 without partisan control are now present (three specific cases cited in footnote 12).
- The power analysis for Prong 3 hypothesis tests (Liang's minor concern) is now present: with ~2,600 Alabama precincts, the minimum detectable effect size at 80% power is β_race ≈ 0.06, well below the observed coefficient of 0.43.

## Recommendation

Accept with minor revisions. The three legal errors that drove the Round 1 score of 2/4 are corrected. The Prong 1 threshold calibration is substantially improved, the *Allen* worked example is correctly framed, and the *Callais* characterization is now precise. The paper can now serve as a credible expert witness methodology guide. I raise my score from 2/4 to 3/4.
