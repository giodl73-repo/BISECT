> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Review: VRASection: Geographic Alignment Score for Minority Opportunity District Bisection

**Reviewer**: George Karypis (University of Minnesota, METIS developer)
**Expertise**: Graph partitioning, multilevel algorithms, METIS design, parallel graph algorithms, sparse matrix algorithms
**Date**: 2026-05-02

## Overall Assessment

This paper proposes a clean algorithmic extension to GeoSection's ratio scanning framework by adding a post-hoc alignment bonus to the ratio selection criterion. The design is algorithmically sound, computationally cheap (zero additional METIS calls), and correctly motivated by the constraint scale mismatch failure mode identified in B.3.

My assessment from a graph algorithms perspective is that the algorithm is well-defined and the theoretical analysis is correct. The paper establishes the key propositions (GeoSection reduction at $w = 0$, pure concentration at $w = 1$) cleanly, and the algorithm pseudocode is precise. However, there are several algorithmic questions that the paper does not address adequately, and one definition that needs clarification to ensure the score is well-posed.

The primary algorithmic concern is that the alignment score $A$ is computed from the best-seed bisection at each ratio, but the best seed is selected by minimum edge-cut, not by alignment. This means the alignment score is evaluated on a bisection that was not optimized for alignment --- a bisection selected for compactness may not be representative of the achievable alignment at that ratio. The paper's scoring function combines these two quantities as if they were jointly optimized, but they are not.

## Score: 3/4

**My score**: 3/4 --- Sound algorithm with a well-defined objective, but the decoupled optimization of compactness and alignment introduces a bias that should be analyzed and either justified or corrected.

## Major Strengths

1. **Zero-cost implementation**: The alignment score is computed from METIS output already produced for the ratio scan. Adding this computation costs $O(\lfloor k/2 \rfloor \cdot |V|)$ arithmetic operations, which is negligible compared to the METIS runtime. This is an important practical property for a production redistricting system.

2. **Clean reduction to GeoSection**: Proposition 1 (VRASection reduces to GeoSection at $w_\text{vra} = 0$) is correctly proven and establishes a clean baseline. This makes the algorithm's behavior at $w_\text{vra} = 0$ formally verifiable.

3. **Correct MetisVra failure diagnosis**: The constraint scale mismatch explanation (60--800x scale difference between population balance and minority VAP fraction) is the correct diagnosis of why ncon=2 fails. VRASection's architectural response --- removing minority VAP from the METIS constraint layer entirely --- is the right fix.

4. **Bounded alignment weight**: The observation that $w_\text{vra} = 0.40$ gives a 60/40 compactness/alignment split, with the legal interpretation that compactness is quantifiably predominant, is a useful design property. It makes the algorithm's behavior interpretable and the parameter choice defensible.

## Major Issues (Must Address)

### Issue 1: The Alignment Score Is Evaluated on a Compactness-Optimized Bisection
**Severity**: High
**Description**: Algorithm 1, lines 7--10, compute the alignment score $A_i$ from $L_{\min}$, which is the minimum-edge-cut bisection at ratio $i:(k-i)$ across $N$ seeds. But $L_{\min}$ is selected by $\min \mathrm{EC}(L)$, not by $\max A(L)$. The alignment score of the compactness-optimal bisection may be substantially lower than the alignment achievable at ratio $i$ with a different seed.

This creates a problem for the scoring function interpretation. The paper defines:
$$\text{score}(i) = \mathrm{EC}_\text{norm}(i) - w_\text{vra} \cdot A(L_{\min}) \cdot \max(\mathrm{EC}_\text{norm}(i), 1)$$
and interprets this as combining the best compactness at ratio $i$ with the alignment of the best compactness bisection. But the "best compactness at ratio $i$" bisection may have lower alignment than a slightly worse compactness bisection at the same ratio. The score is therefore not the optimal combination of compactness and alignment at each ratio --- it is the compactness-optimal bisection's alignment.

To see why this matters: suppose at ratio 2:5 in Alabama, the minimum-EC bisection has $A = 0.20$ (Birmingham isolation, which is compact but not alignment-optimal), while a nearby bisection has $A = 0.55$ at an EC cost of 2%. The paper's algorithm would compute $\text{score}(2:5)$ using $A = 0.20$, not $A = 0.55$. The algorithm might therefore prefer a different ratio when ratio 2:5 with better alignment would have won if alignment had been jointly optimized.

In the empirical results, this appears not to matter: Alabama's 2:5 wins with the compactness-optimal seed's alignment score of 0.42. But the theoretical gap between "compactness-optimal alignment" and "achievable alignment" at each ratio is undefined and could affect results in states where the best compact bisection is alignment-poor.

**Recommendation**: Add $N$ alignment-maximizing seeds per ratio in addition to the $N$ compactness-minimizing seeds, and compute the score on the Pareto frontier of (EC, A) at each ratio. Alternatively, justify that the compactness-optimal bisection at each ratio is approximately alignment-optimal (which would require an empirical or theoretical argument about the correlation between compactness and alignment in geographic graphs). The current formulation should be explicitly noted as using compactness-optimal seeds, with a footnote acknowledging the decoupling.

### Issue 2: The $\max(\mathrm{EC}_\text{norm}(i), 1)$ Factor Has Surprising Edge-Case Behavior
**Severity**: Medium
**Description**: Definition 4 uses $\max(\mathrm{EC}_\text{norm}(i), 1)$ to ensure the alignment bonus scales with the compactness cost. The motivation is: for ratios with very small edge-cuts (near 0), the alignment bonus should not dominate. At the same time, for typical states where $\mathrm{EC}_\text{norm} \gg 1$, the formula reduces to $\mathrm{EC}_\text{norm}(i) \cdot (1 - w_\text{vra} \cdot A)$.

However, the $\max(\cdot, 1)$ floor creates a discontinuity in the score's behavior as $\mathrm{EC}_\text{norm}$ passes through 1. For ratios with $\mathrm{EC}_\text{norm}$ near 1, small changes in EC can cause the alignment term to jump between using EC and using 1, which destabilizes the comparison between nearby ratios.

More concretely: suppose two ratios have $\mathrm{EC}_\text{norm} = 0.9$ and $\mathrm{EC}_\text{norm} = 1.1$, both with $A = 0.5$. Their scores under $w_\text{vra} = 0.4$ are:
- Ratio A: $0.9 - 0.4 \cdot 0.5 \cdot 1.0 = 0.9 - 0.2 = 0.70$ (uses floor)
- Ratio B: $1.1 - 0.4 \cdot 0.5 \cdot 1.1 = 1.1 - 0.22 = 0.88$ (uses EC)

The first ratio wins even though it has lower EC, which is the intended behavior. But the size of the gap is discontinuous at EC = 1. For states with small $k$ (Vermont, Alaska, one or two district states) where EC values may be smaller, this floor could produce counterintuitive ratio rankings.

**Recommendation**: Report the distribution of $\mathrm{EC}_\text{norm}$ values across the six test states to confirm that all values are $\gg 1$ and the floor is never active. If the floor is inactive in practice, note this in the text and simplify the formula to the approximation $\mathrm{EC}_\text{norm}(i) \cdot (1 - w_\text{vra} \cdot A)$ for clarity.

### Issue 3: Sensitivity to Seed Count $N$ Is Not Analyzed
**Severity**: Medium
**Description**: The algorithm uses $N = 50$ seeds per ratio. For a state with $k = 14$ (Georgia, North Carolina), the ratio scan evaluates 7 distinct ratios, each requiring 50 METIS calls, for a total of 350 METIS invocations at level 1. For a state with $k = 7$ (Alabama), this is 3 ratios $\times$ 50 seeds = 150 calls.

B.7 (the seed sensitivity paper) analyzed the effect of seed count on solution quality for GeoSection. Does that analysis carry over to VRASection? Specifically: is $N = 50$ sufficient for the alignment score estimates to be stable, or does the alignment score of the minimum-EC seed vary substantially across runs?

For Alabama, the paper reports a specific alignment score (0.42) for the 2:5 split. If this value is unstable across seeds, the score comparison between 2:5 (322.5) and 1:6 (350.2) could flip on a different run. The gap of 27.7 points suggests reasonable stability, but this should be verified.

**Recommendation**: Report the standard deviation of alignment scores across the $N$ seeds for each ratio in Alabama. If the alignment score is stable (low variance across seeds), note this. If it is not, discuss the implications for the ratio selection decision.

## Minor Issues

- **Algorithm terminates when $k=1$**: Line 1 of Algorithm 1 returns $\{V\}$ when $k=1$, which is correct. But the recursion at the end of Algorithm 1 calls $\mathrm{GeoSection}(G_L, i^*, N, \delta)$ and $\mathrm{GeoSection}(G_R, k-i^*, N, \delta)$. When $i^* = 1$, the left sub-call becomes $\mathrm{GeoSection}(G_L, 1, N, \delta)$, which should return $\{V_L\}$ immediately. Confirm this is the intended behavior and add a note in the pseudocode for clarity.

- **Balance tolerance propagation**: The algorithm passes balance tolerance $\delta$ to METIS with `ufactor = 1 + δ/k`. This is GeoSection's existing convention. Is this correct for the sub-calls as well? When the recursion reaches $\mathrm{GeoSection}(G_R, k-i^*, N, \delta)$, the `ufactor` computation inside GeoSection will use $k-i^*$ (the number of districts in the sub-problem), not the original $k$. Confirm this is intended and is consistent with the balance tolerance guarantee.

- **Complexity statement**: The paper states the overhead is $O(\lfloor k/2 \rfloor \cdot |V|)$ additional arithmetic operations. This is correct for the alignment score computation per se, but the total computational cost of the first-level ratio scan is $O(\lfloor k/2 \rfloor \cdot N \cdot T_\text{METIS})$ where $T_\text{METIS}$ is the METIS runtime for a single bisection. The alignment score computation is dominated by this cost. The complexity statement is correct but should be contextualized relative to the METIS runtime.

- **The `min(i, k-i)` normalization**: The GeoSection normalization $\mathrm{EC}/\sqrt{\min(i, k-i)}$ is carried over from B.8. The paper should include a brief justification of why this normalization makes ratios comparable for the combined score --- specifically, that a 2:5 and a 5:2 split receive the same normalization, so the algorithm evaluates ratios $i \leq \lfloor k/2 \rfloor$ without double-counting.

## Questions for Authors

1. Is the alignment score of the compactness-optimal seed correlated with the maximum achievable alignment at each ratio? Have you measured this for Alabama or any other state?

2. What is the distribution of $\mathrm{EC}_\text{norm}$ values across the six test states? Is the $\max(\cdot, 1)$ floor ever active?

3. For Alabama, what is the variance of the alignment score across the 50 seeds at ratio 2:5? How large would the variance need to be for the 2:5 vs. 1:6 comparison to be unreliable?

4. The recursion calls GeoSection (not VRASection) on the sub-problems. Is this a principled choice for all states, or only for states where the minority population is expected to be already concentrated in the $L^*$ sub-region?

## Recommendation

Revise to address the decoupled optimization concern (Issue 1). The algorithm is well-designed and the empirical results are promising, but the theoretical gap between "compactness-optimal alignment" and "achievable alignment" needs to be addressed before the score function can be presented as a principled combination of the two objectives. The other issues are straightforward to address.

This is a clean algorithmic paper with a sound design. The legal analysis is outside my expertise, but the algorithmic core is solid pending the revision on Issue 1.
