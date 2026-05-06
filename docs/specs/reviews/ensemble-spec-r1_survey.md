---
reviewer: SURVEY
role: Practitioner — usability
spec: Ensemble Search Algorithms (G series extensions)
round: 1
score: 3
date: 2026-05-06
---

## Summary

The CLI surface is largely clean but contains a latent flag conflict, the YAML config example is incomplete and inconsistent with the existing `percentile-sweep` format, and the spec does not tell a practitioner when to reach for each new mode. A redistricting analyst approaching this spec after reading the PercentileSweep and BisectionEnsemble specs would be left uncertain whether Short-Burst competes with BisectionEnsemble or complements it, and would not know whether Flip is appropriate for state court filings or only for exploratory research. The runtime guidance is entirely absent — the only compute budget information is buried in the research questions as informal "expected" claims.

## Strengths

- The decision to keep `--search` and `--structure` as distinct CLI namespaces (rather than overloading a single `--algorithm` flag) is the right call. SA as `--structure simulated-annealing` and Flip as `--search flip` are conceptually clear once you understand the three-layer compositor, and the spec correctly notes this separation is intentional.
- The implementation priority ordering (Flip → Short-Burst → SA → SMC) is sensible and practitioner-friendly: the simplest mode ships first, the most complex last. A practitioner doing incremental validation will appreciate this.
- The SMC standalone design (`redist ensemble --method smc` rather than `redist state --search smc`) is correct. SMC generates a full posterior sample over plans, not a single plan, and forcing it through the `redist state` pipeline would obscure its output semantics.

## P1 — Required changes

**Flag conflict between `--burst-length` and `--ensemble-steps`.** The PercentileSweep spec defines `--ensemble-steps` as the number of local ReCom steps per bisection node for BisectionEnsemble. Short-Burst's `--burst-length` is the same conceptual parameter (steps per burst = steps per local ReCom chain), but uses a different flag name. A practitioner running both modes will have:

```bash
redist state --search bisection-ensemble --percentile 0.5 --ensemble-steps 500
redist state --search short-burst --burst-length 20 --n-bursts 50
```

The two parameters measure different things (BisectionEnsemble runs one ensemble per bisection node; Short-Burst runs many independent bursts over the full plan), so the different names are arguably defensible. However, the spec must explicitly document why these are different parameters, and whether a user can set `--ensemble-steps` in a Short-Burst context (where it would presumably be ignored) or whether it will produce a confusing error. The YAML config example should also specify that `ensemble_steps` and `burst_length` are mutually exclusive per `search` mode.

**The YAML config example is incomplete.** The example shows:

```yaml
algorithm:
  structure: prime-factor
  weights: county
  search: short-burst
  burst_length: 20
  n_bursts: 100
  percentile: 0.0
```

But the PercentileSweep spec's YAML format places `percentile` and `seeds` as flat keys alongside `search`. The existing `redist state` config format (from the CLI reference in REDIST_CLI.md) uses a different indentation convention for algorithm sub-keys. The ensemble-search spec's YAML is inconsistent in two ways: (1) it lacks a top-level `version` or `schema` key that other config files use; (2) for Flip, no YAML example is given at all. Add a complete YAML example for each of the four modes showing all configurable parameters.

**No guidance on when to use each mode.** The spec lists four algorithms and their research questions, but a practitioner reading it cannot answer: "I need to file a plan with the North Carolina state legislature next week. Should I use BisectionEnsemble, Short-Burst, or ConvergenceSweep?" The mode-selection decision should be documented in a table or paragraph covering at least: optimisation target (compactness minimisation vs posterior sampling), compute budget requirements, legal defensibility (modes that produce a single deterministic plan vs a weighted sample), and connection to the existing ConvergenceSweep/PercentileSweep baseline. Without this, the practitioner will default to the mode they read about first, regardless of fit.

## P2 — Suggested improvements

- Runtime estimates are completely absent. The only guidance is informal: "O(1) vs O(n/k)" in the research question for Flip, and "yes for small maps; similar for large maps" for Short-Burst vs ConvergenceSweep. Add a runtime table analogous to the performance table in the ReCom spec: target step times for VT (89 tracts), NC (208 tracts), TX (5265 tracts) for each mode.
- The `--n-bursts 50` default is not motivated. Cannon et al. (2022) report experiments with n_bursts ranging from 10 to 1000. Without guidance on how to choose n_bursts, a practitioner will either under-invest (too few bursts for reliable percentile estimation) or over-invest (more bursts than needed for the objective). Add a brief note: "50 bursts is sufficient for rank-ordering at p=0.0; increase to 200+ for reliable p=0.5 estimates."
- The SMC `--steps 1` in the CLI example (`redist ensemble --method smc --particles 5000 --state NC --steps 1`) is confusing — SMC does not have a "steps" parameter in the same sense as MCMC. If `--steps` here means something other than MCMC steps (e.g., resampling rounds or number of output plans to generate), the spec must clarify this. As written, a practitioner will assume it means one MCMC step, which would produce a trivial result.

## Score: 3/4

The CLI structure and compositor placement are sound. The issues (flag name collision with `--ensemble-steps`, incomplete YAML, absent runtime guidance, missing mode-selection guidance) are all fixable in a revision pass without changing the algorithm designs. The SMC `--steps 1` anomaly should be clarified before implementation to avoid a confusing CLI default.
