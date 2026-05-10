---
reviewer: ROLE PANEL
roles: BOUNDARY, WARD, COVENANT, CONTOUR, MERIDIAN, BENCHMARK, SCALE, PRECINCT, DATUM, COMMONS, LEDGER, SURVEY, TRENCH
spec: RPLAN Incubation And Crate Factoring / RPLAN v0.1 Format
round: 2
date: 2026-05-10
score: 2.6
---

# Round 2 Role Review: RPLAN Incubation And Crate Factoring

## Summary

The RPLAN direction is correct: use RPLAN as the public interchange artifact,
incubate `rplan-*` crates inside this repo, and forbid dependencies from
`rplan-*` back into `bisect-*`.

Round 2 does **not** approve crate implementation yet. The current RPLAN v0.1
format is a useful compatibility format, but it is not yet the right foundation
for `rplan-core`, `rplan-io`, or `rplan-audit`. Before implementation, add a
RPLAN v0.2 schema revision that defines unit identity, graph context, canonical
hashing, provenance, and the split between format validity and audit validity.

Decision: **revise before implementation**.

## Scores

| Role | Score | Reason |
|------|-------|--------|
| BOUNDARY | 3/4 | The bisect/RPLAN dependency boundary is correct, but PlanManifest leakage must be blocked. |
| WARD | 2.5/4 | Legal-profile inputs belong in audit context, not in the base plan format; v0.2 must say this. |
| COVENANT | 2/4 | v0.1 lacks canonical hash rules and stable identity; not yet court-audit ready. |
| CONTOUR | 2.5/4 | Graph knowledge is needed, but as context rather than required embedded plan content. |
| MERIDIAN | 3/4 | Contiguity can be audited cleanly once `UnitGraph` is specified. |
| BENCHMARK | 2/4 | Implementation tests cannot be stable until canonical JSON and v0.1/v0.2 conversion are fixed. |
| SCALE | 3/4 | Keeping graph context separate avoids bloating `.rplan` files. |
| PRECINCT | 2.5/4 | Tract-only v0.1 is too narrow for blocks, precincts, imported units, and local plans. |
| DATUM | 2.5/4 | Source hashes and unit-universe hashes need first-class fields. |
| COMMONS | 3/4 | Tool-neutral RPLAN is a good contribution if bisect-specific fields are removed. |
| LEDGER | 2/4 | `HashMap` + pretty JSON is not an audit ledger; canonical projection is required. |
| SURVEY | 3/4 | `rplan audit` is a better public surface than `bisect certify`. |
| TRENCH | 2.5/4 | Extraction is feasible, but only if `bisect-report` is not moved wholesale. |
| **Average** | **2.6/4** | Revise before implementation. |

## Required Fixes Before Crate Work

### R2-A: Write a RPLAN v0.2 schema spec

RPLAN v0.1 is tract-GEOID keyed and assignment-map oriented. RPLAN v0.2 must
define the generic model that `rplan-core` will implement:

- `unit_kind`
- `unit_ids`
- `canonical_order`
- `assignment`
- `k`
- `display_labels`
- `unit_universe_hash`
- `source_hashes`
- `producer`
- optional `extensions`

RPLAN v0.1 remains a compatibility input. It should not be the internal
`rplan-core` model.

### R2-B: Define graph context explicitly

RPLAN needs graph knowledge for audit, but the graph is not the plan. Add a
separate context artifact, either as `.rctx` or as a named JSON schema:

```text
plan.rplan      # district assignment artifact
context.rctx    # unit universe, adjacency, populations, source hashes
audit.json      # plan + context + legal profile result
```

Minimum `rplan-core` graph model:

```rust
pub struct UnitGraph {
    pub units: PlanUnitIndex,
    pub adjacency: Vec<Vec<UnitEdge>>,
}

pub struct UnitEdge {
    pub to: u32,
    pub kind: EdgeKind,
    pub weight: Option<f64>,
}
```

`rplan-audit` may require `UnitGraph` for contiguity checks, but `.rplan` files
must remain valid assignment artifacts without embedded adjacency.

### R2-C: Resolve 1-based file labels vs 0-based internal labels

RPLAN v0.1 stores district ids as `1..k`. The audit model wants canonical
internal ids `0..k-1`. The conversion rule must be owned by `rplan-io`:

- v0.1 read: `1..k` file labels -> `0..k-1` internal ids
- v0.2 write: choose either canonical `0..k-1` plus `display_labels`, or keep
  external labels and define the canonical hash projection
- audit certificates must state which representation was hashed

Do not let each algorithm crate make its own label conversion.

### R2-D: Keep `PlanManifest` out of `rplan-core`

`PlanManifest` is a bisect run artifact. RPLAN needs generic provenance types:

- `ProducerInfo`
- `SourceHash`
- `UnitUniverse`
- `ConversionLineage`
- `RuntimeProvenance`

`bisect-cli` can adapt `PlanManifest` into those types. `rplan-*` must not
import or serialize `PlanManifest` directly.

### R2-E: Do not extract `bisect-report` wholesale

The refactor must not rename `bisect-report` into `rplan-*`. The generic pieces
to extract are narrow:

- RPLAN structs
- RPLAN read/write
- RPLAN validation
- canonical assignment/hash helpers after generalization

Leave reports, HTML, narrative, BISECT path handling, verification scripts, and
PlanManifest management in `bisect-report`.

### R2-F: Fix RFC 7946 geometry language

RPLAN v0.1 says geometry follows RFC 7946 but shows a `crs` member and permits
NAD83. v0.2 must either:

- be strict RFC 7946: WGS84 longitude/latitude and no `crs` member, or
- define an explicit non-RFC extension field outside the GeoJSON object

The recommended path is strict RFC 7946 for the embedded GeoJSON geometry.

## Suggested Implementation Gate

Start crate work only after these docs exist:

1. `docs/specs/2026-05-10-rplan-v0.2-schema.md`
2. updated `2026-05-10-rplan-incubation.md` linking this review
3. updated `2026-05-10-plan-audit-certificates.md` naming the context artifact

Status as of the fixed-point update: these documents now exist, but they still
need implementation review before crate work starts.

Then implement in this order:

1. `rplan-core`
2. `rplan-io` with v0.1 compatibility reader and v0.2 writer
3. `bisect-report` adapter layer
4. `rplan-audit`
5. `rplan-cli`

## Non-Blocking Notes

- `rplan` remains the right public name.
- Coexisting in this repo remains the right incubation strategy.
- A future standalone repo is reasonable after v0.2 migration tests and at
  least one non-bisect import/audit path.
