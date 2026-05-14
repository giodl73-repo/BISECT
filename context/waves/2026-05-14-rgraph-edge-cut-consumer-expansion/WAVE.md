---
wave: rgraph-edge-cut-consumer-expansion
date_open: 2026-05-14
status: archived
source_goal: docs/specs/2026-05-13-shared-math-graph-kernels.md
---

# RGRAPH Edge Cut Consumer Expansion

## Mission

Route remaining slice-based edge-cut consumers through `rgraph-core` after the
initial edge-cut kernel landed.

## Claim Boundary

This wave may replace duplicated `&[Vec<usize>]` plus slice-assignment edge-cut
loops. It must not alter HashMap/HashSet-shaped domain helpers, recursive
bisection orchestration, clustering objectives, flow repair semantics, or
spectral split behavior.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - Slice consumer expansion | DONE | Wired apportion spectral, clustering, flow, and CLI zero-based spectral helper |
