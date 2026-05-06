# Review 5 — Reviewer: Christina Liang (Computational Social Science / Statistical Methodology)
**Paper:** D.5 — Quantifying VRA Section 2 Evidence with Algorithmic Redistricting: A Gingles Prong-by-Prong Methodology
**Round:** 2
**Score:** 3/4

## Summary

The revision addresses all four of my principal concerns: the bootstrap is now fully specified, the VIF is reported for Alabama, the power analysis is now present, and the ecological regression validation is at least acknowledged. I maintain my score at 3/4. The paper is now a methodologically defensible guide for quantitative VRA analysis.

## Addressed Issues

The bootstrap specification (my primary concern) is now fully stated in Section 6.2: 1000 replicates, county-clustered (sampling counties with replacement, then all precincts within selected counties), percentile CI method. The comparison of clustered vs. unclustered CI widths for the 2020 AL-07 general election is reported — clustered CIs are approximately 40% wider. This is the precise specification I requested, and it demonstrates that spatial autocorrelation is present and material. The county-clustered bootstrap is the right choice and it is now the default method.

The VIF diagnostic (Section 4.5) resolves the multicollinearity concern I raised. Alabama VIF=3.2 is within the acceptable range. The paper correctly interprets VIF values in the 2-4 range as indicating that collinearity is present but manageable.

The power analysis is now present: with ~2,600 Alabama precincts, minimum detectable effect size at 80% power is β_race ≈ 0.06. The observed Alabama coefficient (0.43) is 7× larger than the minimum detectable effect, which means the Prong 3 finding has very high power and a "not significant" result in Alabama would not be a false negative. This is the information I requested for courts to interpret null findings.

The ecological regression validation discussion is now more honest. The paper acknowledges in Section 3.3 that WLS ecological regression estimates individual-level behavior from aggregate precinct data, and cites the King (1997) ecological inference problem. The paper notes that Alabama individual-level validation is available in principle (voter file data exists in some counties) but has not been performed for this paper, and recommends it for future work. This is honest — the paper does not claim validation it cannot provide.

## Remaining Concerns

The ecological regression validation remains absent as an empirical contribution. The paper acknowledges the limitation and recommends it as future work, which I accept. But for a methodology paper intended to guide expert witness practice, the absence of even informal validation against individual-level data is a genuine limitation. I note it here but do not consider it blocking for publication.

The model specification assumption — that the partisan control variable (P_t = Republican presidential vote share) is orthogonal to the residual racial component — is addressed through the VIF diagnostic but the mediation issue (that Black voters are Democrats because they are Black, not independently) is acknowledged only in a footnote. As Rodden also notes, this is the deeper identification challenge. The paper's VIF approach addresses collinearity but not mediation. A one-paragraph discussion in the methodology would be appropriate.

The `redist analyze --ensemble-size 1000` command clarification is now present: the ensemble is generated specifically for the Prong 1 alignment score analysis (not reusing the redistricting ensemble), which is the correct architectural choice.

## Minor Issues

- The turnout weighting note (Section 3.2) now acknowledges that turnout weighting can produce implicit demographic weighting in minority-majority precincts, and that this is not necessarily a problem but should be acknowledged in expert reports.
- The Prong 2 WLS model is now numbered as Equation 2.1.
- All five elections in Table 1 are confirmed to be from the eight-election sample (with a note about why the other three are not shown in the main table — they are in the supplementary LOO table).

## Recommendation

Accept. The bootstrap specification, VIF reporting, and power analysis are genuine improvements that make the methodology reproducible and appropriate for expert testimony. The ecological regression validation remains a limitation honestly disclosed.
