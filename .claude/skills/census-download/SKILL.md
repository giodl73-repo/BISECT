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

# Census Data Download

Download census tract-level population, demographics, and geographic boundaries for redistricting. Handles year-specific data sources, API access, and format conversions.

## Prerequisites
**2020/2010**: Census API key (https://api.census.gov/data/key_signup.html) → `export CENSUS_API_KEY="key"`
**2000**: NHGIS account (https://www.nhgis.org/) - manual download required

## When to Use
User says "Download census data for [year]/Need tract data for [state]", pipeline fails with missing tract data, starting new redistricting project

## Data Sources

**2020 Census**: API (PL 94-171 Redistricting Data), POP100 + P003001-P003008 (race/ethnicity), TIGER/Line boundaries, JSON → Parquet
**2010 Census**: API (SF1 Summary File 1), P001001 + P003001-P003008 (race), TIGER/Line boundaries, JSON → Parquet
**2000 Census**: NHGIS (SF1), FXS001 (total pop) + race fields, TIGER/Line 2000, Fixed-width text → Parquet (manual process)

## Workflow

### Step 1: Download Tract Data
**2020/2010** (API):
```bash
python scripts/data/census/download_census_data.py --year 2020 --state california --api-key $CENSUS_API_KEY
```

**2000** (manual):
1. https://www.nhgis.org/ → Create account
2. Select: Census 2000 → SF1 → Tract level
3. Download → Extract to `data/raw/2000/`
4. Parse: `python scripts/data/census/parse_nhgis_2000.py --state california`

### Step 2: Download Geographic Boundaries
```bash
python scripts/data/geography/download_tiger_shapefiles.py --year 2020 --state california --geo-type tract
```
Outputs: Tract polygons, GEOIDs, geographic attributes

### Step 3: Merge and Convert
```bash
python scripts/data/census/merge_data_geometries.py --year 2020 --state california
```
Output: `data/tracts/2020/california_tracts_2020.parquet`

### Step 4: Validate Data
```bash
python scripts/data/validation/validate_tract_data.py --year 2020 --state california
```
Checks: All GEOIDs present, no missing populations, valid geometries, required demographic fields

## Required Fields

**All years**: `GEOID` (tract ID 11 chars SSCCCTTTTTT), `population` or `POP100` (total pop), `geometry` (tract polygon)
**Optional**: `white`, `black`, `hispanic`, `asian`, `other` (race/ethnicity counts), `area_sqkm`, `density`

## Year-Specific Field Names

| Field | 2020 | 2010 | 2000 |
|-------|------|------|------|
| Total Pop | POP100 | P001001 | FXS001 |
| GEOID | GEOID | GEOID10 | CTIDFP00 |
| White | P003003 | P003003 | NHX002 |
| Black | P003004 | P003004 | NHX003 |

Scripts must handle these variations.

## Performance

| Task | Time/State |
|------|-----------|
| API download (2020/2010) | 1-2 min |
| TIGER shapefiles | 30s |
| NHGIS manual (2000) | 5-10 min |
| Merge + convert | 30s |
| **Total per state** | **2-3 min (API) or 10-15 min (manual)** |
| **All 50 states** | **2-3h (API) or 8-10h (manual)** |

## Troubleshooting

**API rate limiting**: `Error: API rate limit exceeded (500 requests/day)` → Wait 24h or download incrementally
**GEOID mismatches**: GEOIDs don't match between pop/geography → Check year-specific field names (GEOID vs GEOID10 vs CTIDFP00)
**Missing tracts**: 12 tracts in shapefile but not in pop data → Census boundary updates, use intersection join
**Invalid geometries**: `Geometry is invalid for tract 06001400100` → Repair with `df.geometry = df.geometry.buffer(0)`

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
Census tract data with pop/demographics, geographic boundaries (polygons), cleaned + validated data ready for redistricting, Parquet format (fast loading)

## Related Skills
`/adjacency-build` (create adjacency graphs), `/data-validate` (verify completeness), `/run-redistricting` (generate districts)

## Scripts
`scripts/data/census/download_census_data.py` (API downloader), `scripts/data/census/parse_nhgis_2000.py` (2000 parser), `scripts/data/geography/download_tiger_shapefiles.py` (boundaries), `scripts/data/census/merge_data_geometries.py` (merger), `scripts/data/validation/validate_tract_data.py` (validator)
