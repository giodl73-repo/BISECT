# Review: ensemble-methods.md
**Reviewer**: SCALE (statistician)
**Date**: 2026-05-05

---

## What's accurate

The R-hat formula `sqrt((N-1)/N + B/(N*W))` is stated correctly and matches the Gelman-Rubin (1992) formulation. The guide's statement that "chains are split in half before computing R-hat to detect trends within chains" correctly describes the split-chain correction from Gelman et al. (2013). The ESS formula `N / (1 + 2 * sum rho_t)` with Geyer's (1992) monotone truncation is correctly specified. The guide correctly identifies that ReCom has high lag-1 autocorrelation ("each step changes only two districts"), which is the primary reason raw step count is misleading and ESS is necessary. The Hamming autocorrelation is correctly defined as the fraction of tracts changing assignment between consecutive plans, and the integrated autocorrelation time tau_int is the correct summary statistic for mixing speed.

Critically, the R-hat convergence threshold in the guide (R-hat < 1.1) is consistent with the looser threshold used in some applied redistricting literature. However, SCALE notes a material discrepancy: the actual code implementation in `redist-analysis::ensemble_diagnostics` uses a threshold of 1.05, not 1.1 as stated in the guide's table. This is a factual error (see P1 below).

---

## P1 — Required fixes

**R-hat threshold discrepancy**: The guide's convergence table states "R-hat < 1.1: Converged." The implementation in `redist-analysis::ensemble_diagnostics` uses a threshold of 1.05 (`above_threshold: r >= threshold` where `threshold = 1.05`). The `RhatRecord` struct records `threshold: 1.05`. This is a direct contradiction between the guide and the code. Any practitioner following the guide would consider a run with R-hat = 1.08 as converged, but the CLI would flag it as `above_threshold`. This discrepancy must be resolved: either the code threshold should be updated to 1.1, or the guide's table should state 1.05. Given that 1.05 is the field standard (Gelman et al., 2013), the code is correct and the guide should be updated.

**ESS threshold discrepancy**: The guide states "ESS < 400: Run longer chains. Tail percentiles (0.1st, 99.9th) are unreliable." The implementation in `redist-analysis::ensemble_diagnostics` flags `below_threshold: ess < 100.0`. The guide's 400-threshold table and the code's 100-threshold are different by a factor of 4. A practitioner with ESS=350 would follow the guide's advice to run longer chains, but the CLI would not flag this as a problem. SCALE requires that the guide and code agree on operationalised thresholds. Since ESS < 100 is a well-established minimum in the MCMC literature, the code threshold appears correct and the guide's 400-threshold for "Run longer chains" should be clarified as a recommendation (not a CLI flag threshold), with the 100 threshold identified as the hard CLI flag.

**Section "GerryChain speed"**: The guide states "approximately 21 steps per second for North Carolina (k=14, approximately 2,700 tracts)." SCALE accepts this as a plausible measurement, but notes that GerryChain speed depends heavily on whether it uses the optimised Rust backend or the pure Python backend. The guide should state the GerryChain version (Python, with or without the Rust backend accelerator) and the hardware used for this measurement. Without this, the 21 steps/second figure cannot be reproduced or compared.

**Section "The legal argument", maximum-compactness certificate**: The certificate states the ApportionRegions plan is "more compact than approximately 99.8-99.9% of ensemble plans" for WI, GA, PA. The "approximately" range covers percentiles from 0.1st to 0.2nd. This range is reported without standard error on the percentile estimate. For a 1,000-plan ensemble, the 95% confidence interval on the 0.15th percentile is approximately ±0.10 percentile points (by binomial confidence interval: sqrt(0.0015 * 0.9985 / 1000) ≈ 0.039, so CI ≈ 0.15 ± 0.08). The guide should state the ensemble size used to derive these percentiles and the confidence interval on the percentile estimate.

**Section "For North Carolina at GerryChain speed..."**: The guide states "achieving ESS = 1,000 typically requires approximately 3,000-5,000 total steps across 4 chains (750-1,250 per chain)." This estimate is derived from a lag-1 autocorrelation of 0.15-0.25, which gives an autocorrelation time of roughly 3-5 steps. The calculation is approximately correct but is presented as if it applies universally. For a different state with a different graph topology, the required steps could be much higher. The guide should qualify this as an NC-specific estimate and note that states with larger or more complex graphs may require substantially more steps.

---

## P2 — Suggested improvements

The guide would benefit from a worked numeric example for R-hat: given 4 chains, each of length 500, with realistic between-chain and within-chain variances, what is the computed R-hat and how does it compare to the threshold? Abstract formulas are correct but harder to apply without a calibration example.

The Hamming autocorrelation section mentions that "a lag-1 autocorrelation above 0.5 suggests the chain is stuck." This threshold also lacks a citation. SCALE would prefer "above 0.5 as a heuristic consistent with [Autry et al. 2020] or similar redistricting Markov chain literature" rather than presenting it as a universal rule.

---

## Score: 2/4

Two concrete code-guide threshold mismatches (R-hat: 1.05 vs 1.1; ESS flag: 100 vs 400) are the most serious issues, as they create a direct contradiction between what the guide says and what the CLI does. Beyond these, the percentile estimates in the legal certificates lack confidence intervals and the GerryChain speed benchmark is missing hardware and version context. The statistical framework is correctly specified at the mathematical level; the operationalisation of that framework in the guide needs alignment with the code.
