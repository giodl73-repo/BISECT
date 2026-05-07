# Revision Plan — G.1: GerryChain Congressional Comparison
**Round 1 → Round 2**

## Scores

| Reviewer | Score | Recommendation |
|---|---|---|
| Karypis | 3/4 | Major revision (NC inconsistency, source attribution) |
| Rodden | 3/4 | Moderate revision (NC baseline mismatch, WI percentile) |
| Duchin | 2/4 | Major revision (three critical issues) |
| Stephanopoulos | 3/4 | Moderate revision (VRA, legal precision) |
| Liang | 2/4 | Major revision (NC inconsistency, uncertainty) |
| **Mean** | **2.6/4** | |

## BLOCKER: NC Polsby-Popper Inconsistency (must resolve first)

### CRITICAL: Normal approximation vs. empirical percentile discrepancy
**Issue**: The paper states ensemble mean $\mu_{PP} = 0.318$, SD $\sigma_{PP} = 0.031$, AR score $\bar{PP}_{\rm AR} = 0.412$, and empirical percentile = 68th. The z-score is 3.03 ($\approx$ 99.9th percentile under normality). The empirical 68th percentile requires that approximately 32% of ensemble plans have PP above $\bar{PP}_{\rm AR} = 0.412$. For the Herschlag distribution with skewness $+0.3$–$+0.7$, the Cornish-Fisher correction brings the predicted percentile to approximately 99.7th, not 68th.

**Required action**: Verify the primary data. One of the following must be corrected:
1. The AR plan's PP score ($0.412$): verify against actual `redist analyze` output for NC 2020.
2. The ensemble mean ($0.318$): verify against Herschlag 2020 published figures.
3. The ensemble SD ($0.031$): verify against Herschlag 2020 published figures.
4. The stated 68th percentile: verify with direct empirical count.

**If $\bar{PP}_{\rm AR}^{\rm NC} = 0.412$ is correct**, and the empirical percentile is genuinely 68th, then the ensemble parameters (mean, SD) must be wrong and should be corrected in the paper.

**If the 68th percentile is wrong** and the correct value is approximately 95th–99th, this changes the paper's headline narrative (AR is "more compact than average" becomes "AR is at the extreme of compactness") with significant legal implications.

The paper cannot proceed to Round 2 without resolving this with primary data.

## Additional Duchin-Blocking Issues

### D1. DeFord 2021 source attribution for WI, GA, PA
**Issue**: DeFord et al. (2021) is a methods paper that does not publish $N = 50{,}000$ ensemble results for WI, GA, PA with the specific mean/SD values cited.
**Required action**: Provide precise data provenance. Options:
(a) If these are the author's own new ReCom chains: state this explicitly in Section 3, report all chain parameters (step count, initial plan, software version, hardware).
(b) If these are from a DeFord 2021 supplementary dataset: cite the specific dataset with DOI/URL.
(c) If these are estimated from DeFord 2021 figures: state this explicitly, mark all derived statistics with \est, and describe the estimation method.

### D2. TX/CA source attribution
**Issue**: The states table cites Chikina 2017 for TX/CA bounds. Chikina 2017 studied Pennsylvania.
**Required action**: Replace Chikina 2017 with the correct source for TX/CA compactness bounds. If Autry 2021 provides TX bounds (as cited for TX compactness in Section 6.1), use that. For CA, either cite a specific source or label as "author estimate" with methodology.

### D3. Uncertainty quantification uses N instead of ESS
**Issue**: Formula $\hat{\pi} \pm 1.645\sqrt{\hat{\pi}(1-\hat{\pi})/N}$ uses nominal chain length. Correct formula uses ESS $\approx N \times (1-\rho_1)/(1+\rho_1)$. For NC ($\rho_1 \approx 0.87$): $\ess \approx 1{,}703$ out of 24,518. Actual uncertainty $\approx 3.8\times$ wider.
**Required action**: Recompute all stated uncertainty intervals using ESS from G.4 Table 1. Update Section 3.4. Remove the claim of "negligible sampling error."

## High-Priority Revisions

### H1. NC baseline mismatch (Herschlag 2020 uses 2016 vote; AR uses 2020 vote)
**Issue**: The comparison conflates different baselines.
**Fix**: Add a paragraph noting the baseline difference. Show robustness: "Repeating the NC analysis with the AR plan evaluated under the 2016 presidential baseline, we find [X]th Democratic seat percentile vs. 54th under the 2020 baseline, consistent within 5 percentage points."

### H2. WI 61st percentile explanation
**Issue**: AR produces 4D/4R which the paper says is the ensemble median, but the percentile is 61st, not 50th.
**Fix**: Add explanation: for discrete distributions, the median of $S_D$ at 4 may have more than 50% of mass at $S_D \leq 4$ (the CDF is a step function). If $P(S_D \leq 4) = 0.61$, this means 61% of plans produce 4D or fewer — perfectly consistent with 4 being the median.

### H3. VRA analysis for MM district results
**Issue**: MM district percentiles (71st for NC, 62nd for GA) have VRA implications not discussed.
**Fix**: Add Section 8.1: "Minority Representation and VRA Compliance." Note that the AR plan produces MM district counts at or near the ensemble median for all states. For NC, the 71st percentile means the AR plan has slightly more concentrated minority districts than average — a potential Section 2 packing concern that requires VRA analysis beyond this paper (cross-reference the Callais evidence layer paper).

### H4. Allen v. Milligan (2023) update
**Issue**: Legal section does not engage with Allen v. Milligan.
**Fix**: Add a sentence in Section 8 noting that Allen v. Milligan (2023) upheld Section 2 remedies for racial gerrymandering and that MM district analysis in G.1 is relevant to VRA compliance for the AR plan.

## Moderate-Priority Revisions

### M1. 70th percentile threshold labeling
**Issue**: Presented as a "heuristic for legal purposes" but has no precedential basis.
**Fix**: Add explicit disclaimer: "This threshold is the author's proposed practical standard, not an established constitutional benchmark. Courts have not adopted a specific percentile threshold for ensemble-based gerrymandering analysis."

### M2. The "exactly one AR plan" claim
**Issue**: Requires precision about what uniqueness means under METIS.
**Fix**: Carried over from G.0 revision. State that uniqueness is given CS algorithm at $T = 600$ from the specified seed and census release.

## Propagation Notes

- NC PP percentile correction propagates to G.3 (Section 3, PP percentile table) and G.0 (Section 5.2 estimate of "65th–75th percentile").
- DeFord 2021 attribution fix propagates to G.2 (identical results table).
- Uncertainty correction propagates to G.2 (corridor fractions should also use ESS-based uncertainty).
- H3 (VRA analysis) is a new section needed in G.1 only.
