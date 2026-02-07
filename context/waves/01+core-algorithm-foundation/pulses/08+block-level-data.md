---
uuid: cf47ae4d-f0af-386c-68a6-4357a102973a
slug: block-level-data
name: Block-Level Data
wave_uuid: 79fae8
created: '2026-01-10'
status: PLANNED
---

# E8: Block-Level Data Support for Multi-Year Census 📋 DATA AVAILABLE

**Status**: 📋 PLANNED
**Priority**: Medium
**Estimated Complexity**: Medium
**Created**: January 2026
**Commits**: [4a08bd4](https://github.com/giodl_microsoft/redistricting/commit/4a08bd4e2be0173b8b7bbe61116fab72b828e643)
**Size**: M - 777 lines changed (2 files)

### Goal
Support census block-level redistricting for 2000, 2010, and 2020, with automatic tract aggregation for older census years.

**Data Status:** ✅ Block-level PL 94-171 redistricting data for all three census years (2000, 2010, 2020) has been copied to `data/census/` directory (January 13, 2026). Data includes geographic headers and population files for all 50 states.

**Comprehensive Analysis:** See `docs/CENSUS_DATA_ANALYSIS.md` for complete assessment of available data, file formats, geographic hierarchies, and implementation roadmap.

### Background
**Census Geographic Hierarchy:**
- **Blocks**: Smallest unit (~11M nationwide in 2020)
- **Block Groups**: Aggregation of blocks (~240K nationwide)
- **Tracts**: Aggregation of block groups (~85K nationwide)

**Current Implementation:**
- Uses tract-level data (2020 only)
- 84,414 tracts for 2020 census
- Trade-off: Balance between granularity and computation time

**Proposed Enhancement:**
- Download block-level data for 2000, 2010, 2020
- Support both block and tract redistricting
- Auto-aggregate blocks → tracts for 2000 and 2010 when needed

### Why Block-Level Data?

**Advantages:**
1. **Higher Resolution**: 11M blocks vs 85K tracts (130x finer)
2. **Better Compactness**: Smaller units allow tighter district boundaries
3. **Historical Compatibility**: Blocks available back to 2000
4. **Flexibility**: Can aggregate up to tracts if computation too expensive

**Challenges:**
1. **Computation**: 130x more nodes in adjacency graph
2. **Memory**: Larger graphs and GeoDataFrames
3. **Time**: METIS partitioning scales with graph size
4. **Storage**: Larger parquet/pickle files

### Available Data Structure

**Location:** `data/census/`

**Census 2000:**
- Directory: `Census 2000/geos/`
- Format: Geographic files (.upl format)
- Content: Block-level geography for all 50 states
- Example: `algeo.upl` (Alabama), `cageo.upl` (California)

**Census 2010:**
- Directory: `Census 2010/[state]2010.pl/`
- Format: PL 94-171 redistricting data
- Files per state:
  - `[state]000012010.pl` - File 1: P1 Population counts
  - `[state]000022010.pl` - File 2: P2-P4 Race/ethnicity data
  - `[state]geo2010.pl` - Geographic header with GEOID, coordinates, area
- Example: `al2010.pl/algeo2010.pl` contains 222,189 blocks for Alabama

**Census 2020:**
- Directory: `Census 2020/[state]2020.pl/`
- Format: PL 94-171 redistricting data
- Files per state:
  - `[state]000012020.pl` - File 1: P1 and P2 population tables
  - `[state]000022020.pl` - File 2: Additional race/ethnicity tables
  - `[state]000032020.pl` - File 3: Supplemental tables
  - `[state]geo2020.pl` - Geographic header
- Documentation: `2020Census_PL94_171Redistricting_StatesTechDoc_English.pdf`
- Field definitions: `2020_PLSummaryFile_FieldNames.xlsx`
- Headers: `aa_geo_header.csv`, `aa_000012020header.csv`

**Data Format Details:**
- Geographic header includes: SUMLEV (summary level), STATE, COUNTY, TRACT, BLKGRP, BLOCK
- SUMLEV=750 indicates census block level
- Includes AREALAND, AREAWATR (in square meters)
- Includes INTPTLAT, INTPTLON (internal point coordinates)
- Includes POP100 (official 2020 population count)

### Implementation Plan

#### Phase 0: Tract Aggregation for Historical Years (2000, 2010) ✅ COMPLETED (2010), ⏳ PARTIAL (2000)

**Completion Date:** January 13, 2026
**Status:** 2010 pipeline fully complete; 2000 requires manual NHGIS shapefile download

**Objective:** Generate tract-level data for 2000 and 2010 from block-level PL 94-171 files for historical comparison and longitudinal analysis.

**Why Needed:**
- Current pipeline uses tract-level shapefiles for 2020 only
- 2000 and 2010 tract shapefiles not readily available in processed format
- Longitudinal analysis requires consistent tract-level data across all three census years

**Implementation Summary:**

**Census 2010 (COMPLETE ✅):**
1. ✅ **Parse Tract-Level PL 94-171 Files**: `scripts/data/census/parse_pl94171_tracts_2010.py`
   - Extracted 74,224 tracts from fixed-width geographic headers (SUMLEV=140)
   - Output: `data/processed/census_2010/[state]_tracts_2010_population.csv` (all 50 states)

2. ✅ **Download TIGER/Line 2010 Tract Shapefiles**: `scripts/data/geography/download_tiger_tracts_2010.py`
   - Downloaded directly from Census Bureau FTP (all 50 states)
   - Output: `data/geography/tiger_2010_tracts/tl_2010_[fips]_tract10.zip`

3. ✅ **Merge Population + Geometries**: `scripts/data/merge_tracts_with_geometries_2010.py`
   - Combined census data with tract boundaries (all 50 states)
   - Output: `data/tracts/2010/[state]_tracts_2010.parquet` (all 50 states)

**Census 2000 (PARTIAL ⏳):**
1. ✅ **Parse Tract-Level .upl Files**: `scripts/data/census/parse_pl94171_tracts_2000.py`
   - Extracted 65,734 tracts from .upl format geographic files (SUMLEV=14000000)
   - Output: `data/processed/census_2000/[state]_tracts_2000_population.csv` (all 50 states)

2. ⏳ **Download NHGIS 2000 Tract Shapefiles**: `scripts/data/geography/download_tiger_tracts_2000.py`
   - Script provides instructions for manual NHGIS download
   - 2000 shapefiles not available via direct download (requires free NHGIS account)
   - **Action Required:** Visit https://www.nhgis.org/ to download 2000 tract boundaries

3. ✅ **Merge Script Ready**: `scripts/data/merge_tracts_with_geometries_2000.py`
   - Script created and ready to merge once NHGIS shapefiles are available
   - Output: `data/tracts/2000/[state]_tracts_2000.parquet` (will be created after NHGIS download)

**Current Outputs:**
- ✅ `data/tracts/2010/[state]_tracts_2010.parquet` (all 50 states) - **READY FOR REDISTRICTING**
- ⏳ `data/tracts/2000/[state]_tracts_2000.parquet` (pending NHGIS shapefiles)
- Compatible with existing 2020 tract-level pipeline

**Benefit:**
- ✅ **2010**: Can now run `run_redistricting.py --year 2010` for historical validation
- ⏳ **2000**: Will enable `run_redistricting.py --year 2000` once NHGIS shapefiles are obtained

#### Phase 1: Data Processing Infrastructure (Block-Level)

**New Scripts:**
```bash
# Parse block-level PL94-171 data into unified format
scripts/data/census/parse_pl94171_blocks.py --year 2000
scripts/data/census/parse_pl94171_blocks.py --year 2010
scripts/data/census/parse_pl94171_blocks.py --year 2020

# Download block-level TIGER/Line shapefiles
scripts/data/geography/download_block_shapefiles.py --year 2000
scripts/data/geography/download_block_shapefiles.py --year 2010
scripts/data/geography/download_block_shapefiles.py --year 2020

# Merge block population with block geometries
scripts/data/preprocessing/merge_block_geography.py --year 2000
scripts/data/preprocessing/merge_block_geography.py --year 2010
scripts/data/preprocessing/merge_block_geography.py --year 2020
```

**Output Format:**
```
data/raw/blocks/
├── 2000/
│   ├── blocks_01_2000.parquet  # Alabama
│   ├── blocks_02_2000.parquet  # Alaska
│   └── ...
├── 2010/
│   └── blocks_*_2010.parquet
└── 2020/
    └── blocks_*_2020.parquet
```

**Block GEOID Format:**
- **15 digits**: SSSCCCTTTTTTBBB
  - SSS: State FIPS (3 digits)
  - CCC: County FIPS (3 digits)
  - TTTTTT: Tract code (6 digits)
  - BBB: Block code (3 digits)
- Example: `010010201001000` = Alabama (01), Autauga County (001), Tract 020100, Block 1000

#### Phase 2: Block-to-Tract Aggregation

**New Module:** `src/apportionment/data/aggregation.py`

```python
def aggregate_blocks_to_tracts(blocks_gdf):
    """
    Aggregate census blocks to tracts.

    Args:
        blocks_gdf: GeoDataFrame with block-level data
                   Columns: GEOID (15 digits), population, geometry

    Returns:
        GeoDataFrame with tract-level data
        Columns: GEOID (11 digits), population, geometry
    """
    # Extract tract GEOID (first 11 digits of block GEOID)
    blocks_gdf['tract_geoid'] = blocks_gdf['GEOID'].str[:11]

    # Aggregate population by tract
    tracts_gdf = blocks_gdf.groupby('tract_geoid').agg({
        'population': 'sum',
        'geometry': lambda x: unary_union(x)  # Merge geometries
    }).reset_index()

    # Rename tract_geoid to GEOID
    tracts_gdf.rename(columns={'tract_geoid': 'GEOID'}, inplace=True)

    return gpd.GeoDataFrame(tracts_gdf, geometry='geometry')
```

**Usage:**
```python
# Load blocks
blocks_gdf = load_blocks('CA', year='2010')

# Aggregate to tracts
tracts_gdf = aggregate_blocks_to_tracts(blocks_gdf)

# Use tracts for redistricting (smaller graph)
graph = build_adjacency_graph(tracts_gdf)
```

#### Phase 3: Configuration and Granularity Selection

**Update config files:**
```python
# scripts/config_2000.py
STATE_CONFIG_2000 = {
    'alabama': {
        'name': 'Alabama',
        'code': '01',
        'districts': 7,
        'granularity': 'tract',  # or 'block'
    },
    # ...
}

# scripts/config_2010.py
STATE_CONFIG_2010 = {
    # Same structure
}

# scripts/config_2020.py
STATE_CONFIG_2020 = {
    # Same structure
}
```

**Command-Line Option:**
```bash
# Use tract-level (default, faster)
python scripts/pipeline/run_state_redistricting.py --state CA --year 2010 --granularity tract

# Use block-level (high resolution, slower)
python scripts/pipeline/run_state_redistricting.py --state CA --year 2020 --granularity block
```

#### Phase 4: Update Pipeline Scripts

**Files to Modify:**

1. **`scripts/pipeline/process_single_state.py`**
   ```python
   # Add granularity parameter
   parser.add_argument('--granularity', choices=['block', 'tract'], default='tract')

   # Load data based on granularity
   if args.granularity == 'block':
       units_gdf = load_blocks(state_code, args.year)
   else:  # tract
       if args.year in [2000, 2010]:
           # Load blocks and aggregate
           blocks_gdf = load_blocks(state_code, args.year)
           units_gdf = aggregate_blocks_to_tracts(blocks_gdf)
       else:  # 2020
           units_gdf = load_tracts(state_code, args.year)
   ```

2. **`scripts/data/geography/build_adjacency_graphs.py`**
   - Support block-level adjacency construction
   - Cache block adjacency graphs (larger files)

3. **`scripts/pipeline/run_complete_redistricting.py`**
   - Thread `--granularity` parameter through to state processing

#### Phase 5: Performance Optimization

**Block-Level Challenges:**
- California: ~710K blocks vs 23K tracts (31x larger)
- Graph construction: O(N²) → 961x slower worst case
- METIS partitioning: O(N log k) → 31x slower

**Optimization Strategies:**

1. **Spatial Indexing:**
   ```python
   # Use R-tree for faster neighbor queries
   spatial_index = blocks_gdf.sindex
   candidates = blocks_gdf.iloc[list(spatial_index.query(block.geometry))]
   ```

2. **Parallel Processing:**
   - Build adjacency graphs in parallel (by county)
   - Merge county graphs into state graph

3. **Progressive Coarsening:**
   - Start with blocks for first few rounds
   - Aggregate to block groups after districts get smaller
   - Trade-off: resolution vs speed

4. **Caching:**
   - Cache block adjacency graphs (reuse across runs)
   - Large files: CA blocks ~2GB graph

### Multi-Year Support

**2000 Census:**
- 8,205,582 blocks
- Uses 2000 tract definitions
- PL94-171 data format (legacy)

**2010 Census:**
- 11,166,336 blocks
- Uses 2010 tract definitions
- PL94-171 data format

**2020 Census:**
- 8,173,739 blocks (fewer than 2010 due to consolidation)
- Uses 2020 tract definitions
- PL94-171 data format (modern)

**Tract Compatibility:**
- 2000 and 2010 have different tract boundaries
- Must use year-matched tract definitions
- Can't compare 2000 blocks → 2010 tracts directly

### Output Structure

```
data/raw/blocks/
├── 2000/
│   ├── blocks_01_2000.parquet
│   └── ...
├── 2010/
│   ├── blocks_01_2010.parquet
│   └── ...
└── 2020/
    ├── blocks_01_2020.parquet
    └── ...

data/processed/tracts_from_blocks/
├── 2000/
│   ├── tracts_01_2000.parquet  # Aggregated from blocks
│   └── ...
└── 2010/
    ├── tracts_01_2010.parquet
    └── ...
```

### Benefits
- **Historical Analysis**: Compare redistricting across 2000, 2010, 2020
- **Higher Resolution**: Block-level allows finer-grained districts
- **Flexibility**: Choose granularity based on computation budget
- **Research**: Study effect of unit size on compactness/fairness

### Challenges
- **Computation Time**: Block-level 10-100x slower than tracts
- **Memory Usage**: Large states may require 16GB+ RAM
- **Storage**: Block data ~50GB compressed across 3 years
- **Complexity**: More code paths, more edge cases

### Implementation Steps

1. **Phase 1**: Download block shapefiles for 2000, 2010, 2020 (4 hours)
2. **Phase 2**: Implement block-to-tract aggregation (2 hours)
3. **Phase 3**: Add configuration and CLI options (2 hours)
4. **Phase 4**: Update pipeline scripts (3 hours)
5. **Phase 5**: Optimize for block-level performance (4 hours)
6. **Phase 6**: Test on small states (Vermont, Wyoming) (2 hours)
7. **Phase 7**: Document findings and best practices (1 hour)

**Total Estimated Time**: 18 hours

### Files to Create
- `scripts/data/geography/download_block_shapefiles.py`
- `scripts/data/census/download_block_population.py`
- `src/apportionment/data/aggregation.py`

### Files to Modify
- `scripts/pipeline/process_single_state.py`
- `scripts/pipeline/run_complete_redistricting.py`
- `scripts/data/geography/build_adjacency_graphs.py`
- `scripts/config_2000.py`, `scripts/config_2010.py`, `scripts/config_2020.py`

### Success Criteria
- Block-level data available for 2000, 2010, 2020
- Automatic tract aggregation works for 2000/2010
- Pipeline supports both block and tract granularity
- Block-level redistricting produces valid districts
- Performance acceptable for small-medium states
- Documentation includes granularity trade-offs

### Estimated Complexity
**Very High** (18+ hours)
- Multi-year data management
- Performance optimization required
- Large data volume
- Complex aggregation logic

---
