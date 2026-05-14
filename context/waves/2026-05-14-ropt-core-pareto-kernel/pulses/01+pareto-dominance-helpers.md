---
pulse: 01
title: Pareto dominance helpers
status: done
wave: ropt-core-pareto-kernel
---

# Pulse 01 - Pareto Dominance Helpers

## Purpose

Move minimization-only Pareto dominance, fast non-dominated sorting, and
crowding distance into `ropt-core`.

## Pre-implementation Scout

```powershell
rg "fast_non_dominated_sort|crowding_distance|dominates\(" crates -n
```

## Deliverables

- [x] Add `ropt-core` with generic objective-vector helpers and typed input errors.
- [x] Add L0/L1/L2 coverage for dominance, sorting, crowding, dimension mismatch,
      non-finite inputs, and larger deterministic clouds.
- [x] Wire `bisect-pareto` dominance wrappers through `ropt-core` while retaining
      redistricting `Objectives`.
- [x] Update the shared kernel spec and wave status.

## Completion Notes

The extraction is minimization-only, matching the existing NSGA-II usage. The
shared crate does not know about edge cuts, partisan seats, VRA, plan validity,
or audit packages.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p ropt-core -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p ropt-core -- --ignored --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-pareto -- --test-threads=1
git diff --check
```
