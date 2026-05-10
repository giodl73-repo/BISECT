# R1 Review — Jonathan Rodden
**Paper**: U.1 ConvergenceSweep: T=600 Statutory Seed Formula
**Score**: 3.5/4
**Verdict**: Minor Revision

## Summary
U.1 certifies T=600 as the statutory stopping criterion for ConvergenceSweep, an algorithm that sweeps METIS seeds forward from a SHA-256-derived starting point and halts after T consecutive non-improving seeds. The central empirical finding is that Georgia (14 districts) is the worst-case state with a convergence tail of 511 seeds, requiring T >= 512 for certification; T=600 provides an 89-seed margin above this observed maximum. The paper also establishes the SHA-256 content-derived seed formula as the constitutional mechanism for preventing pre-census seed cherry-picking.

## Strengths
- The political economy of the seed-manipulation attack is correctly identified and the SHA-256 temporal neutrality property is the right countermeasure. The paper understands that the adversarial model is a government official who can run the algorithm with multiple seeds before publishing one, not a random attacker — and the SHA-256 formula defeats this model specifically.
- The distinction between T_stat=600 (statutory, all states certified) and T_prac=500 (research, 49 states) is practically useful. The statutory engineering — "values below 600 shall not be used to generate maps with legal effect" — is the correct enforcement design.
- The Georgia failure analysis is documented at the right level of specificity: the algorithm last improves at seed s_0+489, and T=500 halts at seed s_0+501, returning the correct plan by luck but not certifying it. This distinction between "happens to be correct" and "certifiably correct" is exactly the legal precision a statutory paper requires.
- The Certificate of Seed Neutrality (Section 4.4) is a genuine litigation artifact that no prior redistricting proposal has offered.

## Concerns
1. **Partisan outcome at Georgia's optimal vs. suboptimal seed is not reported.** The paper establishes that T=500 fails Georgia geometrically (misses the lowest EC_norm), but does not show whether the geometric difference corresponds to any partisan difference. For a political science audience, the question is whether T=500 would have produced a plan with different D seat counts than T=600. If the two plans have identical partisan outcomes, the T=600 certification matters only for geometric purity, not political neutrality. A Georgia case study comparing D seat counts under the T=500 termination plan and the T=600 certified optimum is needed.
2. **The complete j* column in Table 1 is absent.** Table 1 reports j* (last improving seed index) only for Georgia (489) and Wisconsin (1023), with em-dashes for all other states. Since j* together with tau determines T_needed, practitioners running the 2030 sweep need the complete j* dataset. The full column must be populated from the B.7 sweep log.
3. **The 89-seed margin is misstated as "approximately one standard deviation."** The standard deviation of Gumbel(mu, sigma) is sigma * pi / sqrt(6) ≈ 1.28 * sigma. For sigma_hat = 150, the standard deviation is approximately 192 seeds, not 150. The 89-seed margin is approximately 0.46 standard deviations above the empirical worst case, not 1 standard deviation. This arithmetic error should be corrected; the underlying recommendation is unchanged.
4. **No stratification of convergence tails by state characteristics.** Table 1 sorts by tau but does not include n (tract count), k (districts), or a geographic concentration measure. Understanding which state properties predict long tails (k, n, urban concentration) would allow practitioners to flag likely hard cases in the 2030 census before running the full sweep. Even an informal regression note would help.
5. **Partisan significance of the over-optimising design is not acknowledged.** ConvergenceSweep produces the minimum EC_norm plan — the most compact plan in the bisection family. In states with urban Democratic cores, this tends to pack Democratic votes into fewer districts. The paper should briefly acknowledge the compactness-proportionality tradeoff (already established in B.0) and note that the DIA accepts it in exchange for algorithmic determinism.

## Required Changes
- **P1**: Add a Georgia case study showing D seat count at the T=500 termination plan versus the T=600 certified optimum, and EC_norm at each.
- **P1**: Populate the j* column in Table 1 for all 50 states from the B.7 sweep log.
- **P1**: Correct the "approximately one standard deviation" characterisation of the 89-seed margin to "approximately 0.46 standard deviations."
- **P2**: Add a footnote or short paragraph noting which state characteristics (k, n, geographic compactness) predict longer tails, to help practitioners identify hard cases in future census sweeps.
