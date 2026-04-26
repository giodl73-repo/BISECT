# Practitioner Quick Start

This guide walks a redistricting commission or state legislature through using `redist` to draw, analyze, and submit a redistricting plan.

**Example**: Washington state redistricting commission drawing house (98 districts), senate (49), and congressional (10) maps for 2020 census data.

---

## Prerequisites

```bash
# Build the binary (one-time, ~2 minutes)
cargo build --release --manifest-path redist/Cargo.toml

# Download 2020 census data (TIGER shapefiles + population data)
redist fetch --year 2020

# Convert adjacency files to fast native format (one-time)
python scripts/data/generate_adj_bin.py --year 2020

# Verify setup
redist --version
```

---

## Step 1: Draw your districts

### Option A: Draw all chambers as a suite (recommended)

A suite draws all three chambers in the right order, validates that senate districts nest inside house districts, and produces a single named plan package:

```bash
redist suite --state WA --year 2020 --version WA_Plans \
  --name wa_commission_v1 \
  --house-districts 98 \
  --senate-districts 49 \
  --nest senate-in-house \
  --seed 42
```

This produces:
- `outputs/WA_Plans/2020/plans/wa_commission_v1_house/` — 98 house districts
- `outputs/WA_Plans/2020/plans/wa_commission_v1_senate/` — 49 senate districts (nested in house)
- `outputs/WA_Plans/2020/plans/wa_commission_v1_congressional/` — 10 congressional districts
- `outputs/WA_Plans/2020/suites/wa_commission_v1/suite.json` — suite manifest

### Option B: Draw a single chamber

```bash
# House only
redist state --state WA --year 2020 --version WA_Plans \
  --districts 98 --chamber house --label wa_house_v1 \
  --balance-tolerance 5.0 --seed 42

# Congressional (uses official district count from census config)
redist state --state WA --year 2020 --version WA_Plans \
  --chamber congressional --label wa_congress_v1 \
  --balance-tolerance 0.5 --seed 42
```

### Reproducibility

The `--seed` flag makes every run exactly reproducible. Same seed + same data = same map. Record the seed in your commission minutes.

```bash
# Run with a specific seed
redist state --state WA --districts 98 --chamber house \
  --label wa_house_final --seed 7823461 --year 2020 --version WA_Plans
```

### Population source

By default, districts are equalized by **total population** (required for congressional districts). State legislative chambers may use different bases:

```bash
# Citizen voting-age population (CVAP) — some states require this
redist state --state WA --districts 98 --chamber house \
  --population-source cvap --label wa_house_cvap_v1

# Voting-age population (VAP)
redist state --state WA --districts 98 --chamber house \
  --population-source vap --label wa_house_vap_v1
```

---

## Step 2: Analyze your plan

```bash
# Run all analyses (demographic, political, compactness, contiguity, splits, VRA)
redist analyze --label wa_house_v1 --year 2020 --version WA_Plans --types all

# Or specific types
redist analyze --label wa_house_v1 --year 2020 --version WA_Plans \
  --types contiguity splits compactness
```

### What to check

**Contiguity** (`analysis/contiguity.json`): Every district must be geographically connected. If `all_contiguous: false`, the plan has disconnected districts and must be redrawn.

**County splits** (`analysis/splits.json`): WA constitution requires minimizing county splits. Check `counties.split` and `counties.legal_standard` — the output includes WA's specific constitutional language.

**Population balance** (`analysis/summary.json`): Check `population_balance_valid: true`. Congressional maps must be within ±0.5%; state legislative maps typically within ±5%.

**Partisan fairness** (`analysis/partisan.json`): Efficiency gap, mean-median difference, partisan bias — each with 95% confidence interval. These are not legal requirements but are commonly scrutinized in litigation.

---

## Step 3: Compare to the enacted map

```bash
# Download the current enacted WA house districts from Census
redist fetch --type enacted --year 2020 --states WA

# Compare your plan to enacted
redist compare --plan-a wa_house_v1 --enacted \
  --year 2020 --version WA_Plans \
  --format table json
```

This shows side-by-side:
- Population equality (your plan vs enacted)
- Compactness (your plan vs enacted)
- County splits (your plan vs enacted)
- Geographic stability (Jaccard similarity — how much did the map change?)
- Partisan metrics (both plans)

---

## Step 4: Generate the commission report

```bash
# Full formal report (HTML + JSON)
redist report --label wa_house_v1 --year 2020 --version WA_Plans \
  --format html json \
  --out reports/wa_house_v1/

# Suite report (all three chambers)
redist report --suite wa_commission_v1 --year 2020 --version WA_Plans \
  --format html --out reports/wa_commission_v1/
```

The HTML report is self-contained — email it, print it, or submit it to the public comment docket. No external dependencies needed to open it.

---

## Step 5: Export for external review

```bash
# GeoJSON (for DRA, PlanScore, QGIS)
redist export --label wa_house_v1 --year 2020 --version WA_Plans \
  --format geojson --out exports/

# Shapefile (for Maptitude, ArcGIS)
redist export --label wa_house_v1 --format shapefile --out exports/

# RPLAN (open format, verifiable by any tool)
redist export --label wa_house_v1 --format rplan --out exports/

# All formats at once
redist export --label wa_house_v1 --format geojson shapefile rplan csv \
  --out exports/wa_house_v1/
```

Submit `exports/wa_house_v1/wa_house_v1.rplan` to the public record. Any party can independently verify the plan using `redist validate`.

---

## Step 6: Audit trail

Every plan produced by `redist` has a machine-verifiable audit trail. Check `manifest.json` in the plan directory:

```bash
cat outputs/WA_Plans/2020/plans/wa_house_v1/manifest.json
```

The manifest contains:
- SHA-256 of the source TIGER shapefile (from Census.gov)
- SHA-256 of the adjacency data
- Binary version and download URL
- Exact command to reproduce the plan
- Timestamp

To reproduce the plan on any machine:

```bash
# The manifest shows you exactly what to run:
# "verification_command": "redist state --state WA --year 2020 --version WA_Plans --districts 98 --chamber house --label wa_house_v1 --seed 42 --balance-tolerance 5.0"

redist state --state WA --year 2020 --version WA_Plans \
  --districts 98 --chamber house --label wa_house_v1_verify \
  --seed 42 --balance-tolerance 5.0
```

The output assignments should be identical to the original.

---

## Iterating on plans

```bash
# Draft 1
redist state --state WA --districts 98 --chamber house \
  --label wa_house_draft1 --seed 42 --version WA_Plans

# Draft 2 (different seed)
redist state --state WA --districts 98 --chamber house \
  --label wa_house_draft2 --seed 7823 --version WA_Plans

# Compare the two drafts
redist compare --plan-a wa_house_draft1 --plan-b wa_house_draft2 \
  --year 2020 --version WA_Plans

# Analyze both
redist analyze --label wa_house_draft1 --types all
redist analyze --label wa_house_draft2 --types all

# Side-by-side report
redist report --label wa_house_draft1 wa_house_draft2 --format html
```

---

## Next steps

- [Working with external plans](./importing-external-plans.md) — import from DRA, PlanScore, or any GeoJSON
- [Understanding partisan metrics](./partisan-metrics-guide.md) — EG, MM, PB explained
- [County and municipal preservation](./splits-guide.md) — state-specific standards
- [Multi-chamber nesting](./nesting-guide.md) — senate-in-house requirements
- [Court submission guide](./court-submission-guide.md) — audit trail and expert witness package
