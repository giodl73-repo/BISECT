# Review: G.4 — Ensemble Diagnostics Paper
**Reviewer**: George Karypis (Graph partitioning, METIS)
**Round**: 1
**Score**: 3/4

## Summary

G.4 formalises three convergence diagnostics for redistricting ensembles: Rhat, ESS, and Hamming autocorrelation. The paper is technically solid and represents the G-series' most methodologically self-contained contribution — it does not depend on the contested ensemble percentile claims of G.1–G.3. The application to six states with state-specific threshold calibration is well-executed. From a computational perspective, the comparison between ReCom convergence requirements and ConvergenceSweep runtime is the paper's most practically valuable contribution.

## Strengths

The Rhat formula (Section 2) correctly implements the Vehtari 2021 update and applies rank normalization for the integer-valued Democratic seat count. The consistency check at $n \in \{500, 1{,}000, 2{,}000, 5{,}000, 10{,}000\}$ steps is a useful design pattern for convergence monitoring.

The ESS calculation (Section 3) is correctly derived with the geometric autocorrelation approximation. The acknowledgment that $\rho_1 \approx 0.85$–$0.95$ for ReCom is consistent with the algorithm's local-move structure (each step changes $\approx 1/k$ of tracts). The resulting ESS range of 333–1,000 for 10,000-step chains is realistic.

The minimum ESS calculation (Section 3.4) — $\ess_{\min} \approx 526$ for 95th-percentile tail estimates at 1-percentage-point precision — is correctly derived and matches the statutory minimum of 10,000 steps for states with $k \leq 20$.

The state-specific convergence results table (Table 2) shows internally consistent scaling: larger states (TX, CA) require more steps. The characterisation of ESS as the binding constraint for large states (vs. Rhat for small states) is correct.

## Weaknesses

**The Rhat threshold inconsistency with G.0 is not resolved.** G.0's body text states $\hat{R} < 1.05$ while G.0's table states $\hat{R} < 1.1$. G.4 uses $\hat{R} < 1.1$ throughout. The framework paper must be consistent with the empirical paper. If the threshold is 1.1 (which G.4 supports), G.0 should be corrected.

**The Hamming autocorrelation definition (Section 4.2) requires specification of the reference plan $P_{\rm ref}$.** The paper defines the Hamming autocorrelation as $\hat{\rho}_1 = \mathrm{Corr}(d_H(\pi_0, \pi_t), d_H(\pi_0, \pi_{t+1}))$ using $\pi_0$ (the initial plan) as the reference. But the paper also acknowledges an alternative formulation using a scalar statistic. For the state-specific table (Table 4), which formulation was used? The lag-1 Hamming values (0.87–0.94) are consistent with what one would expect from a ReCom chain — each step changes $\approx 1/k$ of tracts, giving $d_H \approx 1/k$ per step and high lag-1 correlation — but the reference plan choice should be stated explicitly.

**The statutory minimum formula $n_{\rm min}(k) = \max(10{,}000, 500k)$ would give $n_{\rm min}(38) = 19{,}000$ for Texas.** But Table 3 recommends 25,000 steps for Texas. The discrepancy is 25,000 vs. 19,000 — a 32% difference. The paper should either use the formula consistently with the table or explain why Texas requires more than the formula predicts.

## Minor Issues

- The "Rank-Normalized Rhat" subsection (Section 2.4) correctly follows Vehtari 2021 but the paper should state which results in the Application section use rank-normalized vs. original Rhat. "Both the original and rank-normalized Rhat" are mentioned but the reported values are not distinguished.
- The minimum ESS for 5-percentile-point precision is computed as "approximately 526" — this should be the minimum for the 95th percentile estimate. The paper should also compute the ESS minimum for the 99th percentile test (which is the relevant threshold for gerrymandering litigation), which requires approximately $\ess_{\min} \approx 1/(4 \times 0.01 \times 0.99 \times 0.01^2) \approx 2{,}525$.

## Recommendation

Accept with minor revisions. Resolve the Rhat 1.05/1.1 threshold with G.0, specify the Hamming reference plan, and reconcile the TX statutory minimum (25,000 vs. 19,000).
