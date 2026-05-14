---
wave: rgraph-mask-edge-cut-consumer
date_open: 2026-05-14
status: archived
source_goal: docs/specs/2026-05-13-shared-math-graph-kernels.md
---

# RGRAPH Mask Edge Cut Consumer

## Mission

Route bitmask-based subset edge-cut counting through the existing
`rgraph-core::undirected_edge_cut_by` adapter.

## Claim Boundary

This wave may reuse the graph-only cut traversal for bitmask membership labels.
It must not move column-generation pricing policy, feasible-subset enumeration,
or weighted cut scoring into `rgraph-core`.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - Bitmask edge-cut consumer | DONE | Wired `bisect-column::pricing::subset_edge_cut` through `undirected_edge_cut_by` |
