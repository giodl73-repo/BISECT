---
reviewer: ROLE PANEL
roles: BOUNDARY, WARD, COVENANT, CONTOUR, MERIDIAN, BENCHMARK, SCALE, PRECINCT, DATUM, COMMONS, LEDGER, SURVEY, TRENCH
spec: RPLAN v0.2 Schema
round: 1
date: 2026-05-10
score: 3.0
---

# Role Review: RPLAN v0.2 Schema

## Summary

RPLAN v0.2 is the right fixed point for crate work: `.rplan` is the plan
assignment artifact, `.rctx` is context, `rplan-io` owns v0.1 compatibility,
and `rplan-core` owns generic plan identity without depending on bisect.

The schema is close, but not implementation-ready. The main remaining issue is
identity: `display_labels` are described as human labels but currently
participate in the canonical plan hash. That makes a cosmetic relabeling change
the plan identity. The second issue is that `.rctx` is reserved but not yet
specified enough for `rplan-core`/`rplan-io` tests.

Decision: **revise before crate implementation**.

## Scores

| Role | Score | Reason |
|------|-------|--------|
| BOUNDARY | 3.5/4 | Cleanly separates `.rplan` from audit/context and keeps bisect out of `rplan-*`. |
| WARD | 3/4 | Legal rules are kept out of the base file, which is correct. |
| COVENANT | 2.5/4 | Canonical hashing exists, but label semantics need correction. |
| CONTOUR | 3/4 | Graph context is correctly separated, but `.rctx` is underspecified. |
| MERIDIAN | 3/4 | Unit graph model is plausible for contiguity. |
| BENCHMARK | 2.5/4 | Acceptance criteria are good but need concrete test vectors. |
| SCALE | 3.5/4 | Keeping adjacency out of `.rplan` avoids large plan files. |
| PRECINCT | 3/4 | Adds precinct/imported units, but unit-id rules need per-kind validation. |
| DATUM | 3/4 | Source hashes are present; unit-universe hash needs normative construction. |
| COMMONS | 3.5/4 | Strong public interchange story. |
| LEDGER | 2.5/4 | Hash projection must stop mixing identity with presentation. |
| SURVEY | 3/4 | CLI/tool story is clear enough for users. |
| TRENCH | 3/4 | Migration path is feasible. |
| **Average** | **3.0/4** | Revise, then implementation can start. |

## Required Fixes

### R1-A: Remove `display_labels` from plan identity or redefine it

The spec says external labels belong in `display_labels`, but also says changing
`display_labels` changes the plan hash. If labels are display-only, this is the
wrong identity rule.

Choose one:

- **Recommended:** exclude `display_labels` from `plan_hash`; include it in a
  separate `document_hash` or `presentation_hash`.
- Alternative: rename it to `district_id_map` and state that labels are
  semantically binding.

For audit and plan equivalence, the recommended rule is:

```text
plan_hash = units + assignment + k + unit_universe_hash
presentation_hash = display_labels + metadata
```

### R1-B: Move `allow_empty` out of metadata

The assignment rules depend on `metadata.allow_empty`. Metadata is described as
human-facing and should not control structural validity.

Move this to one of:

- `plan.allow_empty_districts`, if it is a file-level structural rule
- `LegalProfile`, if empty districts are only meaningful during audit

Default should be `false`.

### R1-C: Specify `unit_universe_hash`

The schema requires `unit_universe_hash` but does not define how to compute it.
Add the canonical projection, including:

- `unit_kind`
- `unit_ids`
- `canonical_order`
- optional `state`
- optional `year`
- source/vintage id when known

The v0.1 compatibility rule that computes the hash from sorted GEOIDs must use
the same projection.

### R1-D: Specify `.rctx` enough for tests

The `.rctx` section is currently a placeholder. Before crate work, define:

- top-level required fields
- `UnitGraph` JSON shape
- `EdgeKind` allowed values
- whether adjacency is directed or undirected on disk
- whether duplicate edges are legal
- population vector type and unit alignment rule
- context hash

At minimum, `rplan-io` should have a fixture that round-trips a 5-node path
graph context.

### R1-E: Define per-`unit_kind` unit-id validation

`sorted-geoid` says unit ids must be numeric, but v0.2 supports block,
block-group, tract, county, precinct, and imported units.

Add validation rules:

- `tract`: 11 digits
- `block-group`: 12 digits
- `block`: 15 digits
- `county`: 5 digits
- `precinct`: non-empty string, preferably namespaced
- `imported`: non-empty string, no Census format assumption

### R1-F: Add concrete test vectors

Add small canonical examples:

- v0.2 two-district plan hash fixture
- same plan with reordered JSON keys has same hash
- same assignments with different metadata has same `plan_hash`
- same assignments with different `display_labels` has same `plan_hash` if
  R1-A follows the recommended path
- v0.1 conversion fixture with expected v0.2 internal assignment

## Non-Blocking Notes

- Strict RFC 7946 geometry is the right default.
- v0.1 should remain read-only or compatibility-write only.
- The spec should say whether top-level `extensions` participate in any future
  document hash; they should not affect `plan_hash`.
