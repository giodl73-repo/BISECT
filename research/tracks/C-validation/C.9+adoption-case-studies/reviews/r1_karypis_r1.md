# Review 1 — Reviewer: George Karypis (Algorithm Design / Implementation)
**Paper:** C.9 — From Algorithm to Map: Implementation Case Studies for Three Adoption Pathways
**Round:** 1
**Score:** 3/4

## Summary

This paper is primarily a policy and legal paper rather than a technical one, but it contains significant technical claims about the redist system's implementation that I can evaluate. The workflow table is practical and useful. The main technical weakness is that the paper makes claims about algorithmic performance (criterion satisfaction, compactness comparison to CRC enacted map) without providing the supporting data.

## Strengths

The workflow table (Table 1, Section 5) is the paper's most practical contribution. The mapping from redistricting stage to responsible actor to redist command is exactly the kind of operational specification that an adopting jurisdiction would need. The differentiation across three pathways (commission, statute, court-order) at each stage is clear and correct.

The California criteria analysis (Section 3.2) is technically accurate. The mapping from California's six statutory criteria to redist weight parameters is correctly done: population equality → hard constraint; VRA → VRASection; contiguity → adjacency graph invariant; county splits → county-sticky weight; compactness → METIS edge-cut minimization; communities of interest → Commission adjustment. The priority ordering (VRA before county-sticky) is correctly stated.

The island tract handling (Section 6.1, item 3) is a technically important detail that deserves mention: coastal islands and offshore census tracts require special handling, and the paper correctly identifies this as a preprocessing concern.

## Weaknesses and Concerns

Several technical claims are asserted without supporting data:

1. Section 3.2 states that "Polsby-Popper and Reock compactness scores for California algorithmic districts exceed the CRC's enacted map on average." This is a strong empirical claim that should be supported by the actual comparison data. If this comparison has been run, cite or include the data. If it hasn't been run, do not make the claim.

2. Section 3.2 also states that "VRASection produces more majority-minority districts than the compactness-only baseline" with a citation to "dellaLibera2026vra." But this paper is not available for review, and the claim is central to the California VRA criterion satisfaction argument. The paper should include sufficient technical detail about VRASection's performance on California data to support the criterion-2 claim independently.

3. Section 6.1 states that "total data preparation time for a single state is approximately 30 minutes to 4 hours depending on state size." This claim is credible for the redistricting/adjacency-building stage, but it should be broken down: download time (network-dependent), adjacency graph construction (computation-dependent), and validation time (dataset-dependent).

The paper's treatment of the algorithm parameter choices for each jurisdiction (AZ, CA, NC) is vague. The workflow table lists "redist state --state AZ --year 2020 --version v1" but does not specify which parameter values (ufactor, acounty, aswing, wvra) would be used. For an implementation paper, this is a significant gap. The parameter choices should be justified for each jurisdiction, or the paper should note that parameter selection is covered by B.17 and cross-reference that paper.

## Minor Issues

- The VRASection threshold for VRA protection is described as "census blocks with ≥40% minority population" in B.17 but the C.9 paper notes that "the specific threshold for VRA protection can be set to reflect the CVAP percentages required by VRA analysis rather than the total population." This is an important distinction (CVAP vs. total population) that is inconsistent between papers and should be resolved.
- The county-sticky weight calibration for California ("larger counties receiving higher weights") is mentioned but not operationalized. What specifically is the recommended calibration for LA County vs. Alpine County?

## Recommendation

Accept with minor revisions. Support the California compactness comparison claim with data, and specify the parameter values for each case study jurisdiction.
