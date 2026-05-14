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
| 03 - Articulation point kernel | DONE | `rgraph-core` undirected articulation helpers over directed adapter; L0/L1/L2 tests |
| 04 - Close and validation | DONE | full `rgraph-core` L0/L1/L2 ladder; focused BISECT contiguity consumer suite; `git diff --check` |

## Close Summary

`rgraph-core` now owns deterministic connected-component helpers over all graph
nodes or a caller-provided node subset. `bisect-analysis::contiguity` delegates
component discovery to that shared helper while keeping GEOID, district, and
county-context semantics local.

The wave also adds deterministic undirected bridge detection over the existing
directed graph adapter. Bridge detection is kept kernel-only for now so ROUTE
redundancy analysis and future BISECT contiguity-fragility diagnostics can adopt
the same primitive without moving domain scoring into `rgraph-core`.

Articulation point detection now shares the same undirected projection boundary
as bridge detection and remains kernel-only until a domain crate needs to label
network or district fragility.

## Deferred

- ROUTE consumption remains deferred until the shared crates have a portable
  git/local dependency plan.
- BISECT contiguity-fragility reporting remains a domain-layer follow-up; the
  generic bridge/articulation primitives are ready but intentionally unlabeled.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rgraph-core -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p rgraph-core -- --ignored --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-analysis contiguity -- --test-threads=1
git diff --check
```

