# Review: G.5 — Convergence and Mixing Analysis
**Reviewer**: Percy Liang (ML, statistical methodology)
**Round**: 1
**Score**: 2/4

## Summary

G.5 derives theoretical mixing time bounds for redistricting chains. The spectral gap framework is correctly set up. However, the proof of the main theorem is incomplete (the tightening from $O(n^3 \log n)$ to $O(n^2 \log n)$ is not shown), the empirical scaling law ($\hat{t}_{\rm mix} \approx 400k$) is numerically inconsistent with the data, and the theoretical chain being analyzed differs from the chain used in practice. These are serious issues for a theory paper whose purpose is to provide rigorous mixing time bounds.

## Critical Issues

**Issue 1: Proof sketch is incomplete for the $O(n^2 \log n)$ claim.**
The canonical path argument establishes $\Phi = \Omega(1/n)$ and thus $\sgap = \Omega(1/n^2)$. The standard mixing time bound from spectral gap gives:
$$\tmix(\epsilon) \leq \frac{\log(1/(\epsilon \pi_{\min}))}{\sgap}$$
For a state space of size $|\mathcal{P}|$ with uniform $\pi$: $\pi_{\min} = 1/|\mathcal{P}|$, so $\log(1/\pi_{\min}) = \log|\mathcal{P}| \approx n\log n$ (for exponential state spaces). This gives $\tmix = O(n^2 \cdot n\log n) = O(n^3 \log n)$.

To get $O(n^2 \log n)$ requires either:
(a) A path congestion bound that gives $\max$-congestion $\leq C/n$ directly, yielding $\tmix \leq C \cdot n^2/n = C \cdot n$ — too tight; or
(b) A direct path argument where paths have length $O(n)$ and congestion $O(n)$ per edge, giving $\rho \cdot T = O(n^2)$ and $\tmix \leq O(n^2 \log|\mathcal{P}|/\sgap) = O(n^2 \cdot n\log n / (1/n^2))$ — still $O(n^5)$.

The paper does not show how $O(n^2 \log n)$ arises. This is not a minor gap — it is the central theoretical claim. The theorem should either be proven or restated to match what can be established from the sketch.

**Issue 2: The $\hat{t}_{\rm mix} \approx 400k$ scaling law is numerically incorrect.**
From the data in Table 4 of G.5 (identical to G.4 Table 4):

| State | $k$ | $\hat{t}_{\rm mix}$ | $\hat{t}/k$ |
|-------|-----|-------------------|----|
| NC | 14 | 2,000 | 143 |
| WI | 8  | 1,500 | 188 |
| GA | 14 | 2,500 | 179 |
| PA | 17 | 3,000 | 176 |
| TX | 38 | 10,000 | 263 |
| CA | 52 | 20,000 | 385 |

The ratio $\hat{t}/k$ ranges from 143 to 385. The claim of "approximately 400" is only accurate for California. For North Carolina (the most litigated state in the dataset), the coefficient is 143, not 400. A claim of "$\hat{t}_{\rm mix} \approx 400k$" for NC would predict 5,600 steps while the actual is 2,000. The formula over-predicts by 2.8×.

The scaling appears to be sub-linear: $\hat{t}_{\rm mix}$ grows faster than $k$ but the proportionality constant increases with $k$. A power-law fit $\hat{t} \propto k^\alpha$ would be more appropriate. The paper should fit this relationship properly and report the fitted parameters.

**Issue 3: The theoretical chain is not the chain used in the empirical analysis.**
Theorem 1 applies to the "lazy reversible ReCom chain" — a Metropolized version that holds with probability 1/2 at each step. The empirical mixing times (Sections 4, G.4) are measured from the standard ReCom chain with uniform spanning-tree generation, which is not reversible. The paper applies the theoretical bound to explain the empirical gap, but the theoretical bound is not for the chain that was empirically analyzed. This makes the comparison in Table 3 (theoretical $\hat{t}_{\rm mix}$ vs. empirical $\hat{t}_{\rm mix}$) misleading — the theoretical bound is for a different (slower, lazy) chain.

## Secondary Issues

**The concentration explanation for the theory-practice gap is incorrect for uniform chains.**
Section 3.4 states the chain concentrates near compact plans. For a uniform stationary distribution (standard ReCom), the mass is NOT concentrated near compact plans — compact plans are a small fraction of $|\mathcal{P}|$. Starting from a compact plan gives fast empirical mixing because the chain starts near a typical point for *the experiments*, not because the distribution concentrates there. The paper conflates start-state dependence with distribution concentration.

**The $O(T \cdot k \cdot m_{\rm METIS}(n,k))$ complexity for CS is stated as $O(T \cdot k \cdot n\log n)$.** METIS partitioning is $O(n\log n)$ for the multilevel Kernighan-Lin heuristic — this is approximately correct. But $T = 600$ seeds each require $k-1$ METIS calls in the factorization tree (not just $k$). For a two-level tree, the cost is $1 + p$ calls (one top-level, $p$ bottom-level). The total is $(1 + p) \cdot m_{\rm METIS}$ where $p$ is the second factor in the prime factorization. For $k = 14 = 2 \times 7$, this is $1 + 2 = 3$ METIS calls. The paper's formula overstates the CS cost by a factor of approximately $k/\text{depth}$.

## Recommendation

Major revision required. Theorem 1's proof must be completed or the claim reduced. The scaling law must be corrected. The theoretical-vs.-empirical chain mismatch must be addressed.
