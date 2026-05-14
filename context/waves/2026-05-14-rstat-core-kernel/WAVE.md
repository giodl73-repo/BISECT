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
| 03 - Close and validation | DONE | `cargo test -p rstat-core`; `cargo test -p bisect-analysis` |

## Close Summary

`rstat-core` now owns reusable MCMC diagnostics and probability helpers:

- Gelman-Rubin R-hat;
- effective sample size;
- Hamming-distance autocorrelation;
- integrated autocorrelation time;
- regularized incomplete beta with Lanczos-gamma support.

`bisect-analysis` still owns redistricting-specific record shapes and
permutation-test report semantics, but delegates reusable math to `rstat-core`.

