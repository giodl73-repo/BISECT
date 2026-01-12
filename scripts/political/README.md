# Political Analysis Scripts

Analyzes partisan lean, demographics, and electoral characteristics of congressional districts.

## Overview

These scripts provide political and demographic analysis of redistricting results:
1. **Political Analysis**: Partisan lean, competitive districts, margins
2. **Demographic Analysis**: Gender, race/ethnicity, diversity
3. **Visualization**: Maps showing political and demographic characteristics

## Prerequisites

### Election Data

**Required for political analysis**:
```bash
# Download presidential election results by tract
python scripts/data/elections/download_election_data.py --year 2020

# Process to parquet format
python scripts/data/elections/process_election_data.py --year 2020
```

**Output**: `data/processed/elections/2020_president_tract.parquet`

### Demographic Data

**Required for demographic analysis**:
```bash
# Download 2020 Census DHC demographic data
python scripts/data/demographics/download_demographic_data_robust.py --year 2020

# Process to parquet format
python scripts/data/demographics/process_demographic_data.py --year 2020
```

**Output**: `data/processed/demographics/2020_demographics_tract.parquet`

## Political Analysis

### analyze_districts.py

**Purpose**: Calculate political metrics for each district

**Usage**:
```bash
python scripts/political/analyze_districts.py outputs/us_2020_v1/states/california --year 2020
```

**Calculates**:
- Biden two-party vote percentage
- Trump two-party vote percentage
- Democratic margin (Biden % - Trump %)
- Partisan lean classification:
  - Strong D: Dem margin > 15%
  - Lean D: Dem margin 5-15%
  - Toss-up: Dem margin -5% to 5%
  - Lean R: Dem margin -15% to -5%
  - Strong R: Dem margin < -15%

**Output**: `{state}/political_analysis/district_political_2020.csv`

**Columns**:
```csv
district,num_tracts,population,total_votes,biden_votes,trump_votes,biden_two_party_pct,trump_two_party_pct,dem_margin,lean
```

### visualize_partisan_lean.py

**Purpose**: Create maps showing partisan lean of districts

**Usage**:
```bash
python scripts/political/visualize_partisan_lean.py outputs/us_2020_v1/states/california --year 2020 --dpi 150
```

**Creates two maps**:

1. **Partisan Lean Map** (`partisan_lean.png`)
   - Districts colored by lean (Strong D, Lean D, Toss-up, Lean R, Strong R)
   - Legend with district counts per category

2. **Partisan Lean with Margins** (`partisan_lean_with_margins.png`)
   - Main map: Colored by lean
   - Side table: Shows margins for all districts
   - Highlights most competitive districts

**Color Scheme**:
- Strong D: `#1a4da6` (Dark blue)
- Lean D: `#6495ed` (Medium blue)
- Toss-up: `#cccccc` (Gray)
- Lean R: `#f08080` (Light red)
- Strong R: `#c41e3a` (Dark red)

### run_political_analysis.py

**Purpose**: Batch process political analysis for all states

**Usage**:
```bash
python scripts/political/run_political_analysis.py --census-year 2020 --version v1 --election-year 2020
```

**For each state**:
1. Run `analyze_districts.py`
2. Run `visualize_partisan_lean.py`
3. Skip if outputs already exist (unless --force)

**Note**: Alaska and Hawaii may fail due to missing tract-level election data.

## Demographic Analysis

### analyze_district_demographics.py

**Purpose**: Calculate demographic characteristics for each district

**Usage**:
```bash
python scripts/political/analyze_district_demographics.py outputs/us_2020_v1/states/california
```

**Calculates**:

**Gender**:
- Male count and percentage
- Female count and percentage

**Race/Ethnicity**:
- White (non-Hispanic) count and percentage
- Black (non-Hispanic) count and percentage
- Asian (non-Hispanic) count and percentage
- Hispanic count and percentage
- Other count and percentage

**Derived Metrics**:
- Majority race (largest group)
- Majority race percentage
- Minority-majority status (non-white > 50%)

**Output**: `{state}/demographic_analysis/district_demographics.csv`

**Columns**:
```csv
district,num_tracts,population,male,female,male_pct,female_pct,white,black,asian,hispanic,other,white_pct,black_pct,asian_pct,hispanic_pct,other_pct,majority_race,majority_race_pct,minority_majority
```

**CRITICAL Pattern**:
```python
# ALWAYS use this pattern for GEOID handling
demo_df['GEOID'] = demo_df['GEOID'].astype(str).str.zfill(11)
assignments_df['GEOID'] = assignments_df['GEOID'].astype(str).str.zfill(11)
result = demo_df.merge(assignments_df, on='GEOID')
```

**Why**: Census tract GEOIDs are 11 digits but often stored as integers, losing leading zeros.

### visualize_district_demographics.py

**Purpose**: Create demographic visualization maps

**Usage**:
```bash
python scripts/political/visualize_district_demographics.py outputs/us_2020_v1/states/california --dpi 150
```

**Creates three maps**:

1. **Gender Balance Map** (`gender_balance.png`)
   - Male-leaning (>51% male): Blue `#4A90E2`
   - Female-leaning (>51% female): Pink `#E24A90`
   - Balanced (within 51-49): Purple `#9B59B6`

2. **Majority Race Map** (`majority_race.png`)
   - Shows which demographic group is the plurality in each district
   - Colors: Light blue (White), Light orange (Black), Light green (Asian), Light red (Hispanic), Light gray (Other)
   - Legend shows count of each majority type

3. **Diversity Index Map** (`diversity_index.png`)
   - Shannon entropy-based diversity measure
   - 5 levels from "Very Homogeneous" (light) to "Very Diverse" (dark red)
   - Identifies coalition/competitive districts
   - Formula: `-Σ(p_i * log(p_i))` normalized to 0-1 scale

**Diversity Levels**:
- Very Homogeneous: < 0.3 (one group dominates)
- Homogeneous: 0.3-0.5
- Moderate: 0.5-0.7
- Diverse: 0.7-0.85
- Very Diverse: > 0.85 (no clear majority)

### run_demographic_analysis.py

**Purpose**: Batch process demographic analysis for all states

**Usage**:
```bash
python scripts/political/run_demographic_analysis.py --census-year 2020 --version v1
```

**For each state**:
1. Load demographic data
2. Load district assignments
3. Calculate demographics per district
4. Save to `demographic_analysis/district_demographics.csv`

### run_demographic_visualization.py

**Purpose**: Batch create demographic maps for all states

**Usage**:
```bash
python scripts/political/run_demographic_visualization.py --census-year 2020 --version v1 --dpi 150
```

**For each state**:
1. Load demographic statistics
2. Create 3 demographic maps
3. Skip if outputs already exist

## Map Visualization Patterns

### Standard Pattern (ALL maps use this)

```python
import matplotlib.pyplot as plt
import geopandas as gpd

# Plot tracts with THIN WHITE boundaries
tracts_gdf.plot(
    ax=ax,
    column='attribute',
    edgecolor='white',      # Thin white
    linewidth=0.1,          # Very thin
    alpha=0.8
)

# Add THICK BLACK district boundaries as overlay
districts_dissolved = tracts_gdf.dissolve(by='district', as_index=False)
districts_dissolved.boundary.plot(
    ax=ax,
    edgecolor='black',      # Thick black
    linewidth=1.5,          # Thick
    zorder=10               # On top
)
```

**Why This Works**:
- Thin white boundaries show tract-level detail
- Thick black boundaries make districts prominent
- `dissolve()` merges tracts into district polygons
- `boundary.plot()` draws only the outline
- `zorder=10` ensures boundaries render on top

### Title Formatting

**Don't use `\n` in titles** (causes rendering issues):
```python
# BAD
ax.set_title('California\nCongressional Districts')

# GOOD
ax.set_title('California - Congressional Districts')
```

## Data Sources

### Election Data

**Source**: MIT Election Data + Science Lab

**Coverage**: 2020, 2016 presidential elections

**Level**: Census tract (geocoded from precincts)

**Format**: `{year}_president_tract.parquet`
```
GEOID,total_votes,biden_votes,trump_votes,other_votes
```

**Limitations**:
- Alaska: No tract-level data available
- Hawaii: Incomplete tract coverage

### Demographic Data

**Source**: 2020 Census DHC (Demographic and Housing Characteristics)

**Coverage**: All 50 states + DC

**Level**: Census tract

**Variables**:
- Total population
- Sex (Male, Female)
- Race/Ethnicity (White NH, Black NH, Asian NH, Hispanic, Other)

**Format**: `{year}_demographics_tract.parquet`
```
GEOID,state,county,tract,total_pop,male,female,white_non_hispanic,black_non_hispanic,asian_non_hispanic,hispanic,other,male_pct,female_pct,white_pct,black_pct,asian_pct,hispanic_pct,other_pct
```

## Output Structure

```
outputs/us_2020_v1/states/{state}/
├── political_analysis/
│   ├── district_political_2020.csv
│   └── maps/
│       ├── partisan_lean.png
│       └── partisan_lean_with_margins.png
│
└── demographic_analysis/
    ├── district_demographics.csv
    └── maps/
        ├── gender_balance.png
        ├── majority_race.png
        └── diversity_index.png
```

## Common Issues

### Issue: "GEOID type mismatch" error

**Symptom**: `ValueError: You are trying to merge on int64 and object columns for key 'GEOID'`

**Cause**: One DataFrame has GEOID as integer, other as string

**Fix**: Ensure both are strings with proper zero-padding
```python
df1['GEOID'] = df1['GEOID'].astype(str).str.zfill(11)
df2['GEOID'] = df2['GEOID'].astype(str).str.zfill(11)
result = df1.merge(df2, on='GEOID')
```

### Issue: Missing election data

**Symptom**: "Election data not found" or "Political analysis failed"

**Cause**: Haven't downloaded/processed election data

**Fix**:
```bash
python scripts/data/elections/download_election_data.py --year 2020
python scripts/data/elections/process_election_data.py --year 2020
```

### Issue: Missing demographic data

**Symptom**: "Demographic data not found"

**Cause**: Haven't downloaded/processed demographic data

**Fix**:
```bash
python scripts/data/demographics/download_demographic_data_robust.py --year 2020
python scripts/data/demographics/process_demographic_data.py --year 2020
```

### Issue: Alaska/Hawaii failures

**Symptom**: Political analysis fails for AK, HI

**Cause**: Missing tract-level election data for these states

**Solution**: This is expected - these states don't have complete tract-level election data

## Performance

**Political Analysis** (per state):
- Small states: ~5-10 seconds
- Large states: ~30-60 seconds
- Total (48 states): ~40 minutes

**Demographic Analysis** (per state):
- All states: ~1-2 seconds each
- Total (50 states): ~1-2 minutes

**Demographic Visualization** (per state):
- Small states: ~5-10 seconds
- Large states: ~20-30 seconds
- Total (50 states): ~7-10 minutes

## Integration with Pipeline

These scripts are automatically called by `run_complete_redistricting.py`:

```python
# Pipeline runs in order:
1. State redistricting (50 states)
2. Create US aggregates
3. Create US national map
4. run_political_analysis.py        # ← Political
5. run_demographic_analysis.py      # ← Demographic
6. run_demographic_visualization.py # ← Demographic viz
```

**Skip options**:
```bash
# Skip all political analysis
python scripts/pipeline/run_complete_redistricting.py --skip-political

# Skip all demographic analysis
python scripts/pipeline/run_complete_redistricting.py --skip-demographic
```

## See Also

- `CODING_PATTERNS.md` - GEOID handling, map patterns, error handling
- `ARCHITECTURE.md` - System design
- `scripts/pipeline/README.md` - Main pipeline orchestration
- `scripts/data/README.md` - Data acquisition
