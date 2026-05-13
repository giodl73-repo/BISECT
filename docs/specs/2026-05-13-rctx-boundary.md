# RCTX Boundary

**Status:** Phase 0 boundary spec  
**Date:** 2026-05-13  
**Scope:** Shared reproducible machine context for RPLAN, RCOUNT, RHIST, RMAP,
and later civic evidence packages  
**Related specs:** [`2026-05-13-civic-evidence-package-family.md`](2026-05-13-civic-evidence-package-family.md),
[`2026-05-10-rplan-v0.2-schema.md`](2026-05-10-rplan-v0.2-schema.md),
[`2026-05-12-rcount-incubation.md`](2026-05-12-rcount-incubation.md)

## Decision

RCTX is the shared machine-context package concept for canonical unit identity,
unit order, graph identity, source hashes, aligned attributes, and crosswalks.

The current implemented `.rctx` artifact remains owned by `rplan-core` and
`rplan-io` for phase 1. It is treated as RCTX-compatible, not obsolete. Do not
split a new `rctx-core` crate until a second production consumer needs direct
context construction or validation.

Immediate rule:

```text
RPLAN/RCOUNT/RHIST/RMAP may depend on RCTX concepts.
RCTX must not depend on RPLAN/RCOUNT/RHIST/RMAP.
```

## Existing Implementation

The current `rplan_core::RplanContext` is the as-built RCTX v0.1 shape:

| RCTX concept | Existing field |
|---|---|
| schema version | `rctx_version` |
| context digest | `context_hash` |
| canonical unit universe | `units: PlanUnitIndex` |
| adjacency graph | `graph: Option<UnitGraph>` |
| aligned population vector | `populations: Option<Vec<i64>>` |
| aligned subdivision ids | `subdivisions: Option<SubdivisionContext>` |
| aligned demographic vectors | `demographics: Option<DemographicContext>` |
| geometry source references | `geometry: Option<GeometryContext>` |
| source registry | `source_hashes: SourceHashes` |

The current implementation already validates:

- unit ids are valid, unique, and sorted when `canonical_order` is
  `sorted-geoid`;
- graph adjacency length matches unit count;
- graph edge targets are valid;
- duplicate graph edges are rejected;
- undirected graph edges are symmetric;
- graph weights are finite and non-negative;
- population, subdivision, demographic, and geometry vectors align to unit
  count;
- geometry hashes are `sha256:` values.

## RCTX Owns

RCTX owns the following shared concepts:

- stable unit ids and unit id namespace;
- unit kind, state, year, source id, and canonical order;
- `unit_universe_hash`;
- context hash;
- graph identity and edge semantics used by verifiers;
- aligned non-domain attribute arrays needed by more than one package;
- source hashes for unit, graph, population, demographic, subdivision, and
  geometry inputs;
- crosswalks between unit universes.

RCTX does not own plan assignments, vote totals, ballot records, audit samples,
lineage events, certification claims, court records, or rendered maps.

## RMAP Boundary

RCTX may carry geometry identity, but not cartographic presentation.

Allowed in RCTX:

- geometry source id;
- geometry source hash;
- coordinate reference system metadata when needed to interpret the source;
- per-unit geometry hashes aligned to `unit_ids`.

Owned by RMAP:

- rendered maps;
- styling, symbols, colors, and labels;
- layer composition;
- projection choices for display;
- raster/vector tile products;
- public map image hashes.

RMAP may reference an RCTX `context_hash`. RCTX must be useful without RMAP.

## Crosswalk Records

Crosswalks are the missing RCTX primitive needed by RCOUNT, RHIST, and RMAP.
They should be added before deeper district aggregation or multi-cycle count
work.

Minimal record shape:

```json
{
  "crosswalk_id": "cw-ri-2024-precinct-to-district",
  "from_context_hash": "sha256:...",
  "to_context_hash": "sha256:...",
  "from_unit_id": "PVD-0001",
  "to_unit_id": "HD-28",
  "weight": { "num": 1, "den": 1 },
  "weight_kind": "ballots",
  "exhaustive": true,
  "source_refs": ["ri-2024-precinct-district-crosswalk.csv"]
}
```

Allowed `weight_kind` values:

- `population`
- `area`
- `ballots`
- `registered-voters`
- `manual`
- `unit-count`

Use exact rational weights for package identity. CLI display may show decimals,
but verifier logic should not depend on floating-point equality.

## Crosswalk Checks

Minimal verification:

- `crosswalk_id` is non-empty;
- `from_context_hash` and `to_context_hash` are valid `sha256:` values;
- endpoint unit ids exist in the declared contexts when those contexts are
  supplied;
- weights are finite, non-negative rationals with non-zero denominators;
- duplicate `(crosswalk_id, from_unit_id, to_unit_id, weight_kind)` records are
  rejected;
- when a group is marked `exhaustive`, weights for each
  `(crosswalk_id, from_unit_id, weight_kind)` sum exactly to `1`;
- `source_refs` resolve to known source-hash entries.

Non-exhaustive crosswalks are allowed, but any verifier using them must emit a
claim caveat.

## Hash Domains

Future extraction should use explicit hash domain prefixes so RCTX hashes cannot
collide semantically with plan, count, or history hashes:

```text
RCTX_UNIT_UNIVERSE_V1\0
RCTX_CONTEXT_V1\0
RCTX_CROSSWALK_V1\0
```

The existing phase-1 `.rctx` canonical JSON hash remains valid for backwards
compatibility. The first `rctx-core` extraction adds the
`RCTX_CROSSWALK_V1\0` hash domain for shared crosswalk records only; full
context hashing remains with the existing `.rctx` path until a second
full-context producer needs it.

## Build Plan

1. Keep `.rctx` read/write and validation in `rplan-core`/`rplan-io`.
2. Add this spec as the shared boundary before RHIST crate work.
3. Add `rctx-core` for crosswalk structs and verifier checks because RHIST and
   RCOUNT now both need the shared primitive.
4. Extract full context construction into `rctx-core` only after a second
   production crate needs to construct or validate contexts directly.
5. Make RMAP reference RCTX rather than duplicating unit identity.

## RCOUNT Guidance

RCOUNT should not grow a permanent `.rctxc` context dialect. Count packages may
embed count-specific reporting-unit metadata, but shared unit universes and
crosswalks should conform to this RCTX boundary.

RCOUNT package directories can bind RCTX inputs through
`normalized/rctx-refs.ndjson`. The RCOUNT verifier checks reference id
uniqueness, `sha256:` context hashes, optional `sha256:` crosswalk hashes, and
the declared role. It does not re-own crosswalk row semantics; those stay in
`rctx-core`.

The district aggregation CLI can take an explicit RCTX crosswalk NDJSON file.
The bridge validates that file with `rctx-core`, computes the canonical
crosswalk-set hash, and rejects the aggregation if the computed hash differs
from the declared RCOUNT `rctx_refs` crosswalk hash.

District aggregation should eventually consume:

```text
RCOUNT reporting units
RPLAN assignment
RCTX context hash
RCTX crosswalk records
```

The aggregation claim should bind all four inputs by hash.

## RHIST Guidance

RHIST owns lineage events over time. RCTX owns the unit universes those events
connect.

RHIST should reference RCTX contexts for each cycle, then provide lineage and
crosswalk records between them. This avoids making RHIST the owner of every
unit attribute and avoids making RCOUNT the owner of long-run precinct identity.

## Open Questions

- Resolved for the first slice: crosswalk implementation lives in
  `crates/rctx-core`; full `.rctx` remains in RPLAN-owned crates.
- Should `source_refs` use source ids only, source ids plus hash expectations,
  or package-relative paths?
- Do we need embedded geometry support, or should RCTX remain hash/reference
  only and leave geometry payloads to RMAP or source bundles?
- Which package owns public display of crosswalk uncertainty: RMAP, RSTAT, or
  the consuming domain package?
