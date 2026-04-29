# Practitioner Redistricting Toolkit — Overview

**Date**: 2026-04-26
**Specs**: 6 total (see individual spec files)

---

## Vision

The `redist` binary becomes the open-source reference implementation for redistricting practitioners — the tool a Washington state commission, a court-appointed special master, or an academic expert uses to generate, analyze, compare, and certify redistricting plans.

---

## The six specs

| Spec | Name | What it enables |
|------|------|----------------|
| [Spec 1](2026-04-26-spec1-custom-parameters.md) | Custom Parameters | Any chamber, any district count, any population source, reproducible seeds, signed audit manifest |
| [Spec 2](2026-04-26-spec2-plan-comparison.md) | Plan Comparison | Generated vs enacted, plan A vs plan B, enacted map download, Jaccard similarity |
| [Spec 3](2026-04-26-spec3-constraint-analysis.md) | Constraint Analysis | Contiguity, county splits, municipal splits, external format import/export |
| [Spec 4](2026-04-26-spec4-partisan-metrics.md) | Partisan Metrics | Efficiency gap, mean-median, partisan bias, 95% CI, external election data |
| [Spec 5](2026-04-26-spec5-multi-chamber.md) | Multi-Chamber / Nested | House + senate + congressional as a suite, nesting validation, house-district adjacency graph |
| [Spec 6](2026-04-26-spec6-commission-reports.md) | Commission Reports | HTML/PDF/JSON formal reports, chain-of-custody audit, GeoJSON/shapefile export, external tool import, plugin analyzer hook |

---

## Dependency order (implement in this sequence)

```
Spec 1  ──► Spec 2
        ──► Spec 3  ──► Spec 5
        ──► Spec 4
All 5   ──► Spec 6 (terminal)
```

Spec 1 is the foundation. Specs 2, 3, 4 can be implemented in parallel after Spec 1. Spec 5 requires Spec 3 (contiguity for nesting validation). Spec 6 requires all five.

---

## Shared data model (cross-spec contracts)

### `Plan` (Spec 1)
The atomic unit. Every spec produces or consumes plans.

```
outputs/{version}/{year}/plans/{label}/
  manifest.json            ← PlanManifest (chain of custody)
  final_assignments.json   ← tract GEOID → district ID
  analysis/                ← Specs 3, 4 outputs
  maps/                    ← redist map outputs
  intermediate/            ← bisection rounds
```

### `PlanManifest` (Spec 1)
Full provenance record written at plan creation. Includes: seed, binary SHA-256, all input file SHA-256s, config used, timestamp.

### `AnalyzerType` extension (Specs 3, 4)
New values added to existing enum: `Contiguity`, `Splits`, `Partisan`. Same extension model as demographic/political/compactness.

### `PlanSuite` (Spec 5)
A named collection of plans for multiple chambers of the same state/year.

### `Report` (Spec 6)
Aggregates all spec outputs into HTML/PDF/JSON. Includes `redist export` and `redist import` for external tool interoperability.

---

## External tool interoperability (Spec 3 + Spec 6)

A key design principle: `redist` is an **open platform**, not a closed system.

**Import any plan** for analysis:
- GeoJSON district polygons (DRA, PlanScore, Districtr)
- ESRI shapefile (Maptitude, ArcGIS)
- CSV tract-to-district (any tool)
- MGGG / GerryChain JSON format

**Export any plan** to any tool:
- Same formats as import

**Plugin analyzer hook** (Spec 6):
```bash
redist analyze --label plan1 --types external \
  --external-analyzer "python my_org_analyzer.py {assignments_json} {output_dir}"
```
Orgs with proprietary split-scoring methods or court-mandated tools can plug them in without modifying `redist`.

**PlanScore integration** (Spec 4):
Export GeoJSON compatible with PlanScore's API for independent partisan fairness verification.

---

## Washington state walkthrough

```bash
# Step 1: Draw all three chambers
redist suite --state WA --year 2020 --version WA_Plans \
  --name wa_commission_v1 \
  --congressional-districts 10 \
  --house-districts 98 \
  --senate-districts 49 \
  --nest senate-in-house \
  --seed 42

# Step 2: Run all analytics for each chamber
redist analyze --suite wa_commission_v1 --types all

# Step 3: Compare house plan vs enacted house districts
redist compare \
  --plan-a wa_commission_v1_house \
  --enacted --chamber house \
  --year 2020 --version WA_Plans

# Step 4: Generate formal commission report
redist report --suite wa_commission_v1 \
  --format html pdf json \
  --out reports/wa_commission_v1/

# Step 5: Export for external review
redist export --suite wa_commission_v1 \
  --format geojson shapefile \
  --out exports/wa_commission_v1/

# Step 6: Import an alternative plan from DRA for comparison
redist import --file dra_alternative.geojson \
  --state WA --year 2020 --label wa_dra_alternative
redist compare --plan-a wa_commission_v1_house --plan-b wa_dra_alternative
```

---

## New CLI commands added across all 6 specs

| Command | Spec | Description |
|---------|------|-------------|
| `redist state --districts N --chamber X --label Y --population-source Z` | 1 | Custom chamber redistricting |
| `redist suite` | 5 | Multi-chamber plan creation and validation |
| `redist compare` | 2 | Plan-to-plan or plan-to-enacted comparison |
| `redist analyze --types contiguity splits partisan` | 3, 4 | New analyzer types |
| `redist report` | 6 | Formal commission report (HTML/PDF/JSON) |
| `redist export` | 6 | GeoJSON/shapefile/CSV export |
| `redist import` | 6 | Import external plan for analysis |
| `redist fetch --type enacted geography` | 2, 3 | Download enacted districts + geographic relationship files |

---

## New crates

| Crate | Spec | Description |
|-------|------|-------------|
| (extends `redist-analysis`) | 1, 3, 4 | New analyzer modules: contiguity, splits, partisan |
| (extends `redist-cli`) | 1, 2, 5 | New subcommands: suite, compare |
| `redist-compare` | 2 | Plan comparison logic |
| `redist-report` | 6 | Report generation, export, import |

---

## Implementation priority

For Washington's immediate needs (house + senate redistricting):
1. **Spec 1** (custom parameters) — enables `--districts 98`, `--chamber house`
2. **Spec 3** (constraint analysis) — county splits are a WA constitutional requirement
3. **Spec 5** (nesting) — senate nesting in house is required

For a full commission submission:
4. **Spec 4** (partisan metrics)
5. **Spec 2** (comparison vs enacted)
6. **Spec 6** (formal report)

---

## R3 Board Review Amendments (2026-04-26)

**[DATUM] CONCERN — Finding 8: Normative analysis file table**
The cross-spec data contracts for `analysis/` files were implicit. Making them normative ensures each spec knows exactly what it produces and what it consumes.

Normative analysis file table:

| Filename | Produced by | Consumed by |
|----------|-------------|-------------|
| `analysis/demographic.json` | Spec 1 (analyzer) | Spec 6 (report) |
| `analysis/political.json` | Spec 1 | Spec 6 |
| `analysis/compactness.json` | Spec 1 | Spec 6 |
| `analysis/contiguity.json` | Spec 3 | Spec 5 (nesting), Spec 6 |
| `analysis/splits.json` | Spec 3 | Spec 2 (comparison), Spec 6 |
| `analysis/partisan.json` | Spec 4 | Spec 2 (comparison), Spec 6 |
| `analysis/summary.json` | Spec 1 | Spec 6 |

Each file is written atomically to `outputs/{version}/{year}/plans/{label}/analysis/`. Consumers must not read a file that does not exist; they must check for presence and degrade gracefully (e.g., omit the report section with a note: "Analysis not available — run `redist analyze --types partisan` first").

**[LEDGER] CRITICAL — Path convention migration (cross-spec)**
`plans/{label}/` tree (Spec 1) vs legacy `states/{state_name}/` tree (existing CLI). Both trees are preserved. Unlabeled runs continue using `states/{state_name}/`. Labeled runs use `plans/{label}/`. `redist analyze` and `redist map` accept either `--state` (legacy path) or `--label` (new path). A `redist migrate --state WA --label wa_congressional_2020` command copies a legacy plan into the new tree. See Spec 0 R3 amendments for full detail.
