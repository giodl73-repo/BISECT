# Review 1 — Reviewer: George Karypis (Algorithm Design / Statistical Computing)
**Paper:** D.5 — Quantifying VRA Section 2 Evidence with Algorithmic Redistricting: A Gingles Prong-by-Prong Methodology
**Round:** 2
**Score:** 3/4

## Summary

The revision addresses all three of my principal concerns: the COI identification algorithm is now described formally, the Alabama worked example now includes ensemble uncertainty bounds for the alignment scores, and the bootstrap specification is now precise. I maintain my score at 3/4. The paper is now a technically sound methodology guide for VRA expert witnesses.

## Addressed Issues

The COI identification algorithm is now described in Section 2.2: connected components on tracts with ≥40% minority CVAP, using queen contiguity, with the largest connected component identified as the COI. The threshold (40%) and connectivity criterion (queen contiguity) are specified. The justification for the 40% threshold (aligning with the VRA boost weight parameter) is present. This is the formal description I requested, and it makes the alignment score formula (Equation 2.1) fully reproducible.

The Alabama worked example now reports ensemble uncertainty bounds for the alignment scores: for the 7th district (Birmingham corridor), the 5th-95th percentile range across 1000 ensemble configurations is [0.68, 0.79]; for the new Black Belt district, it is [0.55, 0.68]. Both ranges are above the 0.5 threshold across the full ensemble, which is a stronger Prong 1 statement than a single-point estimate.

The bootstrap specification (Section 6.2) now states: 1000 replicates, county-clustered (sampling counties with replacement, then all precincts within selected counties), percentile CI method. The county-clustered vs. unclustered CI width comparison for the 2020 AL-07 general election is reported: clustered CI is approximately 40% wider than unclustered, confirming that spatial autocorrelation is present and that the clustered bootstrap is the right choice.

## Remaining Concerns

The ensemble size justification (why 1000 replicates?) is now present in a footnote: the simulation standard error on the 5th/95th percentile is approximately 0.7%, which is adequate for the stated precision. This is the calculation I requested in my minor issue from Round 1.

Table 1 now includes all eight elections analyzed (not just five), which resolves my minor concern about the election count discrepancy.

The VIF reporting is now present in Section 4.5 (the new collinearity diagnostics subsection): VIF=1.8 for Wisconsin and VIF=2.1 for North Carolina. The Alabama-specific VIF is reported in Section 5 as part of the worked example: VIF=3.2 for Alabama, within the acceptable range (threshold: VIF<5). This resolves Stephanopoulos's C4 concern.

## Minor Issues

- The ensemble robustness check for Prong 1 now clearly distinguishes between option (a) (does some district in the ensemble have alignment > 0.5?) and option (b) (does the same geographic district consistently appear?). The paper uses option (a), which is correctly framed as answering "can a compliant district be drawn?" This resolves Duchin's I4 concern.
- The Prong 2 WLS model is now numbered as Equation 2.1, consistent with the Prong 3 model's numbering.

## Recommendation

Accept. The COI identification description, the ensemble uncertainty bounds, and the bootstrap specification are all genuine improvements that make the methodology reproducible and defensible in expert testimony. The paper is now ready for publication.
