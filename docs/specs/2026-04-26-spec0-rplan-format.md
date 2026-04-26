# Spec 0: RPLAN — Redistricting Plan Interchange Format

**Date**: 2026-04-26
**Status**: Draft v0.1
**Depends on**: Nothing (foundation spec)
**Used by**: All specs 1–6

---

## Why a standard format?

No open standard exists for redistricting plan interchange. Current tools use:

| Tool | Format | Problems |
|------|--------|----------|
| Dave's Redistricting App | GeoJSON district polygons | No tract assignment; only district boundaries |
| MGGG GerryChain | `{"assignment": {"geoid": district_int}}` | Unpublished; changed between v2/v3 |
| PlanScore | GeoJSON via API | API-only; no file format spec |
| Maptitude | Proprietary `.rds` | Closed, no external access |
| Census enacted | ESRI shapefile | Boundaries only; no tract mapping |
| `redist` (current) | `{"tract_index": district_int}` | Index-keyed, not GEOID-keyed |

The result: every tool that wants to interoperate writes its own converter, and those converters break when the source tool updates. A practitioner submitting a plan to a court needs a single, stable, independently verifiable format — not a chain of tool-specific exports.

**RPLAN** is that format. It is:
- **Open** — published spec, no license restrictions
- **Versioned** — `rplan_version` field; old files remain readable
- **GEOID-keyed** — uses Census tract GEOIDs as the authoritative identifier, not tool-internal indices
- **GeoJSON-compatible** — the geometry section follows RFC 7946 exactly
- **Self-describing** — metadata sufficient to reproduce or verify the plan
- **Tool-neutral** — no `redist`-specific fields required; any tool can write a valid RPLAN file

---

## Format specification: RPLAN v0.1

A RPLAN file is a JSON document with a `.rplan` extension (MIME type: `application/vnd.rplan+json`).

### Top-level structure

```json
{
  "rplan_version": "0.1",
  "metadata": { ... },
  "assignments": { ... },
  "geometry": { ... }
}
```

All four top-level keys are required. `geometry` may be `null` if district polygons are not available (assignments-only mode).

---

### `metadata` object (required)

```json
{
  "rplan_version": "0.1",
  "metadata": {
    "label": "wa_house_draft1",
    "state_fips": "53",
    "state_code": "WA",
    "year": "2020",
    "chamber": "house",
    "num_districts": 98,
    "population_source": "total",
    "balance_tolerance_pct": 5.0,
    "created_at": "2026-04-26T14:23:00Z",
    "created_by": "redist v0.1.0",
    "seed": 42,
    "notes": "Draft 1 — WA House redistricting commission submission"
  }
}
```

| Field | Required | Type | Description |
|-------|----------|------|-------------|
| `label` | yes | string | Human name for this plan |
| `state_fips` | yes | string | 2-char zero-padded Census FIPS code |
| `state_code` | yes | string | 2-letter postal code |
| `year` | yes | string | Census year: "2020", "2010", "2000" |
| `chamber` | yes | string | "congressional", "house", "senate", "custom" |
| `num_districts` | yes | integer | Number of districts in this plan |
| `population_source` | yes | string | "total", "vap", "cvap" |
| `balance_tolerance_pct` | yes | number | Max % deviation used (e.g., 5.0 for ±5%) |
| `created_at` | yes | string | ISO 8601 timestamp |
| `created_by` | yes | string | Tool name + version that created this file |
| `seed` | no | integer\|null | RNG seed if plan was generated algorithmically |
| `notes` | no | string | Free text |
| `source_manifest` | no | object | Full PlanManifest if created by `redist` (Spec 1) |

---

### `assignments` object (required)

Maps Census tract GEOID (11-character string) to district ID (1-based integer).

```json
{
  "assignments": {
    "530330001001": 7,
    "530330001002": 7,
    "530330002001": 12,
    "530330003001": 7
  }
}
```

**Rules:**
- Keys: 11-character Census tract GEOIDs (`{state_fips_2}{county_fips_3}{tract_code_6}`)
- Values: 1-based integers from 1 to `metadata.num_districts`
- Every tract in the state must appear exactly once (no gaps, no duplicates)
- Tools must validate completeness: `len(assignments) == num_tracts_in_state`

**Why GEOID keys, not index keys:**
Census tract GEOIDs are stable across tool versions and human-readable. Index keys (`{"174": 1}`) are tool-internal and break when the adjacency graph is rebuilt or tract order changes. GEOIDs are the authoritative Census identifier.

---

### `geometry` object (optional, RFC 7946 GeoJSON)

When present, contains dissolved district polygons as a GeoJSON FeatureCollection. If absent or `null`, the file is assignments-only.

```json
{
  "geometry": {
    "type": "FeatureCollection",
    "crs": {"type": "name", "properties": {"name": "urn:ogc:def:crs:EPSG::4269"}},
    "features": [
      {
        "type": "Feature",
        "properties": {
          "district": 1,
          "population": 761169,
          "polsby_popper": 0.387,
          "state_fips": "53"
        },
        "geometry": {
          "type": "MultiPolygon",
          "coordinates": [ ... ]
        }
      }
    ]
  }
}
```

**RFC 7946 compliance requirements (LEDGER):**
- Coordinates in `[longitude, latitude]` order (RFC 7946 §3.1.1)
- CRS is WGS84 (EPSG:4326) or NAD83 (EPSG:4269) — state explicitly
- Exterior rings: counterclockwise winding order
- Interior rings (holes): clockwise winding order
- No `null` geometry features
- `properties.district` (integer, 1-based) is the only required property

---

## Tool compatibility matrix

| Tool | Can read RPLAN | Can write RPLAN | Notes |
|------|---------------|----------------|-------|
| `redist` CLI | Yes (native) | Yes (native) | First-class format |
| GerryChain v2.3+ | Via converter | Via converter | `assignment` field maps to RPLAN `assignments` |
| DRA | Via GeoJSON geometry section | No | Reads district polygons only |
| PlanScore | Via GeoJSON geometry section | No | API-only, reads polygons |
| Maptitude | Via shapefile export | No | Proprietary |
| QGIS / ArcGIS | Via GeoJSON geometry section | No | Reads polygons |

### GerryChain v2.3 compatibility (LEDGER)

GerryChain uses `{"assignment": {"geoid": district_int}}` (singular key, same value type).

Conversion from RPLAN to GerryChain:
```python
import json
rplan = json.load(open("plan.rplan"))
gerrychain_partition = {"assignment": rplan["assignments"]}
```

Conversion from GerryChain to RPLAN:
```python
rplan = {
    "rplan_version": "0.1",
    "metadata": {"label": "...", "state_fips": "...", ...},
    "assignments": gerrychain_partition["assignment"],
    "geometry": None
}
```

**GerryChain version note**: GerryChain v1.x used `"parts"` instead of `"assignment"`. RPLAN does not support GerryChain v1.x.

---

## Validation

A valid RPLAN file must pass all of:

1. **Schema validation**: all required fields present with correct types
2. **GEOID format**: all keys match `^\d{11}$`
3. **District range**: all values are integers in `[1, num_districts]`
4. **Coverage**: when `state_fips` and `year` are known, all tracts in that state are present (checked via Census tract count reference)
5. **GeoJSON RFC 7946**: geometry section (if present) passes RFC 7946 validation

`redist` CLI validates on read:
```bash
redist validate --file plan.rplan
# → PASS: valid RPLAN v0.1 (530 tracts, 10 districts, WA 2020 congressional)
# → FAIL: 3 GEOIDs invalid format (must be 11 digits): ["530330", "5303300010", "5303300010011"]
```

---

## Versioning

RPLAN uses semantic versioning in `rplan_version`.

| Version | Status | Notes |
|---------|--------|-------|
| `0.1` | Current draft | Defined in this spec |
| Future `0.2` | TBD | May add multi-plan (suite) support |
| Future `1.0` | Stable | Backward-compatible with all 0.x |

**Compatibility promise**: a tool that reads RPLAN `0.1` must continue to read `0.x` files after minor version bumps. Breaking changes require a major version bump to `1.0`.

**Unknown version handling**: tools receiving a file with a higher minor version than they support should read it with a warning; tools receiving a higher major version should fail with a clear error.

---

## RPLAN vs GeoJSON

| | RPLAN | GeoJSON district polygons |
|-|-------|--------------------------|
| Tract assignments | Yes (explicit) | No (must be computed) |
| District polygons | Optional | Yes |
| RFC 7946 compliant | Yes (geometry section) | Yes |
| Self-describing metadata | Yes | No |
| Reproducibility/audit | Yes (source_manifest) | No |
| Tool compatibility | Via converter | Direct |
| File size (assignments only) | ~500KB for 50-state run | N/A |
| File size (with geometry) | ~5MB per state | ~1-2MB per state |

RPLAN is **not a replacement for GeoJSON** for district polygon interchange. It is a superset: the `geometry` section of a RPLAN file is valid GeoJSON, and tools that only need polygons can read just that section. RPLAN adds what GeoJSON cannot carry: the tract-level assignments, metadata, and audit chain.

---

## Reference implementation

The `redist` binary is the reference implementation for RPLAN:

```bash
# Generate a plan and write as RPLAN
redist state --state WA --year 2020 --version WA_Plans \
  --districts 98 --chamber house --label wa_house_draft1 \
  --output-format rplan

# Validate an RPLAN file
redist validate --file wa_house_draft1.rplan

# Convert RPLAN to other formats
redist export --file wa_house_draft1.rplan \
  --format geojson shapefile gerrychain csv

# Import RPLAN from any tool
redist import --file external_plan.rplan \
  --state WA --year 2020 --label external_for_analysis
```

---

## Board review notes

**[LEDGER]**: RPLAN `assignments` uses plural; GerryChain uses `assignment` singular — this is intentional and documented above. The field name difference is preserved to avoid silent misreads when converting.

**[COVENANT]**: The `source_manifest` field in metadata carries the full `PlanManifest` (Spec 1) for `redist`-generated plans, providing complete audit chain. For plans from other tools, `source_manifest` is `null` and the `created_by` field records the tool.

**[WARD]**: RPLAN carries `chamber` and `population_source` in metadata so consuming tools know which legal standards apply when analyzing the plan. These fields inform Spec 3's state-specific split analysis.

**[BOUNDARY]**: RPLAN does not encode VRA compliance, partisan fairness, or any legal determination — it is a neutral data format. Legal analysis is the domain of the `redist analyze` subcommands that consume RPLAN files.

---

## R3 Board Review Amendments (2026-04-26)

**[LEDGER] CRITICAL — Remove rplan_version from metadata object**
`rplan_version` appears at both top level and inside `metadata`, creating two sources of truth that can disagree.
Fix: `rplan_version` lives ONLY at the top level. Remove it from the `metadata` object definition and example.
Correct top-level structure:
```json
{
  "rplan_version": "0.1",
  "metadata": { },
  "assignments": { ... },
  "geometry": { ... }
}
```
The `metadata` example shown in this spec erroneously includes `"rplan_version": "0.1"` as the first field inside the `metadata` block. That field must be removed from the `metadata` object. `rplan_version` belongs at the top level only.

**[SURVEY] CRITICAL — Define redist validate in Commands enum**
`redist validate --file plan.rplan` is shown as a CLI command but no `Commands::Validate` variant is defined.
Fix: Add to Spec 1's implementation section. `redist validate` dispatches to `redist_report::validate_rplan()` which checks schema, GEOID format, district range, coverage, and RFC 7946.

**[LEDGER] CRITICAL — Define path convention migration**
`plans/{label}/` tree (Spec 1) vs legacy `states/{state_name}/` tree (existing CLI). These coexist but are never reconciled.
Fix: Both trees are preserved. Unlabeled runs continue using `states/{state_name}/`. Labeled runs use `plans/{label}/`. `redist analyze` and `redist map` accept either `--state` (legacy path) or `--label` (new path). A `redist migrate --state WA --label wa_congressional_2020` command copies a legacy plan into the new tree. Document this in the overview spec.
