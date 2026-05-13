# X.2 RCTX/RMAP Boundary

Goal: make RCTX a shared machine-context contract without disrupting the
existing RPLAN `.rctx` implementation.

## Status

- [x] Document RCTX as the owner of unit identity, context hashes, graph
  identity, source hashes, aligned attributes, and crosswalks.
- [x] Document RMAP as the owner of rendered/cartographic presentation.
- [x] Map the existing `rplan_core::RplanContext` fields to the shared RCTX
  boundary.
- [x] Record the decision not to split `rctx-core` until there is a second
  production consumer.
- [x] Add crosswalk structs and verifier checks when RCOUNT or RHIST needs
  them for a real workflow.
- [x] Decide whether the first crosswalk implementation belongs in
  `rplan-core`, `rcount-core`, or a new `rctx-core`.

## Specification

Primary spec:

- `docs/specs/2026-05-13-rctx-boundary.md`

Related specs:

- `docs/specs/2026-05-13-civic-evidence-package-family.md`
- `docs/specs/2026-05-10-rplan-v0.2-schema.md`
- `docs/specs/2026-05-12-rcount-incubation.md`

## Implementation Rule

Keep current `.rctx` read/write and validation in `rplan-core` and `rplan-io`
for now. Treat that code as RCTX-compatible phase-1 infrastructure.

The first shared extraction is `crates/rctx-core`, limited to exact rational
crosswalk records, source refs, context unit indexes, and verifier checks.
RHIST delegates crosswalk verification to this shared primitive. Do not move
full `.rctx` read/write out of RPLAN until there is direct pressure from a
second full-context producer.

RCOUNT now has a consumer reference path in `normalized/rctx-refs.ndjson` for
context hashes and optional crosswalk hashes. This is a binding layer, not a
second crosswalk verifier.

## Next Slice

Build RHIST and RCOUNT consumers on top of this boundary:

- cycle contexts reference RCTX-compatible unit universes;
- lineage events connect units across cycles;
- crosswalk records use `rctx-core` exact rational checks;
- RCOUNT can consume RHIST without owning long-run precinct history.
