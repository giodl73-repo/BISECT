---
name: data-validate
description: Validate data completeness and quality for redistricting pipeline. Use before running pipeline to check census tracts, adjacency graphs, and required fields are present for all states and years.
allowed-tools:
  - Read
  - Bash
  - Glob
  - Grep
user-invocable: true
---

# Data Validation Skill

Comprehensive validation of census data, adjacency graphs, pipeline outputs. Ensures prerequisites met before redistricting, identifies missing/corrupted data.

## When to Use
User says "Validate data/Check if data ready", before full 50-state redistricting, after downloading census data, debugging missing data errors

## Workflow

### Step 1: Validate Census Tract Data
```bash
python scripts/data/validation/validate_tract_data.py --year 2020
```

**Validates**: Tract files exist (all states), required fields (GEOID, population, geometry), no missing GEOIDs, populations > 0, valid polygon geometries, correct format (Parquet)

**Output**:
```
[OK] california: 9,000 tracts, all fields present
[FAIL] wyoming: File not found
[WARN] hawaii: 12 invalid geometries
Summary: 49/51 states (96%), Missing: wyoming, Issues: hawaii
```

### Step 2: Validate Adjacency Graphs
```bash
python scripts/data/validation/validate_adjacency_graphs.py --year 2020
```

**Validates**: Graph files exist, contains 'adjacency' + 'edge_weights' dicts, node count matches tracts, graph connected (single component), edge weights positive integers, no isolated nodes, avg degree reasonable (2-4)

**Output**:
```
[OK] california: 9,000 nodes, 27,000 edges, 1 component
[FAIL] hawaii: 2 components (disconnected islands)
Summary: 48/51 states (94%), Connectivity issues: hawaii (needs water connections)
```

### Step 3: Validate Pipeline Outputs
```bash
python scripts/validation/validate_pipeline_outputs.py --year 2020 --version v1
```

**Validates**: District assignments exist, all tracts assigned, population balance ±0.5%, compactness analysis complete, maps generated, dashboard created

**Output**:
```
Stage 1 - Redistricting: [OK] 50/51 states (98%), [FAIL] wyoming
Stage 2 - Analysis: [OK] Compactness/Political/Demographic 50/51 states
Stage 3 - Visualization: [OK] 300/306 maps (98%)
Stage 4 - Post-Processing: [OK] National maps 5/5, Metro 20/20, Dashboard exists
Overall: 97% complete, Missing: wyoming (all stages)
```

### Step 4: Check Data Quality
```bash
python scripts/data/validation/check_data_quality.py --year 2020
```

**Checks**: Population distributions (reasonable), geometric validity (no self-intersections), GEOID formats (11 chars, correct prefixes), demographic totals (sum to total pop), edge weight distributions (no extreme outliers)

**Output**:
```
Population: Total 331M, Min 0 (FAIL), Max 124K (OK), Mean 4.5K, 0-pop: alaska (2)
Geometries: Valid 72,998/73,000 (99.99%), Invalid: 2 (hawaii)
GEOIDs: Format valid 73,000/73,000 (100%)
Edge Weights: Min 10cm, Max 98,432cm (~1km OK), Mean 2,145cm (~21m), Outliers >100km: 0
```

## Validation Levels

**Level 1 - Quick** (1-2 min): Check files exist: `ls data/tracts/2020/*.parquet | wc -l` (51), `ls data/adjacency/2020/*.pkl | wc -l` (51)

**Level 2 - Standard** (5-10 min): Check files + basic fields:
```bash
python scripts/data/validation/validate_tract_data.py --year 2020
python scripts/data/validation/validate_adjacency_graphs.py --year 2020
```

**Level 3 - Deep** (30-60 min): Full quality checks + geometric validation:
```bash
python scripts/data/validation/check_data_quality.py --year 2020 --deep
```

## Common Issues

**Issue 1 - Missing States**: Missing wyoming, alaska → `/census-download` for missing
**Issue 2 - Invalid Geometries**: hawaii: 12 invalid → Repair with `buffer(0)` in tract data
**Issue 3 - Disconnected Graphs**: hawaii: 2 components → Rebuild adjacency with water connections
**Issue 4 - Zero Population**: alaska: 2 tracts pop=0 → Remove uninhabited or merge with neighbors
**Issue 5 - GEOID Format**: california: 5 GEOIDs length≠11 → Check field name (GEOID vs GEOID10 vs CTIDFP00)

## Validation Checklist

**Census Data**:
- [ ] All 51 files exist (50 states + DC)
- [ ] GEOID field present, correct type (string)
- [ ] Population field present, all > 0
- [ ] Geometry field present, all valid
- [ ] Demographic fields present (optional)

**Adjacency Graphs**:
- [ ] All 51 graph files exist
- [ ] All graphs single connected component
- [ ] Node counts match tract counts
- [ ] Edge weights positive integers
- [ ] No isolated nodes

**Optional Data**:
- [ ] Election data (if political analysis for 2020)
- [ ] CBSA data (if metro area maps)
- [ ] Historical data (if comparisons)

## Output Formats

**JSON Report**:
```json
{
  "year": 2020,
  "census_data": {"complete": 49, "total": 51, "missing": ["wyoming", "alaska"], "issues": {"hawaii": "12 invalid geometries"}},
  "adjacency_graphs": {"complete": 48, "total": 51, "connectivity_issues": ["hawaii"]},
  "overall_readiness": "94%"
}
```

**Summary Stats**: Total tracts ~73K (expected), pop 331M (matches Census), states ready 48/51, estimated runtime 2-4h

## What You'll Get
Completeness report (which states have data), quality report (issues identified), readiness assessment (can pipeline run), issue list (specific problems), recommendations (next steps)

## Next Steps

**Validation passes (95%+)**: Run `/run-redistricting` → print-only mode → small state test → full 50-state
**Validation fails (<95%)**: Fix missing (use `/census-download` for tracts, `/adjacency-build` for graphs), fix quality (repair invalid geometries, rebuild graphs with water, remove/merge 0-pop tracts), re-run validation

## Related Scripts
`scripts/data/validation/validate_tract_data.py` (census validator), `scripts/data/validation/validate_adjacency_graphs.py` (graph validator), `scripts/validation/validate_pipeline_outputs.py` (output validator), `scripts/data/validation/check_data_quality.py` (quality checker)
