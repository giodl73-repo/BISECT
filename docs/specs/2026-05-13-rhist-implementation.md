# RHIST Implementation Spec

**Status:** Implementation-ready package spec draft  
**Date:** 2026-05-13  
**Scope:** Minimal reproducible unit-history package, verifier checks, hash
projections, and fixtures  
**Related specs:** [`2026-05-13-rhist-boundary.md`](2026-05-13-rhist-boundary.md),
[`2026-05-13-rctx-boundary.md`](2026-05-13-rctx-boundary.md),
[`2026-05-13-civic-evidence-layer-access-patterns.md`](2026-05-13-civic-evidence-layer-access-patterns.md)

## Decision

Build RHIST as a small package substrate for cross-cycle unit history. The first
implementation should validate declared lineage and crosswalk evidence; it
should not ingest vote totals, district assignments, rendered map products, or
certification actions.

The first crate slice may be `rhist-core`/`rhist-io`, but the package schema
below is the stable target even if the first validator lives elsewhere.

## Package Layout

```text
package.rhist/
  manifest.json
  sources/
    source-index.json
    <preserved source files>
  contexts/
    context-index.ndjson
  units/
    cycles.ndjson
    lineage-events.ndjson
    crosswalks.ndjson
  proofs/
    package-hashes.json
  claims/
    claim-boundary.json
  transcripts/
    verify-transcript.json
```

Required for L0:

- `manifest.json`
- `sources/source-index.json`
- `contexts/context-index.ndjson`
- `units/cycles.ndjson`
- `units/lineage-events.ndjson`
- `proofs/package-hashes.json`
- `claims/claim-boundary.json`

`units/crosswalks.ndjson` is required when a lineage event declares weighted
coverage or when a consumer needs aggregation across cycles.

## Manifest

```json
{
  "rhist_version": "0.1",
  "package_id": "syn-rhist-l0-rename",
  "jurisdiction": "SYN",
  "cycle_ids": ["syn-2024-general", "syn-2028-general"],
  "producer": "rhist-fixture",
  "created_at": "2026-05-13T00:00:00Z",
  "package_content_hash": "sha256:..."
}
```

Checks:

- `rhist_version == "0.1"` for the first reader;
- `package_id` is non-empty;
- every manifest cycle id appears in `units/cycles.ndjson`;
- `package_content_hash` matches the canonical package projection.

## Source Index

Each source index entry records byte identity for preserved inputs:

```json
{
  "source_id": "syn-2024-precincts.csv",
  "path": "sources/syn-2024-precincts.csv",
  "sha256": "sha256:...",
  "media_type": "text/csv",
  "description": "Synthetic 2024 precinct list"
}
```

Checks:

- `source_id` is unique;
- `path` is package-relative and must not escape the package root;
- `sha256` is a valid hash and matches preserved bytes when the file is
  present;
- every `source_refs` value resolves to a known `source_id`.

## Context Index

RHIST references RCTX-compatible contexts by hash. The context payload may be a
preserved `.rctx`, an external package reference, or a compact fixture context.

```json
{
  "context_id": "syn-2024-precinct-context",
  "context_hash": "sha256:...",
  "rctx_version": "0.1",
  "unit_kind": "precinct",
  "cycle_id": "syn-2024-general",
  "unit_ids": ["syn:precinct:P-001", "syn:precinct:P-002"],
  "source_refs": ["syn-2024-precincts.csv"]
}
```

Checks:

- `context_id` is unique;
- `context_hash` is a valid `sha256:` value;
- `cycle_id` exists in `cycles.ndjson`;
- `unit_ids` are non-empty and unique inside the context;
- if a full `.rctx` is supplied, `unit_ids` must match its unit order.

## Cycles

```json
{
  "cycle_id": "syn-2024-general",
  "jurisdiction": "SYN",
  "cycle_kind": "general-election",
  "effective_date": "2024-11-05",
  "context_id": "syn-2024-precinct-context",
  "context_hash": "sha256:...",
  "source_refs": ["syn-2024-precincts.csv"]
}
```

Allowed `cycle_kind` values:

- `general-election`
- `primary-election`
- `special-election`
- `redistricting-cycle`
- `administrative-snapshot`
- `imported`

Checks:

- `cycle_id` is unique;
- `effective_date` is ISO `YYYY-MM-DD`;
- `context_id` resolves to the context index;
- `context_hash` matches the referenced context index row.

## Lineage Events

```json
{
  "event_id": "lineage:P-004-split",
  "event_kind": "split",
  "from_cycle_id": "syn-2024-general",
  "to_cycle_id": "syn-2028-general",
  "from_unit_ids": ["syn:precinct:P-004"],
  "to_unit_ids": ["syn:precinct:P-004A", "syn:precinct:P-004B"],
  "effective_date": "2028-11-07",
  "authority": "SYN County Election Office",
  "confidence": "official",
  "source_refs": ["syn-2028-precinct-change-notice.txt"],
  "explanation": "P-004 split into two precincts after municipal growth."
}
```

Allowed `event_kind` values:

- `unchanged`
- `create`
- `close`
- `rename`
- `split`
- `merge`
- `boundary-change`
- `administrative-recode`

Allowed `confidence` values:

- `official`
- `derived`
- `manual-review`
- `unknown`

Cardinality rules:

| Event kind | `from_unit_ids` | `to_unit_ids` |
|---|---:|---:|
| `unchanged` | 1 | 1 |
| `rename` | 1 | 1 |
| `administrative-recode` | 1 | 1 |
| `split` | 1 | 2 or more |
| `merge` | 2 or more | 1 |
| `boundary-change` | 1 or more | 1 or more |
| `create` | 0 | 1 or more |
| `close` | 1 or more | 0 |

Checks:

- `event_id` is unique;
- `from_cycle_id` and `to_cycle_id` exist unless omitted for package boundary
  events;
- source and target units exist in their cycle contexts;
- cardinality matches `event_kind`;
- `effective_date` is not earlier than the source cycle effective date;
- source refs resolve.

## Crosswalks

Crosswalk records reuse the RCTX crosswalk shape, with RHIST cycle ids added for
verifier convenience:

```json
{
  "crosswalk_id": "cw-syn-2024-to-2028",
  "from_cycle_id": "syn-2024-general",
  "to_cycle_id": "syn-2028-general",
  "from_context_hash": "sha256:...",
  "to_context_hash": "sha256:...",
  "from_unit_id": "syn:precinct:P-004",
  "to_unit_id": "syn:precinct:P-004A",
  "weight": { "num": 3, "den": 5 },
  "weight_kind": "registered-voters",
  "exhaustive": true,
  "source_refs": ["syn-2028-precinct-change-notice.txt"]
}
```

Allowed `weight_kind` values match RCTX:

- `population`
- `area`
- `ballots`
- `registered-voters`
- `manual`
- `unit-count`

Checks:

- `crosswalk_id` is non-empty;
- cycle ids and context hashes agree with `cycles.ndjson`;
- endpoint unit ids exist;
- rational denominator is positive;
- rational numerator is non-negative;
- duplicate `(crosswalk_id, from_unit_id, to_unit_id, weight_kind)` rows are
  rejected;
- exhaustive rows for each `(crosswalk_id, from_unit_id, weight_kind)` sum
  exactly to `1`;
- non-exhaustive crosswalk groups appear in `claim-boundary.json` caveats.

## Claim Boundary

```json
{
  "package_id": "syn-rhist-l0-rename",
  "proves": [
    "declared lineage records are internally consistent",
    "declared source refs resolve to preserved source hashes"
  ],
  "does_not_prove": [
    "official legal validity of boundary changes",
    "completeness of all historical sources",
    "vote totals or district assignments"
  ],
  "caveats": []
}
```

Checks:

- `package_id` matches the manifest;
- `proves` and `does_not_prove` are non-empty;
- non-exhaustive crosswalk groups have a caveat.

## Package Hash Projection

The L0 package content hash is computed over canonical JSON with this shape:

```json
{
  "rhist_version": "0.1",
  "manifest_without_package_content_hash": {},
  "source_index": [],
  "context_index": [],
  "cycles": [],
  "lineage_events": [],
  "crosswalks": [],
  "claim_boundary": {}
}
```

Use domain-separated prefixes:

```text
RHIST_CYCLE_V1\0
RHIST_LINEAGE_EVENT_V1\0
RHIST_CROSSWALK_V1\0
RHIST_PACKAGE_V1\0
```

The package projection excludes `transcripts/verify-transcript.json` so a
verifier can regenerate transcripts without changing package identity.

## Verifier Check IDs

| Check id | Purpose |
|---|---|
| `manifest_cycle_refs` | manifest cycle ids are present |
| `source_refs_resolve` | source references exist and preserved bytes match |
| `context_cycle_refs` | context rows reference known cycles |
| `context_unit_ids_unique` | each context has unique units |
| `cycle_context_refs` | cycles point to context rows and hashes |
| `lineage_unit_refs` | lineage unit ids exist in source/target contexts |
| `lineage_cardinality` | event cardinality matches event kind |
| `crosswalk_unit_refs` | crosswalk endpoint units exist |
| `crosswalk_weight_sum` | exhaustive rational weights sum to `1` |
| `claim_boundary_present` | package claim limits are explicit |
| `package_content_hash` | canonical package hash matches manifest |

## RCOUNT Migration Mapping

Existing `RcountPackage.lineage` records can map to RHIST as follows:

| RCOUNT field | RHIST field |
|---|---|
| `lineage_id` | `event_id` |
| `kind: Unchanged` | `event_kind: unchanged` |
| `kind: Split` | `event_kind: split` |
| `kind: Merge` | `event_kind: merge` |
| `prior_cycle` | `from_cycle_id` |
| `current_cycle` | `to_cycle_id` |
| `prior_reporting_unit_ids` | `from_unit_ids` |
| `current_reporting_unit_ids` | `to_unit_ids` |
| `authority` | `authority` |
| `explanation` | `explanation` |

Missing in current RCOUNT lineage and required by RHIST:

- source refs;
- confidence;
- effective date;
- per-cycle context references;
- crosswalk weights.

RCOUNT may keep embedded lineage for current fixtures, but new cross-cycle
features should either reference RHIST or use RHIST-compatible records.

## Fixture Ladder

### L0 Positive: Rename

Two cycles, one one-to-one rename, no crosswalk file needed:

```text
syn-2024-general: syn:precinct:P-001
syn-2028-general: syn:precinct:P-001A
event: rename P-001 -> P-001A
```

Expected checks: all pass.

### L0 Negative: Missing Unit

Same as L0 positive, but the lineage event targets
`syn:precinct:P-001B`, which is absent from the 2028 context.

Expected failure: `lineage_unit_refs`.

### L1 Positive: Split And Merge

Two cycles:

```text
P-004 -> P-004A, P-004B
P-007, P-008 -> P-078
```

Crosswalk:

```text
P-004 -> P-004A 3/5
P-004 -> P-004B 2/5
P-007 -> P-078 1/1
P-008 -> P-078 1/1
```

Expected checks: all pass.

### L1 Negative: Bad Weights

Same as L1 positive, but `P-004` weights sum to `6/5`.

Expected failure: `crosswalk_weight_sum`.

### L2 Positive: Public Source

Use a small public jurisdiction source bundle with preserved source bytes,
manual source index, and one documented split/rename/merge. L2 should not be
implemented until source permissions and stable public URLs are confirmed.

## Implementation Gate

Before crate work:

- add L0 fixture directories to `docs/fixtures/rhist/`;
- add one role review over this implementation spec;
- decide whether RHIST reads compact context index rows only or full `.rctx`
  sidecars in L0.

L0 fixture directories now exist:

- `docs/fixtures/rhist/l0-rename`
- `docs/fixtures/rhist/l0-missing-unit`

They use real source-file SHA-256 values and computed RHIST package hashes.

The first `rhist-core` slice validates L0 and L1 fixtures in memory:

- `docs/fixtures/rhist/l0-rename`
- `docs/fixtures/rhist/l0-missing-unit`
- `docs/fixtures/rhist/l1-split-merge`
- `docs/fixtures/rhist/l1-bad-weights`
- `docs/fixtures/rhist/l2-three-cycle`

The first `rhist-io` slice now loads and writes package directories, verifies
preserved source-file hashes, and runs the `rhist-core` verifier over loaded
packages.

The first `rcount-rhist` bridge slice maps existing RCOUNT embedded lineage
events into RHIST-compatible lineage events and verifies the mapped synthetic
split/merge package against `rhist-core`.

RHIST crosswalk verification now delegates the shared exact-rational crosswalk
checks to `rctx-core`, while RHIST keeps ownership of cycle and lineage
semantics. This keeps the RCTX primitive reusable for RCOUNT/RPLAN aggregation
without moving full `.rctx` read/write out of RPLAN yet.

Canonical package hashing is implemented in `rhist-core` with the
`RHIST_PACKAGE_V1\0` domain prefix. `rhist-io` writes and verifies package
hashes, and the checked-in RHIST fixtures are regenerated with computed
package hashes.

The tiny `rhist-cli verify` command now verifies RHIST package directories and
emits JSON or pretty JSON transcripts. RCOUNT now has a consumer reference
surface for RHIST packages: `RcountPackage.rhist_refs` writes
`normalized/rhist-refs.ndjson`, validates `sha256:` package hashes, requires at
least one referenced cycle, and restricts roles to `unit-lineage`,
`aggregation-crosswalk`, or `context-lineage`. This lets RCOUNT bind to RHIST
cross-cycle evidence without growing independent history semantics.

The first real-source pressure fixture is
`docs/fixtures/rhist/real-ri-tract-unchanged`. It preserves narrow
Census-derived Rhode Island tract source slices for GEOID `44001030601` across
2000, 2010, and 2020. It proves source preservation and unchanged-lineage
verification only, not full Census tract relationship coverage.
