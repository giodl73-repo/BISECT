# Review: G.5 — Convergence and Mixing Analysis
**Reviewer**: Moon Duchin (Ensemble methods, metric geometry of redistricting)
**Round**: 1
**Score**: 2/4

## Summary

G.5 attempts to provide theoretical mixing time bounds for redistricting Markov chains. The paper's central theorem ($\sgap = \Omega(1/n^2)$, $\tmix = O(n^2 \log n)$) is potentially significant — it would be the first quantitative mixing time bound for a redistricting-specific chain. However, the proof sketch as presented is incomplete: the key step claiming the bound "tightens" from $O(n^3 \log n)$ to $O(n^2 \log n)$ is asserted without argument. Additionally, the paper analyzes a "lazy reversible ReCom" chain that is not the same as the chain used in any published redistricting study — and the relationship between the theoretical chain and the practical chain is not established.

## Critical Issues

**Issue 1: The spectral gap claim $\sgap = \Omega(1/n^2)$ is stated as a theorem but the proof sketch is incomplete.**

The proof uses the canonical path method to establish $\Phi = \Omega(1/n)$ (conductance), then claims:
- From Cheeger: $\sgap = \Omega(\Phi^2) = \Omega(1/n^2)$ ✓
- Mixing time: $\tmix = O(\log(|\mathcal{P}|)/\sgap)$

Then $\tmix = O(n\log n / (1/n^2)) = O(n^3 \log n)$ — consistent with the "intermediate step" the paper mentions.

The paper then states this "is tightened to $O(n^2 \log n)$ using the path congestion bound directly." This is a non-trivial claim. The path congestion bound can give a direct mixing time estimate without going through the spectral gap, but obtaining $O(n^2 \log n)$ from a path congestion argument on a space of size $|\mathcal{P}| \gg 2^n$ requires that the congestion is $O(n^2/\log|\mathcal{P}|) = O(n/\log n)$. This is possible but requires specific structural properties of the canonical paths that are not demonstrated in the proof sketch.

Concretely: the proof sketch says "the number of plans that can be 'on either side' of the separator is at most $|\mathcal{P}|/|\mathcal{P}^*|$ where $\mathcal{P}^*$ is the set of plans consistent with the separator assignment." This requires knowing the ratio $|\mathcal{P}|/|\mathcal{P}^*|$, which is not bounded in the sketch. Without this bound, the path congestion argument is incomplete.

The paper should either: (a) provide the complete path congestion argument with the $|\mathcal{P}|/|\mathcal{P}^*|$ bound, or (b) state the theorem as $\tmix = O(n^3 \log n)$ from the spectral gap, and note that a tighter bound of $O(n^2 \log n)$ is conjectured.

**Issue 2: The theoretical chain and the practical chain are different, and the relationship between them is unestablished.**

Section 5 analyzes the "lazy reversible ReCom" chain — a Metropolized version that holds with probability 1/2 and applies ReCom with Metropolis correction otherwise. This chain has a well-defined stationary distribution and spectral gap. However:

(a) The standard ReCom implementation (used in GerryChain and by Herschlag/DeFord) is NOT reversible — the spanning-tree selection probabilities depend on the current plan in a way that makes the detailed balance equations fail. The chain is not reversible with respect to any known distribution.

(b) The Metropolized Forest ReCom (Autry 2021) is reversible, but it has a different proposal distribution and different computational properties than standard ReCom.

The paper's Theorem 1 applies to the lazy reversible chain. The empirical mixing times in Section 4 are from what the paper calls "standard ReCom with uniform spanning-tree generation" — which is the non-reversible chain. The gap between theory (lazy reversible) and practice (non-reversible) is not analyzed. The spectral gap lower bound may not transfer from the reversible to the non-reversible chain.

**Issue 3: The "concentration near compact plans" explanation for the theory-practice gap is presented as explanation when it is a conjecture.**

Section 3.4 states: "the high-compactness region of $\mathcal{P}$ ... captures most of the stationary measure (in a compactness-weighted chain) or is rapidly visited (in a uniform chain starting from a compact plan)." For a uniform chain, the stationary measure is uniform over $\mathcal{P}$ — the high-compactness region does NOT capture most of the stationary measure. The paper acknowledges this parenthetically but then draws the wrong conclusion: if the stationary distribution is uniform, a chain starting from a compact plan will eventually visit non-compact regions, and the empirical mixing time measures how long before it does. The fact that the empirical mixing time is short means the chain visits approximately the right fraction of compact vs. non-compact regions quickly — not that it concentrates on compact plans permanently.

The real explanation for the gap is likely that the worst-case initial state (which drives the theoretical mixing time) is a maximally non-compact plan that is deeply embedded in a low-probability region of the chain's natural trajectory. Starting from a random spanning-tree plan (as done in G.4) is not the worst case. The empirical mixing time is effectively a typical-case, not worst-case, measurement.

## Secondary Issues

- The $\hat{t}_{\rm mix} \approx 400k$ scaling law is presented in Section 4.2 without uncertainty intervals or formal regression. Given that the data points are NC ($k=14$, $\hat{t}=2{,}000$), WI ($k=8$, $\hat{t}=1{,}500$), GA ($k=14$, $\hat{t}=2{,}500$), PA ($k=17$, $\hat{t}=3{,}000$), TX ($k=38$, $\hat{t}=10{,}000$), CA ($k=52$, $\hat{t}=20{,}000$), the coefficient is not 400 for NC: $2{,}000/14 = 143$, not 400. For WI: $1{,}500/8 = 187$. For TX: $10{,}000/38 = 263$. For CA: $20{,}000/52 = 385$. The coefficient is NOT approximately 400 for small states — it ranges from 143 to 385. The linear scaling claim is misleading.
- The epistemic comparison between CS and MCMC (Section 5.3) correctly notes that neither is "more rigorous in an absolute sense." This is the right framing and should be emphasized more.

## Recommendation

Major revision required for Issue 1 (incomplete proof) and Issue 2 (misidentified chain). The scaling law claim ($\approx 400k$) should be corrected — the coefficient is not 400 for small states.
