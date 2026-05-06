# Review: G.4 — Ensemble Diagnostics Paper
**Reviewer**: Moon Duchin (Ensemble methods, metric geometry of redistricting)
**Round**: 1
**Score**: 3/4

## Summary

G.4 is the G-series paper I find most technically credible. The three-diagnostic framework (Rhat, ESS, Hamming) is standard in the MCMC literature and appropriately adapted to redistricting. The calibration work — deriving state-specific ESS requirements and minimum step counts — is exactly what the ensemble redistricting literature needs. My concerns are about specific threshold choices, the relationship to G.1's data provenance issues, and a subtle problem with the Rhat threshold.

## Strengths

The paper correctly adopts the Vehtari 2021 rank-normalized Rhat for integer-valued statistics (Democratic seat counts), which is superior to the original Gelman-Rubin statistic for discrete outcomes. This methodological choice is appropriate and is consistently applied.

The ESS minimum derivation (Section 3.4) is correctly executed: $\ess_{\min} \approx 526$ for 95th-percentile estimates at 1-percentage-point precision is the right calibration. The extension to 5-percentage-point precision ($\ess_{\min} \approx 2{,}100$) is also correctly derived. The statutory minimum formula $n_{\rm min}(k) = \max(10{,}000, 500k)$ is a clean and defensible recommendation.

The ESS-as-binding-constraint finding for large states (TX, CA) is a genuine empirical contribution. Previous ensemble studies have focused on Rhat as the primary diagnostic; the observation that ESS is the bottleneck for high-autocorrelation large-delegation chains is correct and practically important.

The identification that starting chains from random initial plans (rather than from compact plans) prevents apparent convergence from masking poor mixing is methodologically sound.

## Issues

**Issue 1: The Rhat threshold of 1.1 is not calibrated to redistricting-specific requirements.**
The paper adopts $\hat{R} < 1.1$ from Gelman 1992 via Vehtari 2021. This threshold was calibrated for continuous distributions in Bayesian inference. For redistricting applications with discrete outcomes (Democratic seat counts on a 0–52 integer scale), the threshold may be too permissive. Specifically, for a state like Wisconsin with $k = 8$ where the outcome is an integer between 0 and 8, $\hat{R} < 1.1$ could be satisfied even if the chains disagree on whether the most likely outcome is 3D or 4D. The paper should either (a) justify the 1.1 threshold specifically for redistricting integer outcomes, or (b) provide a redistricting-specific calibration (e.g., requiring $\hat{R} < 1.05$ for integer outcomes and $\hat{R} < 1.1$ for continuous outcomes like PP).

This issue is compounded by the G.0 inconsistency (G.0 body text uses 1.05, G.0 table uses 1.1, G.4 uses 1.1 throughout). Before this paper is published, G.0 and G.4 must agree.

**Issue 2: The Hamming autocorrelation reference plan is underspecified.**
The Hamming distance definition uses $d_H(\pi, \pi') = \frac{1}{n}\#\{v: \pi(v) \neq \pi'(v)\}$ with Hungarian matching to make it label-invariant. But the autocorrelation $\hat{\rho}_1$ is defined as $\mathrm{Corr}(d_H(\pi_0, \pi_t), d_H(\pi_0, \pi_{t+1}))$ — which depends on the choice of $\pi_0$. The paper says "the initial plan" but different random initial plans will give different $\hat{\rho}_1$ values. For a fully mixed chain starting from a random plan, $\hat{\rho}_1$ converges to a well-defined value as the chain length grows (the correlation of consecutive distances from a fixed reference in a stationary chain). But for a chain that has not yet reached stationarity, $\hat{\rho}_1$ will be artificially high (consecutive plans are far from the non-stationary initial plan). The paper should address this.

**Issue 3: The paper certifies convergence for its own chains but does not certify the G.1 source ensembles.**
G.4 runs 10 new chains per state and reports convergence diagnostics. But the ensembles used in G.1 (Herschlag 2020, DeFord 2021) were not run by this author and their Rhat and ESS are not reported. The paper's purpose — establishing an evidentiary standard for redistricting ensembles — requires applying the standard to the specific ensembles used in G.1. Without this application, G.4 and G.1 are methodologically disconnected.

## Minor Issues

- The ESS for Democratic seat counts is estimated as $\approx 1{,}429$–$1{,}765$ for NC and WI (higher than PP due to lower autocorrelation). But the paper notes $\rho_1(D\text{-seats}) \approx 0.70$–$0.80$ vs. $\rho_1(PP) \approx 0.85$–$0.95$. A chain with $\rho_1 = 0.70$ and $N = 10{,}000$ gives $\ess \approx 10{,}000 \times 0.30/1.70 \approx 1{,}765$ — consistent. But $\rho_1 = 0.80$ gives $\ess \approx 1{,}111$, and the table shows NC $D$-seats ESS = 1,538. The discrepancy ($\rho_1 = 0.80 \to \ess = 1{,}111$ vs. stated $1{,}538$) needs checking.
- The statutory Hamming threshold $\hat{\rho}_1 < 0.95$ would reject a chain only if its lag-1 autocorrelation is above 0.95. Given that all states are reported at 0.87–0.94, this threshold is barely binding. The more useful diagnostic would be a Hamming ESS requirement.

## Recommendation

Accept with moderate revisions. Resolve the Rhat threshold across G.0/G.4, specify the Hamming reference plan protocol, and apply G.4 diagnostics to the G.1 source ensembles.
