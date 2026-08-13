---
pulse: 48
wave: nrs-2010-national-baseline
date: 2026-08-13
status: complete
---

# Block-Ensemble v3 Registered Analysis Closeout

## Outcome

The registered analysis recomputed from all six retained v3 primary traces
after the frozen 500-step burn-in. It reports each State and kernel separately,
including split-chain R-hat, per-chain and pooled ESS, descriptive quantiles,
acceptance, maximum population deviation, permutation-aligned snapshot Hamming
diagnostics, and descriptive State-specific cross-kernel KS statistics.

NH and NM passed the registered scalar convergence rules for both kernels. GA
did not. Wilson cut fraction had split R-hat `1.04656` but pooled ESS `66.47`,
below the required `100`. Kruskal weighted boundary cut had split R-hat
`1.07507` and pooled ESS `53.88`, failing both thresholds. The complete
execution and exact replay evidence remains valid, but the protocol's
all-or-nothing decision is therefore `gate_passed: false`.

The package is closed non-converged without retry or extension. A deterministic
analyzer writes `analysis.json` and `summary.csv`; the package verifier
recomputes both byte-for-byte and checks the final SHA-256 manifest. Fourteen
focused v3 tests and the complete package verification pass.

## Claim Boundary

V3 supports bounded 2020 NH/NM/GA block-graph feasibility, deterministic
replay, the registered diagnostics, and State-specific Wilson-versus-Kruskal
sensitivity. It does not establish mixing, sampler equivalence, national
representativeness, neutrality, fairness, VRA or legal validity, polygon
compactness, or coverage of all valid plans. GA's non-convergence is retained
as negative evidence and cannot be replaced by an unregistered longer run.
