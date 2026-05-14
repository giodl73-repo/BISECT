---
wave: shared-kernel-crates
date_open: 2026-05-13
status: complete
source_goal: docs/specs/2026-05-13-shared-math-graph-kernels.md
---

# Shared Kernel Crates

## Mission

Create reusable pure-Rust algorithm kernels that BISECT, ROUTE, and future civic
evidence projects can share without importing domain models or R/Python runtime
dependencies.

## Claim Boundary

This wave may create generic graph/statistical crates and migrate narrow
consumer code. It must not move redistricting legality, route scoring, election
audit semantics, package hashing, or civic claim interpretation into the shared
crates.

## Inputs

| Input | Source |
|---|---|
| Active spec | `docs/specs/2026-05-13-shared-math-graph-kernels.md` |
| Role review | `docs/specs/reviews/shared-math-graph-kernels-r1_roles.md` |
| ROUTE candidate code | `C:\src\route\crates\route-network\src\centrality.rs` |
| ROUTE connectivity candidate | `C:\src\route\crates\route-network\src\connectivity.rs` |

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - RGRAPH core skeleton | DONE | `pulses/01+rgraph-core-skeleton.md`; `rgraph-core` shortest-path and reachability tests; L0/L1/L2 coverage |
| 02 - Brandes centrality compatibility | DONE | `pulses/02+brandes-centrality-compatibility.md`; ROUTE synthetic centrality tests ported to `rgraph-core`; L1/L2 centrality coverage |
| 03 - BISECT graph consumer | DONE | `pulses/03+bisect-graph-consumer.md`; `bisect-analysis::contiguity` uses `rgraph-core` reachability |
| 04 - ROUTE adapter decision | DONE | `pulses/04+route-adapter-decision.md`; cross-repo adapter deferred until dependency location is stable |
| 05 - Wave close and extraction decision | DONE | `pulses/05+close-extraction-decision.md`; `rgraph-core` remains incubated here pending ROUTE consumer |

## Validation Gate

Run after each implementation pulse:

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rgraph-core -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p rgraph-core -- --ignored --test-threads=1
git diff --check
```

When a consumer crate changes, add its focused package test to the same gate.

## Close Summary

The wave created the first shared kernel crate, `rgraph-core`, with deterministic
shortest-path, filtered reachability, and weighted Brandes edge-betweenness
primitives. `bisect-analysis::contiguity` now consumes `rgraph-core` for
component traversal. ROUTE integration is deliberately deferred until the shared
dependency location is stable enough for `C:\src\route` to consume without a
repository-local path leak.

`rstat-core` remains a later wave. `rgraph-core` should stay incubated in this
workspace until a ROUTE adapter or standalone shared repository provides the
second real consumer.

The `rgraph-core` test ladder includes inline L0 tests, L1 integration tests in
`crates/rgraph-core/tests/l1_paths_centrality.rs`, and ignored L2 graph stress
tests in `crates/rgraph-core/tests/l2_graph_stress.rs`.

