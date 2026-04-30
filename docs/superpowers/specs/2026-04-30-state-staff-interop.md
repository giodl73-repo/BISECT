# State Staff Interop: Don't Build a GUI; Be a Backend
**Date:** 2026-04-30
**Status:** Proposed; pending review
**Closes gap for:** state legislative staff (★★→★★★★)
**Depends on:** existing redist export + existing redist import (partial)
**Estimated effort:** 2-3 days

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

6. **Round-trip property tests**
   - Districtr → redist → Districtr: assignment identical
   - DRA → redist → DRA: assignment identical
   - GerryChain → redist → GerryChain: already in Researcher spec; cross-reference

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
- L1: round-trip Districtr → redist → Districtr on a small Vermont sample
- L1: round-trip DRA → redist → DRA on the same sample
- L2: skipped-by-default — fetch a real Districtr plan from their public examples and validate the import

## Risks

| Risk | Mitigation |
|---|---|
| Districtr's JSON format changes | Pin the Districtr version we test against; document the schema we accept |
| DRA's CSV format varies | Sniff column names; accept common variants |
| Block→tract crosswalk is large per state | Pre-compute and cache; reuse Census-published crosswalks |
| Precinct-level imports need precinct shapefiles we don't have | Document the prerequisite; user provides them |
| Staff want features we don't have ("compare against last cycle's enacted") | Cross-reference Plan Comparison spec |

## Definition of done

- `redist import --format districtr` round-trips a sample Districtr export
- `redist export --format districtr` produces a file that re-loads in Districtr's web app
- DRA import/export verified on a sample DRA plan
- Shapefile import/export verified with QGIS as an external test client
- `quickstart-state-staff.md` walks through the full Districtr → redist → PDF flow
- A real state staffer (not us) gets through the quickstart in under 15 minutes
