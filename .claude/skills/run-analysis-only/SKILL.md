---
name: run-analysis-only
description: Run analysis stages without redistricting when district assignments already exist. Use when you want to regenerate analysis, update maps, or add new analysis types without rerunning METIS partitioning.
allowed-tools:
  - Read
  - Bash
  - Glob
  - Grep
user-invocable: true
---

# Run Analysis Only

Execute analysis stages (compactness/political/demographic) + regenerate visualizations without rerunning redistricting. Useful when districts exist and you want updated analysis or changed map styling.

## Prerequisites
District assignments exist (`outputs/us_{year}_{version}/states/*/data/districts.csv`), census tract data available, 2020 election data for political analysis

## When to Use
User says "Regenerate analysis/Update maps without redistricting", districts exist + want new analysis type, change map DPI/styling, testing new analysis code

## Workflow

### Step 1: Verify Districts Exist
```bash
ls outputs/us_2020_v1/states/*/data/districts.csv     # Check files
python scripts/validation/validate_pipeline_outputs.py --year 2020 --version v1  # Or validate
```
If missing → `/run-redistricting` first

### Step 2: Run with Skip Flag
```bash
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version v1 --skip-redistricting
```

**Executes**: ✓ Per-state analysis (parallel), ✓ State maps (parallel), ✓ Post-processing (national maps, metro), ✓ Dashboard, ✗ Redistricting (skipped)

### Step 3: Monitor Progress
Progress bars show: Analysis stages per state (compactness/political/demographic), map generation per state, national aggregation, dashboard generation
**Runtime**: ~1-2h (much faster than full redistricting)

## Common Use Cases

**Use Case 1 - Change Map Resolution**: Original DPI 150 → Regenerate DPI 300: `--dpi 300 --skip-redistricting --force` (use `--force` to override skip logic)

**Use Case 2 - Add Political Analysis**: Original run 2010 without election data → Download 2020 election data → Rerun: `--skip-redistricting`

**Use Case 3 - Fix Analysis Bug**: Fix bug in compactness code → Regenerate: `--skip-redistricting --force`

**Use Case 4 - Test New Analysis**: Add new analysis script → Test: `--skip-redistricting --states "VT,DE"`

## What Gets Regenerated

**Per-State** (parallel): `compactness.csv` + maps, `political_lean.csv` + maps (2020), `demographic_composition.csv` + maps, all state maps in `maps/`
**National** (post-processing): `maps/us_all_districts.png`, `maps/us_political_lean.png` (2020), `maps/us_demographic_*.png`, `maps/rounds/round_*.png`
**Metro Areas** (if CBSA data): Top 20 metro focused maps
**Dashboard**: `index.html` with updated data

## What Does NOT Change
`districts.csv` (assignments), `district_summary.csv`, `rounds_hierarchy.csv`, METIS partitioning (not rerun)

## Performance

| Task | With Redistricting | Analysis Only |
|------|-------------------|---------------|
| Small (VT) | 30s | 10s |
| Medium (AL) | 2min | 30s |
| Large (CA) | 5min | 2min |
| All 50 states | 2-4h | 1-2h |

Analysis-only **~50-60% faster** (METIS skipped)

## Combining Flags

```bash
# Analysis + specific states + force
--skip-redistricting --states "CA,TX,FL" --force

# Analysis + validation
--skip-redistricting --validate

# Analysis + print only
--skip-redistricting --print-only
```

## Error Handling

**Missing districts**: `Error: District assignments not found` → Run full redistricting (remove `--skip-redistricting`)
**Mismatched versions**: `Error: Census year doesn't match` → Ensure `--year` matches original run

## What You'll Get
Updated analysis CSVs (latest algorithms), regenerated maps (current styling/DPI), updated dashboard (refreshed data), all without expensive METIS rerun

## Next Steps
Review updated dashboard, compare results before/after, `/validate-compactness` to check metrics, `/run-statistical-analysis` for quantitative comparison
