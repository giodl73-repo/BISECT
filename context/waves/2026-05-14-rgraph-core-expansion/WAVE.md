---
wave: rgraph-core-expansion
date_open: 2026-05-14
status: complete
source_goal: docs/specs/2026-05-13-shared-math-graph-kernels.md
---

# RGRAPH Core Expansion

## Mission

Extend `rgraph-core` with small generic graph helpers only when BISECT has an
immediate consumer and the helper remains portable to ROUTE.

## Claim Boundary

This wave may add graph-only algorithms and adapt BISECT graph consumers. It
must not move redistricting legality, district records, route scoring, or package
semantics into `rgraph-core`.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - Connected components kernel | DONE | `rgraph-core` components helpers; BISECT contiguity consumer; L0/L1/L2 tests |
| 02 - Bridge detection kernel | DONE | `rgraph-core` undirected bridge helpers over directed adapter; L0/L1/L2 tests |

## Close Summary

`rgraph-core` now owns deterministic connected-component helpers over all graph
nodes or a caller-provided node subset. `bisect-analysis::contiguity` delegates
component discovery to that shared helper while keeping GEOID, district, and
county-context semantics local.

The wave also adds deterministic undirected bridge detection over the existing
directed graph adapter. Bridge detection is kept kernel-only for now so ROUTE
redundancy analysis and future BISECT contiguity-fragility diagnostics can adopt
the same primitive without moving domain scoring into `rgraph-core`.

