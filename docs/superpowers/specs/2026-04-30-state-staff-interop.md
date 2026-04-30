# State Staff Interop: Don't Build a GUI; Be a Backend
**Date:** 2026-04-30
**Updated:** 2026-04-30 (v2 — incorporates 6-role consensus on atomic import + version handshake + Callais preflight)
**Status:** Revised; pending re-review
**Closes gap for:** state legislative staff (★★→★★★★), civic group submitting a counter-proposal (bypass-staff path)
**Depends on:** existing redist export + existing redist import (partial)
**Estimated effort:** 3-4 days (v2: atomic import + version handshake + civic bypass)

## Why this exists

State legislative staff want to draw maps interactively, see population balance update in real time, and submit a final plan. That's a Districtr / Dave's Redistricting App workflow. Building a competing GUI is a multi-year project; **MGGG already does it for free.**

The honest move: ship clean import/export so staff use Districtr (or DRA, or QGIS) as the front-end and our CLI as the analytical backend. This spec defines the interop surface.

A staffer's workflow becomes:
1. Draw the map in Districtr, save as JSON
2. `redist import --format districtr <PATH> --label senate_proposal_v3`
3. `redist analyze --label senate_proposal_v3 --types all`
4. `redist report --label senate_proposal_v3 --format pdf`
5. Iterate in Districtr; repeat

The staffer never sees Rust, Python, or our internal data formats. They get fast analysis + court-quality reports out of a familiar GUI.

## Scope

### In scope

1. **Districtr import** — `redist import --format districtr <PATH>`
   - Districtr exports plans as JSON with assignment by precinct or block
   - We map their geometry IDs to our GEOIDs
   - Validate population balance, contiguity (warn but don't fail)
   - Write a plan label with manifest

2. **Districtr export** — `redist export --format districtr --label LABEL`
   - Produce a Districtr-compatible JSON that can be loaded back into the web app
   - Useful for: "I drew a plan via the CLI; load it into Districtr to share/edit"

3. **DRA (Dave's Redistricting App) import/export**
   - DRA's CSV export format: `geoid, district`
   - We already export `--format csv`; verify it's DRA-compatible
   - Import: `redist import --format dra <PATH>`

4. **QGIS / Shapefile interop**
   - Export plan as shapefile (district polygons + attribute table)
   - Import: same — accept a shapefile with a `district` column

5. **State staff quickstart** (extends Onboarding spec)
   - `quickstart-state-staff.md`: 5-step walkthrough using Districtr → redist → PDF

6. **Round-trip property tests (v2 — formal equality)**
   - **Definition of round-trip equality**: for each upstream tool T ∈ {Districtr, DRA, GerryChain}, a plan P satisfies `T(redist(T(P))) == P` iff for every geometry ID g in T's source format, the destination district assigned to g is identical, after canonicalizing district labels (district numbers can be permuted by the tool but the partition must be the same).
   - **Canonical form**: assignments are compared after re-numbering districts in increasing order of their lowest-GEOID member.
   - **Test scope**: assignment-only equality. Metadata (color, name, comment) is not required to round-trip; if it does, that's a bonus.
   - Districtr → redist → Districtr: passes round-trip equality
   - DRA → redist → DRA: passes round-trip equality
   - GerryChain → redist → GerryChain: already in Researcher spec; cross-reference

7. **Atomic import (v2 — TRENCH PP-22)** — `redist import` writes to a tmp directory and renames into place only after every validation passes. Half-imported plan labels MUST NOT be visible to subsequent commands. On failure: tmp directory is deleted, no plan label is created, exit code is non-zero, and the error message names the validation that failed.

8. **Schema version handshake (v2 — TRENCH/COVENANT)**
   - Districtr export JSON has a top-level `schema_version` field (or we infer the Districtr build from a known set of fingerprints when it's missing)
   - DRA exports embed a column-set fingerprint in their CSV header
   - On import, we record the upstream tool name + version + schema fingerprint in the import manifest
   - We pin a tested-against version range per format; out-of-range raises a warning naming the supported range
   - The pinned ranges live in `redist-cli/src/import_compat.json` so they're a one-line bump

9. **Callais p.36 mutex preflight (v2 — BOUNDARY)** — every `redist import` (and every subsequent `redist analyze`) checks the plan's manifest for both VRA-aware bisection markers and partisan-weighted bisection markers. If both appear, the command exits with the Callais p.36 disentanglement error before any analysis runs. State-staff workflows are the highest-volume entry point for this footgun; we catch it here.

10. **Civic-group bypass path (v2 — COMMONS)** — `redist import --as-civic-counter-proposal` flag tags the import manifest with `submission_type=civic_counter_proposal` and:
    - Skips the "is this from an authoritative state mapping tool" warning
    - Records `submitted_by` (free text) + `submitted_at` (ISO-8601)
    - Wires the plan into the Plan Comparison + Civic Bidirectional pipelines so the public counter-proposal becomes a first-class comparable plan, not a second-class import

### Out of scope

- Native GUI (the whole point of this spec)
- Real-time collaborative editing (Districtr's domain)
- Mobile app (web works)
- Legal-compliance check pre-submission ("does this plan comply with state X's redistricting criteria?") — separate spec / partner with state-specific tools

## Implementation notes

### GEOID translation is the hard part

Districtr identifies geometries by Census GEOID (TIGER), block ID (Census), or precinct ID (state-specific). Our internal IDs are 11-char TIGER tract GEOIDs.

- **Tract-level Districtr plans:** trivial (1:1 mapping)
- **Precinct-level Districtr plans:** need precinct→tract reallocation. Reuse the existing `scripts/data/political/build_dem_shares.py` aggregation logic (or its successor)
- **Block-level Districtr plans:** aggregate blocks → tracts via Census-published block-to-tract crosswalk
- **DRA plans:** typically tract or block; same logic applies

### Validation on import

- Population balance check (warning, not error — staff might be mid-draw)
- Contiguity check (warning if any district is split)
- Population sum check (does the total match expected state population? Warn if drift)
- District count check (does it match expected state seats? Warn)

Warnings get written to the plan's manifest so subsequent analyze/report commands can surface them.

### The CLI surface

```
redist import --format <FORMAT> <PATH> --label <LABEL> [--year YYYY]
  formats: districtr, dra, gerrychain, shapefile, geojson, csv

redist export --format <FORMAT> --label <LABEL> [--out <PATH>]
  formats: districtr, dra, gerrychain, shapefile, geojson, csv, reproducibility-package
```

`redist export --format districtr` is the new addition (we already have geojson, gerrychain, csv, reproducibility-package).

## Outputs

```
outputs/{version}/imports/{label}/
├── manifest.json                # Source format, source URL/path, import warnings
├── final_assignments.json       # Standardized internal format
├── original.{ext}               # Verbatim copy of imported file
└── translation_log.txt          # GEOID-mapping decisions (if non-trivial)
```

## Tests

- L0: schema validation for each format (JSON / CSV / shapefile readers)
- L0: GEOID translation logic (tract↔block↔precinct) on synthetic small graph
- L0 **atomic import failure**: inject a contiguity validation failure mid-import; assert no plan label is visible in `outputs/` and the tmp directory is gone
- L0 **canonical-form round-trip**: Districtr → redist → Districtr on a 5-district synthetic plan where district labels are permuted; assert canonical-form equality
- L0 **Callais p.36 preflight**: import a plan whose manifest claims both VRA + partisan markers; assert the import exits non-zero with the disentanglement error string
- L0 **schema-version handshake**: import a Districtr file with `schema_version` outside the pinned range; assert the warning fires and names the supported range
- L0 **civic bypass path**: import with `--as-civic-counter-proposal --submitted-by "League of Women Voters"`; assert manifest has `submission_type=civic_counter_proposal` and the submitter is recorded
- L1: round-trip Districtr → redist → Districtr on a small Vermont sample
- L1: round-trip DRA → redist → DRA on the same sample
- L2: skipped-by-default — fetch a real Districtr plan from their public examples and validate the import

## Risks

| Risk | Mitigation |
|---|---|
| Districtr's JSON format changes | Pin the Districtr version we test against in `import_compat.json`; schema-version handshake warns out-of-range; document the schema we accept |
| DRA's CSV format varies | Column-set fingerprinting; accept known variants; out-of-range warns |
| Block→tract crosswalk is large per state | Pre-compute and cache; reuse Census-published crosswalks |
| Precinct-level imports need precinct shapefiles we don't have | Document the prerequisite; user provides them |
| Half-imported plan label observed by another command (TRENCH PP-22) | Atomic tmp-then-rename; on failure no label is created |
| Staff accidentally combine VRA + partisan bisection (Callais p.36) | Mutex preflight at import + analyze gates; loud error |
| Civic counter-proposal silently treated as authoritative state submission | `--as-civic-counter-proposal` flag tags manifest; comparison reports surface the submission type |
| Staff want features we don't have ("compare against last cycle's enacted") | Cross-reference Plan Comparison spec |

## Definition of done

- `redist import --format districtr` round-trips a sample Districtr export under the canonical-form definition
- `redist export --format districtr` produces a file that re-loads in Districtr's web app
- DRA import/export verified on a sample DRA plan
- Shapefile import/export verified with QGIS as an external test client
- Atomic-import + Callais preflight + schema-version handshake all enforced and tested
- `--as-civic-counter-proposal` produces a tagged plan that flows through Plan Comparison + Civic Bidirectional pipelines
- `quickstart-state-staff.md` walks through the full Districtr → redist → PDF flow
- A real state staffer (not us) gets through the quickstart in under 15 minutes
