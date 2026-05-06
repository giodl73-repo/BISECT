# Review: G.1 — GerryChain Congressional Comparison
**Reviewer**: Moon Duchin (Ensemble methods, metric geometry of redistricting)
**Round**: 1
**Score**: 2/4

## Summary

G.1 is the most important paper in the G-series and has the most serious problems. The paper makes specific quantitative claims — NC at the 68th PP percentile, WI at the 72nd, GA at the 38th Democratic seat percentile — and presents these as the AR plan's position in published ensemble distributions. I have examined the cited sources carefully and find that: (1) the DeFord 2021 citations for WI, GA, and PA do not support the specific distributional summaries quoted; (2) the NC normal-approximation inconsistency suggests data error; and (3) the uncertainty quantification treats correlated ensemble samples as independent. These are collectively fatal for a paper that will be cited in litigation.

## Critical Issues

**Issue 1: Source attribution for WI, GA, PA ensembles is insufficient.**
DeFord et al. (2021) "Recombination: A Family of Markov Chains for Redistricting" is a methods paper. It demonstrates ReCom on several states but does not publish complete ensemble distributions at $N = 50{,}000$ with the specific PP means and standard deviations quoted in G.1. The paper cites "DeFord 2021 ($N = 50{,}000$) \est" for Wisconsin, Georgia, and Pennsylvania — but this is not what DeFord 2021 contains.

Three possibilities: (a) the author ran new ReCom chains and is calling them "DeFord 2021 ensembles" — this is a serious misattribution; (b) the author extrapolated the means/SDs from the DeFord paper's figures — this is impermissible without stating it explicitly; (c) there is a supplementary dataset from DeFord 2021 that the paper does not cite. Regardless, the paper must state precisely which data it used and where it came from. In litigation, opposing experts will pull DeFord 2021 and find that the specific numbers are not there.

**Issue 2: The NC normal approximation discrepancy is unexplained and numerically implausible.**
The paper computes $\Phi((0.412 - 0.318)/0.031) = \Phi(3.03) \approx 0.999$, then states the empirical count gives 68th percentile. The gap is 31 percentile points. For a right-skewed distribution with skewness $\gamma \approx +0.5$ (as reported in G.3), the Cornish-Fisher correction to the normal CDF at $z = 3.03$ gives approximately:

$\Phi(3.03) - \phi(3.03) \cdot \frac{\gamma}{6}(3.03^2 - 1) \approx 0.999 - 0.0034 \cdot \frac{0.5}{6}(8.18) \approx 0.999 - 0.0023 \approx 0.997$

Even with substantial skewness correction, the corrected percentile is 99.7th, not 68th. The empirical 68th percentile is only consistent with the mean and SD if the AR PP score is substantially lower than stated — approximately $0.318 + 0.031 \times \Phi^{-1}(0.68) \approx 0.318 + 0.031 \times 0.47 = 0.333$, not 0.412.

One of the following must be true: (a) $\bar{PP}_{\rm AR}^{\rm NC}$ is not 0.412 but approximately 0.333–0.340; (b) the ensemble mean is not 0.318 but approximately 0.39; (c) the ensemble SD is not 0.031 but approximately 0.11; or (d) the stated empirical percentile of 68 is wrong and the actual value is much higher. The paper cannot proceed with these numbers as stated — this is a clear data error and the G-series' most prominent headline result (NC at 68th PP percentile) is potentially wrong.

**Issue 3: The uncertainty formula applies to independent samples; ensemble draws are autocorrelated.**
The paper uses $\hat{\pi} \pm 1.645\sqrt{\hat{\pi}(1-\hat{\pi})/N}$ with $N = 24{,}518$ (nominal chain length). From G.4's analysis, the ESS for PP compactness in an NC-sized chain is approximately $\ess \approx 769$ for 10,000 steps, implying $\ess/N \approx 0.077$. For the Herschlag ensemble of $N = 24{,}518$, the ESS is approximately $24{,}518 \times 0.077 \approx 1{,}888$. The correct uncertainty width is $\sqrt{24518/1888} \approx 3.6\times$ wider than stated. For a claim as specific as "54th percentile," this matters: the true 90% interval could be approximately $54 \pm 4$ rather than $54 \pm 1$.

**Issue 4: The TX bounds cite Chikina 2017, a paper that studied Pennsylvania.**
The paper uses Autry 2021 for TX compactness bounds but cites Chikina 2017 for TX/CA in the states table. Chikina et al. (2017) is an estimation method paper that did not study Texas or California. The bounds for TX and CA must be derived from something else — the paper should state the actual source.

## Secondary Issues

- The Georgia mechanistic explanation (binary split of Atlanta from rest-of-state, 7-way partition producing 5D districts) is compelling but entirely qualitative. It should be supported by showing the actual binary split line and its edge-cut value versus alternative splits.
- The 70th percentile threshold proposed in Section 8 ("a plan at the $p$th percentile of compactness with $p \in [60, 85]$") is presented as a "heuristic for legal purposes derived from the ensemble literature" but no such standard exists in the literature. This is an original normative claim that needs to be labeled as such.
- The legal argument that verifiability "eliminates one entire class of challenge" is overstated. A court could still challenge the METIS algorithm itself (is it the right objective?), the census tract granularity (why not blocks?), or the Huntington-Hill seat count derivation (is the prime factorization the right districting scheme?).

## Recommendation

Major revision required. The paper needs to either reproduce the correct empirical percentile for NC (resolving the 68th vs. 99th percentile discrepancy) or restate the AR plan's PP score. The DeFord 2021 ensemble attribution must be clarified with exact data provenance. The TX/CA source citation must be corrected.
