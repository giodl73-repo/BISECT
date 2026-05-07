# Review: G.1 — GerryChain Congressional Comparison
**Reviewer**: Moon Duchin (Ensemble methods, metric geometry of redistricting)
**Round**: 2
**Score**: 3/4

## Summary

The revision resolves the most critical issue: the NC PP score has been corrected from 0.412 to 0.337, which is consistent with the ensemble parameters ($\mu = 0.318$, $\sigma = 0.031$) and the claimed 68th empirical percentile (z = 0.61 → 73rd normal approximation, 68th empirical conservative estimate). The internal arithmetic no longer contradicts itself. The DeFord 2021 attribution problem has been partially addressed. The Chikina 2017 misattribution for TX has been corrected to Autry 2021. There are remaining issues at high and moderate priority.

## Blocking Issues — Resolution

**BLOCKER (NC PP inconsistency): Resolved.** The corrected PP score of 0.337 satisfies internal consistency: $z = (0.337 - 0.318)/0.031 = 0.61$, which gives approximately the 73rd normal-approximation percentile, and the paper conservatively reports 68th under direct empirical count, acknowledging right skewness. This is appropriate. The paper can now be cited with this headline number without arithmetic embarrassment.

I note the formula in Section 4.2 now uses $q_\pp$ notation (G.0 fix cascaded correctly). Good.

**D1 (DeFord 2021 attribution): Partially resolved.** The revised text in Section 2 now states that results for WI, GA, PA use "published ensemble statistics from Herschlag et al. 2020 (NC) and DeFord et al. 2021 methodology (other states); state-specific ensemble parameters calibrated to published figures." This is honest in acknowledging calibration to published figures, but it does not fully specify the estimation method. The phrase "calibrated to published figures" leaves open whether the parameters were read from figures (imprecise), derived from a model (needs statement), or from a supplementary dataset (needs DOI). The \est{} markers remain throughout, which is appropriate. For litigation use, further specificity would be required — but for research publication this is adequate.

**D2 (Chikina 2017 misattribution for TX): Resolved.** The states table now correctly attributes TX compactness bounds to Autry 2021 and labels CA as "author estimate." The footnote is consistent.

**D3 (ESS-based uncertainty): Not resolved.** Section 3.4 still uses the formula $\hat\pi \pm 1.645\sqrt{\hat\pi(1-\hat\pi)/N}$ with $N$ as nominal chain length, and still claims "negligible sampling error" for the direct-comparison states. The REVISION-PLAN explicitly required recomputing all uncertainty intervals using ESS. ESS for the Herschlag NC chain is approximately 1,703 out of 24,518, making the actual uncertainty ~3.6× wider. For a claim as specific as "54th percentile," this means the true 90% CI is approximately 54 ± 4, not 54 ± 1. The "negligible sampling error" claim is materially incorrect and must be removed. This is a high-priority item that has not been addressed.

## High-Priority Issues — Status

**H1 (NC baseline mismatch):** Not addressed. The paragraph noting that the Herschlag ensemble uses 2016 vote while the AR plan is evaluated under 2020 vote has not been added. This is a methodological concern that must be addressed — without it, the comparison is not apples-to-apples.

**H2 (WI 61st percentile explanation):** Not addressed. The discrete distribution explanation (61% of plans produce 4D or fewer, consistent with 4 being the median) has not been added. This is an easy clarification.

**H3 (VRA analysis for MM districts):** Not addressed. The requested Section 8.1 on minority representation and VRA compliance has not been added.

**H4 (Allen v. Milligan 2023):** Not addressed.

## Recommendation

Accept with major revisions. The blocking NC inconsistency is resolved, which is the paper's most critical fix. However, three high-priority items remain unaddressed: ESS-based uncertainty (D3), NC baseline mismatch (H1), and the VRA section (H3). The paper cannot be published in current form for litigation use given the "negligible sampling error" claim — this is factually wrong and exposes the paper to cross-examination. The other high-priority items are less blocking but should be addressed before the G-series proceeds to submission.

Score of 3/4 is conditional on the ESS uncertainty correction being made in response to this review.
