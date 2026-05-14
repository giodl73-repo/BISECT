---
wave: rgraph-label-connectivity-kernel
date_open: 2026-05-14
status: archived
source_goal: docs/specs/2026-05-13-shared-math-graph-kernels.md
---

# RGRAPH Label Connectivity Kernel

## Mission

Extract duplicated assignment-label connectivity checks into `rgraph-core` and
route existing BISECT consumers through the shared graph kernel.

## Claim Boundary

This wave may add graph-only checks that a label's induced subgraph is
connected. It must not own population balance, repair policy, local-search
acceptance, clustering semantics, flow construction, or ILP branch ordering.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - Assignment label connectivity | DONE | Added `rgraph-core` label connectivity helpers and wired local-search, clustering, flow, and ILP consumers |
