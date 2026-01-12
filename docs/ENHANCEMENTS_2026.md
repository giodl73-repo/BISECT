# Pipeline Enhancements - January 2026

## Overview

Four enhancements to integrate into the main redistricting pipeline to provide more comprehensive analysis and visualization of congressional districts.

---

## Enhancement 1: Integrate Compactness into Main Pipeline

### Current State
- Compactness calculation exists as standalone script: `scripts/pipeline/calculate_compactness_metrics.py`
- Computes Polsby-Popper and Reock scores
- Must be run manually after redistricting

### Goal
- Automatically calculate compactness metrics as part of the main pipeline
- Add compactness scores to `district_summary.csv` during the Summary stage
- No separate manual step required

### Implementation Plan

#### Files to Modify
1. **`scripts/pipeline/run_complete_redistricting.py`**
   - No changes needed (orchestrator calls run_all_states)

2. **`scripts/pipeline/create_final_district_summary.py`**
   - Import compactness calculation functions from `calculate_compactness_metrics.py`
   - Add compactness score calculation after district geometry creation
   - Add `polsby_popper` and `reock` columns to district_summary.csv

#### Changes Required

**create_final_district_summary.py:**
```python
# Add imports
from calculate_compactness_metrics import polsby_popper_score, reock_score

# In create_summary function, after creating district geometries:
# Calculate compactness metrics
districts_gdf['polsby_popper'] = districts_gdf.geometry.apply(polsby_popper_score)
districts_gdf['reock'] = districts_gdf.geometry.apply(reock_score)

# Include in output CSV
summary_df['polsby_popper'] = districts_gdf['polsby_popper']
summary_df['reock'] = districts_gdf['reock']
```

#### Output Changes
- `district_summary.csv` gains two new columns:
  - `polsby_popper` (float 0-1)
  - `reock` (float 0-1)

#### Benefits
- Compactness automatically calculated for all 50 states
- No manual post-processing required
- Data immediately available for dashboard and analysis

---

## Enhancement 2: Add D/R Seat Totals to Political Maps

### Current State
- Political maps show partisan lean by district (red/blue coloring)
- Created by `scripts/political/visualize_partisan_lean.py`
- No summary statistics shown on map

### Goal
- Add text annotation to political maps showing:
  - Total Democratic-leaning seats (blue ≥ 50%)
  - Total Republican-leaning seats (red > 50%)
  - Format: "D: 27 | R: 25" (example)

### Implementation Plan

#### Files to Modify
1. **`scripts/political/visualize_partisan_lean.py`**
   - Calculate D/R seat counts after assigning colors
   - Add text box annotation to upper-right corner of map
   - Use clean, readable font and contrasting background

#### Changes Required

**visualize_partisan_lean.py:**
```python
# After assigning dem_share and colors to districts
d_seats = (districts_gdf['dem_share'] >= 0.5).sum()
r_seats = (districts_gdf['dem_share'] < 0.5).sum()

# Add text annotation
ax.text(0.98, 0.98, f'D: {d_seats} | R: {r_seats}',
        transform=ax.transAxes,
        fontsize=16,
        fontweight='bold',
        verticalalignment='top',
        horizontalalignment='right',
        bbox=dict(boxstyle='round', facecolor='white', alpha=0.9, edgecolor='black'))
```

#### Output Changes
- Political maps gain seat count annotation
- Example: `california_political.png` shows "D: 42 | R: 10"

#### Benefits
- Quick visual summary of partisan balance
- Easy comparison across states
- Useful for dashboard and paper figures

---

## Enhancement 3: Create National Political and Demographic Maps

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

---

## Enhancement 4: Create Urban Metro Area District Maps (MSA/MCSA)

### Current State
- Individual district maps exist for each district
- State-level overview maps exist
- No focused views of major metropolitan areas

### Goal
- Create focused maps for major metro areas showing:
  - All districts within the MSA/MCSA boundary
  - Surrounding context (faded neighboring districts)
  - Major city labels
  - Highway networks (optional)

### Implementation Plan

#### New Script
**`scripts/visualization/create_metro_area_maps.py`**

##### Metro Areas to Cover
Using Census Bureau MSA/MCSA definitions, focus on largest metros:

**Top 20 Metropolitan Areas** (by population):
1. New York-Newark-Jersey City, NY-NJ-PA
2. Los Angeles-Long Beach-Anaheim, CA
3. Chicago-Naperville-Elgin, IL-IN-WI
4. Dallas-Fort Worth-Arlington, TX
5. Houston-The Woodlands-Sugar Land, TX
6. Washington-Arlington-Alexandria, DC-VA-MD-WV
7. Philadelphia-Camden-Wilmington, PA-NJ-DE-MD
8. Miami-Fort Lauderdale-Pompano Beach, FL
9. Atlanta-Sandy Springs-Alpharetta, GA
10. Boston-Cambridge-Newton, MA-NH
11. Phoenix-Mesa-Chandler, AZ
12. San Francisco-Oakland-Berkeley, CA
13. Riverside-San Bernardino-Ontario, CA
14. Detroit-Warren-Dearborn, MI
15. Seattle-Tacoma-Bellevue, WA
16. Minneapolis-St. Paul-Bloomington, MN-WI
17. San Diego-Chula Vista-Carlsbad, CA
18. Tampa-St. Petersburg-Clearwater, FL
19. Denver-Aurora-Lakewood, CO
20. St. Louis, MO-IL

##### Inputs
- Census MSA/MCSA boundary shapefiles (year-specific definitions)
- District shapefiles (from tract unions)
- City place boundaries
- District summary data

**IMPORTANT**: MSA/MCSA definitions change by decade:
- 2020 census → 2020 MSA definitions
- 2010 census → 2010 MSA definitions
- Year parameter must be threaded through download, build, and run scripts

##### Processing Steps
1. **Load MSA boundaries**
   - Download from Census TIGER/Line
   - Filter to top 20 by population

2. **For each MSA:**
   - Identify districts that intersect MSA boundary
   - Load district geometries for focal state(s)
   - Load neighboring districts for context
   - Spatial join cities within MSA

3. **Create focused map**
   - Set extent to MSA boundary + 10% margin
   - Plot focal districts (bright colors)
   - Plot neighboring districts (faded gray)
   - Add city labels (largest 10 cities in metro)
   - Add district numbers at centroids
   - Title: "{Metro Area Name} - Congressional Districts"

##### Outputs
- `outputs/us_YEAR_VERSION/metro_areas/new_york.png`
- `outputs/us_YEAR_VERSION/metro_areas/los_angeles.png`
- ... (one per metro area)

#### Visual Specifications
- **Size**: 14x10 inches (landscape)
- **DPI**: 150
- **Colors**: Qualitative colormap (each district distinct)
- **Context**: Gray neighboring districts at 30% opacity
- **Labels**: City names (8-12pt), district numbers (14pt bold)

#### Integration
Add as optional step in `run_complete_redistricting.py`:
```python
# Optional: CREATE METRO AREA MAPS
if args.create_metro_maps:
    run_subscript(
        'scripts/visualization/create_metro_area_maps.py',
        args=['--year', year, '--version', version, '--output-dir', output_dir],
        description='Creating metro area focused maps'
    )
```

---

## Implementation Order

### Priority 1: Enhancement 1 (Compactness Integration)
- **Effort**: Low (code already exists, just needs integration)
- **Impact**: High (adds critical metric to all outputs)
- **Time**: 30 minutes

### Priority 2: Enhancement 2 (D/R Seat Totals)
- **Effort**: Low (simple text annotation)
- **Impact**: Medium (improves political map readability)
- **Time**: 20 minutes

### Priority 3: Enhancement 3 (National Maps)
- **Effort**: Medium (new script, handle all states, insets)
- **Impact**: High (enables national-level analysis)
- **Time**: 2 hours

### Priority 4: Enhancement 4 (Metro Area Maps)
- **Effort**: High (new script, MSA data, 20+ maps)
- **Impact**: Medium (nice-to-have for urban analysis)
- **Time**: 4 hours

---

## Testing Plan

### Enhancement 1
```bash
# Run full pipeline for one state
python scripts/pipeline/run_state_redistricting.py --state CA --year 2020 --output-dir test
# Verify district_summary.csv has polsby_popper and reock columns
```

### Enhancement 2
```bash
# Run political analysis for one state
python scripts/political/run_political_analysis.py --state CA --year 2020
# Verify political map has D/R seat counts
```

### Enhancement 3
```bash
# Create national maps
python scripts/political/create_us_national_political_demographic_maps.py --year 2020 --version v1
# Verify output files exist and look correct
```

### Enhancement 4
```bash
# Create metro area maps
python scripts/visualization/create_metro_area_maps.py --year 2020 --version v1
# Verify 20 metro area PNG files created
```

---

## Success Criteria

### Enhancement 1 ✓
- All state `district_summary.csv` files have compactness columns
- Values are reasonable (0-1 range)
- No manual compactness calculation needed

### Enhancement 2 ✓
- All political maps show D/R seat counts
- Annotation is readable and well-positioned
- Counts match actual district data

### Enhancement 3 ✓
- National political map shows all 435 districts
- National demographic map shows all 435 districts
- Alaska/Hawaii shown as insets
- Legends are clear and accurate

### Enhancement 4 ✓
- 20 metro area maps created
- Each map shows correct districts for that metro
- City labels and district numbers visible
- Context districts provide geographic reference

---

**Date**: January 2026
**Status**: Design complete, ready for implementation
**Order**: Implement 1 → 2 → 3 → 4
