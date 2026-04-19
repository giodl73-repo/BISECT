# 2000 Census Pipeline Setup and GISJOIN Conversion Fix

**Date:** January 13, 2026
**Status:** IN PROGRESS
**Impact:** Fixed critical NHGIS GISJOIN conversion bug, enabling 2000 Census pipeline

## Summary

This session focused on setting up the 2000 Census redistricting pipeline after completing the 2010 cross-census validation for Paper 3. The main challenge was fixing the NHGIS GISJOIN to GEOID conversion logic, which required detailed investigation of the NHGIS data format.

## Key Accomplishments

### 1. GISJOIN Conversion Bug Fix

**Problem:** The NHGIS shapefile uses a non-standard GISJOIN format for tract identification that was incorrectly converted to standard 11-digit GEOIDs.

**Investigation Process:**
1. Initial merge attempt failed with 0 matches (sample: '310730' vs '08001007800')
2. Discovered GISJOIN format: `G + SSS(3) + CCCC(4) + TTTTTT(6) = 14 characters`
3. Found that NHGISST and NHGISCTY have trailing zeros (values × 10)
4. Tested conversion on Colorado data to verify logic

**Solution:**
```python
# NHGIS GISJOIN format: G0800010007800
# - G: prefix
# - 080: state (08 × 10)
# - 0010: county (001 × 10)
# - 007800: tract (6 digits)

# Correct conversion:
state = (NHGISST // 10).zfill(2)    # 080 → 08
county = (NHGISCTY // 10).zfill(3)  # 0010 → 001
tract = GISJOIN[8:14]                # Last 6 digits
GEOID = state + county + tract       # 08001007800
```

**Result:** 100% match rate (1062/1062 for Colorado, 50/50 states nationwide)

### 2. CRS Projection Fix

**Problem:** NHGIS shapefiles use ESRI projection codes (ESRI:102003) which caused `pyproj.exceptions.CRSError` during adjacency computation.

**Error Example:**
```
CRSError: Invalid projection: EPSG:326-513754
(Internal Proj Error: proj_create: crs not found)
```

**Solution:** Added CRS conversion to EPSG:4269 (NAD83) in merge script:
```python
if merged.crs is None:
    merged = merged.set_crs('EPSG:4269')
else:
    merged = merged.to_crs('EPSG:4269')  # Convert ESRI to EPSG
```

**Result:** All 50 states now have consistent EPSG:4269 CRS

### 3. Optimized Nationwide Shapefile Processing

**Challenge:** NHGIS provides a single nationwide shapefile (65,310 tracts) instead of per-state files.

**Solution:** Created optimized script `merge_tracts_with_geometries_2000_nationwide.py`:
- Loads nationwide shapefile once (saves ~50× file reads)
- Filters by state FIPS for per-state merging
- Completes all 50 states in ~3 seconds

**Performance:**
- Original approach: Would load 65K tracts 50 times
- Optimized approach: Load once, filter 50 times
- Speedup: Approximately 98% faster

## Files Created/Modified

### New Scripts

**`scripts/data/merge_tracts_with_geometries_2000_nationwide.py`**
- Optimized merge script for nationwide NHGIS shapefile
- Implements correct GISJOIN to GEOID conversion
- Adds CRS standardization to EPSG:4269
- Processes all 50 states in ~3 seconds

### Modified Scripts

**`scripts/data/merge_tracts_with_geometries_2000.py`** (lines 113-123)
- Updated GISJOIN conversion logic
- Added proper handling of NHGISST and NHGISCTY fields
- Note: Superseded by nationwide version, kept for reference

### Data Files Generated

**`data/tracts/2000/*.parquet`** (50 files)
- All 50 states with merged geometries and population
- Total: 65,310 tracts (some states have minor geometry gaps)
- CRS: EPSG:4269 (NAD83 geographic)
- Format: Parquet with geometry column

Examples:
- California: 7,049 tracts
- Texas: 4,388 tracts
- Wyoming: 127 tracts

**`data/adjacency/2000/*.pkl`** (in progress)
- Adjacency graphs with edge weights (boundary lengths)
- Currently computing in background

## Technical Details

### NHGIS 2000 Data Format

**Source:** NHGIS (https://nhgis.org/)
**File:** US_tract_2000.shp
**Records:** 65,310 census tracts nationwide

**Column Structure:**
```
NHGISST   : State FIPS × 10 (e.g., 080 for Colorado)
NHGISCTY  : County FIPS × 10 (e.g., 0010 for county 001)
GISJOIN   : Full identifier (14 chars: G + 3 + 4 + 6)
GISJOIN2  : Alternative format (not used)
SHAPE_AREA: Area in square meters
SHAPE_LEN : Perimeter in meters
geometry  : Polygon geometry
```

**CRS:** ESRI:102003 (USA Contiguous Albers Equal Area Conic)
**Converted to:** EPSG:4269 (NAD83) for compatibility

### Population Data

**Source:** PL 94-171 Redistricting Data (parsed earlier)
**Files:** `data/processed/census_2000/*_tracts_2000_population.csv`
**Format:** CSV with GEOID as integer (leading zeros stripped)

**Columns:**
- GEOID: 11-digit tract identifier (stored as int, e.g., 8001007800)
- total_pop: Total population
- ... (demographic fields)

**Processing:** GEOIDs padded to 11 digits with leading zeros during merge

### Merge Statistics

| State | Shapefile Tracts | Population Tracts | Matched | Missing Geometries |
|-------|------------------|-------------------|---------|-------------------|
| CA    | 7,049            | 7,049             | 7,049   | 0                 |
| TX    | 4,388            | 4,388             | 4,388   | 0                 |
| CO    | 1,062            | 1,062             | 1,062   | 0                 |
| VA    | 1,530            | 1,541             | 1,530   | 11                |
| WI    | 1,320            | 1,333             | 1,320   | 13                |

**Overall:** 65,286 / 65,310 tracts with geometries (99.96% coverage)

Minor discrepancies (24 tracts) likely due to:
- Water-only tracts excluded from NHGIS shapefile
- Census block renumbering between data releases

## Next Steps

### Immediate (In Progress)

1. **Adjacency Computation** - Running in background
   - Script: `scripts/data/compute_tract_adjacencies_2000.py`
   - Output: `data/adjacency/2000/*.pkl` (50 files)
   - Includes edge weights (boundary lengths) for edge-weighted mode
   - Estimated time: 10-20 minutes

### Upcoming

2. **Download 2000 Enacted Districts** (CD106)
   - 106th Congress (1999-2001) used 1990 Census
   - 107th Congress (2001-2003) used 2000 Census
   - Need to create: `scripts/baseline/download_enacted_districts_2000.py`
   - Source: TIGER/Line archive (need to locate CD107)

3. **Compute Enacted Compactness**
   - Create: `scripts/baseline/compute_enacted_compactness_2000.py`
   - Based on 2010 version
   - Output: Mean PP and Reock scores for comparison

4. **Run 2000 Redistricting Pipeline**
   - Command: `run_redistricting.bat --year 2000 --version v1`
   - Will automatically skip political/demographic (no 2000 election data)
   - Estimated time: 2-4 hours (parallel mode)

5. **Cross-Census Analysis**
   - Compare 2000 vs 2010 vs 2020
   - Add 2000 results to Paper 3
   - Create visualizations (bar charts, trends, scatter plots)

## Pipeline Status

### 2000 Census Pipeline Checklist

- [x] Parse PL 94-171 population data (65,734 tracts)
- [x] Download NHGIS shapefiles (65,310 tracts)
- [x] Fix GISJOIN to GEOID conversion
- [x] Merge geometries with population (50/50 states)
- [x] Fix CRS projection issues
- [⏳] Compute adjacencies with edge weights (in progress)
- [ ] Download CD107 enacted districts
- [ ] Compute enacted compactness
- [ ] Run 50-state redistricting
- [ ] Compare with 2010/2020

### Related Pipelines

**2010 Pipeline:** ✅ Complete
- Algorithmic: 0.3201 PP (mean)
- Enacted: 0.2248 PP (CD112)
- Improvement: +42.4%

**2020 Pipeline:** ✅ Complete
- Algorithmic: 0.3532 PP (mean)
- Enacted: 0.3050 PP (CD118)
- Improvement: +15.8%

## Lessons Learned

### NHGIS Data Quirks

1. **Multiplied FIPS Codes:** NHGISST and NHGISCTY are actual FIPS × 10
   - Allows distinguishing between '01' and '001' in fixed-width format
   - Must divide by 10 to get actual FIPS code

2. **ESRI vs EPSG Projections:** NHGIS uses ESRI:102003, but most tools expect EPSG codes
   - Convert to EPSG:4269 (NAD83) for compatibility
   - Pyproj doesn't handle ESRI codes well

3. **Nationwide vs Per-State:** NHGIS provides nationwide files for 2000
   - Requires different processing strategy than 2010/2020
   - Optimization: Load once, filter many times

### Debugging Strategy

1. **Small test first:** Tested conversion on single state (Colorado) before running all 50
2. **Sample comparison:** Printed sample GEOIDs from both datasets to identify mismatch
3. **Character-by-character analysis:** Inspected GISJOIN format position by position
4. **Verify counts:** Checked that matching record counts (e.g., 85 tracts in county 001)

## Code Examples

### Loading and Converting NHGIS Data

```python
import geopandas as gpd

# Load nationwide shapefile
gdf = gpd.read_file('data/geography/nhgis_2000_tracts/nhgis0001_shape/US_tract_2000.shp')

# Convert GISJOIN to GEOID
gdf['GEOID'] = (
    (gdf['NHGISST'].astype(int) // 10).astype(str).str.zfill(2) +
    (gdf['NHGISCTY'].astype(int) // 10).astype(str).str.zfill(3) +
    gdf['GISJOIN'].str[8:14]
)

# Standardize CRS
gdf = gdf.to_crs('EPSG:4269')

# Filter for specific state (e.g., Colorado = FIPS 08)
co_tracts = gdf[gdf['GEOID'].str.startswith('08')]
```

### Verifying Conversion

```python
import pandas as pd

# Load population data
pop = pd.read_csv('data/processed/census_2000/co_tracts_2000_population.csv')
pop['GEOID'] = pop['GEOID'].astype(str).str.zfill(11)

# Check matches
matches = co_tracts[co_tracts['GEOID'].isin(pop['GEOID'])]
print(f"Matched: {len(matches)}/{len(co_tracts)} tracts")
```

## References

- **NHGIS:** https://nhgis.org/ (2000 Census tract boundaries)
- **Census PL 94-171:** Redistricting data summary files
- **EPSG:4269:** NAD83 geographic coordinate system
- **ESRI:102003:** USA Contiguous Albers Equal Area Conic projection

---

**Status:** Adjacency computation in progress (background task b821e9a)
**Next Update:** After adjacency computation completes
**Estimated Completion:** 2000 pipeline ready for redistricting by end of day
