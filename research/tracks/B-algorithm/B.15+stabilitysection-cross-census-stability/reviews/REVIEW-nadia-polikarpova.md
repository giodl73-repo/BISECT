> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Review: StabilitySection: Cross-Census Stability of GeoSection-Optimal Redistricting Maps

**Reviewer**: Nadia Polikarpova (UC San Diego, Programming Systems and Programming Languages)
**Expertise**: Formal verification, program synthesis, specification languages, correctness-by-construction software, automated reasoning
**Date**: 2026-05-02

## Overall Assessment

Reviewed from a formal methods perspective: this paper proposes a composite metric (CSS) for measuring algorithmic stability, states two propositions about its behavior, and reports empirical findings from a partial evaluation. The paper's technical contributions are clear and the propositions are well-motivated, but neither is proved --- they are stated as formal claims and then supported only by intuition or informal argument. For a paper that makes explicit use of formal notation (definitions, propositions, algorithms), this is a significant gap.

More technically, the CSS formula raises definitional questions that the paper does not address: the weights are stated to sum to 1.0 and the formula produces a score in $[0,1]$ only under specific assumptions about the components, which the paper does not verify. The Lorenz Drift Proposition involves an unspecified constant $C_{\text{pop}}$ that is not defined as a computable quantity. The district Jaccard Similarity definition uses a matching operator $\sigma(d)$ that is well-defined only if the optimal assignment matching is unique, which it may not be in general.

These are correctability issues, not fatal flaws. The paper has genuine mathematical content and the definitions are mostly precise. The issues listed below should be addressed before the formal claims are relied upon in a legal or policy context.

## Score: 3/4

**My score**: 3/4 --- Formally well-structured with mostly precise definitions; propositions are stated without proof and one involves an unspecified constant; the CSS formula has edge-case behavior that needs attention.

## Major Strengths

1. **CSS components are clearly defined**: The three CSS components ($s_{\text{seat}}$, $s_{\text{ratio}}$, $s_{\text{gap}}$) are defined separately with explicit formulas. This is good formal practice: a composite metric is more interpretable and more easily validated when its components are independently defined and can be computed separately.

2. **Two-source decomposition is formally clean**: The distinction between Type I (population-shift) and Type II (tract-boundary-redesign) changes is a well-formed partition of the perturbation space. The proposed experiment (fix 2020 graph, use 2000 populations) correctly isolates the Type I effect, assuming the spatial intersection approximation is negligible.

3. **Algorithm 1 is specified correctly**: The StabilitySection algorithm (Algorithm 1) is a straightforward loop over census years with clearly defined inputs and outputs. The specification is adequate.

4. **Proposition 2 (CSS as Geographic Burden Shift) is well-scoped**: The proposition states a conditional: if CSS$_{3\text{yr}} \geq 0.90$, then the legislature cannot justify deviation from the GeoSection map on geographic grounds alone. This is a precise logical claim, not a vague assertion, and it is the right kind of formal statement for a legal argument.

## Major Issues (Must Address)

### Issue 1: Proposition 1 (Lorenz Drift Predicts CSS) Is Not Proved
**Severity**: High
**Description**: Proposition 1 states a stability bound:
$$\Delta p^*(s, y, y') < \frac{\epsilon}{2 \cdot C_{\text{pop}}}$$
where $C_{\text{pop}}$ is "the maximum rate of change of normalised edge-cut per unit of population fraction shift."

This proposition has two problems:

First, $C_{\text{pop}}$ is not defined as a computable quantity. The paper describes it in words but does not give a formula for computing it from the graph $G$ and the population vector. Without a computable definition, the bound cannot be evaluated and the proposition cannot be used as a practical predictor. Is $C_{\text{pop}}$ the Lipschitz constant of the normalised edge-cut function with respect to node weights? If so, how is it estimated from the METIS output?

Second, the proposition is stated without proof and the informal argument after it ("this proposition formalises the intuition that states with large isoperimetric gaps are more robust") does not constitute a proof. The claim depends on the edge-cut function being locally linear in population weights, which is not obviously true for the METIS graph partitioning objective.

**Recommendation**: Either (a) prove the proposition rigorously, providing a definition of $C_{\text{pop}}$ and a proof that the bound holds for the normalised edge-cut objective; or (b) downgrade it from a proposition to a "conjecture" or "empirical observation" and add a note that the bound is informal. For the paper's legal application, an informal but clearly labeled heuristic is more defensible than an unproved formal claim.

### Issue 2: CSS Formula Has Edge-Case Behavior for Seat-Count-Changing States
**Severity**: Medium
**Description**: For states where the seat count $k$ changes between censuses, the paper uses seat *share* $\text{dem\_seats}/k$ rather than raw seat count, with a tolerance of $\pm 0.04$. This means $s_{\text{seat}}$ for these states is:
$$s_{\text{seat}}(s) = \mathbf{1}\left[\left|\frac{\text{dem\_seats}_y}{k_y} - \frac{\text{dem\_seats}_{y'}}{k_{y'}}\right| \leq 0.04\right]$$

But the paper also defines $s_{\text{seat}}$ as a binary indicator (0 or 1) for states with fixed seat counts. This creates a mixed definition where some states are evaluated with a tolerance and others are not. The tolerance of 0.04 for states with changing seat counts makes $s_{\text{seat}}$ easier to achieve for those states than for states with fixed seat counts (where even a one-seat shift produces $s_{\text{seat}} = 0$).

The CSS formula should use a consistent definition of $s_{\text{seat}}$ across all states, or the weighting should account for the fact that the metric is easier to achieve for seat-count-changing states.

Additionally: for Texas ($k_{2000} = 30$, $k_{2020} = 38$), the tolerance of $\pm 0.04$ corresponds to approximately $\pm 1.5$ seats at $k=38$. At $k=30$, the same 0.04 tolerance corresponds to $\pm 1.2$ seats. These are different absolute tolerances for different-sized delegations. A tolerance of $\pm 1/k_{\max}$ (one seat at the larger delegation size) would be more consistent.

**Recommendation**: Unify the $s_{\text{seat}}$ definition by using a tolerance of $\pm 1/k_{\max}$ for all states (including those with fixed seat counts, for which $k_{\max} = k$ and the tolerance is below 0.5 seats, effectively making it binary). Document the tolerance choice as a parameter and discuss its implications.

### Issue 3: District Jaccard Similarity Has a Non-Unique Matching
**Severity**: Medium
**Description**: Definition 2 (District Jaccard Similarity) uses:
$$\sigma(d) = \arg\max_{d''} \text{pop}(d_y \cap d''_{y'})$$
which maps each year-$y$ district to the best-matching year-$y'$ district by population overlap. This maximum is not guaranteed to be unique: two year-$y'$ districts may have identical population overlap with district $d_y$. The paper does not address this.

More subtly, the greedy maximum-overlap matching $\sigma$ is not guaranteed to be a permutation. Two year-$y$ districts might both match the same year-$y'$ district as their argmax, making $\sigma$ not injective. The Jaccard formula requires a bijective matching between districts to be well-defined as stated.

The correct definition should specify that $\sigma$ is chosen to maximize the total population overlap (i.e., $\sigma$ is the solution to a linear sum assignment problem), which is both well-defined and can be computed efficiently.

**Recommendation**: Replace the $\arg\max$ matching with an assignment-problem formulation: $\sigma^* = \arg\max_{\sigma \in S_k} \sum_d \text{pop}(d_y \cap d'_{\sigma(d)})$ where $S_k$ is the set of permutations of $k$ elements. Note that this is a maximum-weight bipartite matching, solvable in polynomial time by the Hungarian algorithm.

### Issue 4: The Binary $s_{\text{ratio}}$ Component Is Discontinuous
**Severity**: Low
**Description**: The ratio stability component is defined as a binary indicator:
$$s_{\text{ratio}}(s) = \mathbf{1}[r_y = r_{y'} \text{ for all } y, y' \in Y]$$

This means a state that has ratio 2:2 in 2000 and 2010 but ratio 2:2 in 2020 scores $s_{\text{ratio}} = 1$, while a state that has ratio 2:2 in 2000 and 2010 but ratio 3:5 in 2020 scores $s_{\text{ratio}} = 0$. The latter state is certainly less stable, but the binary indicator treats them as maximally different regardless of the magnitude of the ratio change.

For a state like NC (going from 6:8 to 5:8 to possibly 6:8 again), the binary indicator would score $s_{\text{ratio}} = 0$ even though the ratio changed by only one unit out of 14. A continuous ratio stability score that rewards small deviations would be more informative.

**Recommendation**: Define $s_{\text{ratio}}$ as a continuous score in $[0,1]$, e.g., $s_{\text{ratio}} = 1 - \max_{y,y'} |\min(r_y, k-r_y) - \min(r_{y'}, k-r_{y'})| / (k/2)$, which is 0 for identical ratios and decreases as the ratio difference increases. Report both the binary and continuous versions in the evaluation.

## Minor Issues

- **The CSS thresholds ($\geq 0.90$, $0.70$--$0.90$, etc.) are not calibrated to any data**: The paper assigns legal interpretations to specific CSS threshold ranges, but these thresholds are not derived from any distribution of actual CSS values. Until the full three-census CSS is computed for all 50 states, the thresholds cannot be empirically calibrated. They should be presented as provisional.

- **The two-year vs three-year CSS averaging is not specified precisely**: For the three-year CSS, the paper says each component is "the average across all three pairwise year comparisons." For $s_{\text{seat}}$, which is binary, the average of three binary comparisons gives a score in $\{0, 1/3, 2/3, 1\}$. This is a reasonable construction but should be stated explicitly in the definition.

- **Algorithm 1 does not specify how ties are resolved in the natural ratio scan**: If two ratios produce identical normalised edge-cuts, the algorithm must select one. GeoSection's tie-breaking rule should be specified here (or a pointer given to the GeoSection paper's specification).

## Questions for Authors

1. Is $C_{\text{pop}}$ in Proposition 1 intended to be a constant or a state-specific quantity? If state-specific, how is it estimated from the graph and population data?

2. For the Iowa $\Delta f = 0.31$ finding: what is the actual ratio in 2000 versus 2010? The tract-fraction $f$ shift from 0.18 to 0.49 is large, but the corresponding ratio shift (e.g., from 1:3 to 2:2) might be a single integer change. Are these two equivalent descriptions of the same shift?

3. Does the Lorenz drift proxy (Proposition 1) make predictions that can be validated against the 47 states with complete 2000 data? If so, what is the false positive and false negative rate?

4. For states where $s_{\text{ratio}}$ uses the normalised ratio comparison (seat-count changes), is the binary indicator applied to the normalised ratios or to the raw ratios? The paper describes both but the formula only shows the raw indicator.

## Recommendation

The paper is formally sound in most respects, with clear definitions and a well-structured framework. The main issues are: an unproved proposition with an unspecified constant, a binary $s_{\text{ratio}}$ that is less informative than a continuous version, and a Jaccard matching definition that should use a proper assignment formulation. These are correctible. Revise with attention to the proof of Proposition 1 or its downgrade to a conjecture, the Jaccard matching formalization, and a continuous $s_{\text{ratio}}$ definition.
