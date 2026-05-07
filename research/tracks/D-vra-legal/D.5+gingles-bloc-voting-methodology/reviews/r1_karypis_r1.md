# Review 1 — Reviewer: George Karypis (Algorithm Design / Statistical Computing)
**Paper:** D.5 — Quantifying VRA Section 2 Evidence with Algorithmic Redistricting: A Gingles Prong-by-Prong Methodology
**Round:** 1
**Score:** 3/4

## Summary

This paper is primarily a legal and statistical methodology paper rather than an algorithm paper, so my review focuses on the statistical and computational aspects. The WLS+HC3+Holm methodology for Prong 3 is correctly implemented and well-described. The VRASection alignment score for Prong 1 is a novel algorithmic contribution that deserves more technical elaboration. The Alabama worked example is appropriately chosen.

## Strengths

The WLS ecological regression for Prong 2 is correctly specified. The estimator β̂_min (estimated minority vote share for the minority-preferred candidate) is the standard quantity of interest in racially polarized voting analysis. The use of HC3 heteroskedasticity-consistent standard errors is appropriate for precinct-level data, and the citation to MacKinnon and White (1985) is correct.

The Prong 3 regression (Equation 3.1) is correctly formulated. The inclusion of the partisan control variable P_t (Republican presidential vote share) as the Callais disentanglement mechanism is technically correct: it isolates the racial component of majority bloc voting from the partisan component by conditioning on district-level partisan baseline. The WLS weighting (proportional to precinct turnout) is standard and appropriate.

The Holm sequential rejection procedure is correctly applied and better-powered than Bonferroni while controlling family-wise error rate. The LOO (leave-one-out) robustness check is an excellent methodological addition that addresses the election-selection uncertainty identified in Section 6.1.

The expert witness checklist (Table 2) is a practical contribution: mapping redist command outputs to legal elements of each Gingles prong provides a clear workflow for expert witnesses.

## Weaknesses and Concerns

The VRASection alignment score (Equation 2.1) lacks sufficient technical documentation for a methodology paper. The score is defined as the ratio of minority CVAP in the district to minority CVAP in the "community of interest (COI)" identified by the VRASection algorithm — but the COI identification algorithm is not described. How does VRASection identify the geographic core of the minority community? Is it a kernel density estimate? A connected-components algorithm on high-minority-CVAP tracts? A Voronoi region? The Proposition 1 (Prong 1 threshold) depends critically on the COI identification, and this identification method should be described formally.

The Alabama worked example shows alignment scores of 0.73 (7th district) and 0.61 (new Black Belt district). These point estimates have no confidence intervals in the text (the confidence intervals are described in Section 6 but not reported for the specific Alabama districts). The expert checklist says to report "5th-95th percentile alignment range" — but this range is not reported for Alabama. At minimum, the worked example should demonstrate the full methodology including the ensemble uncertainty bounds.

The bootstrap confidence interval methodology is underdeveloped. Section 6.2 states "each bootstrap resample draws precincts with replacement and refits the WLS regression" — but does not specify the number of bootstrap replicates, whether clustered bootstrap (by county or district) is used, or how the confidence interval is computed (percentile method vs. bias-corrected vs. BCa). For an expert witness methodology paper, these details matter: opposing experts will challenge the CI methodology in deposition.

## Minor Issues

- The "redist analyze --ensemble-size 1000" command in the expert checklist is the first place ensemble size is specified. This should be justified: why 1000? What is the accuracy of the 5th-95th percentile range as a function of ensemble size? For a 90% interval, the simulation standard error on the 5th/95th percentile is roughly sqrt(0.05 × 0.95 / 1000) ≈ 0.7%, which seems adequate. But this calculation should be provided.
- Table 1 (Alabama Prong 3 results) shows five elections but Section 5.3 says eight elections were analyzed. The table should include all eight elections, or the text should explain why only five are shown.

## Recommendation

Accept with minor revisions. Document the COI identification algorithm for VRASection (which underpins the Prong 1 alignment score), report ensemble uncertainty bounds in the Alabama worked example, and specify bootstrap resampling details (number of replicates, clustering, interval type).
