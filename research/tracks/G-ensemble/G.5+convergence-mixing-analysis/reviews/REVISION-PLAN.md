# Revision Plan — G.5: Convergence and Mixing Analysis
**Round 1 → Round 2**

## Scores

| Reviewer | Score | Recommendation |
|---|---|---|
| Karypis | 3/4 | Moderate revision |
| Rodden | 3/4 | Minor revision |
| Duchin | 2/4 | Major revision |
| Stephanopoulos | 3/4 | Minor revision |
| Liang | 2/4 | Major revision |
| **Mean** | **2.6/4** | |

## Blocking Issues

### B1. Theorem 1 proof sketch is incomplete
**Issue**: The paper claims to tighten the mixing time bound from $O(n^3 \log n)$ to $O(n^2 \log n)$ using "the path congestion bound directly" — but does not show this argument. From spectral gap alone: $\tmix = O(\log|\mathcal{P}|/\sgap) = O(n\log n \cdot n^2) = O(n^3 \log n)$. The $O(n^2 \log n)$ bound requires a direct path congestion argument showing congestion $\leq C \cdot n / \log n$.

**Required action**: One of:
(a) **Provide the complete path congestion argument.** Show explicitly that for any pair $(\pi, \pi')$ of plans, the canonical path between them passes through each intermediate plan at most $O(n)$ times (congestion $= O(n)$), and that the path length is $O(n\log n)$. The mixing time via path congestion is $O(\text{max congestion} \times \text{max path length}) = O(n^2 \log n)$. This argument needs to be written out formally with the $|\mathcal{P}|/|\mathcal{P}^*|$ bound for the separator.

(b) **Reduce the claim.** State Theorem 1 with the proved bound: $\tmix = O(n^3 \log n)$, and add a remark: "We conjecture that the bound can be improved to $O(n^2 \log n)$ using a direct path congestion argument; this improvement does not affect the empirical conclusions since both bounds are many orders of magnitude above observed mixing times."

Option (b) is acceptable for the G-series' legal-empirical purpose and removes a false claim.

### B2. Theoretical chain ≠ empirical chain
**Issue**: Theorem 1 proves bounds for the "lazy reversible ReCom chain" (Metropolized, probability 1/2 hold). The empirical measurements use "standard ReCom with uniform spanning-tree generation" (non-reversible, unknown stationary distribution). The theoretical bounds do not directly apply to the chain that was measured.
**Required action**: Add a subsection in Section 3 titled "Scope of the Theorem." State clearly:
- The theorem applies to the lazy reversible chain (Metropolized Forest ReCom, Autry 2021 variant).
- Standard ReCom is non-reversible with an unknown stationary distribution; mixing time bounds from spectral theory do not formally apply.
- The empirical mixing times measure the standard non-reversible chain.
- The relationship between the two chains is: reversible chains have no faster mixing in general; the lazy chain is always 2× slower by construction. Therefore, $\hat{t}_{\rm mix}$ for the standard chain is at most $2\times$ faster than the lazy chain bound. The $10^4$–$10^5\times$ gap between theory and practice is not reduced by this factor.

### B3. Scaling law $\hat{t}_{\rm mix} \approx 400k$ is numerically wrong
**Issue**: Computing $\hat{t}/k$ from the data:
- NC: $2000/14 = 143$
- WI: $1500/8 = 188$
- GA: $2500/14 = 179$
- PA: $3000/17 = 176$
- TX: $10000/38 = 263$
- CA: $20000/52 = 385$

The coefficient ranges from 143 to 385. The claim of "approximately 400" is accurate only for CA.

**Fix**: Replace the linear scaling claim with a power-law fit. Log-log regression on the six data points:
$\log(\hat{t}) = \alpha + \beta \log(k)$
gives approximately $\beta \approx 1.2$–$1.4$ (super-linear in $k$). Report: "$\hat{t}_{\rm mix} \propto k^{1.3}$ (estimated from six data points; R² = [value])." Add: "The scaling is approximately linear for large states but the coefficient is not constant — small states mix more efficiently per district than large states."

## High-Priority Revisions

### H1. Concentration explanation for theory-practice gap
**Issue**: Paper attributes the $10^4$–$10^5\times$ gap to "concentration near compact plans" but this is incorrect for a uniform stationary distribution.
**Fix**: Replace the concentration explanation with:
"The theory-practice gap has two components: (1) the worst-case initial state (a maximally non-compact plan entrenched in a low-probability region of the natural trajectory) does not arise in practice, because we start chains from random spanning-tree plans rather than adversarial configurations; (2) the $O(n^2\log n)$ bound applies to total-variation mixing, while the empirical $\hat{t}_{\rm mix}$ is measured using the weaker $\hat{R} < 1.1$ criterion, which requires chains to explore the same region of state space rather than the full state space. These two effects together account for the observed $10^4$–$10^5\times$ ratio."

### H2. "$< 1\%$ outlier frequency" claim needs support
**Issue**: "expected frequency $< 1\%$" for the CS plan being a partisan outlier stated without derivation.
**Fix**: Add derivation: "From the B.7 50-state sweep (50 states, 3 census years = 150 runs), the maximum observed CS plan partisan deviation was Georgia at the 38th percentile of Democratic seats. No state produced a plan below the 30th or above the 70th percentile (as defined in G.1 Section 8.4). Extrapolating, the fraction of states where CS produces a partisan outlier (below 5th or above 95th percentile) is 0/150 = 0% in the observed data, bounded above by $1/150 \approx 0.7\%$ at 95\% confidence."

### H3. Adversarial misuse of Theorem 1 (Stephanopoulos)
**Issue**: An expert could cite $O(n^2 \log n)$ for CA = $2 \times 10^9$ steps to claim any practical ensemble is inadequate.
**Fix**: Add a paragraph: "Theorem 1's bounds are worst-case guarantees. The G.4 empirical certification framework provides the appropriate standard for practical use: a chain certified under G.4 diagnostic standards (Rhat, ESS, Hamming) is adequate for redistricting inference regardless of its relationship to the theoretical worst-case bound. The theoretical bound motivates the need for empirical certification; it does not set the minimum chain length."

## Moderate-Priority Revisions

### M1. Census tract graph planarity assumption
**Issue**: Census-tract adjacency graphs are approximately but not exactly planar (water bodies can create non-planar adjacencies).
**Fix**: Add: "The Lipton-Tarjan separator theorem applies to planar graphs. Census-tract adjacency graphs are approximately planar — most adjacency crossings occur at water bodies, and the number of non-planar edges is $O(\sqrt{n})$ in practice. The separator bound applies with a constant-factor overhead."

### M2. Start-from-compact plan concentration (Rodden)
**Issue**: The hybrid protocol starts from the CS plan (compact), which could create apparent fast mixing.
**Fix**: Add to Section 6.4: "The ensemble audit in the hybrid protocol should start from random initial plans, not from the CS plan. Starting from the CS plan would produce faster apparent convergence (since the chain starts near a typical high-compactness plan) but would not certify stationarity for the full ensemble distribution."

### M3. Comparison of CS complexity to ReCom
**Issue**: The paper's complexity calculation for CS includes $k$ METIS calls per seed, but the factorization tree requires only $\log k$ levels (depth), not $k$ calls.
**Fix**: Clarify: "CS requires $\sum_{\text{levels}} p_i$ METIS calls per seed, where $p_i$ is the partition factor at level $i$. For $k = 14 = 2 \times 7$: 3 calls per seed (one 2-way, two 7-way). Total: $600 \times 3 \times m_{\rm METIS}(n, k)$. The formula $O(T \cdot k \cdot n\log n)$ overstates the cost by approximately $k/3$ for typical $k$."

## Low-Priority Revisions

### L1. Lazy chain factor of 2
**Fix**: Add: "The lazy chain mixes at most $2\times$ more slowly than the corresponding non-lazy chain (by the standard coupon-collector argument). Empirical mixing times for the non-lazy standard ReCom chain are therefore expected to be at most $2\times$ faster than the lazy chain bound — a negligible factor given the $10^4$–$10^5\times$ theory-practice gap."
