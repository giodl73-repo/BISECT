# Review: G.1 — GerryChain Congressional Comparison
**Reviewer**: George Karypis (Graph partitioning, METIS)
**Round**: 1
**Score**: 3/4

## Summary

G.1 is the empirical centrepiece of the G-series: it places the AR plan within published ensemble distributions for six states on three axes (Polsby-Popper compactness, partisan seat share, minority-majority districts). The state-by-state analysis is detailed and the summary table is well-structured. From a graph-partitioning perspective, the AR plan's performance characteristics are as expected — the METIS edge-cut objective drives compactness above the ReCom baseline, and the partisan-outcome near-median result for competitive states is the natural consequence of compact, geographically neutral partitioning.

## Strengths

The NC case is the best-documented and the numbers are credible. The Herschlag ensemble ($N = 24{,}518$ plans) is well-characterised in the literature, and the AR plan's placement at the 68th percentile of PP compactness and 54th percentile of Democratic seats is plausible given the METIS objective. The explanation of the normal approximation over-predicting the percentile due to right skew — and then using the direct empirical count as authoritative — is methodologically correct.

The prime-$k$ case for Pennsylvania ($k = 17$) is appropriately flagged: METIS's 17-way direct partition produces lower compactness (61st percentile) than hierarchical factorization states, and the paper correctly attributes this to the absence of the factorization tree structure rather than any algorithmic failure.

The Georgia deviation analysis (Section 5, 38th percentile of Democratic seats) is the paper's most important qualitative result. The mechanistic explanation — that the binary split divides Atlanta from the rest of the state, and the 7-way partition of the Atlanta half produces 5 Democratic districts due to compactness constraints — is plausible and well-argued.

## Weaknesses

**For WI, GA, and PA, the paper cites DeFord 2021 with $N = 50{,}000$ plans, but the DeFord 2021 paper did not publish full redistricting ensembles at these scales for these states at census-tract resolution.** DeFord et al. (2021) is a methods paper introducing the ReCom algorithm with smaller illustrative ensembles. If the paper is using DeFord 2021's illustrative results as the source for WI/GA/PA ensemble statistics, this needs much clearer attribution. If the author ran new ReCom ensembles, this needs to be stated and the methodology reported. The current citation of "$N = 50{,}000$ \est" with a vague footnote is insufficient for the level of specificity claimed (e.g., "mean $\mu_{PP} \approx 0.318$, $\sigma_{PP} \approx 0.031$" for NC — but the NC ensemble is from Herschlag 2020, not DeFord 2021).

**The NC Polsby-Popper calculation contains a major mathematical inconsistency.** The normal approximation gives $\Phi(3.03) \approx 0.999$, which would place the AR plan at the 99.9th percentile. The paper then states this is "bounded to the 68th percentile under direct empirical count." A discrepancy of 99th vs. 68th percentile is not explained by right skew alone — it indicates either that the ensemble mean/SD values are wrong, or that the AR plan's PP score is wrong. The paper dismisses the discrepancy as "the distribution is right-skewed and the empirical count is the authoritative figure," but a 30-percentile-point gap requires substantive explanation, not a wave of the hand. This is a serious credibility problem for the NC result.

**The TX and CA results are presented with very wide uncertainty ($\pm 5$–$9$ percentile points) but the source "Chikina 2017" for California bounds is not clearly linked to any Texas- or California-specific ensemble.** Chikina et al. (2017) studied Pennsylvania. The paper should not cite Chikina 2017 as the source for CA/TX bounds without explaining the extrapolation methodology. "Literature bounds" is too vague for a claim as specific as "52nd $\pm 8$ percentile."

## Minor Issues

- The uncertainty formula $\hat{\pi} \pm 1.645 \sqrt{\hat{\pi}(1-\hat{\pi})/N}$ in Section 3.4 treats the ensemble as a simple random sample, but plans within a single ReCom chain are correlated. The correct formula should use $N_{\rm eff}$ (ESS) rather than $N$. With ESS $\approx 700$–$1{,}000$ for NC and WI, the uncertainty intervals would be $\sqrt{24000/700} \approx 5.8\times$ wider than reported.
- The AR plan's Wisconsin result (72nd PP percentile, 61st Democratic seat percentile) is the highest Democratic seat percentile among the six states, yet Wisconsin is described as a "competitive" state. The 61st percentile for Democratic seats in a 50/50 state is mildly surprising and deserves a sentence of explanation.

## Recommendation

Major revision required for the NC normal-approximation discrepancy, and the source attribution for WI/GA/PA ensembles. The paper makes strong empirical claims that require clearly identified data sources.
