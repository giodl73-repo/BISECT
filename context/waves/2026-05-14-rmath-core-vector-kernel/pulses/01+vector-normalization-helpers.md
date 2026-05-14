---
pulse: 01
title: Vector normalization helpers
status: done
wave: rmath-core-vector-kernel
---

# Pulse 01 - Vector Normalization Helpers

## Purpose

Move duplicated deterministic vector primitives from spectral smoothing and
Fiedler power iteration into `rmath-core`.

## Pre-implementation Scout

```powershell
rg "fn dot|fn norm|normalize_centered|project_out_ones|smooth_spectral_vector|compute_fiedler" crates -n
```

## Deliverables

- [x] Add `rmath-core` vector helpers for dot product, L2 norm, centering, and
      normalization with typed input errors.
- [x] Add L0/L1/L2 coverage for finite input validation, dimension mismatch,
      centering, normalization, and near-zero vector behavior.
- [x] Wire `bisect-apportion::spectral` through `rmath-core` centered
      normalization.
- [x] Wire `bisect-data::fiedler` through `rmath-core` dot, centering, and
      normalization helpers.
- [x] Update the shared kernel spec and this wave's pulse table.

## Completion Notes

The pulse keeps spectral smoothing, balanced sweeps, weighted Laplacian
construction, and Fiedler certificate interpretation outside `rmath-core`.
Only reusable vector primitives moved into the shared numeric kernel.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rmath-core -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p rmath-core -- --ignored --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-apportion spectral -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-data fiedler -- --test-threads=1
git diff --check
```
