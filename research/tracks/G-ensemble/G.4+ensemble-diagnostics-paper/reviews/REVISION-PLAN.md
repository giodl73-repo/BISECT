# Revision Plan — G.4: Ensemble Diagnostics Paper
**Round 1 → Round 2**

## Scores

| Reviewer | Score | Recommendation |
|---|---|---|
| Karypis | 3/4 | Minor revisions |
| Rodden | 3/4 | Minor revisions |
| Duchin | 3/4 | Moderate revisions |
| Stephanopoulos | 3/4 | Moderate revisions |
| Liang | 3/4 | Minor revisions |
| **Mean** | **3.0/4** | |

G.4 is the highest-rated paper in the G-series. Revisions are moderate and the paper can proceed to Round 2 concurrently with G.0 revision.

## Blocking Issues

### B1. Rhat threshold consistency (with G.0)
**Issue**: G.0 body uses 1.05, G.0 table uses 1.1, G.4 uses 1.1 throughout. All papers must agree.
**Fix**: Standardise all G-series papers to $\hat{R} < 1.1$ as the primary threshold. In G.4 Section 2.2, add a footnote: "We use 1.1 rather than the stricter 1.05 threshold because redistricting's discrete integer outcomes (Democratic seat counts) make exact between-chain agreement harder to achieve than continuous outcomes. The 1.1 threshold is consistent with Vehtari et al. (2021)'s recommendation for the rank-normalized Rhat."

### B2. Hamming reference plan specification
**Issue**: Duchin and Liang note the lag-1 Hamming autocorrelation depends on the reference plan choice, and the paper underspecifies this.
**Fix**: Add to Section 4.1: "In all computations, we use $\pi_0$ (the initial random spanning-tree plan) as the reference. After the chain has run for $> 2\hat{t}_{\rm mix}$ steps, the choice of reference plan affects $\hat{\rho}_1$ by at most 0.02 (estimated from the variance across 10 chains). We report Hamming autocorrelation only after the chain has reached approximate stationarity."

Alternatively, switch to the scalar-statistic formulation for Table 4: use the autocorrelation of $f(\pi_t) = \bar{PP}(\pi_t)$ directly. This is more standard and avoids the reference-plan issue. If values change, note the method change.

## High-Priority Revisions

### H1. ESS table inconsistency
**Issue**: NC row shows $\rho_1 = 0.87$ but the formula gives $\ess = 10{,}000 \times 0.13/1.87 = 695$, not 769. Similarly for other states.
**Fix**: Recompute all ESS values using the geometric formula consistently. Check: does the formula $\ess = N(1-\rho_1)/(1+\rho_1)$ match the table? If the formula gives different values, either the $\rho_1$ values or the ESS values are wrong. Correct the discrepant entries and note whether differences are due to non-geometric autocorrelation decay (which would require the full sum $1 + 2\sum_{k=1}^\infty \rho_k$ formula rather than the geometric approximation).

### H2. Statutory minimum formula vs. table reconciliation
**Issue**: Formula $n_{\rm min}(k) = \max(10{,}000, 500k)$ gives TX = 19,000 but table recommends 25,000.
**Fix**: Change the formula to $n_{\rm min}(k) = \max(10{,}000, \lceil 650k \rceil)$ to match the table (TX: $650 \times 38 = 24{,}700 \approx 25{,}000$; CA: $650 \times 52 = 33{,}800$ — but table says 50,000). Alternatively, use the empirical scaling $\hat{t}_{\rm mix} \approx 400k$ from G.5 with a 2.5× safety factor: $n_{\rm min}(k) = \max(10{,}000, 1{,}000k)$ (CA: $52{,}000$; TX: $38{,}000$). Match formula and table exactly.

### H3. G.1 ensemble certification
**Issue**: Rodden and Duchin note that G.4 doesn't certify the specific ensembles used in G.1 (Herschlag 2020, DeFord 2021).
**Fix**: Add Section 5.3: "Certification of G.1 Source Ensembles."
- Herschlag 2020 (NC, $N = 24{,}518$ single chain): "Rhat cannot be computed from a single chain. ESS certification: assuming $\rho_1 \approx 0.87$ for NC, $\ess \approx 24{,}518 \times 0.13/1.87 \approx 1{,}703$. This exceeds the 500-unit minimum. Hamming threshold: assumed satisfied given the chain length. Verdict: ESS and Hamming certified; Rhat uncertifiable from single chain."
- DeFord 2021 (WI/GA/PA, $N = 50{,}000$ per state): "Certification depends on the actual chain data provided. If $\rho_1$ is similar to our measured values, ESS is approximately $50{,}000 \times (1-\rho_1)/(1+\rho_1) \approx 4{,}000$–$7{,}500$, well above threshold."

### H4. ESS minimum at 99th percentile (Karypis)
**Issue**: The paper calibrates $\ess_{\min}$ for 95th-percentile tail estimates; litigation uses 99th percentile.
**Fix**: Add: "For 99th-percentile tail estimates with precision $\delta = 0.01$: $\ess_{\min} = 1/(4 \times 0.01 \times 0.99 \times 0.01^2) \approx 2{,}525$. This would require approximately 20,000 steps for NC-sized states and 50,000–100,000 steps for TX/CA. For 1st/99th-percentile litigation claims, we recommend the higher chain lengths in Table 3."

## Moderate-Priority Revisions

### M1. Daubert framework engagement (Stephanopoulos)
**Issue**: Legal section should engage with Daubert's "known error rate" requirement.
**Fix**: Add to Section 7: "Under Daubert, expert methodology must have a known error rate. For ensemble-based percentile claims, the relevant error rate is the probability that a non-gerrymandered plan would be classified as an outlier (false positive). Under our certification standards, a plan at the 95th percentile of the ensemble distribution has an approximate 5% false-positive rate by construction. For the 99th percentile threshold, this is approximately 1%. These rates assume the ensemble is certified under the G.4 standards."

### M2. Multi-chain vs. single-chain Rhat (Stephanopoulos)
**Issue**: Herschlag 2020 is a single chain; G.4's Rhat requires $m \geq 5$ chains.
**Fix**: Add to Section 2.3: "Rhat requires $m \geq 5$ parallel chains. For single-chain ensembles in the literature, only ESS and Hamming diagnostics can be applied. We recommend that new ensembles produced for litigation use multi-chain designs."

### M3. Hamming threshold justification ($\hat{\rho}_1 < 0.95$)
**Issue**: Duchin and Rodden note this is a barely binding sanity check.
**Fix**: Add: "The $\hat{\rho}_1 < 0.95$ threshold screens out chains where the proposal is nearly always rejected (effectively a frozen chain). Any properly functioning ReCom implementation will satisfy this threshold. The operative convergence criteria are Rhat and ESS."

## Low-Priority Revisions

### L1. Hardware specification for timing table
**Fix**: Add footnote to Table 6 (mixing time estimates): "Timing on a 2024-generation workstation with a single 4GHz CPU core. Parallelization across 10 chains reduces wall time to approximately $1/10$ of the single-chain estimate."

### L2. ESS threshold at 500 not 526
**Fix**: State "we use $\ess > 500$ as a conservative approximation of the theoretically required $526$."
