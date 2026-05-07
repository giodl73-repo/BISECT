# Review: G.5 — Convergence and Mixing Analysis
**Reviewer**: George Karypis (Graph partitioning, METIS)
**Round**: 1
**Score**: 3/4

## Summary

G.5 derives theoretical mixing time bounds for redistricting Markov chains on planar graphs and contrasts these bounds with empirical mixing times from G.4. The main theoretical claim — $\Omega(1/n^2)$ spectral gap and $O(n^2 \log n)$ mixing time — is a standard result from the theory of random walks on planar graphs. The paper's most interesting contribution is explaining the $10^4$–$10^5\times$ gap between theory and practice via concentration near compact plans. The ConvergenceSweep comparison is effective.

## Strengths

The spectral gap framework (Cheeger inequality, conductance, planar separator) is correctly set up. The Lipton-Tarjan separator bound ($O(\sqrt{n})$ separator for planar graphs) is correctly cited and correctly applied to the bottleneck argument. The resulting $\Omega(1/n^2)$ spectral gap bound is consistent with the standard literature on random walks on planar-graph-based state spaces.

The practical scaling law $\hat{t}_{\rm mix} \approx 400k$ (empirical mixing time scales with district count, not tract count) is one of the most useful practical results in the G-series. The observation that the relevant state-space diameter is approximately $400$ ReCom steps per district — rather than the $O(n^2)$ spectral-gap prediction — is a genuine empirical finding that should be highlighted more prominently.

The "when ensemble is irreplaceable" section (Section 6) is well-structured and correctly identifies the two use cases where ensemble methods cannot be replaced by CS: auditing enacted maps and characterising the feasible space. This division of labor is important for the G-series legal argument.

## Weaknesses

**The proof sketch for Theorem 1 has a logical gap.** The proof uses a canonical path argument to establish $\Phi = \Omega(1/n)$ (conductance), then applies the Cheeger inequality to get $\sgap = \Omega(\Phi^2) = \Omega(1/n^2)$. The mixing time bound is then stated as $\tmix = O(\log(|\mathcal{P}|)/\sgap) = O(n^2 \log n)$.

But the intermediate step "$O(n \log n \cdot n^2) = O(n^3 \log n)$, which is tightened to $O(n^2 \log n)$ using the path congestion bound directly" is not derived — it is asserted. If the mixing time from spectral gap and state space size is $O(n^3 \log n)$, saying it is "tightened" by a different argument without showing the tighter argument is insufficient for a theorem. The paper should either:
(a) Provide the path congestion argument that gives the $O(n^2 \log n)$ bound, or
(b) State the proven bound as $O(n^3 \log n)$ and note that the $O(n^2 \log n)$ is a conjecture pending a tighter analysis.

In practice, this distinction does not affect the empirical conclusions (both bounds are many orders of magnitude above observed mixing times), but as stated, Theorem 1's proof is incomplete.

**The gap between theory and practice ($10^4$–$10^5\times$) is attributed to "concentration near compact plans" but this explanation is underdeveloped.** The paper states: "the high-compactness region of $\mathcal{P}$ captures most of the stationary measure (in a compactness-weighted chain) or is rapidly visited (in a uniform chain starting from a compact plan)." But standard ReCom uses a uniform (not compactness-weighted) stationary distribution. If the chain is started from a compact plan (as is typical in practice), the chain's initial behavior reflects exploration of the compact neighborhood — but the mixing time measures the time to reach stationarity from the WORST initial state, not from a compact initial state. The paper conflates "fast mixing from compact initial conditions" with "the chain mixes fast in practice." The theoretical bound is a worst-case bound; the empirical mixing time is an average-case measurement from specific initial conditions.

## Minor Issues

- The lazy chain assumption (probability $1/2$ at each step) in the reversible ReCom formulation adds a factor of 2 to the mixing time. The paper should state whether the empirical results (Table 3, 4) use the lazy chain or the standard non-lazy chain.
- The comparison table (Table 1 in Section 5) correctly shows CS as "determined" (deterministic) and certified ReCom as having "only with fixed seed" determinism. But the table entry for "Theory basis" for CS is "Gumbel tail model" — this is correct but should reference the B.16 paper explicitly.

## Recommendation

Accept with moderate revisions. The proof sketch for Theorem 1 needs either the missing tightening argument or a reduced claim ($O(n^3 \log n)$). The gap explanation needs to distinguish worst-case theoretical bounds from average-case empirical behavior.
