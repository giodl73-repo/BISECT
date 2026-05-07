# Review: G.0 — Ensemble Methodology
**Reviewer**: Moon Duchin (Ensemble methods, metric geometry of redistricting)
**Round**: 1
**Score**: 2/4

## Summary

G.0 attempts to establish a methodological framework for placing the AR algorithmic plan within ensemble distributions. The two-goal distinction is well-motivated, and the statutory argument is novel. However, the paper has several technical problems that undermine its credibility as a framework document for the entire G-series. Framework papers must be technically impeccable because errors here propagate forward. I find three significant issues that require revision before the G-series can proceed.

## Strengths

The two-goal framing (ensemble as evaluator, algorithm as canonical plan) is the paper's genuine contribution and is correctly stated. The paper does not claim that ensembles and AR are competitors, which is the right framing. The percentile definition (Eq. 1) is standard and correctly stated.

The short-burst discussion (Section 2.3) is accurate and appropriately cautious. Noting that short-burst ensembles "are not samples from the geographic distribution" is an important caveat that many applied papers miss.

## Critical Issues

**Issue 1: The ReCom stationary distribution claim is incorrect (Section 2.2).**
The paper states that ReCom "generates plans with different properties than the flip chain" and attributes this to the recombination move. This is true but the deeper issue is not stated: standard ReCom (as implemented in GerryChain) does NOT have a known stationary distribution. DeFord et al. (2021) is explicit about this — the chain targets a distribution that is proportional to the inverse of a function of spanning trees, which approximates the uniform distribution over plans but is not exactly uniform. The paper lists "Stationary dist. known: Implicit" in Table 1, but "implicit" is too weak — the correct description is "unknown/approximately uniform." This distinction matters enormously for the validity of any percentile inference: if we do not know the stationary distribution, we cannot formally interpret percentile position as a statement about the probability of a plan under geographic neutrality.

The Metropolized Forest ReCom (Autry et al. 2021) partially addresses this, but the paper mentions it only in passing without clarifying that the non-Metropolized version used in published ensemble studies has an unknown stationary distribution. All subsequent G-series papers that use DeFord 2021 ensemble results inherit this problem.

**Issue 2: The convergence diagnostic section uses an incorrect Rhat formula (Section 4).**
The formula presented computes $\hat{V} = \frac{n-1}{n} W + \frac{m+1}{mn} B$. The original Gelman-Rubin (1992) formula is $\hat{V} = \frac{n-1}{n} W + \frac{1}{n} B$. The version in this paper matches neither the 1992 paper nor the Vehtari et al. (2021) rank-normalized update (which uses $\hat{V} = \frac{n-1}{n} W + \frac{m+1}{mn} B$ — actually this IS the Vehtari formula). Checking more carefully: the formula as written is consistent with Vehtari 2021 (Eq. 4), but the paper cites only Gelman 1992. This creates confusion. The paper should cite Vehtari 2021 for this formula and note that it uses the updated version rather than the original.

More importantly: the paper states the threshold is $\hat{R} < 1.05$ in Section 4 but then states $\hat{R} < 1.1$ in the diagnostic standards table at the end of the same section. This internal inconsistency — 1.05 vs. 1.1 — must be resolved. G.4 uses 1.1 throughout. The framework paper must be consistent with G.4.

**Issue 3: The CS-to-ensemble bridge is a false analogy (Section 5).**
The paper claims that "both T and ESS measure the robustness of a result to additional sampling." This is not true. ESS measures the variance reduction from additional draws from a known distribution. T=600 measures the length of the non-improving tail in a deterministic search. These are operationally and conceptually different:
- ESS is defined relative to independent sampling from a target distribution.
- The CS tail T is defined relative to a Gumbel tail model of the objective landscape.

The paper provides no formal analogy between these two quantities. The claim that they are "complementary certificates for different goals" is reasonable as a rhetorical framing, but presenting them as parallel measures of "robustness to additional sampling" is technically incorrect and will be challenged in legal or expert witness settings.

The bridge section should either (a) formally define a new notion of "seed-sweep ESS" that is genuinely parallel to MCMC ESS, or (b) honestly state that the two certificates measure different things and cannot be numerically compared.

## Secondary Issues

- The percentile notation $\pi_f(P^*)$ conflicts with standard MCMC notation where $\pi$ denotes the stationary distribution. This needs to be changed throughout the G-series.
- The claim that the AR plan is "constitutionally grounded" in Article I §2 is made in Section 6 but was presumably established in B.11. Framework papers should not introduce new constitutional claims without full argument.
- The phrase "local optimum under small perturbations" in Section 5.2 is ambiguous. METIS finds a local optimum under the Kernighan-Lin local search. The paper should specify what "small perturbations" means: perturbations of the seed, of district assignment of individual tracts, or of the graph structure.

## Recommendation

Major revision required. The ReCom stationary distribution issue is the most serious because it propagates to G.1 through G.3. The Rhat inconsistency (1.05 vs. 1.1) must be resolved before G.4 can be published. The CS-bridge section should be substantially rewritten to avoid a false analogy.
