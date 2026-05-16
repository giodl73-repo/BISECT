# BISECT FLETCH Source Orchestration

**Status:** Implemented first migration slice
**Date:** 2026-05-15
**Scope:** `bisect fetch` source acquisition, cacheline handoff, and non-mutating source gate
**Review record:** [`reviews/bisect-fletch-source-orchestration-r1_roles.md`](reviews/bisect-fletch-source-orchestration-r1_roles.md)

## Decision

BISECT should use FLETCH for neutral source acquisition while BISECT keeps all
redistricting semantics, derived artifacts, and admissibility claims.

FLETCH owns the generic URL/cacheline boundary:

- stable cacheline identity for each requested source item;
- generic HTTP acquisition for public source bytes;
- registry/flight/handoff reporting for source readiness;
- a reusable cache root under `data/.fletch`.

BISECT owns the domain boundary:

- `data/manifest.json` and local manifest overrides;
- state/year/type expansion in `bisect fetch`;
- GitHub release adjacency pulls;
- ZIP extraction into existing BISECT paths;
- LODES aggregation from block files to tract CSVs;
- ACS housing derivation;
- done-marker semantics;
- build/analyze/report validation and all redistricting claims.

Downloaded source bytes are evidence inputs, not validated redistricting claims.

## Source Families

| Family | Source | FLETCH role | BISECT role |
|---|---|---|---|
| `tiger` | Census TIGER tract ZIP | Generic HTTP cacheline | Extract shapefile into existing `data/{year}/tiger/tracts/...` target |
| `pl94171` | Census PL 94-171 ZIP | Generic HTTP cacheline | Preserve existing redistricting ZIP target and done marker |
| `school-districts` | Census TIGER UNSD ZIP | Generic HTTP cacheline | Extract school district shapefile |
| `eia-861` | EIA Form 861 ZIP | Generic HTTP cacheline | Extract service territory shapefile |
| `lodes-wac` | LEHD LODES WAC `.csv.gz` | Generic HTTP cacheline when present | Preserve 404 soft-skip and aggregate blocks to tract CSV |
| `lodes-od` | LEHD LODES OD `.csv.gz` | Generic HTTP cacheline when present | Preserve 404 soft-skip and aggregate block pairs to tract-pair CSV |
| `acs-housing` | Census ACS API JSON | Generic HTTP cacheline when present | Preserve 404 soft-skip and derive housing CSV |
| `adjacency` | GitHub release assets | Adapter-required | Keep `gh release download --release` path |
| `elections` | Python downloader | Adapter-required | Remains outside `bisect fetch` for now |

## CLI Contract

`bisect fletch-sources` is non-mutating except for writing the handoff ledger.

```text
bisect fletch-sources --year 2020 --states VT --type tiger --gate
```

It expands the same state/year/type surface as `bisect fetch`, builds a BISECT
FLETCH registry in memory, writes `data/fletch-source-handoff.csv`, and fails
`--gate` only on registry or validation defects. Adapter-required rows are
allowed because they are explicit handoffs, not hidden failures.

## Fetch Contract

`bisect fetch` keeps the existing user-facing behavior:

- local manifest overrides still apply before built-in manifest data;
- `--check-only` still prints the existing check report;
- `--force` forces the FLETCH cache acquisition and BISECT target refresh;
- GitHub release adjacency stays with `gh`;
- source bytes land in `data/.fletch`;
- BISECT writes the same local targets and done markers as before.

## Validation Floor

The migration is accepted when:

- `bisect fletch-sources --gate` passes for the default source surface;
- focused fetch tests compile and pass;
- existing BISECT fetch tests continue to protect done-marker behavior;
- formatting and diff checks pass.

