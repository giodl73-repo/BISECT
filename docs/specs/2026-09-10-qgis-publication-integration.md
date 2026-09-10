# QGIS publication integration — proposal, 2026-09-10

Status: architecture with a completed RI publication pilot; see the
[implementation guide](../PUBLICATION_QUICKSTART.md) and its release evidence.
Desktop setup is complete: the CLI builds from the lockfile, the M:
vault is linked, all 150 canonical contexts hash-verify, and the six-arm RI
portable replay passes. Separate RPLAN, RLINE and RCOUNT checkouts are present
for contract inspection. BISECT's embedded snapshots and pinned dependencies
have not been replaced with sibling checkout heads.

## Where the mapping technology fits

BISECT already has open-source SVG/PNG rendering in `crates/bisect-map` and
GeoJSON/GerryChain/CSV interchange in `crates/bisect-cli/src/export_cmd.rs`.
The QGIS/GDAL/GeoPackage workflow developed for SEAWALKS and PUCK adds editable
cartography, local basemaps, print layouts, atlases and relocatable recovery
packages. Existing fast renderers remain useful for routine research runs.
The existing GeoJSON exporter can emit null geometry when the plan package
lacks geometry. The pilot must check this explicitly and join the matching
versioned source geography; successfully writing GeoJSON is not sufficient.

| Owner | Appropriate work | Boundary |
|---|---|---|
| BISECT map/report and Python publication layer | Optional QGIS project generator; statewide and district-detail atlas; side-by-side scenario maps; legends, labels, insets, source notes; PNG/PDF release checks | Consumer of verified plans/contexts and metrics. QGIS/GDAL is an optional publication environment, not a required solver runtime. |
| BISECT data preparation | Read-only inspection of TIGER/PL joins, geometry coverage, nulls, CRS and subdivision boundaries; versioned GeoPackage working views | Do not repair geometry, substitute Census vintages or change adjacency implicitly. Any source change must follow existing data-admission and replay gates. |
| RPLAN | Neutral, versioned interchange contract for plan identity, unit IDs, assignments, geometry references and provenance; potential optional GeoPackage adapter after a demonstrated consumer need | RPLAN's design rule leaves map styling, atlas layouts and BISECT workflows in the application. Existing canonical hashes and certificate meanings remain authoritative. |
| RCOUNT | Existing verified district aggregation and count-status outputs become map attributes consumed by BISECT | Product maps remain outside RCOUNT. Preserve count lifecycle, lineage and privacy gates; no ballot-level geographic export is implied. |
| RLINE | Existing RCTX/crosswalk, graph and history contracts support the input and evidence chain | Its current remit is reusable kernels. Do not add QGIS layouts or a large GIS runtime to the foundation merely to share application code. |

The upstream boundaries are explicit in RPLAN `README.md`/`docs/compatibility.md`,
RLINE `README.md`/`docs/compatibility.md`, and RCOUNT `docs/family-contract.md`.
Foundation changes require their tests and the documented RCOUNT downstream
rehearsal. Adopting a new dependency revision in BISECT is a separate,
intentional compatibility change with pinned commits and fixture/hash checks.

## First useful pilot

1. Select one existing, versioned BISECT plan and its matching RCTX/source
   geometry. A completed small-State plan is simpler for the first atlas;
   the RI adjacent-swap result is a two-label diagnostic and must be labeled
   that way if used instead. Do not imply it is a new certified final plan.
2. Export a publication GeoPackage with units, district assignments,
   subdivisions and display geometry. Preserve GEOIDs as text, plan/district
   IDs, Census vintage, population definitions, source hashes, plan/context
   hashes and original verification status. Make any dissolve/reprojection
   an explicit derivative with its own hash, not a replacement canonical file.
3. Generate a QGIS project with relative paths and local basemap data. Provide
   an overview and per-district pages with consistent scales where comparison
   requires them. Keep full-precision geometry separate from any display-only
   simplification. Templates belong under BISECT's publication tooling.
4. Check exact assignment equality and join coverage; compare per-district
   and statewide population totals; audit geometry coverage and CRS/axis order;
   bind release metadata to the existing plan/context evidence. Verify the
   complete page set, labels, legends, dates and qualifications.
5. Reopen and export from a relocated package using standalone QGIS. Perform
   active publication work in the ignored local `.work/publication` workspace;
   archive verified releases to the configured vault after local completion.
   Commit compact templates, manifests, checks and reviewed summaries. Follow
   [the local-work and distribution contract](../PUBLICATION_PACKAGING.md).

Suggested package structure: `project.qgs` and its attachment companion,
`publication.gpkg`, pinned basemap data/credits, `source-manifest.json`,
`verification.json`, and reviewed exports. Styling edits are allowed. An edit
to assignments or canonical geometry creates a new candidate requiring the
normal BISECT/RPLAN checks; saving a QGIS project does not renew a certificate.

After the pilot, add multi-scenario comparison atlases and optional maps of
verified RCOUNT aggregates. Revisit a shared adapter only when more than one
consumer needs the same neutral contract; avoid moving product templates into
RPLAN/RLINE/RCOUNT.

## Review through the local roles

This is an agent self-review through `.roles` definitions, not an independent
human review or implementation approval.

- **CONTOUR:** require text GEOIDs, exact join coverage, correct Census vintage
  and explicit population/VAP definitions. A convenient newer basemap must
  not replace the source geography used by the plan.
- **MERIDIAN:** accept publication as a downstream consumer. Graph adjacency,
  assignment decisions, objectives and computed metrics stay governed by the
  existing engine; apparent visual improvement is not an algorithm result.
- **DATUM:** distinguish reproducible rendering from evidence of better plans.
  Preserve original result status, uncertainty and unresolved proof stages.
- **LEDGER:** require versioned interchange and CRS/coordinate-order checks;
  retain canonical hashes alongside separate derivative hashes. Test actual
  GeoJSON/GeoPackage round trips before claiming compatibility.

The RI-specific publication builder and portable packages are implemented.
A general RPLAN mapping adapter, new research result, dependency-snapshot
migration and historical Cal* ArcGIS conversion remain outside this pilot.
