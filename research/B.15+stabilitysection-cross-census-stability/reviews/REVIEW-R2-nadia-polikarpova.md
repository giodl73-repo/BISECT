# Review R2: StabilitySection: Cross-Census Stability of GeoSection-Optimal Redistricting Maps

**Reviewer**: Nadia Polikarpova (UC San Diego, Programming Systems and Programming Languages)
**Expertise**: Formal verification, program synthesis, specification languages, correctness-by-construction software, automated reasoning
**Round**: 2
**Date**: 2026-05-02

## Overall Assessment

The revision addresses three of the four issues I raised in Round 1 with varying degrees of completeness. The most important improvement is in the Jaccard matching formulation: Section 4.4 now includes pseudocode that explicitly uses the Hungarian algorithm (via `scipy.optimize.linear_sum_assignment`) for injective matching, resolving the non-uniqueness and injectivity concerns I raised. The s_seat definition has been clarified to distinguish "structurally changed" states from "unstable" states, resolving the inconsistency for seat-count-changing states. The threshold sensitivity table directly addresses the binary $s_{\text{ratio}}$ discontinuity concern by providing a concrete threshold range.

The major unresolved issue is Proposition 1: the $C_{\text{pop}}$ constant remains undefined as a computable quantity, and the proposition is still stated without proof in the revision. This was the highest-severity issue in Round 1 and has not been substantially addressed. My score rises to 3.0/4 reflecting the Jaccard and s_seat improvements, but Proposition 1 needs formal resolution.

## Score: 3.0/4

**My score**: 3.0/4 — Jaccard matching and s_seat definition substantially improved; Proposition 1 still unproved with unspecified constant; threshold sensitivity addresses the discontinuity concern partially; binary $s_{\text{ratio}}$ continuous alternative not provided.

## Changes Since Round 1: What Was Addressed

### Jaccard Matching Formulation (Issue 3 from R1 — Fully Addressed)

The pseudocode in Section 4.4 now explicitly uses `linear_sum_assignment(-overlap)` from `scipy.optimize`, which implements the maximum-weight bipartite matching. This resolves both the non-uniqueness concern (the assignment problem has a unique solution when the overlap matrix has distinct entries) and the injectivity concern (the output is a permutation by construction). The pseudocode is correct and complete.

One observation: the pseudocode computes `pop_total[i]` as a variable that is not defined in the code block. It appears to be the total population assigned to district $i$ in year 2000. This should be defined explicitly, as `sum(pop[t] for t in assignments_2000[i].tracts)`, to make the Jaccard formula self-contained.

### s_seat Clarification (Issue 2 from R1 — Substantially Addressed)

The revision correctly identifies Montana, Texas, and New York as "structurally changed" rather than "unstable" — states where the seat count changed between census years and the s_seat comparison requires a normalised approach. The CSS definition now explicitly states that for seat-count-changing states, the comparison uses seat share with a tolerance of $\pm 1/k_{\max}$ rather than a raw seat count equality check.

The mixed definition (tolerance for seat-count-changing states vs. exact match for fixed-seat-count states) is resolved by making the tolerance explicit. However, the recommendation to unify using $\pm 1/k_{\max}$ for all states (including fixed-seat-count states, where the tolerance is $< 0.5$ seats and effectively binary) was not adopted. This is a minor remaining inconsistency: states with $k=2$ (MT, WV, RI) use a $\pm 0.5$ tolerance, which corresponds to anything — effectively ignoring s_seat for 2-seat states.

**Recommendation**: State explicitly that for $k=2$ states, $s_{\text{seat}}$ is always 1 because the $\pm 1/k_{\max}$ tolerance exceeds 0.5 seats, making the comparison vacuous. If this is intentional (2-seat states have only one possible Democrat seat count in the presence of a majority), say so.

### Threshold Sensitivity (Issue 4 from R1 — Addressed)

The threshold sensitivity table provides the stability count at four threshold levels. While this does not provide a continuous $s_{\text{ratio}}$ definition (which was my recommendation), it does provide empirical evidence for the discontinuity concern: the jump from 15 to 20 stable states between $\tau = 0.02$ and $\tau = 0.05$ shows that five states are in the sensitive range. This is an acceptable response for a paper in this stage of development.

## Remaining Issues

### Issue 1: Proposition 1 ($C_{\text{pop}}$ Bound) Is Still Unproved
**Severity**: High
**Description**: The revision does not address Proposition 1. The constant $C_{\text{pop}}$ is still described as "the maximum rate of change of normalised edge-cut per unit of population fraction shift" without a computable definition. The proposition is stated as a formal claim without proof, and the informal argument after it ("this proposition formalises the intuition...") is unchanged from Round 1.

For a paper that uses formal notation and makes explicit use of propositions in its legal argument, an unproved proposition with an unspecified constant is a correctness gap. The issue is not just mathematical elegance — the legal section (Proposition 2) depends on Proposition 1 implicitly: if Lorenz drift predicts CSS, then monitoring $\Delta p^*$ gives advance warning of which states have defensible CSS claims. If Proposition 1 is incorrect or uncomputable, the Lorenz proxy's predictive value is ungrounded.

**Recommendation**: Either (a) prove Proposition 1 by defining $C_{\text{pop}}$ as the Lipschitz constant of the normalised edge-cut function with respect to node weight perturbations (this is a computable quantity for a fixed graph) and establishing the bound formally; or (b) downgrade Proposition 1 to a "Conjecture" label with a note that the bound is informal and the stability prediction is empirically supported (referencing the threshold sensitivity results as evidence). Option (b) is the lower-effort resolution and is scientifically honest.

### Issue 2: Continuous $s_{\text{ratio}}$ Not Provided
**Severity**: Low-Medium
**Description**: The threshold sensitivity table addresses the discontinuity concern for the binary stability classification but does not provide a continuous $s_{\text{ratio}}$ definition. The binary $s_{\text{ratio}}$ remains in the CSS formula. For states near the stability boundary (Alabama at $\Delta f = 0.06$, South Carolina at $\Delta f = 0.07$), the binary classification treats them as maximally different from stable states even though they are quite close.

This matters formally: a state that is just above the threshold and one that is just below receive the same $s_{\text{ratio}}$ values (0 and 1 respectively) despite having nearly identical $\Delta f$ values. The CSS formula inherits this discontinuity.

**Recommendation**: Add a note in the CSS definition section acknowledging that $s_{\text{ratio}}$ is discontinuous and that a continuous variant (e.g., $s_{\text{ratio}} = \max(0, 1 - \Delta f / 0.10)$) could be used in applications where smooth comparisons are needed. Present both the binary and continuous formulations and note that the empirical results use the binary version.

### Issue 3: $p^*_{2000}$ Projection Is Circular
**Severity**: Low
**Description**: In the Iowa case study, the paper projects "$p^*_{2000} \approx 0.25$" as "expected direction." This value is derived from the ratio geometry: a 1:3 split at $k=4$ implies $p^* = \min(1,3)/4 = 0.25$ by the $p^* = \min(r, k-r)/k$ formula. The projection is circular — it derives the Lorenz proxy value from the ratio outcome that the proxy is supposed to predict. If $p^*_{2000}$ is computed independently from the Lorenz curve of the population distribution and happens to equal 0.25, that is evidence for the proxy. Deriving it from the ratio is tautological.

**Recommendation**: Either compute $p^*_{2000}$ from the Iowa 2000 sweep output (the Lorenz curve of population density by geographic area), or remove the projection and acknowledge that $p^*_{2000}$ is unknown until the computation is performed.

## Minor Issues

- The two-year vs. three-year CSS averaging is now more clearly specified: Section 3 states "average across all pairwise year comparisons." For the three-year case with binary $s_{\text{seat}}$, the paper should note that the average of three binary comparisons takes values in $\{0, 1/3, 2/3, 1\}$, and that a state with two matching years and one non-matching year receives $s_{\text{seat}} = 2/3$, which is a rational number rather than a "moderately stable" label. The CSS threshold of $0.70$ would then not be achievable by a state with $s_{\text{seat}} = 2/3$ if $s_{\text{ratio}} = 0$ — these edge cases should be documented.

- Algorithm 1 still does not specify tie-breaking for identical normalised edge-cuts. A pointer to the GeoSection specification would suffice.

- The pseudocode in Section 4.4 uses `np.zeros` and `set()` without import statements. While this is pseudocode, adding imports would clarify that this is intended as executable code, not informal notation.

## Questions for Authors

1. For $k=2$ states (MT, WV, RI) under the $\pm 1/k_{\max}$ tolerance: is $s_{\text{seat}}$ effectively always 1, or is the formula undefined for 2-seat states? The paper should clarify.

2. Is $C_{\text{pop}}$ in Proposition 1 intended to be a single constant for the whole dataset, or a state-specific quantity estimated from each state's graph structure? If state-specific, how does it vary across states?

3. For the five states that gain stability between $\tau = 0.02$ and $\tau = 0.05$: do these states show the same instability profile in seed stability (B.7)? Are they consistently borderline across both stability dimensions?

## Recommendation

The revision substantially improves the Jaccard matching formulation and the s_seat definitional consistency. These were correctness issues in Round 1 that are now resolved. The remaining issues — Proposition 1 proof gap, continuous $s_{\text{ratio}}$, and the circular Iowa $p^*$ projection — are all fixable without new data. My score rises to 3.0/4. Revise and resubmit with attention to Proposition 1 (downgrade to conjecture at minimum) and the Iowa $p^*$ computation.
