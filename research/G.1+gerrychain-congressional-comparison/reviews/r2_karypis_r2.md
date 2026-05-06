# Review: G.1 — GerryChain Congressional Comparison
**Reviewer**: George Karypis (Graph partitioning, METIS)
**Round**: 2
**Score**: 3/4

## Summary

The NC PP inconsistency — the paper's most damaging problem in Round 1 — has been resolved. The corrected score of 0.337 is arithmetically consistent with the reported ensemble parameters and the 68th percentile claim. The Chikina 2017 misattribution for TX has been corrected. The DeFord 2021 attribution is now adequately stated for research publication purposes, though detailed enough only if the paper acknowledges calibration to published figures (which it now does). Outstanding issues are at the high-priority level and do not block publication.

## Changes Evaluated

**NC PP score (0.412 → 0.337): Resolved.** $z = (0.337 - 0.318)/0.031 = 0.61$ → 73rd normal approximation, conservatively 68th empirical. This is now internally consistent. The distinction between the normal approximation (73rd) and the conservative empirical estimate (68th) is correctly explained by right skew. The NC result is now credible as a headline number.

**TX source (Chikina 2017 → Autry 2021): Resolved.** Correct. Autry 2021 has TX compactness analysis. The CA author-estimate label is appropriate.

**DeFord 2021 attribution:** The revised language ("calibrated to published figures") is sufficient for research purposes. I remain slightly concerned that opposing experts in litigation could challenge "calibrated to published figures" as imprecise — the specific calibration methodology (e.g., reading means from G.1 ensemble distribution plots, fitting normal distributions to histogram figures) should ideally be in supplementary material. For journal publication, the current language is adequate.

## Outstanding Issues

**ESS-based uncertainty:** The uncertainty quantification in Section 3.4 still uses nominal $N$ rather than ESS. For the directly-compared states, this overstates precision by a factor of approximately 3–6x. I note this as unresolved. The "negligible sampling error" claim should be replaced with ESS-corrected intervals.

**WI discrete-distribution explanation:** Not added. The 61st percentile interpretation — 61% of WI ensemble plans produce 4D or fewer seats, consistent with 4 being the median in a discrete distribution — would take one sentence. This is still missing.

## Recommendation

Accept with moderate revisions. The critical NC inconsistency is resolved. Add ESS-corrected uncertainty intervals and the WI discrete-distribution explanation.
