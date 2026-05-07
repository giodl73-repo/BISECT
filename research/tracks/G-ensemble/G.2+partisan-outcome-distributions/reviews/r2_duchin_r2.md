# Review: G.2 — Partisan Outcome Distributions
**Reviewer**: Moon Duchin (Ensemble methods, metric geometry of redistricting)
**Round**: 2
**Score**: 3/4

## Summary

The two blocking issues from Round 1 have been addressed. The overdispersion direction has been corrected to underdispersion with an accurate mechanistic explanation, and the Rodden-effect framing has been added to distinguish geographic-median from partisan-neutral. These are the right fixes. The paper is now a credible companion to G.1 for the proportionality corridor analysis. Remaining issues are at the high-priority level.

## Blocking Issues — Resolution

**B1 (Overdispersion direction): Resolved.** The correction is both textually accurate and mechanistically well-explained. The revised Section 2.1 now correctly states that contiguous districts in a sorted state "cannot be arbitrarily rearranged, so the effective plan space is narrower than what an unconstrained binomial would predict." This is the right explanation: the Rodden sorting mechanism reduces partisan variance (underdispersion) precisely because geographic compactness and population balance together constrain district configurations more tightly in sorted states. The numerical example — binomial SD of 1.80 vs. observed SD of 1.2 for Georgia — is now correctly described as underdispersion. Well done.

**B2 (Ensemble-median ≠ partisan-neutral): Resolved.** The added paragraph after the results table correctly states: "A plan at the ensemble median is not neutral by any proportionality standard. In geographically sorted states, the ensemble median itself reflects the Rodden effect." The abstract has been revised accordingly. The distinction between "geographic consistency" and "proportional fairness" is now explicit and prominently placed.

## High-Priority Issues — Status

**H1 (Minnesota ensemble source): Not fully resolved.** The Minnesota result still appears in Section 4.3 without a published ensemble source. The \est{} marker has been added, which is the minimum fix, but the estimation method is described only as "binomial model with $p_{\rm geo} \approx 0.5$" — this is stated but not derived. The binomial model gives corridor fraction $\phi \approx$ Prob$(S_D \in \{k/2 \pm 1\})$ under Binomial(8, 0.5), which is $\phi = P(3 \leq S_D \leq 5) \approx 0.71$, not 0.59 as reported. The 59% figure needs a source or a corrected derivation. If the estimate comes from a model other than the pure binomial, that model must be described.

**H2 (Corridor fraction uncertainty): Not addressed.** The corridor fractions (42%, 61%, 28%, 55%) are still reported as point estimates without uncertainty intervals. For an ensemble with ESS $\approx 1{,}000$, the 90% CI for $\hat\phi = 0.42$ is $0.42 \pm 0.026$. These intervals are not wide enough to change any of the paper's conclusions, but their absence is a statistical omission that reviewers will flag.

**H3 (Standard deviation table): Not added.** The range $\sigma_{S_D} \approx 1.0$–$1.5$ for $k = 8$–$17$ is still stated without a table of state-specific values with sources.

## Recommendation

Accept with moderate revisions. The two blocking issues are resolved and the paper's core contribution is now technically correct. Address the Minnesota corridor fraction discrepancy (59% vs. the binomial prediction of 71%), and add uncertainty intervals to corridor fractions. The state-specific SD table would strengthen the paper considerably.
