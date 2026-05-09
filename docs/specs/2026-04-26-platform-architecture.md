# BISECT Platform Architecture

**Date**: 2026-04-26
**Scope**: Full platform including practitioner toolkit (Specs 0–6)

---

## Crate workspace

```
BISECT/ (Cargo workspace)
│
├── crates/
│   │
│   ├── bisect-core/              UNCHANGED — algorithm kernel
│   │   ├── bisection.rs          BisectionTree, split scheduling
│   │   ├── vra.rs                adaptive boost formula (single truth)
│   │   ├── partition.rs          Partition, assert_balanced()
│   │   └── metis_format.rs       METIS .graph writer/parser
│   │
│   ├── bisect-data/              EXTENDED — two new modules
│   │   ├── tiger.rs              TIGER .shp reader → WKB output
│   │   ├── adjacency.rs          R-tree candidate detection + Rayon intersection
│   │   ├── bridge.rs             Union-Find island bridging
│   │   ├── serialize.rs          RADJ binary format v2
│   │   ├── adjacency_loader.rs   .adj.bin first, pkl shim fallback
│   │   ├── enacted.rs            NEW (Spec 2) download enacted district
│   │   │                         shapefiles + PIP tract assignment
│   │   └── geography.rs          NEW (Spec 3) place-tract relationship
│   │                             files (Census geographic relationship files)
│   │
│   ├── bisect-analysis/          EXTENDED — five new analyzer modules
│   │   ├── analyzer.rs           Analyzer trait + AnalyzerContext + AnalyzerType
│   │   ├── demographic.rs        race/ethnicity per district (total pop basis)
│   │   ├── political.rs          partisan aggregation, is_uncontested
│   │   ├── compactness.rs        PP, Reock, CHR metrics
│   │   ├── urban.rs              largest city per district
│   │   ├── summary.rs            merged district_summary + pop_balance_ok
│   │   ├── vra_analysis.rs       majority-minority district analysis (VAP)
│   │   ├── contiguity.rs         NEW (Spec 3) BFS per district,
│   │   │                         component count, disconnected tract list
│   │   ├── splits.rs             NEW (Spec 3) county splits (GEOID-based,
│   │   │                         no extra data), municipal splits (place-tract)
│   │   ├── partisan.rs           NEW (Spec 4) efficiency gap, mean-median,
│   │   │                         partisan bias, bootstrap CI (suppressed N<10)
│   │   └── comparison.rs         NEW (Spec 2) Jaccard similarity, metric
│   │                             comparison between two plans
│   │
│   ├── bisect-map/               UNCHANGED — native Rust SVG→PNG rendering
│   │   ├── projection.rs         equirectangular + InsetProjection (AK/HI)
│   │   ├── dissolve.rs           WKB decode + geo::BooleanOps union
│   │   ├── colorscheme.rs        categorical + choropleth schemes
│   │   ├── labeler.rs            adaptive font sizing, polylabel, halo SVG
│   │   ├── renderer.rs           SVG assembly + resvg PNG + Liberation Sans
│   │   └── rounds.rs             BisectionTree lineage tracking
│   │
│   ├── bisect-report/            NEW CRATE (Specs 0, 2, 6)
│   │   ├── rplan.rs              RPLAN v0.1 reader/writer/validator
│   │   │                         — the open redistricting plan interchange format
│   │   ├── report.rs             report assembly — collects all analysis outputs
│   │   ├── html.rs               tera template rendering → self-contained HTML
│   │   ├── pdf.rs                printpdf pure-Rust PDF (or wkhtmltopdf shim)
│   │   ├── export.rs             GeoJSON (RFC 7946), ESRI shapefile,
│   │   │                         GerryChain v2.3, CSV export
│   │   ├── import.rs             GeoJSON/shapefile → PIP tract assignment
│   │   │                         (point-in-polygon, nearest fallback)
│   │   └── audit.rs              chain-of-custody, SHA-256, verification
│   │                             command generation
│   │
│   ├── bisect-cli/               EXTENDED — four new modules + extended args
│   │   ├── args.rs               +AnalyzeArgs (contiguity/splits/partisan)
│   │   │                         +StateArgs (--districts, --chamber, --label,
│   │   │                           --population-source, --balance-tolerance)
│   │   │                         +SuiteArgs, CompareArgs, ValidateArgs,
│   │   │                           ReportArgs, ExportArgs, ImportArgs, MigrateArgs
│   │   ├── runner.rs             +chamber-aware balance defaults
│   │   │                         +PlanManifest writing after each run
│   │   │                         +resolve_adjacency_path() via manifest
│   │   ├── manifest.rs           NEW — PlanManifest struct, SHA-256 of
│   │   │                         TIGER source + binary provenance
│   │   ├── analyze.rs            +contiguity, splits, partisan dispatch
│   │   │                         +bitfield exit codes (1/2/4/8)
│   │   ├── compare.rs            NEW (Spec 2) — BISECT compare dispatcher
│   │   ├── suite.rs              NEW (Spec 5) — BISECT suite create/validate
│   │   ├── nesting.rs            NEW (Spec 5) — validate_nesting(),
│   │   │                         build_chamber_adjacency() (primary component)
│   │   ├── migrate.rs            NEW — BISECT migrate: states/ → plans/ tree
│   │   ├── geometry.rs           shared dissolve helper (existing)
│   │   ├── adjacency_loader.rs   .adj.bin + pkl shim (existing)
│   │   ├── map_cmd.rs            state + national maps (existing)
│   │   └── aggregate.rs          national rollup (existing)
│   │
│   └── bisect-web/               STUB — dashboard HTML (Python still used)
│
└── python/
    └── bisect_py/                PyO3 bindings (unchanged)
```

---

## Output directory structure

Two trees coexist — legacy (unlabeled) and labeled (new):

```
outputs/{version}/{year}/
│
├── states/{state_name}/          LEGACY — unlabeled bisect state runs
│   ├── data/
│   │   ├── final_assignments.json
│   │   └── vra_analysis.json
│   ├── analysis/
│   │   ├── demographic.json
│   │   ├── political.json
│   │   ├── compactness.json
│   │   ├── contiguity.json       NEW (Spec 3)
│   │   ├── splits.json           NEW (Spec 3)
│   │   ├── partisan.json         NEW (Spec 4)
│   │   └── summary.json
│   ├── maps/
│   │   ├── districts.png
│   │   ├── political.png
│   │   ├── demographic.png
│   │   └── compactness.png
│   └── intermediate/
│       ├── depth_00/assignments.json
│       └── depth_NN/assignments.json
│
├── plans/{label}/                NEW — labeled runs (Spec 1)
│   ├── manifest.json             PlanManifest (full provenance)
│   ├── data/                     same structure as states/
│   ├── analysis/                 same structure as states/
│   ├── maps/                     same structure as states/
│   └── intermediate/
│
├── national/{year}/              national rollup
│   ├── us_demographic.json + .csv
│   ├── us_political.json + .csv
│   ├── us_compactness.json + .csv
│   ├── us_summary.json + .csv
│   └── maps/
│       ├── districts.png
│       ├── political.png
│       └── demographic.png
│
└── suites/{suite_name}/          NEW (Spec 5)
    ├── suite.json                chamber manifest + plan references
    ├── {label}_congressional.rplan
    ├── {label}_house.rplan
    └── {label}_senate.rplan
```

---

## Analysis file producer/consumer table

| File | Produced by | Consumed by |
|------|-------------|-------------|
| `analysis/demographic.json` | `bisect analyze --types demographic` | Spec 6 report §5 |
| `analysis/political.json` | `bisect analyze --types political` | Spec 2 comparison, Spec 6 §5 |
| `analysis/compactness.json` | `bisect analyze --types compactness` | Spec 6 §6 |
| `analysis/contiguity.json` | `bisect analyze --types contiguity` | Spec 5 nesting, Spec 6 §3 |
| `analysis/splits.json` | `bisect analyze --types splits` | Spec 2 comparison, Spec 6 §3 |
| `analysis/partisan.json` | `bisect analyze --types partisan` | Spec 2 comparison, Spec 6 §4 |
| `analysis/urban.json` | `bisect analyze --types urban` | Spec 6 §5 |
| `analysis/summary.json` | `bisect analyze --types summary` | Spec 6 §1 |

---

## Full CLI surface

```
# Core redistricting (extended Spec 1)
bisect state   --state WA --year 2020 --version WA_Plans
               --districts 98          override district count
               --chamber house         congressional|house|senate|custom
               --label wa_house_v1     named plan (uses plans/ tree)
               --population-source vap total|vap|cvap
               --balance-tolerance 5.0 pct (default: chamber-aware)
               --seed 42               reproducible

bisect states  --year 2020 --version WA_Plans --workers 8
BISECT run     --year all --version RustV3 --workers 12

# Multi-chamber suite (Spec 5)
BISECT suite   --state WA --year 2020 --version WA_Plans
               --name wa_commission_v1
               --house-districts 98 --senate-districts 49
               --nest senate-in-house
               --seed 42
BISECT suite validate --name wa_commission_v1

# Plan comparison (Spec 2)
BISECT compare --plan-a wa_house_v1 --plan-b wa_house_v2
               --enacted              compare vs current enacted map
               --metrics all          population|compactness|splits|partisan
               --format table         table|json|csv

# Analytics (Specs 1, 3, 4 — extended)
bisect analyze --state WA --year 2020 --version WA_Plans
               --label wa_house_v1    OR --state WA (legacy path)
               --types all            +contiguity +splits +partisan
               --election-file data/custom/wa_election.csv
               --allow-imbalance      suppress exit bit 1
               --allow-noncontiguous  suppress exit bit 2

# Aggregation + national maps (existing, extended)
BISECT aggregate --year 2020 --version WA_Plans --types all --csv
BISECT map       --state WA --year 2020 --version WA_Plans --types all
BISECT map       --scope national --version WA_Plans --year 2020

# RPLAN format (Spec 0)
BISECT validate  --file wa_house_v1.rplan
BISECT export    --label wa_house_v1 --year 2020 --version WA_Plans
                 --format geojson shapefile gerrychain csv rplan
BISECT import    --file external.rplan --state WA --year 2020 --label external_v1
BISECT migrate   --state WA --year 2020 --version WA_Plans --label wa_congressional

# Commission reports (Spec 6)
BISECT report    --label wa_house_v1 --year 2020 --version WA_Plans
                 --format html json pdf
                 --out reports/wa_house_v1/
BISECT report    --suite wa_commission_v1 --format html

# Data download (extended)
bisect fetch     --type enacted --year 2020 --states WA
bisect fetch     --type geography --year 2020 --states WA
bisect fetch     --type elections --year 2020 --states WA
```

---

## Exit code bitfield (Spec 3 R3 amendment)

| Bit | Value | Condition | Suppressed by |
|-----|-------|-----------|---------------|
| 0 | 1 | Population balance violation | `--allow-imbalance` |
| 1 | 2 | Contiguity violation | `--allow-noncontiguous` |
| 2 | 4 | Nesting violation | (not suppressible) |
| 3 | 8 | Required geographic data absent | (not suppressible) |

Exit code = OR of all active bits. `0` = all constraints satisfied.
Examples: balance only=1, contiguity only=2, both=3, nesting only=4, balance+nesting=5.

---

## Build order (implementation dependency sequence)

```
Phase A (foundation):
  bisect-report/rplan.rs        RPLAN reader/writer/validator (Spec 0)
  bisect-cli/manifest.rs        PlanManifest + SHA-256 (Spec 1)
  bisect-cli args extension     --districts --chamber --label --population-source (Spec 1)

Phase B (analytics — parallel):
  bisect-analysis/contiguity.rs    (Spec 3)
  bisect-analysis/splits.rs        (Spec 3)
  bisect-analysis/partisan.rs      (Spec 4)
  bisect-analysis/comparison.rs    (Spec 2)
  bisect-data/enacted.rs           (Spec 2)
  bisect-data/geography.rs         (Spec 3)

Phase C (multi-chamber):
  bisect-cli/nesting.rs         build_chamber_adjacency + validate_nesting (Spec 5)
  bisect-cli/suite.rs           BISECT suite command (Spec 5)
  bisect-cli/compare.rs         BISECT compare command (Spec 2)

Phase D (reports — last, depends on all):
  bisect-report/export.rs       GeoJSON/shapefile/GerryChain (Spec 6)
  bisect-report/import.rs       PIP tract assignment (Spec 6)
  bisect-report/audit.rs        chain-of-custody (Spec 6)
  bisect-report/html.rs         tera HTML templates (Spec 6)
  bisect-report/report.rs       full report assembly (Spec 6)
```
