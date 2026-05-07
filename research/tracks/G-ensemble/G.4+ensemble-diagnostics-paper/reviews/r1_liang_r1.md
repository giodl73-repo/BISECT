# Review: G.4 — Ensemble Diagnostics Paper
**Reviewer**: Percy Liang (ML, statistical methodology)
**Round**: 1
**Score**: 3/4

## Summary

G.4 is the technically strongest paper in the G-series. The three-diagnostic framework is well-calibrated, the formulas are correctly stated, and the empirical application is internally consistent. My concerns are about specific threshold choices, the Hamming reference plan underspecification, and a minor ESS inconsistency.

## Strengths

The ESS formula and its application are correctly executed. The geometric autocorrelation approximation is appropriate for the described range ($\rho_1 = 0.85$–$0.95$) and the resulting ESS range (333–1,000 for 10,000 steps) is realistic.

The minimum ESS derivation — $\ess_{\min} \approx 1/(4\alpha(1-\alpha)\delta^2)$ for $\alpha$-percentile estimates with precision $\delta$ — is correctly stated and matches standard results from quantile estimation theory.

The Rhat formula correctly implements the Vehtari 2021 update with rank normalization for discrete statistics. The notation is clear and the formula matches the cited source.

The empirical results table (Table 2) is well-organised and the scaling pattern (Rhat-bottleneck for small states, ESS-bottleneck for large states) is clearly shown.

## Weaknesses

**Minor ESS inconsistency in Table 1.** The NC row shows $\rho_1(PP) = 0.87$ and $\ess(PP) = 769$. Using the geometric formula: $\ess = 10{,}000 \times (1-0.87)/(1+0.87) = 10{,}000 \times 0.13/1.87 = 695$. The table shows 769, which corresponds to $\rho_1 = 0.85$, not 0.87. Similarly, WI shows $\rho_1 = 0.85$ and $\ess = 1{,}000$; the formula gives $10{,}000 \times 0.15/1.85 = 811$, not 1,000, which corresponds to $\rho_1 = 0.80$. The table values appear computed with different $\rho_1$ values than stated. The discrepancy is small but the inconsistency should be resolved.

**The Hamming reference plan is underspecified.** Section 4.2 states the Hamming autocorrelation uses $\pi_0$ (the initial plan) as the reference. But as the chain evolves, $d_H(\pi_0, \pi_t)$ increases monotonically from 0 to approximately $(k-1)/k$, never decreasing. A running estimate of the lag-1 correlation of this increasing sequence will be close to 1 simply because both consecutive values are increasing — even if the chain is mixing well. The correct formulation should either: (a) use a reference plan drawn from the stationary distribution (impractical without knowing convergence), (b) use the autocorrelation of $f(\pi_t)$ for a scalar statistic $f$ (which the paper notes as an alternative but does not use for the table values), or (c) compute $d_H(\pi_t, \pi_{t+\ell})$ for consecutive plan pairs (not relative to a fixed reference). The paper should clarify which formulation produces the values in Table 4.

**The statutory minimum formula $n_{\rm min}(k) = \max(10{,}000, 500k)$ predicts $n_{\rm min}(38) = 19{,}000$ but the table recommends 25,000 for TX.** The discrepancy is 6,000 steps (32%). The paper should either update the formula to match the table (e.g., $n_{\rm min}(k) = \max(10{,}000, 650k)$) or explain why Texas requires more than the formula predicts.

**The ESS minimum for 95th-percentile estimates ($\ess_{\min} \approx 526$) is the threshold for the 95th percentile with $\delta = 0.01$.** But the G.4 statutory condition requires $\ess > 500$, not $> 526$. The threshold should be stated as the ceiling of the derivation, i.e., 530, not rounded down to 500. This small inconsistency could be cited in adverse expert review.

## Minor Issues

- The formula $\hat{V} = \frac{n-1}{n}W + \frac{1}{n}B$ in the Rhat derivation in the abstract of Section 2 differs from the formula in the body ($\frac{m+1}{mn}B$). Both appear in Section 2 but they are not the same. The body formula (Vehtari 2021) is correct; the simplified version in the abstract of the section should match.
- The steps-per-second estimates in Section 6 Table (4.2 steps/sec for NC/GA, 7.1 for WI) should specify the hardware used and whether these are single-core or parallelized.

## Recommendation

Accept with minor revisions. Fix the ESS table inconsistency, clarify the Hamming reference plan, resolve the statutory minimum formula vs. table discrepancy.
