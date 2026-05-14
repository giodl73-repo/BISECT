---
wave: rgraph-subset-connectivity-kernel
date_open: 2026-05-14
status: archived
source_goal: docs/specs/2026-05-13-shared-math-graph-kernels.md
---

# RGRAPH Subset Connectivity Kernel

## Mission

Extract repeated adjacency-list node-subset connectivity checks into
`rgraph-core`.

## Claim Boundary

This wave may add graph-only validation and traversal for a caller-provided node
subset. Callers keep empty-subset policy, proposal/repair semantics, pricing
rules, and evidence interpretations.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - Node subset connectivity | DONE | Added `node_subset_connected` and routed production subset-connectivity wrappers |
