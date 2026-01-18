# Enhancement 15: Fix 2010/2000 Pipeline Completeness

**Status**: ✅ COMPLETED
**Priority**: Medium
**Estimated Complexity**: Medium
**Created**: January 2026
**Completed**: January 2026

### Goal
Systematically fix all missing outputs in 2010 and 2000 census runs identified by the validation framework, ensuring all three census years (2000, 2010, 2020) have complete and consistent pipeline outputs.

### Problem
Enhancement 14's validation framework revealed significant gaps in 2010 and 2000 pipeline runs:
- **2010 v1**: 0% complete, 68.5% partially complete
- **2000 v1**: 0% complete (similar to 2010)
- **2020 v3**: 100% complete (baseline)

**Root Cause**: Missing places (cities/towns) data files:
- 2010: Had all 50 `*_places_2010.parquet` files ✅
- 2020: Missing all 50 `*_places_2020.parquet` files ❌
- 2000: Missing all 50 `*_places_2000.parquet` files ❌

Without places data, the pipeline couldn't:
1. Generate `district_cities.csv` (identifies largest city per district)
2. Create individual district maps with city labels

### Solution

**Three-Phase Data Acquisition and Pipeline Re-Run:**

**Phase 1: Download 2020 Places Data (Census API)**
- Used existing `scripts/data/geography/download_all_places.py`
- Downloaded all 50 states with full population data from Census 2020 API
- Output: `data/raw/*_places_2020.parquet` (50 files)

**Phase 2: Download and Convert 2000 Places Data (NHGIS)**
- Downloaded `US_place_2000.shp` (boundaries) and CSV (population) from NHGIS
- Census 2000 API doesn't exist, so used NHGIS instead
- Created `scripts/data/geography/convert_nhgis_places_to_parquet.py`:
  - Reads NHGIS shapefile (all states nationwide)
  - Joins with CSV containing population data (FL5001 column)
  - Converts GISJOIN format to standard GEOID
  - Splits into per-state parquet files
  - Handles all 51 jurisdictions (50 states + DC)
- Output: `data/raw/*_places_2000.parquet` (51 files)

**Phase 3: Re-Run Pipelines**
- Updated `fix_2010_missing_outputs.py` to support 2020/2010/2000
- Ran fix script for each census year:
  - Phase 1: Add cities to districts (`add_cities_to_districts.py`)
  - Phase 2: Create individual district maps (`create_individual_district_maps.py`)
  - Phase 3: National post-processing (where applicable)
- All three years now 100% complete for required outputs

### Implementation

**New Files Created:**
- `scripts/data/geography/convert_nhgis_places_to_parquet.py` - NHGIS conversion utility (273 lines)

**Files Modified:**
- `scripts/data/geography/download_places.py` - Added 2000 census support:
  - 2000 URL pattern for TIGER files
  - 2000 column name mapping (PLCIDFP00, NAME00, NAMELSAD00)
  - 2000 API endpoint (fails gracefully when unavailable)
  - Skip population filter when API data unavailable
- `scripts/pipeline/fix_2010_missing_outputs.py` - Added 2020 support:
  - Load STATE_CONFIG_2020
  - Accept `--year 2020` parameter
  - Support all three census years uniformly
- `docs/ENHANCEMENTS_2026.md` - This documentation

**Data Format Standardization:**
All three census years now have identical format:
```python
Columns: ['GEOID', 'NAME', 'NAMELSAD', 'population', 'geometry']
```

**Data Sources:**
- **2010**: Census TIGER + Census API (SF1) - Complete
- **2020**: Census TIGER + Census API (PL) - Complete
- **2000**: NHGIS TIGER + NHGIS CSV (NP001A from SF1a) - Complete

### Results

**Before Fix:**
```
2020 v3: 100% complete ✅
2010 v1: 0% complete (68.5% partial)
2000 v1: 0% complete (similar)

Missing outputs (all 50 states):
- district_cities.csv
- maps/districts/district_*.png
- *_districts_with_cities.png
```

**After Fix:**
```
2020 v3: 100% complete ✅
2010 v1: 100% complete ✅
2000 v1: 100% complete ✅

All required outputs present:
- district_cities.csv (50/50 states)
- Individual district maps (435 total)
- District maps with city labels (50/50 states)
```

### Usage Examples

**Download 2020 places data:**
```bash
python scripts/data/geography/download_all_places.py --year 2020
```

**Convert NHGIS 2000 data:**
```bash
python scripts/data/geography/convert_nhgis_places_to_parquet.py \
    --input data/raw/US_place_2000.shp \
    --csv data/raw/nhgis0006_ds146_2000_place.csv
```

**Re-run pipeline to fix missing outputs:**
```bash
# Fix 2020
python scripts/pipeline/fix_2010_missing_outputs.py --year 2020 --version v3

# Fix 2010
python scripts/pipeline/fix_2010_missing_outputs.py --year 2010 --version v1

# Fix 2000
python scripts/pipeline/fix_2010_missing_outputs.py --year 2000 --version v1
```

**Validate results:**
```bash
python scripts/validation/validate_pipeline_outputs.py --year 2020 --version v3
python scripts/validation/validate_pipeline_outputs.py --year 2010 --version v1
python scripts/validation/validate_pipeline_outputs.py --year 2000 --version v1
```

### Key Learnings

1. **Root Cause Analysis**: Validation framework identified symptoms; investigation revealed root cause (missing input data)
2. **Data Source Diversity**: Different census years require different data sources (Census API for 2010/2020, NHGIS for 2000)
3. **Format Standardization**: Critical to maintain identical data format across all years for pipeline compatibility
4. **NHGIS Data Structure**: NHGIS uses GISJOIN format and separate boundary/data files that need joining
5. **Population is Essential**: City labels require population data to identify largest city per district
6. **Re-Runnable Scripts**: Fix approach used standard pipeline scripts with skip logic rather than manual patching

### Benefits

1. **Complete Pipeline Runs**: All three census years now fully functional
2. **Consistent Data**: Identical format across 2000/2010/2020 ensures uniform analysis
3. **Better Visualizations**: District maps now have meaningful city labels
4. **Historical Comparison**: Can compare redistricting results across all three census cycles
5. **Reusable Tools**: NHGIS conversion script can be used for future historical data needs

---

**Date Added**: January 14, 2026
**Date Completed**: January 14, 2026
**Implementation Time**: ~4 hours (data acquisition, NHGIS conversion, pipeline re-runs, verification, documentation)
