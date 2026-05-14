---
wave: rgraph-edge-cut-adapter-kernel
date_open: 2026-05-14
status: archived
source_goal: docs/specs/2026-05-13-shared-math-graph-kernels.md
---

# RGRAPH Edge Cut Adapter Kernel

## Mission

Add a closure-based edge-cut adapter so map/set-shaped assignment callers can
reuse the graph-only cut loop without moving their domain defaults into
`rgraph-core`.

## Claim Boundary

This wave may add a label callback for edge-cut counting. Callers must continue
to own missing-assignment defaults, side-membership definitions, evidence
semantics, and recursive bisection selection policy.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - Closure label edge cuts | DONE | Added `undirected_edge_cut_by` and wired CLI HashMap/HashSet edge-cut helpers |
