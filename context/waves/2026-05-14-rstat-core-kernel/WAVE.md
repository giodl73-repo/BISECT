---
wave: rstat-core-kernel
date_open: 2026-05-14
status: complete
source_goal: docs/specs/2026-05-13-shared-math-graph-kernels.md
---

# RSTAT Core Kernel

## Mission

Extract reusable deterministic statistical kernels from BISECT analysis into a
small `rstat-core` crate without moving election-audit semantics out of
`rcount-stats`.

## Claim Boundary

This wave may extract generic statistical computation. It must not make RSTAT a
certification layer, alter RCOUNT package verification, or move RLA method
semantics out of `rcount-stats`.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - MCMC diagnostics kernel | DONE | `rstat-core::mcmc`; `bisect-analysis::ensemble_diagnostics` wrappers |
| 02 - Probability kernel | DONE | `rstat-core::probability::regularized_incomplete_beta`; permutation consumer |
| 03 - Summary and quantile kernel | DONE | `rstat-core::summary`; L0/L1/L2 tests; partisan consumer |
| 04 - Bootstrap resampling kernel | DONE | `rstat-core::resampling`; L0/L1/L2 tests; partisan bootstrap consumer |
| 05 - Hypothesis and multiple-testing kernel | DONE | `rstat-core::hypothesis`; permutation and bloc-voting consumers |
| 06 - Weighted summary kernel | DONE | `rstat-core::summary` weighted helpers; bloc-voting and compactness consumers |
| 07 - Normal CDF probability helper | DONE | `rstat-core::probability::standard_normal_cdf`; bloc-voting consumer |
| 08 - Bootstrap percentile interval reuse | DONE | bloc-voting cluster bootstrap uses `rstat-core::summary` intervals |
| 09 - Close and validation | DONE | `cargo test -p rstat-core`; `cargo test -p rstat-core -- --ignored`; `cargo test -p bisect-analysis` |

## Close Summary

`rstat-core` now owns reusable MCMC diagnostics and probability helpers:

- Gelman-Rubin R-hat;
- effective sample size;
- Hamming-distance autocorrelation;
- integrated autocorrelation time;
- regularized incomplete beta with Lanczos-gamma support.
- descriptive summary statistics;
- deterministic R-7 quantiles and percentile intervals.
- seeded bootstrap statistic and percentile interval helpers.
- empirical p-values, ESS beta correction, Bayesian detection score, and
  multiple-testing corrections.
- weighted descriptive summaries.
- standard Normal CDF approximation.
- R-7 percentile interval reuse for bootstrap reports.

`bisect-analysis` still owns redistricting-specific record shapes and
permutation-test report semantics, but delegates reusable math to `rstat-core`.
The test ladder now includes inline L0 tests, integration L1 tests, and ignored
L2 numeric stress tests that can be run explicitly.

