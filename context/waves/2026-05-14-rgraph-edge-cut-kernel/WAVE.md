---
wave: rgraph-edge-cut-kernel
date_open: 2026-05-14
status: archived
source_goal: docs/specs/2026-05-13-shared-math-graph-kernels.md
---

# RGRAPH Edge Cut Kernel

## Mission

Extract reusable undirected edge-cut counting into `rgraph-core` and route
existing redistricting consumers through it.

## Claim Boundary

This wave may add graph-only cut counting over adjacency lists and assignment
labels. It must not own compactness interpretation, local-search objectives,
Pareto objectives, replica-exchange criteria, or plan validity semantics.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - Undirected edge-cut count | DONE | Added `rgraph-core::undirected_edge_cut`; wired local-search, Pareto, and PT consumers with L0/L1/L2 coverage |
