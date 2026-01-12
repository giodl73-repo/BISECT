# Pipeline Enhancements - January 2026

## Overview

Eight enhancements to integrate into the main redistricting pipeline to provide more comprehensive analysis, visualization, and algorithmic improvements for congressional district generation.

**Status Summary:**
- ✅ Enhancement 1: Compactness Integration - **COMPLETED**
- ✅ Enhancement 2: D/R Seat Totals - **COMPLETED**
- ✅ Enhancement 3: National Maps - **COMPLETED**
- 🚧 Enhancement 4: Urban Metro Areas - **IN PROGRESS**
- 📋 Enhancement 5: National Round Progression - **PLANNED**
- 📋 Enhancement 6: System Architecture Diagrams - **PLANNED**
- 📋 Enhancement 7: Edge-Weighted Recursive Bisection - **PLANNED**
- 📋 Enhancement 8: Block-Level Data Support - **PLANNED**

---

## Enhancement 1: Integrate Compactness into Main Pipeline ✅ COMPLETED

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

**Completion Date:** January 10, 2026
**Implementation:** Compactness metrics integrated into `create_final_district_summary.py`. All district_summary.csv files now include polsby_popper and reock columns. Compactness visualization pipeline added as post-processing steps.

---

## Enhancement 2: Add D/R Seat Totals to Political Maps ✅ COMPLETED

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

**Completion Date:** January 11, 2026
**Implementation:** D/R seat count annotation added to `visualize_partisan_lean.py`. All political maps now display seat totals in upper-right corner with readable styling.

---

## Enhancement 3: Create National Political and Demographic Maps ✅ COMPLETED

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

---

## Enhancement 4: Create Urban Metro Area District Maps (MSA/MCSA) 🚧 IN PROGRESS

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

## Enhancement 5: Create National Round Progression Maps 📋 PLANNED

### Goal
Create national-level visualization of recursive bisection progression showing rounds 1-6+ across all states.

### Description
Generate a series of national maps showing how the US is progressively divided:
- Round 1: 2 regions (first bisection)
- Round 2: 4 regions (second bisection)
- Round 3: 8 regions
- Round 4: 16 regions
- Round 5: 32 regions
- Round 6: 64 regions
- Continue through later rounds as states complete their divisions

### Implementation Plan

#### Data Collection Phase
- Aggregate round data from all 50 states' `rounds/round_N_assignments.pkl` files
- Track which states have completed which rounds
- Handle states with different final round counts (1-district states vs 52-district California)

#### Visualization Script
**Create:** `scripts/pipeline/create_us_national_rounds_progression.py`

##### Outputs
- `us_national_round_1_2020.png` - 2 regions
- `us_national_round_2_2020.png` - 4 regions
- `us_national_round_3_2020.png` - 8 regions
- `us_national_round_4_2020.png` - 16 regions
- `us_national_round_5_2020.png` - 32 regions
- `us_national_round_6_2020.png` - 64 regions
- Continue for later rounds

##### Visual Specifications
- Use consistent color scheme across rounds
- Show state boundaries with districts/regions overlaid
- As states complete their final districts, show them fully divided in subsequent rounds
- Size: 20x12 inches, DPI: 150
- Title: "U.S. Congressional Districts - Round N (2^N regions)"

#### Pipeline Integration
Add as post-processing step after `create_us_rounds_hierarchy.py`:
```python
# CREATE US NATIONAL ROUND PROGRESSION
if output_dir.exists() or args.print_only:
    pipeline_steps.append({
        'name': 'Create national round progression maps',
        'command': f'{sys.executable} scripts/pipeline/create_us_national_rounds_progression.py --year {args.year} --version {args.version} --output-dir {output_dir} --dpi {args.dpi}'.strip(),
        'critical': False
    })
```

#### Dashboard Integration
Add to USA row, Rounds tab:
- Show progressive sequence of national bisection
- Allow users to see national-level recursive division pattern
- Display maps in order: Round 1 → Round 2 → ... → Final

### Benefits
- Visualize national-level recursive bisection strategy
- Understand how equal-population constraint affects regional divisions
- Compare bisection patterns across geographic regions
- Educational tool for understanding METIS recursive algorithm at scale

### Estimated Complexity
**Medium** (2-3 hours)
- Similar to existing national map generation
- Complexity in aggregating round data across states with different completion points

---

## Implementation Order

### Priority 1: Enhancement 1 (Compactness Integration) ✅ COMPLETED
- **Effort**: Low (code already exists, just needs integration)
- **Impact**: High (adds critical metric to all outputs)
- **Time**: 30 minutes

### Priority 2: Enhancement 2 (D/R Seat Totals) ✅ COMPLETED
- **Status**: Completed January 11, 2026
- **Effort**: Low (simple text annotation)
- **Impact**: Medium (improves political map readability)
- **Time**: 20 minutes

### Priority 3: Enhancement 3 (National Maps) ✅ COMPLETED
- **Status**: Completed January 11, 2026
- **Effort**: Medium (new script, handle all states, insets)
- **Impact**: High (enables national-level analysis)
- **Time**: 2 hours

### Priority 4: Enhancement 4 (Metro Area Maps) 🚧 IN PROGRESS
- **Effort**: High (new script, MSA data, 20+ maps)
- **Impact**: Medium (nice-to-have for urban analysis)
- **Time**: 4 hours

### Priority 5: Enhancement 5 (National Round Progression) 📋 PLANNED
- **Effort**: Medium (similar to existing national map generation)
- **Impact**: High (visualize recursive bisection at scale)
- **Time**: 2-3 hours

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

---

## Enhancement 6: Create System Architecture Diagrams 📋 PLANNED

### Goal
Generate visual architecture diagrams showing system components, data flow, and relationships.

### Description
Create comprehensive diagrams to supplement the written documentation:
- **System Overview**: High-level component diagram (data → processing → visualization)
- **Pipeline Flow**: Step-by-step redistricting pipeline with data transformations
- **Script Hierarchy**: Tree showing script dependencies and execution order
- **Data Flow**: How data moves from Census sources through to final outputs
- **Module Structure**: Library organization (src/apportionment)

### Implementation Plan

#### Tools/Format
- **Mermaid**: Markdown-based diagrams (GitHub-compatible, version-controllable)
- **GraphViz/DOT**: More complex layouts if needed
- **Draw.io**: Editable diagrams with source files (.drawio)

#### Diagrams to Create

**1. System Architecture Overview**
```mermaid
graph TB
    A[Census Data] --> B[Data Processing]
    B --> C[Graph Construction]
    C --> D[METIS Partitioning]
    D --> E[District Assignment]
    E --> F[Analysis]
    F --> G[Visualization]
    G --> H[Dashboard]
```

**2. Pipeline Flow Diagram**
- Show complete 50-state pipeline
- Highlight parallel processing
- Indicate skip logic and resumability
- Mark critical vs optional steps

**3. Script Dependency Graph**
- run_complete_redistricting.py at root
- Branch to per-state processing
- Show post-processing aggregation
- Display analysis stages

**4. Data Format Evolution**
- Raw TIGER/Line shapefiles → Parquet tracts
- Election data → Tract-level aggregates
- District assignments → Summary CSVs
- GeoDataFrames → PNG maps

**5. Module Structure**
- src/apportionment/ package layout
- scripts/ directory organization
- Data directory structure

#### Output Locations
- `docs/diagrams/` - Source files (.mmd, .drawio)
- `docs/diagrams/rendered/` - PNG exports
- Embed in `docs/ARCHITECTURE.md` via relative paths

#### Integration
Update `docs/ARCHITECTURE.md` to include diagram embeds:
```markdown
## System Architecture

![System Overview](diagrams/rendered/system_overview.png)

The redistricting system consists of...
```

### Benefits
- Visual understanding for new developers
- Easier to explain system design
- Quick reference for component relationships
- Documentation in ARCHITECTURE.md more accessible

### Estimated Complexity
**Medium** (2-3 hours)
- Straightforward with Mermaid
- Most information already documented
- Main effort in layout and clarity

---

## Enhancement 7: Edge-Weighted Recursive Bisection Variant 📋 PLANNED

### Goal
Implement a variant of the recursive bisection algorithm that minimizes geographic boundary length when partitioning, producing more compact districts.

### Current State
- Current algorithm uses METIS with uniform edge weights
- METIS minimizes edge cuts (number of edges crossing partition boundary)
- Does not consider actual geographic distance/boundary length
- Result: Districts may have long, winding boundaries

### Proposed Enhancement
Use actual boundary lengths as edge weights in METIS partitioning:
- **Edge weight = shared boundary length** between adjacent tracts
- METIS minimizes sum of edge weights (total boundary length)
- Result: Districts with shorter perimeters (better compactness)

### Implementation Plan

#### Phase 1: Update Adjacency Graph Construction

**File**: `src/apportionment/data/adjacency.py`

**Changes**:
```python
def build_adjacency_graph_with_weights(tracts_gdf):
    """
    Build adjacency graph with edge weights = boundary length.

    For each pair of adjacent tracts:
    1. Find intersection of boundaries (shared edge)
    2. Calculate length of shared boundary
    3. Store as edge weight in NetworkX graph
    """
    graph = nx.Graph()

    # Add nodes
    for idx, tract in tracts_gdf.iterrows():
        graph.add_node(tract['GEOID'],
                      population=tract['population'],
                      geometry=tract['geometry'])

    # Add edges with boundary length weights
    for i, tract_i in tracts_gdf.iterrows():
        for j, tract_j in tracts_gdf.iterrows():
            if i >= j:
                continue

            if tract_i.geometry.touches(tract_j.geometry):
                # Calculate shared boundary length
                intersection = tract_i.geometry.intersection(tract_j.geometry)

                if intersection.geom_type in ['LineString', 'MultiLineString']:
                    # Shared edge boundary
                    boundary_length = intersection.length
                elif intersection.is_empty or intersection.geom_type == 'Point':
                    # Only touches at corner/point - use minimal weight
                    boundary_length = 0.001  # Small non-zero value
                else:
                    # Handle water-based adjacency or other special cases
                    boundary_length = 1.0  # Default weight

                graph.add_edge(tract_i['GEOID'], tract_j['GEOID'],
                              weight=boundary_length)

    return graph
```

#### Phase 2: Update METIS Wrapper

**File**: `src/apportionment/partition/metis_wrapper.py`

**Changes**:
```python
def partition_graph_weighted(graph, num_parts=2, **options):
    """
    Partition graph using edge weights.

    METIS will minimize: sum of (edge_weight * cut_indicator)
    With boundary_length weights, this minimizes total boundary length.
    """
    # Extract edge weights
    adjwgt = []  # Edge weights array for METIS
    for u, v, data in graph.edges(data=True):
        weight = data.get('weight', 1.0)
        adjwgt.append(int(weight * 1000))  # Scale and convert to integer

    # Call METIS with adjwgt parameter
    return _call_metis_with_weights(graph, num_parts, adjwgt, **options)
```

#### Phase 3: Add Water-Based Adjacency Handling

**Special Cases to Handle**:

1. **Water-based adjacency** (e.g., tracts separated by river but connected by bridge):
   - Currently: Added as edge with no geometric boundary
   - Proposed: Use fixed penalty weight (e.g., median tract boundary length)
   - Rationale: Discourage but allow splitting across water

2. **Point adjacency** (tracts touching at single corner):
   - Currently: Treated same as edge adjacency
   - Proposed: Use very small weight (0.001)
   - Rationale: Easy to split at corners, minimal boundary length

3. **Validation**:
   ```python
   # Check for edges without geometric boundary
   for u, v, data in graph.edges(data=True):
       if 'weight' not in data or data['weight'] == 0:
           # Water-based or point adjacency
           # Assign default weight
           data['weight'] = calculate_default_weight(graph)
   ```

#### Phase 4: Configuration and Testing

**New Parameters**:
```python
# In scripts/config_2020.py
BISECTION_CONFIG = {
    'use_edge_weights': False,  # Default: original uniform weights
    'weight_type': 'boundary_length',  # or 'uniform'
    'water_adjacency_weight': 'median',  # or fixed value
}
```

**Testing Plan**:
1. Run both algorithms on same state (e.g., Colorado - simple geometry)
2. Compare compactness metrics:
   - Polsby-Popper scores
   - Reock scores
   - Average boundary length per district
3. Compare with ground truth (actual congressional districts)
4. Validate population balance maintained

#### Output Changes

**New Column in district_summary.csv**:
- `boundary_length` - Total perimeter length of district boundary

**Comparison Script**:
```bash
# Run both variants
python scripts/pipeline/run_state_redistricting.py --state CO --version v1 --weight-type uniform
python scripts/pipeline/run_state_redistricting.py --state CO --version v2 --weight-type boundary_length

# Compare compactness
python scripts/analysis/compare_bisection_variants.py --state CO --versions v1 v2
```

### Benefits
- **Better Compactness**: Minimizing boundary length directly optimizes for compact shapes
- **Geographic Intuition**: Edge weights reflect real spatial relationships
- **Fairer Districts**: Compact districts reduce gerrymandering potential
- **Research Value**: Compare algorithmic approaches empirically

### Challenges
- **Computation Time**: Calculating boundary intersections adds overhead
- **METIS Integer Weights**: Need to scale/discretize floating-point lengths
- **Water Bodies**: Requires careful handling of non-geometric adjacencies
- **CRS Selection**: Boundary lengths depend on projection (use equal-area projection)

### Implementation Steps

1. **Phase 1**: Implement weighted graph construction (2 hours)
2. **Phase 2**: Update METIS wrapper to pass edge weights (1 hour)
3. **Phase 3**: Handle water/point adjacencies (2 hours)
4. **Phase 4**: Configuration and testing framework (2 hours)
5. **Phase 5**: Run comparison on 5-10 states (4 hours)
6. **Phase 6**: Document findings and update defaults if beneficial (1 hour)

**Total Estimated Time**: 12 hours

### Files to Modify
- `src/apportionment/data/adjacency.py` - Add boundary length calculation
- `src/apportionment/partition/metis_wrapper.py` - Pass edge weights to METIS
- `scripts/config_2020.py` - Add weight configuration
- `scripts/pipeline/process_single_state.py` - Support weight parameter
- `scripts/analysis/compare_bisection_variants.py` - New comparison script

### Success Criteria
- Edge-weighted variant produces valid districts (population balanced, contiguous)
- Compactness metrics improve by 5-10% on average
- Boundary lengths reduced compared to uniform-weight variant
- Water adjacencies handled gracefully
- Computation time remains reasonable (<2x uniform variant)

### Estimated Complexity
**High** (12+ hours)
- Geometric computation overhead
- METIS integration complexity
- Extensive testing required
- Multiple special cases to handle

---

## Enhancement 8: Block-Level Data Support for Multi-Year Census 📋 PLANNED

### Goal
Support census block-level redistricting for 2000, 2010, and 2020, with automatic tract aggregation for older census years.

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

### Implementation Plan

#### Phase 1: Data Download Infrastructure

**New Scripts:**
```bash
# Download block-level shapefiles for all years
scripts/data/geography/download_block_shapefiles.py --year 2000
scripts/data/geography/download_block_shapefiles.py --year 2010
scripts/data/geography/download_block_shapefiles.py --year 2020

# Download block-level PL94-171 population data
scripts/data/census/download_block_population.py --year 2000
scripts/data/census/download_block_population.py --year 2010
scripts/data/census/download_block_population.py --year 2020
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

**Date**: January 2026
**Status**: Enhancements 1-3 complete, 4 in progress, 5-8 planned
**Order**: ✅ 1 → ✅ 2 → ✅ 3 → 🚧 4 → 📋 5 → 📋 6 → 📋 7 → 📋 8
