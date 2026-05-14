---
pulse: 01
title: Symmetric 2x2 eigensystem
status: done
wave: rmath-core-eigen-kernel
---

# Pulse 01 - Symmetric 2x2 Eigensystem

## Purpose

Move the closed-form symmetric 2x2 covariance eigensystem from GeoSection
orientation into `rmath-core`.

## Pre-implementation Scout

```powershell
rg "compute_minor_axis|Eigenvalues|eigenvector|lambda_min|atan2" crates -n
```

## Deliverables

- [x] Add a reusable symmetric 2x2 eigensystem helper to `rmath-core`.
- [x] Add L0/L1/L2 coverage for axis-aligned, off-diagonal, degenerate, and
      non-finite inputs.
- [x] Wire `bisect-cli::geosection_orientation` through the `rmath-core` helper.
- [x] Update the shared kernel spec and this wave's pulse table.

## Completion Notes

The helper returns the smaller and larger eigenvalues plus the unit eigenvector
for the smaller eigenvalue. GeoSection preserves its existing
`minor_lat.atan2(minor_lon)` angle convention.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rmath-core -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p rmath-core -- --ignored --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-cli geosection_orientation -- --test-threads=1
git diff --check
```
