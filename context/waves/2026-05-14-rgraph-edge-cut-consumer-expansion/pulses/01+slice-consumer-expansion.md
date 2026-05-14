---
pulse: 01
title: Slice consumer expansion
status: done
wave: rgraph-edge-cut-consumer-expansion
---

# Pulse 01 - Slice Consumer Expansion

## Purpose

Replace remaining duplicated slice-based edge-cut loops with
`rgraph-core::undirected_edge_cut`.

## Pre-implementation Scout

```powershell
rg "fn edge_cut|count_edge_cuts|neighbor > node|assignment\\[node\\] != assignment\\[neighbor\\]" crates -n -g "*.rs"
```

## Deliverables

- [x] Add `rgraph-core` dependencies where consumers need the shared helper.
- [x] Wire `bisect-apportion::spectral`, `bisect-clustering::metrics`,
      `bisect-flow`, and CLI zero-based spectral summary edge cuts through
      `rgraph-core`.
- [x] Leave HashMap/HashSet-shaped edge-cut helpers local until a dedicated API
      exists for those assignment forms.
- [x] Update shared-kernel docs and wave status.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-apportion spectral -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-clustering -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-flow -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-cli spectral --lib -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test --workspace -- --test-threads=1
git diff --check
```
