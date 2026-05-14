---
pulse: 01
title: Local weighted cut helper
status: done
wave: cli-weighted-cut-scoring-cleanup
---

# Pulse 01 - Local Weighted Cut Helper

## Purpose

Remove repeated weighted crossing-edge scoring loops in `bisect-cli` without
promoting the logic to a shared graph kernel.

## Pre-implementation Scout

```powershell
rg "filter_map\\(\\|\\(&\\(u, v\\), &w\\)\\||contains\\(&u\\) != contains\\(&v\\)" crates/bisect-cli/src/bisection_runner.rs -n
```

## Deliverables

- [x] Add a local `weighted_edge_cut` helper in `bisection_runner.rs`.
- [x] Replace repeated local weighted cut scoring loops.
- [x] Keep subgraph edge-weight mapping and weighted scoring policy local.
- [x] Update shared-kernel docs and wave status.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-cli weighted --lib -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-cli connected --lib -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test --workspace -- --test-threads=1
git diff --check
```
