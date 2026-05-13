---
wave: rust-port-and-rplan-specs
date_open: 2026-04-24
date_close: 2026-04-26
status: archived
backfill: true
confidence: high
---

# Rust Port And RPLAN Specs

## Mission

Move the project toward a Rust CLI and formal package/spec surface: Rust
workspace, PyO3 bridge, redistricting core/data/map/report crates, RPLAN specs,
state policy, and practitioner toolkit documents.

## Evidence

Representative commits:

- `96b78e7a` Add Rust CLI port plan
- `4a7eacfa` Phase 0: Rust workspace scaffold + PyO3 bindings
- `6b045ecd` Phase 1a: VRA formula to Rust
- `53067afe` Phase 1b: bisection tree in Rust
- `07c56d14` Phase 2a: TIGER tract shapefile reader in Rust
- `00001f72` Phase 3d complete: redist state VT+AL pass
- `28a2dddd` Phase 5: redist-map crate + analytics + analyze/map CLI
- `d94b52bf` Spec 0: RPLAN
- `e15985b2` Implement Spec 0+1: RPLAN format and custom parameters
- `66f62775` Implement Specs 2-6

## Tracks

- Rust CLI and crates.
- RPLAN package/specs.
- Practitioner toolkit.
- State/location policy.
- Testing and invariants.

## Established

- Rust became the production direction.
- RPLAN became the named redistricting plan package.
- The project started treating outputs as verifiable artifacts, not only maps.

## Carry Forward

RCOUNT/RCTX/RHIST later generalized this package thinking beyond redistricting
plans.

