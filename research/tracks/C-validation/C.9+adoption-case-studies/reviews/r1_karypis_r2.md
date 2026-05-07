# Review 1 — Reviewer: George Karypis (Algorithm Design / Implementation)
**Paper:** C.9 — From Algorithm to Map: Implementation Case Studies for Three Adoption Pathways
**Round:** 2
**Score:** 3/4

## Summary

The revision addresses my principal technical concern: the California compactness comparison claim is now qualified. The paper either provides the data or withdraws the claim, as I requested. The parameter specification for each case study jurisdiction is now more developed. My score remains at 3/4 — the paper is primarily a policy and legal paper, and the technical contributions are limited by design, but they are now honest about what is and is not demonstrated.

## Addressed Issues

The California compactness claim (Section 3.2) is now qualified: the paper acknowledges that the comparison to the CRC enacted map is available upon request (from `redist analyze` outputs) but does not embed the full comparison data in the paper. This is the right choice for a primarily policy-oriented paper — making the claim verifiable without turning the paper into a data presentation exercise.

The VRASection California performance claim is now framed more carefully: rather than asserting that VRASection produces more majority-minority districts than the baseline (which required a citation to an unavailable paper), the paper now describes what VRASection's optimization does and cross-references D.5 for the Gingles framework. This is honest.

The parameter values for each case study are now specified: a table in Section 5 shows the recommended parameter choices for Arizona (AIRC pathway), California (CRC pathway), and North Carolina (court-order pathway), with brief justifications. This is the operational specification I requested.

The CVAP vs. total population consistency issue is now resolved: the paper consistently uses CVAP for VRA threshold calculations and notes the legal basis (*Hayden v. Pataki*), which is consistent with the D.5 paper's approach.

## Remaining Concerns

The VRASection technical documentation in the C.9 paper is still lighter than I would prefer. The paper references D.5 for the Gingles methodology, which is appropriate, but C.9's California VRA analysis would benefit from showing the alignment scores for the VRASection-generated California districts. California has 52 congressional districts, and the VRASection analysis would generate alignment scores for each minority-opportunity district — reporting these would make the criterion-satisfaction claim verifiable.

The island tract handling note (Section 6.1) remains a brief mention. For an implementation paper, the preprocessing steps for coastal states (California, Hawaii, Alaska) deserve more systematic treatment.

## Minor Issues

- The county-sticky weight calibration for California is now operationalized with a recommended scaling (LA County weight = 5.0, Alpine County weight = 1.0, with other counties scaled by log(population)). This is a reasonable specification that I find adequate.
- The data preparation time estimates are now benchmarked against a specific reference configuration (16-core workstation), which makes them reproducible.

## Recommendation

Accept. The compactness claim qualification and the parameter specification table are genuine improvements. The paper is now technically honest and operationally useful.
