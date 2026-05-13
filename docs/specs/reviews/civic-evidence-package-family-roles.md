# Civic Evidence Package Family Role Lenses

date: 2026-05-13  
scope: `docs/specs/2026-05-13-civic-evidence-package-family.md`

## Purpose

The existing role panel remains valid for RPLAN and RCOUNT: BOUNDARY, WARD,
COVENANT, CONTOUR, MERIDIAN, BENCHMARK, SCALE, PRECINCT, DATUM, COMMONS,
LEDGER, SURVEY, TRENCH, plus CANVASS, TALLY, and VAULT for election-count work.

The civic evidence package family adds a new failure mode: package boundaries
can drift until every package owns geography, history, custody, statistics, and
presentation at once. These lenses are overlays for reviewing package-family
architecture.

## New Overlay Roles

| Role | Point of view | Blocks when... |
|---|---|---|
| IDENTITY | canonical unit identity, unit order, source universe, crosswalk ids | a presentation artifact or domain package invents canonical ids |
| LINEAGE | temporal unit history, splits, merges, renames, boundary edits | RPLAN or RCOUNT becomes the permanent history owner |
| CARTOGRAPHY | rendered map outputs, projections, visual layers, label provenance | RMAP is treated as machine context or RCTX is polluted with styling |
| CUSTODY | seals, transfers, logs, observers, physical evidence chain | custody claims are inferred from count/audit math alone |
| STATS | statistical models, uncertainty, anomaly scores, risk metrics | exploratory analytics are presented as verifier/certification claims |
| COMPOSITION | bundles, court records, public evidence packages | an umbrella bundle creates new facts not present in child packages |

## How To Use Them

Use these overlays when reviewing package-family specs:

- **RCTX/RMAP specs:** IDENTITY and CARTOGRAPHY must both sign off.
- **RHIST specs:** LINEAGE and IDENTITY must both sign off.
- **RCOUNT multi-cycle specs:** LINEAGE checks whether the feature belongs in
  RHIST instead.
- **RAUDIT/RCHAIN/RLOG specs:** CUSTODY checks whether physical claims exceed
  source evidence.
- **RSTAT/W-series specs:** STATS checks claim language and false-positive
  boundaries.
- **RCASE specs:** COMPOSITION checks that child package hashes and boundaries
  are preserved.

These roles do not replace the base panel. They sharpen it.
