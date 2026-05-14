---
pulse: 01
title: Node subset connectivity
status: done
wave: rgraph-subset-connectivity-kernel
---

# Pulse 01 - Node Subset Connectivity

## Purpose

Move repeated "does this node subset induce one connected component?" traversal
into `rgraph-core`.

## Pre-implementation Scout

```powershell
rg "fn is_connected|subset_connected|is_connected_subset|visited\\.len\\(\\) ==" crates -n -g "*.rs"
```

## Deliverables

- [x] Add `rgraph-core::node_subset_connected` with typed validation errors.
- [x] Add L0/L1/L2 coverage for connected subsets, disconnected subsets,
      singleton/empty subsets, duplicate inputs, out-of-bounds nodes, and
      out-of-bounds neighbors.
- [x] Wire `bisect-column`, `bisect-smc::proposal`, `bisect-pareto::mutation`,
      CLI bisection subset checks, and SA evidence checks through `rgraph-core`.
- [x] Update shared-kernel docs and wave status.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rgraph-core subset -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-column -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-smc connected -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-pareto is_connected_subset -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-cli connected --lib -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test --workspace -- --test-threads=1
git diff --check
```
