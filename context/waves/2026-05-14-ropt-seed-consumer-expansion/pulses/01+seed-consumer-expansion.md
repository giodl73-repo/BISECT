---
pulse: 01
title: Seed consumer expansion
status: done
wave: ropt-seed-consumer-expansion
---

# Pulse 01 - Seed Consumer Expansion

## Purpose

Reuse `ropt-core::derive_seed` for existing deterministic seed helpers beyond
`bisect-pareto`.

## Pre-implementation Scout

```powershell
rg "Sha256|seed_from_u64|chain_seed|particle_seed|resample_seed|step_seed|replica_seed|swap_seed" crates -n
```

## Deliverables

- [x] Extend `ropt-core::SeedPart` with `usize` support for existing ensemble
      chain-seed byte encoding.
- [x] Wire `bisect-smc::seeds`, `bisect-multiscale::seeds`,
      `bisect-ensemble::chain`, and `bisect-ensemble::parallel_tempering` through
      `ropt-core::derive_seed`.
- [x] Preserve all domain prefixes and existing public seed helper APIs.
- [x] Update shared-kernel docs and wave status.

## Completion Notes

No domain prefixes moved into `ropt-core`. The extraction only centralizes the
byte encoding, separator convention, SHA-256 digest, and first-8-byte `u64`
conversion.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p ropt-core -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-smc seeds -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-multiscale seeds -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-ensemble chain_seed replica_seed swap_seed pt_fwd_rev -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test --workspace -- --test-threads=1
git diff --check
```
