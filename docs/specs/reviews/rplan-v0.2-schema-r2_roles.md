---
reviewer: ROLE PANEL
roles: BOUNDARY, WARD, COVENANT, CONTOUR, MERIDIAN, BENCHMARK, SCALE, PRECINCT, DATUM, COMMONS, LEDGER, SURVEY, TRENCH
spec: RPLAN v0.2 Schema
round: 2
date: 2026-05-10
score: 3.6
---

# Round 2 Role Review: RPLAN v0.2 Schema

## Summary

Round 1 blockers are resolved. The schema now separates plan identity from
presentation, moves empty-district behavior out of metadata, defines
`unit_universe_hash`, specifies per-unit validation, and gives `.rctx` enough
structure for crate fixtures.

Decision: **approved for `rplan-core` and `rplan-io` phase 1 implementation**.

## Scores

| Role | Score | Reason |
|------|-------|--------|
| BOUNDARY | 3.5/4 | `.rplan` remains assignment-only; `.rctx` owns context. |
| WARD | 3.5/4 | Legal/audit claims stay outside the base file. |
| COVENANT | 3.5/4 | `plan_hash` now excludes presentation fields and has stable rules. |
| CONTOUR | 3.5/4 | `.rctx` graph semantics are testable for phase 1. |
| MERIDIAN | 3.5/4 | Undirected symmetric adjacency is clear enough for contiguity. |
| BENCHMARK | 3.5/4 | Acceptance criteria now produce deterministic fixture tests. |
| SCALE | 4/4 | Context separation avoids large plan artifacts. |
| PRECINCT | 3.5/4 | Multi-unit support and validation are explicit. |
| DATUM | 3.5/4 | Unit-universe and context hashes are defined. |
| COMMONS | 4/4 | Public interchange story is coherent and not bisect-bound. |
| LEDGER | 3.5/4 | Hash projections are suitable for audit logging. |
| SURVEY | 3.5/4 | v0.1 migration rules are operational. |
| TRENCH | 3.5/4 | Phase 1 is implementable without dragging in report code. |
| **Average** | **3.6/4** | Approved for phase 1. |

## Resolved Since Round 1

### R2-A: Display labels no longer define plan identity

`display_labels` are explicitly presentation fields and do not participate in
`plan_hash`.

### R2-B: Empty-district behavior moved into the plan

`allow_empty_districts` now belongs to the plan object, not metadata.

### R2-C: Unit-universe hashing is specified

The schema now defines the canonical projection for `unit_universe_hash`,
including v0.1 compatibility construction.

### R2-D: `.rctx` is concrete enough for phase 1

The context artifact now has required fields, graph JSON shape, allowed edge
kinds, undirected adjacency rules, duplicate-edge behavior, population
alignment, and `context_hash`.

## Phase 1 Implementation Scope

Approved implementation surface:

- `rplan-core::DistrictPlan`
- `rplan-core::PlanUnitIndex`
- `rplan-core::UnitGraph`
- `rplan-core::RplanContext`
- canonical JSON and hash helpers
- `rplan-io` v0.2 read/write
- `rplan-io` v0.1 read/convert compatibility
- `.rctx` read/write fixture support

Not approved yet:

- broad external adapter support
- geometry-heavy conversion
- standalone repository extraction
