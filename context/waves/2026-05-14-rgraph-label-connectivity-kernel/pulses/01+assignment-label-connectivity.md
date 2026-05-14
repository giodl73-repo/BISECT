---
pulse: 01
title: Assignment label connectivity
status: done
wave: rgraph-label-connectivity-kernel
---

# Pulse 01 - Assignment Label Connectivity

## Purpose

Move duplicated "does this assignment label induce one connected subgraph?"
logic into `rgraph-core`.

## Pre-implementation Scout

```powershell
rg "all_.*connected|district_connected|cluster_connected|VecDeque" crates -n -g "*.rs"
```

## Deliverables

- [x] Add typed `rgraph-core` helpers for single-label and label-set
      connectivity over adjacency lists plus assignments.
- [x] Add L0/L1/L2 coverage for connected labels, disconnected labels, missing
      labels, length mismatches, out-of-bounds neighbors, and larger grids.
- [x] Wire `bisect-local-search`, `bisect-clustering`, `bisect-flow`, and
      `bisect-ilp` wrappers through `rgraph-core`.
- [x] Update shared-kernel docs and wave status.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rgraph-core -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p rgraph-core -- --ignored --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-local-search -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-clustering -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-flow -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-ilp -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test --workspace -- --test-threads=1
git diff --check
```
