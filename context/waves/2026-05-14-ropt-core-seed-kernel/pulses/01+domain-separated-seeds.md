---
pulse: 01
title: Domain-separated seed derivation
status: done
wave: ropt-core-seed-kernel
---

# Pulse 01 - Domain-Separated Seed Derivation

## Purpose

Move SHA-256 based deterministic seed derivation into `ropt-core` while keeping
Pareto-specific seed domains in `bisect-pareto`.

## Pre-implementation Scout

```powershell
rg "seed_from_u64|Sha256|init_seed|cross_seed|mut_seed|base_seed" crates -n
```

## Deliverables

- [x] Add `ropt-core::derive_seed` with explicit byte-domain and typed seed parts.
- [x] Add L0/L1/L2 coverage for determinism, domain separation, empty-domain
      rejection, Pareto legacy shape, and long seed streams.
- [x] Wire `bisect-pareto::seeds` through `ropt-core` without changing public seed
      functions or historical outputs.
- [x] Update shared-kernel docs and wave status.

## Completion Notes

The shared helper is intentionally narrow: byte-domain plus typed integer seed
parts to a deterministic `u64`. Pareto operation labels and RNG usage remain in
`bisect-pareto`.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p ropt-core -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p ropt-core -- --ignored --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-pareto seeds -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test --workspace -- --test-threads=1
git diff --check
```
