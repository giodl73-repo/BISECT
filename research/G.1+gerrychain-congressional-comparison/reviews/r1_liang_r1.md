# Review: G.1 — GerryChain Congressional Comparison
**Reviewer**: Percy Liang (ML, statistical methodology)
**Round**: 1
**Score**: 2/4

## Summary

G.1 presents specific quantitative claims about the AR plan's position in published ensemble distributions. From a statistical methodology perspective, the paper has serious problems with data provenance, uncertainty quantification, and internal consistency. Several headline numbers are potentially wrong or unverifiable. I cannot recommend acceptance in the current form.

## Critical Issues

**Issue 1: Internal consistency of NC Polsby-Popper claim.**
The paper states: ensemble mean $\mu_{PP} = 0.318$, ensemble SD $\sigma_{PP} = 0.031$, AR plan score $\bar{PP}_{\rm AR} = 0.412$, and empirical percentile = 68th. The z-score is $(0.412 - 0.318)/0.031 = 3.03$, corresponding to the 99.9th percentile under normality. The paper attributes the discrepancy to right skew. For the empirical percentile to be 68th with a z-score of 3.03, the distribution would need to have very heavy right tails — specifically, approximately 32% of the distribution would need to lie above $\mu + 3.03\sigma$. This would require a skewness substantially greater than +2 and excess kurtosis greater than +6. The G.3 paper reports skewness values of +0.3 to +0.7 for the same states. These are irreconcilable.

The most parsimonious explanation is a data error: either the AR PP score is not 0.412, or the ensemble parameters (0.318, 0.031) are wrong, or the 68th percentile is wrong. The paper must resolve this inconsistency with actual data, not a narrative explanation about skewness.

**Issue 2: Uncertainty quantification treats N as independent samples.**
The formula $\hat{\pi} \pm 1.645\sqrt{\hat{\pi}(1-\hat{\pi})/N}$ is valid for an i.i.d. sample of size $N$. The ensemble is a dependent sample from an MCMC chain. Let $\rho_1 \approx 0.87$ be the lag-1 autocorrelation (from G.4). Then $\ess \approx N(1-\rho_1)/(1+\rho_1) \approx 24{,}518 \times (0.13/1.87) \approx 1{,}703$ for the Herschlag chain. The actual uncertainty in the reported percentiles is $\sqrt{24{,}518/1{,}703} \approx 3.8\times$ wider than stated. For the 54th Democratic seat percentile in NC, the correct 90% interval is approximately $54 \pm 5.5$ rather than $54 \pm 1.5$. The paper's claim of "negligible sampling error" for direct comparisons is incorrect.

**Issue 3: Proportionality corridor analysis introduces MN without methodology.**
Section 4 of G.2 (which is closely related to G.1's results) introduces Minnesota as "an additional competitive state not studied in G.1." G.1 should either include MN or G.2 should not introduce new state data. More importantly, the corridor analysis in G.2 shows a table with MN included, with $\phi = 59\%$ for MN, but no ensemble source is cited for MN. If G.1 is the empirical paper, MN data should appear here with its source.

**Issue 4: The Georgia result at 38th Democratic seat percentile.**
This is the most politically sensitive number in the paper. The paper states that "38% of random valid plans also produce 5 or fewer Democratic seats." If this is from the DeFord 2021 ensemble, then DeFord 2021 must include a published Georgia-specific ensemble result with enough resolution to determine that 38% of plans produce $S_D \leq 5$. In a chain with many plans, this is a reliable estimate. But if the 38% figure is estimated from distributional summaries (mean 6, SD 1.2, Binomial approximation with overdispersion), the result could be inaccurate by several percentage points. The paper must state clearly which it is.

## Secondary Issues

- The uncertainty formula for TX and CA cites Chikina 2017 for "literature bounds" but Chikina 2017 does not study TX or CA. The source for the TX/CA bounds must be corrected.
- The paper uses both $\bar{PP}$ and $\bar{pp}$ for mean Polsby-Popper in different places. Notation should be consistent.
- The phrase "negligible sampling error given $N \geq 10{,}000$" confuses nominal chain length with ESS. This sentence should be deleted or corrected.

## Recommendation

Major revision required. The NC PP percentile inconsistency is the single most urgent issue — it calls into question the paper's most prominent empirical claim. The uncertainty quantification must be corrected throughout.
