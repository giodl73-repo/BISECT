---
pulse: 01
title: Closure label edge cuts
status: done
wave: rgraph-edge-cut-adapter-kernel
---

# Pulse 01 - Closure Label Edge Cuts

## Purpose

Provide a graph-only edge-cut loop for callers whose labels come from maps,
sets, or other domain-owned assignment views.

## Pre-implementation Scout

```powershell
rg "count_edge_cuts\\(|fn edge_cut\\(adjacency: \\&\\[Vec<usize>\\], left" crates/bisect-cli/src -n
```

## Deliverables

- [x] Add `rgraph-core::undirected_edge_cut_by` accepting a node-label closure.
- [x] Keep slice-based `undirected_edge_cut` as the main assignment-slice helper.
- [x] Wire CLI bisection-runner HashMap and SA-evidence HashSet helpers through
      the closure adapter while keeping defaults local.
- [x] Update shared-kernel docs and wave status.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rgraph-core edge_cut -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-cli edge_cut --lib -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-cli sa_evidence --lib -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test --workspace -- --test-threads=1
git diff --check
```
