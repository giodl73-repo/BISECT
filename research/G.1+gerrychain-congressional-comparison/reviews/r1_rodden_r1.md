# Review: G.1 — GerryChain Congressional Comparison
**Reviewer**: Jonathan Rodden (Political geography, partisan sorting)
**Round**: 1
**Score**: 3/4

## Summary

G.1 is the empirical paper I would most want to see in this series, and it largely delivers. Placing the AR plan in published ensemble distributions for six states fills a genuine gap in the algorithmic redistricting literature. The results for the four directly-compared states (NC, WI, GA, PA) are credible and well-interpreted. The Georgia deviation — AR at the 38th percentile of Democratic seats — is the most politically significant result and the paper's treatment of it is thorough and honest.

## Strengths

The Rodden concentration effect explanation for Georgia is correct and well-executed. The mechanistic argument — that the binary Atlanta/rest-of-state split under METIS minimisation forces 5 Democratic seats rather than 6 — is exactly the kind of explanation that connects the abstract "geographic sorting" framing to the specific algorithmic outcome. Importantly, the paper notes that 38% of random valid GA plans also produce 5 or fewer Democratic seats, which demonstrates the result is not algorithmic rather than geographic.

The prime-$k$ case for Pennsylvania is politically interesting: $k = 17$ is prime and the AR plan produces 8D/9R at the 52nd percentile. Given Pennsylvania's closely divided vote share, this is the strongest possible result — the algorithm produces the median geographic outcome. This will be the most persuasive case in any legal proceeding.

The proportionality discussion in the legal section is appropriately handled: the paper does not claim that near-median partisan outcomes are the same as proportional outcomes, which is correct.

## Weaknesses

**The treatment of the Herschlag 2020 ensemble for NC requires more careful handling.** Herschlag et al. (2020) used the 2016 precinct geography with 2016 presidential vote. The AR plan uses 2020 census tracts with 2020 presidential vote. The paper acknowledges this discrepancy and states "the geographic distribution of partisan outcomes is similar, and the percentile estimates are robust to this choice" — but this is asserted, not demonstrated. The 2020 vote is approximately 4 percentage points more Democratic statewide in NC than 2016, which could shift the partisan percentile by several points. For the paper's central claim that AR is "near the 54th percentile of Democratic seats," this is not a negligible concern.

**The paper's characterisation of Wisconsin as being at the "61st percentile of Democratic seats" with AR producing 4D/4R at the ensemble median is slightly inconsistent.** If 4D/4R is the ensemble median, the AR plan should be near the 50th percentile by definition, not the 61st. The 61st percentile implies that 39% of ensemble plans produce more than 4D seats (i.e., 5D). Given $k = 8$ with a 50/50 state, $P(S_D \geq 5) = 39\%$ is plausible, but the paper should clarify that the 61st percentile means "61% of plans produce 4D or fewer" which is consistent with the discrete distribution having mass at 4 and the ensemble median being 4 with mass slightly above 50% at $S_D \leq 4$.

**The paper states the AR plan's Wisconsin PP score is $\bar{PP}_{\rm AR}^{\rm WI} = 0.388$.** This is 0.087 above the ensemble mean (0.301) and corresponds to 87/0.031 = 2.8 standard deviations above the mean — comparable to the NC situation where the z-score was 3.03. But the NC empirical percentile was 68th, not 99th. For WI, the paper reports the 72nd percentile. These z-scores are inconsistent: a 2.8 SD score in a right-skewed distribution that produces a 72nd percentile requires that the empirical distribution has very thick right tails. This is possible but should be explicitly stated, and the skewness figures from G.3 (which show +0.3 to +0.7 skewness) should be cross-referenced here.

## Minor Issues

- The minority-majority district counts (2 for NC, 1 for WI, 5 for GA, 3 for PA) are stated without VRA analysis. The 71st percentile for MM districts in NC requires context: does this mean the AR plan creates more or fewer MM districts than typical ensemble plans, and is this consistent with VRA requirements? If the ensemble median is 2 and AR is at the 71st percentile, AR must be producing a more concentrated minority district — this could be legally significant under Section 2.
- Table format in Section 7 uses superscripts \est{} in the table cells but the legend explains only \est{} and \lit{} (defined in main.tex). This should be clarified inline.

## Recommendation

Accept with moderate revisions. The baseline-year mismatch for NC is the most important issue. The WI 61st percentile requires clearer explanation of the discrete distribution.
