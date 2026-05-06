---
reviewer: MERIDIAN
role: Computational Geographer — algorithm correctness
spec: Ensemble Search Algorithms (G series extensions)
round: 1
score: 2
date: 2026-05-06
---

## Summary

The spec describes four algorithm additions to the ensemble search infrastructure. Two of the four are described competently (Flip and Simulated Annealing), but Short-Burst contains a meaningful mischaracterisation of the Cannon et al. 2022 algorithm, and the complexity claims throughout are either wrong or ungrounded. The SMC description is accurate at the high level but too thin to evaluate statistical validity. These issues are not cosmetic — two of them affect the scientific claims the G-series papers can make.

## Strengths

- The compositor-layer placement is correct throughout. Short-Burst, Flip, and BisectionEnsemble are correctly classified as Search layer variants. Simulated Annealing is correctly classified as a Structure layer variant (it replaces METIS at each bisection node), and the spec is admirably explicit about this asymmetry.
- The Flip proposal description is accurate: picking a random boundary tract and proposing it to a neighbouring district with a Metropolis-Hastings or always-accept rule is standard in the gerrymandering literature (Chikina et al. 2017; Autry et al. 2021). The distinction from pair-merge (ReCom) is correctly drawn.
- The exponential cooling schedule default (T_0=1.0, T_final=0.01) is reasonable for redistricting subgraphs of ~200–2,000 tracts. For METIS-scale graphs the schedule would need tuning, but as a starting default it is defensible and the spec acknowledges the problem space.

## P1 — Required changes

**Short-Burst algorithm mischaracterises Cannon et al. 2022.** The pseudocode reads:

```
best_plans.push(plans.min_by(objective))
```

and then `return best_plans[floor(p * n_bursts)]`. This is not what Short-Burst does. In Cannon, Duchin, Weighill, and Wolf (2022), each burst is a short ReCom chain of length ℓ, and the *current plan at the end of the burst* — not the minimum across all steps in the burst — is kept as the candidate. The burst is a Markov chain, not a deterministic scan for optima. The optimisation comes from running many bursts and comparing their endpoints, not from scanning within each burst. The `min_by(objective)` inside the burst loop is wrong. It replaces the Markov chain dynamics with a greedy scan, which changes the algorithm's theoretical properties: it no longer approximates the correct ensemble distribution near the objective minimum, and the "proven to outperform long-chain ReCom" claim from the paper does not apply to the greedy-scan variant.

The correct pseudocode is:
```
for burst in 0..n_bursts:
    chain = RecomChain::new(initial=current_plan, seed=chain_seed(base, burst))
    run for burst_length steps
    candidates.push(chain.current_plan())
sort candidates by objective
return candidates[floor(p * n_bursts)]
```

The `p` parameter then selects a rank from the *burst-endpoint* distribution, not a rank from a within-burst scan. This matters because the endpoints have a specific distributional interpretation as approximate samples from the objective-tilted measure.

**Cooling schedule is underspecified for the SA structure variant.** The spec gives T_0=1.0, T_final=0.01 but does not specify whether these temperatures are applied per-step (over `n_steps` steps at each bisection node) or globally (across the entire bisection sequence). For a bisection-node-local application, 1,000 steps with exponential decay from 1.0 to 0.01 gives a decay rate of exp(-ln(100)/1000) ≈ 0.9954 per step — extremely slow cooling that will not reach the optimum within 1,000 steps for graphs of more than ~100 tracts. The default should be specified relative to the subgraph size, e.g., T_final proportional to 1/n_tracts, or the spec should note that n_steps should be set to ~10× the tract count per node for effective cooling.

**Complexity claims are vague or absent.** The spec claims Flip is O(1) per step vs O(n/k) for ReCom (in the research question discussion), but provides no derivation. A ReCom step costs O(subgraph size) for the spanning tree ≈ O(2n/k tracts for a pair merge), which is O(n/k) — correct. A Flip step requires picking a random boundary tract (O(boundary size) if maintained, O(n) if not) and checking contiguity (O(n) naive, O(1) if an incremental connectivity structure is maintained). The O(1) claim is only true if boundary-tract set and contiguity structures are maintained incrementally. The spec must either specify that these data structures are maintained, or correct the complexity claim.

## P2 — Suggested improvements

- The `min_by(objective)` fix will require clarifying whether `p=0.0` means "return the globally best burst endpoint" or "return the burst-endpoint distribution minimum." These are equivalent after sorting, but the intent should be stated.
- The SA cooling schedule for redistricting graphs typically benefits from a restart strategy (reset to best-known plan when temperature drops below a threshold). The spec should note this as a tuning option even if not implemented in v1.
- The SMC section notes "covers the full feasible space better than ReCom chains" — this is a strong claim that should cite the specific theoretical result (Imai et al. 2020 consistency theorem) rather than asserting it as expected behaviour.

## Score: 2/4

The Short-Burst algorithm contains a fundamental error (min-within-burst vs endpoint-of-burst) that would invalidate the G.6 paper's core claim. The SA cooling schedule underspecification will cause reproducibility failures across different subgraph sizes. These require a revision before implementation begins.
