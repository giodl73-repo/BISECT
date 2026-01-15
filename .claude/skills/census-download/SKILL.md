---
name: census-download
description: Download census tract data for a specific year and state. Use when you need population, demographic, or geographic data for 2000, 2010, or 2020. Handles year-specific data sources and formats.
allowed-tools:
  - Read
  - Write
  - Bash
  - Glob
  - WebFetch
user-invocable: true
---

# Census Data Download Skill

## Overview

Download census tract-level population, demographics, and geographic boundaries for use in redistricting. Handles year-specific data sources, API access, and data format conversions.

## When to Use This Skill

- User says: "Download census data for [year]"
- User says: "I need tract data for [state]"
- Pipeline fails with missing tract data
- Starting new redistricting project

## Workflow

### Step 1: Determine Data Source

Census data sources vary by year:

**2020 Census**:
- Source: Census API (requires API key)
- Dataset: PL 94-171 Redistricting Data
- Format: JSON → convert to Parquet
- Fields: POP100, P003001-P003008 (race/ethnicity)
- Boundaries: TIGER/Line Shapefiles

**2010 Census**:
- Source: Census API (requires API key)
- Dataset: SF1 (Summary File 1)
- Format: JSON → convert to Parquet
- Fields: P001001 (total pop), P003001-P003008 (race)
- Boundaries: TIGER/Line Shapefiles

**2000 Census**:
- Source: NHGIS (manual download)
- Dataset: Summary File 1 (SF1)
- Format: Fixed-width text → parse to Parquet
- Fields: FXS001 (total pop), race fields
- Boundaries: TIGER/Line 2000 Shapefiles
- **Note**: Requires manual NHGIS account and download

### Step 2: Get Census API Key (2010, 2020)

If using API:
1. Register at: https://api.census.gov/data/key_signup.html
2. Receive key via email
3. Set environment variable:
   ```bash
   export CENSUS_API_KEY="your_key_here"
   # Or add to ~/.bashrc for persistence
   ```

### Step 3: Download Tract Data

**For 2020**:
```bash
python scripts/data/census/download_census_data.py \
  --year 2020 \
  --state california \
  --api-key $CENSUS_API_KEY
```

**For 2010**:
```bash
python scripts/data/census/download_census_data.py \
  --year 2010 \
  --state california \
  --api-key $CENSUS_API_KEY
```

**For 2000** (manual NHGIS process):
1. Go to: https://www.nhgis.org/
2. Create account
3. Select: Census 2000 → Summary File 1 → Tract level
4. Download and extract to `data/raw/2000/`
5. Run parser:
   ```bash
   python scripts/data/census/parse_nhgis_2000.py --state california
   ```

### Step 4: Download Geographic Boundaries

**TIGER/Line Shapefiles**:
```bash
python scripts/data/geography/download_tiger_shapefiles.py \
  --year 2020 \
  --state california \
  --geo-type tract
```

This downloads:
- Tract boundaries (polygons)
- Tract identifiers (GEOIDs)
- Geographic attributes (area, etc.)

### Step 5: Merge and Convert

Combine population data with geographies:
```bash
python scripts/data/census/merge_data_geometries.py \
  --year 2020 \
  --state california
```

Output: `data/tracts/2020/california_tracts_2020.parquet`

### Step 6: Validate Data

Check data quality:
```bash
python scripts/data/validation/validate_tract_data.py \
  --year 2020 \
  --state california
```

Validates:
- All GEOIDs present
- No missing populations
- Valid geometries
- Required demographic fields

## Required Fields

**All years need**:
- `GEOID`: Census tract identifier (11 characters: SSCCCTTTTTT)
- `population` or `POP100`: Total population
- `geometry`: Tract boundary polygon

**Optional but recommended**:
- `white`, `black`, `hispanic`, `asian`, `other`: Race/ethnicity counts
- `area_sqkm`: Tract area in square kilometers
- `density`: Population density

## Year-Specific Field Names

| Field | 2020 | 2010 | 2000 |
|-------|------|------|------|
| Total Population | POP100 | P001001 | FXS001 |
| GEOID | GEOID | GEOID10 | CTIDFP00 |
| White | P003003 | P003003 | NHX002 |
| Black | P003004 | P003004 | NHX003 |

Scripts must handle these variations.

## Download Time Estimates

| Task | Time per State |
|------|---------------|
| API download (2020/2010) | 1-2 minutes |
| TIGER shapefiles | 30 seconds |
| NHGIS manual (2000) | 5-10 minutes |
| Merge + convert | 30 seconds |
| **Total per state** | 2-3 minutes (API) or 10-15 minutes (manual) |

**All 50 states**: ~2-3 hours (API) or ~8-10 hours (manual)

## Common Issues

**Issue 1: API Rate Limiting**:
```
Error: API rate limit exceeded (500 requests/day)
Solution: Wait 24 hours or download states incrementally
```

**Issue 2: GEOID Mismatches**:
```
Error: GEOIDs don't match between population and geography
Solution: Check year-specific field names (GEOID vs GEOID10 vs CTIDFP00)
```

**Issue 3: Missing Tracts**:
```
Warning: 12 tracts in shapefile but not in population data
Solution: Census sometimes updates tract boundaries - use intersection join
```

**Issue 4: Invalid Geometries**:
```
Error: Geometry is invalid for tract 06001400100
Solution: Use buffer(0) to repair: df.geometry = df.geometry.buffer(0)
```

## Output Structure

```
data/tracts/
├── 2000/
│   ├── california_tracts_2000.parquet
│   ├── texas_tracts_2000.parquet
│   └── ... (50 states)
├── 2010/
│   ├── california_tracts_2010.parquet
│   └── ...
└── 2020/
    ├── california_tracts_2020.parquet
    └── ...
```

## What You'll Get

- Census tract data with population and demographics
- Geographic boundaries (polygons)
- Cleaned and validated data ready for redistricting
- Parquet format for fast loading

## Next Steps

After downloading:
- Use `/adjacency-build` to create adjacency graphs
- Use `/data-validate` to verify completeness
- Use `/run-redistricting` to generate districts

## Related Scripts

- `scripts/data/census/download_census_data.py` - API downloader
- `scripts/data/census/parse_nhgis_2000.py` - 2000 parser
- `scripts/data/geography/download_tiger_shapefiles.py` - Boundaries
- `scripts/data/census/merge_data_geometries.py` - Merger
- `scripts/data/validation/validate_tract_data.py` - Validator
