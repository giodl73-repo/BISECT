---
wave: shared-kernel-crates
pulse: 04
status: done
depends_on: [02]
governing_roles:
  - ROUTE-TRAFFIC-ENGINEER
  - ROUTE-SCOPE-KEEPER
  - ROUTE-NUMERACY-CHECKER
  - LEDGER
---

# Pulse 04 - ROUTE Adapter Decision

## Mission

Decide and document the safest ROUTE integration path: direct path dependency,
standalone shared repository, or temporary compatibility wrapper.

## Deliverables

- [ ] Test a `HighwayGraph` adapter against `rgraph-core` if cross-repo edits are
  acceptable.
- [x] Otherwise record the exact blocker and required follow-up.
- [x] Preserve route-specific units, capacities, and scoring outside
  `rgraph-core`.

## Completion Notes

- Direct ROUTE modification is deferred because `rgraph-core` currently lives in
  the apportionment workspace. Adding a `C:\src\apportionment` path dependency
  inside `C:\src\route` would make ROUTE non-portable.
- Follow-up: once `rgraph-core` either moves to a shared repository or ROUTE
  explicitly accepts a local path dependency for development, adapt
  `HighwayGraph` and replace local Dijkstra/Brandes routines behind compatibility
  tests.

