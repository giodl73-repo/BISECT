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

## Overview

Comprehensive validation of census data, adjacency graphs, and pipeline outputs. Ensures all prerequisites are met before running redistricting and identifies missing or corrupted data.

## When to Use This Skill

- User says: "Validate the data"
- User says: "Check if data is ready"
- Before running full 50-state redistricting
- After downloading new census data
- Debugging missing data errors

## Workflow

### Step 1: Validate Census Tract Data

Check for all 50 states + DC:

```bash
python scripts/data/validation/validate_tract_data.py --year 2020
```

**Validates**:
- ✓ Tract data files exist for all states
- ✓ Required fields present (GEOID, population, geometry)
- ✓ No missing GEOIDs
- ✓ All populations > 0
- ✓ Geometries are valid polygons
- ✓ File format is correct (Parquet)

**Example output**:
```
Validating tract data for 2020...

[OK] california: 9,000 tracts, all fields present
[OK] texas: 5,400 tracts, all fields present
[FAIL] wyoming: File not found
[WARN] hawaii: 12 tracts with invalid geometries

Summary:
- 49/51 states complete (96%)
- Missing: wyoming
- Issues: hawaii (12 invalid geometries)
```

### Step 2: Validate Adjacency Graphs

Check graph quality:

```bash
python scripts/data/validation/validate_adjacency_graphs.py --year 2020
```

**Validates**:
- ✓ Graph files exist for all states
- ✓ Contains 'adjacency' and 'edge_weights' dicts
- ✓ Number of nodes matches tract count
- ✓ Graph is connected (single component)
- ✓ Edge weights are positive integers
- ✓ No isolated nodes (all tracts have neighbors)
- ✓ Average degree is reasonable (2-4)

**Example output**:
```
Validating adjacency graphs for 2020...

[OK] california: 9,000 nodes, 27,000 edges, 1 component
[OK] texas: 5,400 nodes, 16,200 edges, 1 component
[FAIL] wyoming: File not found
[FAIL] hawaii: 2 connected components (disconnected islands)

Summary:
- 48/51 states valid (94%)
- Missing: wyoming
- Connectivity issues: hawaii (needs water connections)
```

### Step 3: Validate Pipeline Outputs

Check redistricting outputs if already run:

```bash
python scripts/validation/validate_pipeline_outputs.py \
  --year 2020 \
  --version v1
```

**Validates**:
- ✓ District assignment files exist
- ✓ All tracts assigned to districts
- ✓ Population balance within ±0.5%
- ✓ Compactness analysis complete
- ✓ Maps generated
- ✓ Dashboard created

**Example output**:
```
Validating pipeline outputs: us_2020_v1

Stage 1: Redistricting
[OK] 50/51 states complete (98%)
[FAIL] wyoming: districts.csv not found

Stage 2: Analysis
[OK] Compactness: 50/51 states
[OK] Political: 50/51 states (2020 election data available)
[OK] Demographic: 50/51 states

Stage 3: Visualization
[OK] State maps: 300/306 maps (98%)
[FAIL] wyoming: Missing 6 maps

Stage 4: Post-Processing
[OK] National maps: 5/5 complete
[OK] Metro area maps: 20/20 complete
[OK] Dashboard: index.html exists

Overall: 97% complete
Missing: wyoming (all stages)
```

### Step 4: Check Data Quality

Run quality checks:

```bash
python scripts/data/validation/check_data_quality.py --year 2020
```

**Checks**:
- Population distributions (reasonable values)
- Geometric validity (no self-intersections)
- GEOID formats (11 characters, correct prefixes)
- Demographic totals (sum to total population)
- Edge weight distributions (no extreme outliers)

**Example output**:
```
Data Quality Report: 2020

Population:
- Total: 331,449,281
- Min tract: 0 (FAIL - should be > 0)
- Max tract: 124,584 (OK)
- Mean: 4,524
- States with 0-pop tracts: alaska (2 tracts)

Geometries:
- Valid: 72,998/73,000 (99.99%)
- Invalid: 2 (hawaii)
- Self-intersecting: 0

GEOIDs:
- Format valid: 73,000/73,000 (100%)
- Length 11: 73,000/73,000 (100%)

Edge Weights:
- Min: 10 cm (point adjacencies)
- Max: 98,432 cm (~1km) (OK)
- Mean: 2,145 cm (~21m)
- Outliers (>100km): 0
```

## Validation Levels

**Level 1: Quick Check** (1-2 minutes)
```bash
# Just check files exist
ls data/tracts/2020/*.parquet | wc -l  # Should be 51
ls data/adjacency/2020/*.pkl | wc -l   # Should be 51
```

**Level 2: Standard Validation** (5-10 minutes)
```bash
# Check files + basic fields
python scripts/data/validation/validate_tract_data.py --year 2020
python scripts/data/validation/validate_adjacency_graphs.py --year 2020
```

**Level 3: Deep Validation** (30-60 minutes)
```bash
# Full quality checks including geometric validation
python scripts/data/validation/check_data_quality.py --year 2020 --deep
```

## Common Data Issues

**Issue 1: Missing States**:
```
Missing: wyoming, alaska
Solution: Run /census-download for missing states
```

**Issue 2: Invalid Geometries**:
```
hawaii: 12 invalid geometries
Solution: Repair with buffer(0) in tract data file
```

**Issue 3: Disconnected Graphs**:
```
hawaii: 2 connected components
Solution: Rebuild adjacency with water connections enabled
```

**Issue 4: Zero Population Tracts**:
```
alaska: 2 tracts with population = 0
Solution: Remove uninhabited tracts or merge with neighbors
```

**Issue 5: GEOID Format Errors**:
```
california: 5 GEOIDs with length != 11
Solution: Check GEOID field name (GEOID vs GEOID10 vs CTIDFP00)
```

## Validation Checklist

Before running full pipeline:

**Census Data**:
- [ ] All 51 files exist (50 states + DC)
- [ ] GEOID field present and correct type (string)
- [ ] Population field present, all > 0
- [ ] Geometry field present, all valid
- [ ] Demographic fields present (optional but recommended)

**Adjacency Graphs**:
- [ ] All 51 graph files exist
- [ ] All graphs have single connected component
- [ ] Node counts match tract counts
- [ ] Edge weights are positive integers
- [ ] No isolated nodes

**Optional Data**:
- [ ] Election data available (if doing political analysis for 2020)
- [ ] CBSA data available (if doing metro area maps)
- [ ] Historical data available (if doing comparisons)

## Output Formats

**Validation Report (JSON)**:
```json
{
  "year": 2020,
  "census_data": {
    "complete": 49,
    "total": 51,
    "missing": ["wyoming", "alaska"],
    "issues": {
      "hawaii": "12 invalid geometries"
    }
  },
  "adjacency_graphs": {
    "complete": 48,
    "total": 51,
    "connectivity_issues": ["hawaii"]
  },
  "overall_readiness": "94%"
}
```

**Summary Statistics**:
- Total tracts: 73,000 (expected ~73,000 for US)
- Total population: 331M (matches Census total)
- States ready: 48/51
- Estimated pipeline runtime: 2-4 hours

## What You'll Get

- **Completeness report**: Which states have data
- **Quality report**: Data quality issues identified
- **Readiness assessment**: Whether pipeline can run
- **Issue list**: Specific problems to fix
- **Recommendations**: Next steps to resolve issues

## Next Steps

**If validation passes (95%+)**:
- Run full pipeline with `/run-redistricting`
- Start with print-only mode
- Then small state test
- Then full 50-state run

**If validation fails (<95%)**:
- Fix missing data:
  - Use `/census-download` for missing tract data
  - Use `/adjacency-build` for missing graphs
- Fix quality issues:
  - Repair invalid geometries
  - Rebuild graphs with water connections
  - Remove/merge zero-population tracts
- Re-run validation

## Related Scripts

- `scripts/data/validation/validate_tract_data.py` - Census data validator
- `scripts/data/validation/validate_adjacency_graphs.py` - Graph validator
- `scripts/validation/validate_pipeline_outputs.py` - Output validator
- `scripts/data/validation/check_data_quality.py` - Quality checker
