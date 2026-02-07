---
uuid: 1f3bba38-65e4-952e-2c8b-deed1c537079
slug: national-maps
name: National Maps
wave_uuid: 79fae8
created: '2026-01-10'
status: COMPLETED
---

# E3: Create National Political and Demographic Maps

**Status**: ✅ COMPLETED
**Priority**: Medium
**Estimated Complexity**: Medium
**Created**: January 2026
**Completed**: January 2026

### Current State
- State-level maps exist for political and demographic analysis
- National aggregate CSV files exist (`us_all_districts.csv`)
- No national visualization showing all 435 districts

### Goal
- Create two national maps showing all 435 congressional districts:
  1. **Political Map**: Red/blue coloring by partisan lean
  2. **Demographic Map**: Color by majority demographic group

### Implementation Plan

#### New Script
**`scripts/political/create_us_national_political_demographic_maps.py`**

##### Inputs
- `outputs/us_YEAR_VERSION/us_all_districts.csv` - All districts with political/demographic data
- Census tract shapefiles for all 50 states
- Political data from MIT Election Lab
- Demographic data from Census

##### Processing Steps
1. **Load all state data**
   - Load tracts for all 50 states
   - Load district assignments from each state's output
   - Merge into single national GeoDataFrame

2. **Political Map**
   - Join with political data (dem_share)
   - Color districts: blue (D≥50%), red (R>50%)
   - Use diverging colormap for intensity
   - Add seat totals: "D: XXX | R: XXX"
   - Add Alaska/Hawaii as insets

3. **Demographic Map**
   - Join with demographic data
   - Determine majority group per district
   - Color by group: White, Hispanic, Black, Asian, Other
   - Use qualitative colormap (5 distinct colors)
   - Add legend with counts per group
   - Add Alaska/Hawaii as insets

##### Outputs
- `outputs/us_YEAR_VERSION/us_national_political.png`
- `outputs/us_YEAR_VERSION/us_national_demographic.png`

#### Visual Specifications
- **Size**: 20x12 inches
- **DPI**: 150 (high quality, reasonable file size)
- **Projection**: Albers Equal Area Conic (standard for US maps)
- **Alaska/Hawaii**: Inset boxes in lower-left corner
- **Title**: "U.S. Congressional Districts 2020 - Political Lean"
- **Legend**: Clear, positioned outside plot area

#### Integration
Add as Step [4/4] in `run_complete_redistricting.py`:
```python
# [4/4] CREATE US NATIONAL MAPS
run_subscript(
    'scripts/political/create_us_national_political_demographic_maps.py',
    args=['--year', year, '--version', version, '--output-dir', output_dir],
    description='Creating national political and demographic maps'
)
```

**Completion Date:** January 11, 2026
**Implementation:** Created `create_us_national_political_map.py` and `create_us_national_demographic_map.py`. Both scripts generate nationwide visualizations of all 435 districts. Integrated into pipeline as post-processing steps. Dashboard updated with USA row showing national maps.
