---
pulse: 01
title: Undirected edge-cut count
status: done
wave: rgraph-edge-cut-kernel
---

# Pulse 01 - Undirected Edge-Cut Count

## Purpose

Move duplicated "count each undirected cross-assignment edge once" logic into
`rgraph-core`.

## Pre-implementation Scout

```powershell
rg "edge_cut|count_edge_cuts|assignment\\[.*\\] !=" crates -n
```

## Deliverables

- [x] Add `rgraph-core::undirected_edge_cut` with typed input errors.
- [x] Add L0/L1/L2 coverage for usize/u32 adjacency, assignment mismatch,
      out-of-bounds neighbors, and larger grid smoke tests.
- [x] Wire `bisect-local-search`, `bisect-pareto`, and
      `bisect-ensemble::parallel_tempering` consumers through `rgraph-core`.
- [x] Update shared-kernel docs and wave status.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rgraph-core -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p rgraph-core -- --ignored --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-local-search edge_cut -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-pareto ec_ -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-ensemble p0_le_p1_ec -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test --workspace -- --test-threads=1
git diff --check
```
