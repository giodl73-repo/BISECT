# RHIST Boundary

**Status:** Phase 0 package spec  
**Date:** 2026-05-13  
**Scope:** Reproducible unit history and lineage package  
**Related specs:** [`2026-05-13-civic-evidence-package-family.md`](2026-05-13-civic-evidence-package-family.md),
[`2026-05-13-rctx-boundary.md`](2026-05-13-rctx-boundary.md),
[`2026-05-13-rhist-implementation.md`](2026-05-13-rhist-implementation.md)

## Decision

RHIST owns unit lineage across time. It should be the first new base package
after the RCTX boundary because it prevents RCOUNT and RPLAN from each growing
their own incompatible precinct/history model.

The implementation-ready package shape is specified in
[`2026-05-13-rhist-implementation.md`](2026-05-13-rhist-implementation.md).

## Owns

- cycle ids and cycle metadata;
- unit identities as references to RCTX-compatible contexts;
- split, merge, rename, closure, creation, and boundary-change events;
- cross-cycle crosswalks and evidence quality;
- source hashes for historical unit files;
- lineage verification transcripts.

RHIST does not own vote totals, plan assignments, audit samples, rendered maps,
or certification claims.

## Minimal Package Shape

```text
manifest.json
sources/source-index.json
contexts/context-index.ndjson
units/cycles.ndjson
units/lineage-events.ndjson
units/crosswalks.ndjson
proofs/package-hashes.json
claims/claim-boundary.json
transcripts/verify-transcript.json
```

## Records

Cycle record:

```json
{
  "cycle_id": "ri-2024-general",
  "jurisdiction": "RI",
  "effective_date": "2024-11-05",
  "context_hash": "sha256:...",
  "source_refs": ["ri-2024-precincts.csv"]
}
```

Lineage event record:

```json
{
  "event_id": "evt-2022-2024-pvd-17-split",
  "event_kind": "split",
  "from_cycle_id": "ri-2022-general",
  "to_cycle_id": "ri-2024-general",
  "from_unit_ids": ["PVD-17"],
  "to_unit_ids": ["PVD-17A", "PVD-17B"],
  "effective_date": "2024-01-15",
  "source_refs": ["pvd-board-minutes-2024-01-15.pdf"],
  "confidence": "official"
}
```

Allowed event kinds:

- `create`
- `close`
- `rename`
- `split`
- `merge`
- `boundary-change`
- `administrative-recode`

Allowed confidence values:

- `official`
- `derived`
- `manual-review`
- `unknown`

## Checks

- every cycle id is unique;
- every cycle references a valid RCTX context hash;
- every lineage event id is unique;
- event unit ids exist in the referenced cycle contexts when contexts are
  supplied;
- split events have one prior unit and multiple current units;
- merge events have multiple prior units and one current unit;
- rename and administrative-recode events preserve one-to-one lineage;
- closed units do not appear as current units in a later cycle without a create
  or reopen-style event;
- exhaustive crosswalk weights sum exactly to `1` for each prior unit;
- all `source_refs` resolve to source-index entries.

## Access Patterns

RCOUNT reads RHIST when it needs cross-election comparison, historical turnout
series, or unit-normalized district aggregation. RPLAN reads RHIST only for
historical plan comparison or when explaining unit changes across plan cycles.

RHIST reads RCTX contexts. It does not read RCOUNT or RPLAN.

## Fixture Ladder

- **L0 positive:** two cycles, one rename, one exact one-to-one crosswalk.
- **L0 negative:** event references an unknown unit.
- **L1 positive:** split and merge with exhaustive rational weights.
- **L1 negative:** exhaustive weights sum to something other than `1`.
- **L2 positive:** small real public source with preserved source hashes.

## Hash Domains

```text
RHIST_CYCLE_V1\0
RHIST_LINEAGE_EVENT_V1\0
RHIST_CROSSWALK_V1\0
RHIST_PACKAGE_V1\0
```
