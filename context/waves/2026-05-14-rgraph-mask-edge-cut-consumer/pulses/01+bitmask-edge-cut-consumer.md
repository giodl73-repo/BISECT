---
pulse: 01
title: Bitmask edge-cut consumer
status: done
wave: rgraph-mask-edge-cut-consumer
---

# Pulse 01 - Bitmask Edge-Cut Consumer

## Purpose

Use the existing closure-based edge-cut adapter for column-pricing bitmask
membership labels.

## Pre-implementation Scout

```powershell
rg "subset_edge_cut|mask &|left_in|right_in" crates/bisect-column -n
```

## Deliverables

- [x] Route `bisect-column::pricing::subset_edge_cut` through
      `rgraph_core::undirected_edge_cut_by`.
- [x] Do not add a new shared API for bitmasks.
- [x] Leave weighted cut scoring loops local until a weighted-edge-cut boundary
      is explicitly justified.
- [x] Update shared-kernel docs and wave status.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-column -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test --workspace -- --test-threads=1
git diff --check
```
