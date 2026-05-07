# Review: G.2 — Partisan Outcome Distributions
**Reviewer**: George Karypis (Graph partitioning, METIS)
**Round**: 1
**Score**: 3/4

## Summary

G.2 focuses on the partisan outcome distributions of ReCom ensembles and the AR plan's position within them. The paper's central negative finding — that AR is not systematically partisan-biased — is well-argued. The proportionality corridor analysis is a useful methodological contribution. From a graph partitioning standpoint, the paper correctly attributes the Georgia deviation to the minimum-edge-cut objective's interaction with geographic sorting, which is technically accurate.

## Strengths

The binomial-with-overdispersion model (Eq. 1) for the partisan seat distribution is an appropriate approximation. The identification of the "geographically determined probability" $p_{\rm geo}$ as the driver of the ensemble distribution's center is correct and connects to the Rodden geographic sorting literature naturally.

The Georgia mechanistic analysis (Section 3.2) is the most technically specific discussion in any of the G-series papers. The description of how the binary split at the top of the $14 = 2 \times 7$ tree divides Atlanta from the rest of the state, and how the 7-way Atlanta-half partition produces 5D/2R rather than 6D/1R, is mechanistically precise. This is exactly the level of algorithmic explanation needed to rebut a claim of partisan design.

The robustness-to-baseline analysis (Section 3.3) showing that percentile rankings are stable across 2016, 2018, and 2020 electoral baselines within 5 percentage points is methodologically important. This would be a standard challenge in litigation.

## Weaknesses

**The overdispersion claim for the binomial model is asserted but not quantified.** Equation 1 states $S_D \sim \text{approximately Binomial}(k, p_{\rm geo})$ "with overdispersion" — but the overdispersion factor is never given. Overdispersion affects the width of the distribution, which directly affects what counts as an outlier. For Georgia with $p_{\rm geo} \approx 0.36$, a pure binomial gives $\sigma_{S_D} = \sqrt{14 \times 0.36 \times 0.64} \approx 1.80$. The paper elsewhere (G.1) uses $\sigma_{S_D} \approx 1.2$ for Georgia — this is actually less than the binomial prediction, not overdispersed. The model as stated is internally inconsistent with the distributional summaries used.

**The standard deviation estimates ($\sigma_{S_D} \approx 1.0$–$1.5$ for $k = 8$–$17$) are presented without source.** These are key parameters for interpreting the significance of deviations from the median. If the GA SD is 1.2, then AR's 5D (versus median 6D) is a deviation of 0.83 standard deviations — notable but not extreme. If the SD is 1.8 (the pure binomial value), the deviation is only 0.56 SDs. The paper should provide a table of state-specific means and SDs with sources.

**Minnesota is introduced in Section 4 without a supporting ensemble source.** The paper reports that AR produces 4D/4R for Minnesota at the 44th percentile with $\phi = 59\%$ corridor fraction — but Minnesota is not one of the six states studied in G.1, and no published ReCom ensemble is cited for Minnesota at congressional district resolution. Either cite a source or label this result with the \est{} marker and describe the estimation method.

## Minor Issues

- The corridor definition (Eq. 2) uses $\lfloor k \cdot v_D \rceil$ (nearest-integer rounding) for the proportional seat count. For Georgia with $v_D = 0.464$ and $k = 14$, this gives $\lfloor 6.496 \rceil = 6$, so the corridor is $\{5, 6, 7\}$ — but the paper shows $\{6, 7\}$ in the table. The corridor should include values within ±1 of the proportional count, which for 6.5 would be $\{5.5, 6.5, 7.5\}$ rounded to integers, giving $\{6, 7\}$ only if the "within one seat" criterion is exclusive of 5. The definition should be clarified.
- The "algorithm choice determines which tail you land in" discussion in Section 5 (connecting to B.0 bakeoff) is under-developed. This is a potentially important claim — that the METIS edge-cut objective drives partisan outcomes differently than a compactness-maximizing or population-variance-minimizing objective. It warrants its own subsection.

## Recommendation

Accept with moderate revisions. Fix the overdispersion inconsistency, add a source for Minnesota, and provide a table of state-specific distributional parameters.
