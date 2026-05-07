---
reviewer: Jonathan Rodden
round: 1
score: 3
date: 2026-05-05
---

## Summary

B.10 addresses a constitutionally significant problem: how to operationalise the "preserve political subdivisions" criterion that appears in 34 state constitutions. The county-sticky approach is a practical and well-motivated solution, and the empirical results are concrete and interpretable. From a political science perspective, the paper correctly identifies that county preservation is a non-partisan criterion (counties are geographic units, not partisan units) and demonstrates that the alpha_c = 3.0 default achieves substantial split reduction at minimal compactness cost. My concerns are about the geographic verification of the county split counts and the political science treatment of county identity.

## Strengths

- **The Iowa case study is excellent.** Iowa's four-region geography (northwest, northeast, southwest, southeast) is exactly the kind of traditional geographic understanding that county preservation is meant to protect. The reduction from 12 to 3 county splits, with the 3 remaining splits explained by population constraints (Polk County, university towns), is a clear and convincing validation of the approach.
- **The California validation is honest.** The paper correctly predicts that California will see minimal benefit (41 to 38 splits, mostly unavoidable) because most of California's counties exceed one district target. This honest characterisation of a case where the approach has limited benefit strengthens the overall credibility.
- **The legal criterion operationalisation is precise.** The paper's formulation of "preserve to the extent possible" as a soft constraint (penalise but don't prohibit intra-county cuts) is a principled operationalisation that tracks the language of constitutional provisions, which use "to the extent possible" or "where practical" language.

## Weaknesses / P1 Items (Required Fixes)

- **The county split counts are not verified against TIGER data.** The paper states that the B.2 baseline produces 487 county splits nationally. This is a specific quantitative claim that should be verifiable from the TIGER/Line tract-to-county FIPS mapping. The paper should state how this count was computed: did the authors compute county splits from the actual district assignment output, or is it derived from a formula? If computed from actual outputs, what was the TIGER data vintage (2020 TIGER/Line boundaries), and are county FIPS codes taken from tract-level TIGER data (which may differ from county-level TIGER data due to water areas and boundary updates)?
- **The partisan implications of county preservation are not discussed.** Counties in the United States are not partisan-neutral geographic units: they have systematic partisan patterns (rural counties lean Republican, urban counties lean Democratic). County preservation that avoids splitting urban counties tends to concentrate Democratic voters, while preserving rural county integrity tends to maintain Republican-leaning districts. The paper should acknowledge this and test whether the 34% county-split reduction at alpha_c = 3.0 produces a systematic partisan effect (even a small one). The claim that county preservation is "non-partisan" needs empirical support, not just theoretical assertion.
- **The 34-state constitutional requirement is mentioned but not operationalised.** The paper states "county preservation is a constitutionally required criterion in 34 state redistricting frameworks" but does not list which 34 states or differentiate between constitutional provisions (mandatory) and statutory provisions (directory). For the 16 states without such requirements, is the DIA default of alpha_c = 3.0 still appropriate? The paper should provide a list of the 34 states and note whether the alpha_c = 3.0 default should be state-conditional.

## P2 Items (Suggestions)

- **Compare the county-sticky results to Iowa's independent redistricting commission output.** Iowa has an independent nonpartisan redistricting body (Legislative Services Agency) that has operationalised county preservation for decades. Comparing the alpha_c = 3.0 results to Iowa's actual enacted congressional maps would provide an external validity check on whether the algorithm achieves the kind of county preservation that human mapmakers have traditionally aimed for.
- **Add a map of Iowa at alpha_c = 1.0 and alpha_c = 3.0.** A side-by-side map showing the baseline vs. county-sticky result for Iowa would make the county-preservation benefit visually concrete and would be compelling for practitioners.

## Score: 3 — Minor Revision

The county split count verification (P1.1) and the partisan implications test (P1.2) are the primary issues. The 34-state list (P1.3) is a documentation gap that is straightforward to fix. The paper's core contribution is sound.
