# X.1 Plan

## Thesis

Precinct and reporting-unit lineage should be a reusable package substrate, not
an incidental section embedded permanently in RPLAN or RCOUNT. RHIST gives both
plans and counts a stable way to reference unit changes across cycles.

## First Slice

- [x] Define `rhist-core` records for cycles, units, lineage events, and
  crosswalk weights.
- [x] Define a minimal RHIST package directory shape.
- [x] Add source-index hashing and package content hash.
- [x] Add verifier checks for split/merge/rename cardinality.
- [x] Add crosswalk weight-sum check.
- [x] Add L1 split/merge synthetic positive package.
- [x] Add one stale/missing-unit negative package.
- [x] Add L1 bad-weight negative package.
- [x] Add minimal `rhist-core` verifier over L0 fixtures.
- [x] Add `rhist-io` package directory loader/writer.
- [x] Add RCOUNT-to-RHIST mapping test.
- [x] Add RHIST canonical package hash implementation.
- [x] Add fixture regeneration command for computed package hashes.
- [x] Add one three-cycle synthetic positive package.
- [x] Add tiny `rhist-cli verify` command.
- [x] Add real-source pressure fixture.
- [x] Add RCOUNT consumer reference to RHIST package hashes.

## Implementation Spec

- `docs/specs/2026-05-13-rhist-implementation.md`
- `docs/specs/reviews/rhist-implementation-r1_roles.md`

RCOUNT embedded lineage now has a bridge path and a package-hash reference path
without moving RHIST ownership into RCOUNT.

L0 fixture directories:

- `docs/fixtures/rhist/l0-rename`
- `docs/fixtures/rhist/l0-missing-unit`
- `docs/fixtures/rhist/l1-split-merge`
- `docs/fixtures/rhist/l1-bad-weights`
- `docs/fixtures/rhist/l2-three-cycle`
- `docs/fixtures/rhist/real-ri-tract-unchanged`

Core crate:

- `crates/rhist-core`
- `crates/rhist-io`
- `crates/rhist-cli`
- `crates/rcount-rhist`
- `crates/rcount-rhist`

## Relationship To RCOUNT

RCOUNT already has reporting-unit lineage records for the current synthetic
multi-election harness. Those records should be treated as RHIST-compatible
seed material. Do not expand RCOUNT into a general history package before this
slice is designed. New RCOUNT packages can also declare `rhist_refs` in
`normalized/rhist-refs.ndjson` to bind unit-lineage, aggregation-crosswalk, or
context-lineage inputs by RHIST package hash.

## Relationship To RPLAN

RPLAN can reference RHIST when comparing plans across cycles or when a plan
assignment uses units that have changed from the count or census source.

## Claim Boundary

RHIST verifies declared lineage relationships and source hashes. It does not
prove that officials chose legally correct boundaries or that every historical
source file is complete.
