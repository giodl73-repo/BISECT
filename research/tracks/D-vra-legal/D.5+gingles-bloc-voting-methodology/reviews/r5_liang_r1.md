# Review 5 — Reviewer: Christina Liang (Computational Social Science / Statistical Methodology)
**Paper:** D.5 — Quantifying VRA Section 2 Evidence with Algorithmic Redistricting: A Gingles Prong-by-Prong Methodology
**Round:** 1
**Score:** 3/4

## Summary

A methodologically sophisticated paper that makes a genuine contribution to the quantitative social science of VRA expert testimony. The WLS+HC3+Holm regression for Prong 3 is correctly implemented and clearly described. The main weaknesses are in the bootstrap confidence interval methodology and the absence of model validation for the ecological regression.

## Strengths

The two-equation framework for post-*Callais* Prong 3 analysis is correctly specified. Equation 3.1 (WLS regression of candidate vote share on majority racial composition and Republican presidential vote share, weighted by precinct turnout) isolates the racial component of majority bloc voting by partialling out partisan alignment. This is the right approach for the disentanglement problem.

The HC3 standard error estimator is the right choice for precinct-level data. HC3 is more conservative than HC1 or HC2 in small samples, which is appropriate for an expert witness context where conservative inference is preferable to liberal inference. The citation to MacKinnon and White (1985) is correct.

The Holm procedure application is methodologically correct and the advantage over Bonferroni (more power while controlling family-wise error rate) is accurately described. The LOO robustness check is a valuable addition: it tests whether the Prong 3 conclusion is driven by a single unusual election, addressing the election-selection uncertainty in Section 6.1.

The Alabama results (β̂_race dropping from 0.71 to 0.43 with partisan control) are plausible and consistent with the structure of Alabama's electorate. The Holm-corrected p-values below 0.001 across five elections provide strong evidence of racial bloc voting that is not merely partisan.

## Weaknesses and Concerns

The ecological regression lacks validation. The WLS ecological regression estimates individual-level behavior from aggregate precinct data — a well-known inference problem with a long literature (King 1997, Imai et al. 2008). The paper cites King (1997) in the limitations but does not validate the WLS estimates against any benchmark. In Alabama, individual-level voting behavior can be approximated using voter file data that links individual voters to their demographic characteristics in some counties. If any validation is available — even informal comparison of WLS estimates to voter file data in a single county — it should be reported.

The bootstrap methodology is underspecified. Section 6.2 describes "bootstrap resampling of the precinct-level data" but does not specify:
1. Number of bootstrap replicates (1000? 10000?)
2. Whether resampling is at the precinct level or clustered (by county? by district?)
3. The confidence interval method (percentile? bias-corrected? BCa?)

These are not pedantic details — they affect the width of the reported confidence intervals, which are the primary uncertainty quantification for expert testimony. Opposing counsel will ask these questions in deposition. The paper must specify the bootstrap procedure precisely.

The model specification assumes that the partisan control variable (P_t = Republican presidential vote share) is orthogonal to the residual racial component of majority bloc voting. This is the key identifying assumption of the Callais disentanglement. In Alabama, where precinct-level racial composition and Republican presidential vote share are very highly correlated (the Black Belt counties have both high Black population and very high Democratic presidential vote), the OLS estimates of β_race and β_party will have high variance due to multicollinearity. The paper should report the correlation between X_maj and P_t in the Alabama sample and the variance inflation factor (VIF) for the Prong 3 regression. If VIF > 10, the disentanglement may be insufficiently powered to separate the two effects.

## Minor Issues

- Section 3.2 states the Prong 2 model uses weights "proportional to precinct total vote." This is the turnout weighting convention, which is standard. However, the paper should note that turnout weighting can produce an implicit demographic weighting if minority-majority precincts have different turnout rates than non-minority-majority precincts. In Alabama, Black voter turnout in Democratic primaries is typically high, which means turnout-weighted estimates may upweight the minority cohesion signal. This is not necessarily a problem but should be acknowledged.
- The paper provides no discussion of statistical power. Given the precinct-level sample size in Alabama (Alabama has approximately 2,600 precincts), what is the minimum detectable effect size for the Prong 3 racial coefficient at 80% power? Knowing the minimum detectable effect size would help courts assess whether a "not statistically significant" finding reflects a truly absent effect or merely insufficient power.
- The "redist analyze --ensemble-size 1000" flag in the expert checklist implies the ensemble can be run as a subcommand of the analysis. The paper should clarify whether this uses the same ensemble as the redistricting run or generates a new ensemble specifically for the Prong 1 alignment score analysis.

## Recommendation

Accept with minor revisions. Specify the bootstrap procedure (number of replicates, clustering level, interval method), report the VIF for the Alabama Prong 3 regression, and add a power analysis for the Prong 3 hypothesis tests.
