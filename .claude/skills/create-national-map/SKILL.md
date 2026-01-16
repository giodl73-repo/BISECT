---
name: create-national-map
description: Generate national-level visualization maps showing all 435 congressional districts across 50 states with Alaska and Hawaii insets. Creates US-wide maps for political lean, demographics, compactness, or round progression. Use when you need to visualize national redistricting patterns.
allowed-tools:
  - Read
  - Bash
  - Glob
  - Grep
user-invocable: true
---

# Create National Map

## Overview

Generate comprehensive national-level visualization maps showing all 435 congressional districts across the United States. Includes special handling for Alaska and Hawaii as insets with proper scaling and positioning.

## Prerequisites

Before creating national maps, validate:
1. **All state data exists** for 50 states + DC (`outputs/us_{year}_{version}/states/*/data/districts.csv`)
2. **Census tract data** available for all states
3. **Analysis completed** for all states (if creating analysis maps)
4. **National aggregation** completed (runs in post-processing stage)

## When to Use This Skill

- User says: "Create a national map" or "US map"
- User says: "Show all 435 districts on one map"
- User wants to visualize national patterns (political, demographic, compactness)
- User needs overview of entire redistricting result
- After completing 50-state pipeline run

## Map Types

### 1. All Districts Map
**Purpose**: Show all 435 congressional districts on single US map

**What it shows**:
- All district boundaries across 50 states
- Distinct colors for each district
- Alaska and Hawaii as insets (scaled appropriately)
- State boundaries in bold
- Total: 435 districts labeled

**Required data**:
- `districts.csv` for all 50 states

**Example command**:
```bash
python scripts/visualization/create_national_map.py \
  --year 2020 \
  --version v1 \
  --type districts \
  --dpi 150
```

### 2. National Political Lean Map
**Purpose**: Show partisan lean across all 435 districts (2020 only)

**What it shows**:
- Blue-to-red gradient for all districts
- Total D/R seat counts (e.g., "D: 222 | R: 213")
- Alaska/Hawaii insets
- Swing districts highlighted

**Required data**:
- `political_lean.csv` for all states (2020 with election data)

**Example command**:
```bash
python scripts/political/analyze_districts.py \
  --year 2020 \
  --version v1 \
  --scope national
```

### 3. National Demographic Maps
**Purpose**: Show racial/ethnic composition patterns nationally

**What it shows**:
- Color intensity by percentage across all districts
- Three variants: White %, Minority %, Majority-minority
- Alaska/Hawaii insets
- National statistics

**Required data**:
- `demographic_composition.csv` for all states

**Example command**:
```bash
python scripts/demographic/analyze_demographics.py \
  --year 2020 \
  --version v1 \
  --scope national
```

### 4. National Compactness Map
**Purpose**: Show compactness scores across all districts

**What it shows**:
- Red (low) to green (high) compactness gradient
- Two metrics: Polsby-Popper and Reock
- Alaska/Hawaii insets
- National mean/median statistics

**Required data**:
- `compactness.csv` for all states

**Example command**:
```bash
python scripts/compactness/analyze_compactness.py \
  --year 2020 \
  --version v1 \
  --scope national
```

### 5. National Round Progression Maps
**Purpose**: Show districts created at each recursive bisection round nationally

**What it shows**:
- Series of maps (one per round: 1-9)
- Round 1: 2 regions → Round 9: 435 districts
- Color-coded by round
- Shows algorithm's hierarchical structure

**Required data**:
- `rounds_hierarchy.csv` for all states
- `rounds_aggregated.csv` (national aggregation)

**Example command**:
```bash
python scripts/visualization/visualize_rounds.py \
  --year 2020 \
  --version v1 \
  --scope national
```

## Workflow

### Step 1: Validate All State Data Complete

Check that all 50 states have finished:
```bash
# Count states with district data
ls outputs/us_2020_v1/states/*/data/districts.csv | wc -l
# Should be 51 (50 states + DC)

# Check for missing states
python scripts/validation/check_data_completeness.py --year 2020 --version v1
```

If any states missing:
- Run redistricting for missing states
- Or continue with partial data (graceful degradation)

### Step 2: Determine Map Type

Ask user if not specified:
- Basic districts map (all 435)?
- Political lean (2020 only)?
- Demographics (White %, Minority %, Majority-minority)?
- Compactness (Polsby-Popper or Reock)?
- Round progression (algorithm visualization)?

### Step 3: Run National Aggregation (If Needed)

National maps require post-processing aggregation:
```bash
# This runs automatically during pipeline
# Or run manually:
python scripts/pipeline/run_complete_redistricting.py \
  --year 2020 \
  --version v1 \
  --skip-redistricting \
  --skip-analysis
```

This creates:
- `outputs/us_2020_v1/data/national_summary.csv`
- `outputs/us_2020_v1/data/rounds_aggregated.csv`

### Step 4: Generate National Map

Run appropriate script with `--scope national`:

**All Districts**:
```bash
python scripts/visualization/create_national_map.py \
  --year 2020 \
  --version v1 \
  --dpi 150
```

**Political Lean** (2020 only):
```bash
python scripts/political/analyze_districts.py \
  --year 2020 \
  --version v1 \
  --scope national
```

**Demographics**:
```bash
python scripts/demographic/analyze_demographics.py \
  --year 2020 \
  --version v1 \
  --scope national
```

**Compactness**:
```bash
python scripts/compactness/analyze_compactness.py \
  --year 2020 \
  --version v1 \
  --scope national
```

**Round Progression**:
```bash
python scripts/visualization/visualize_rounds.py \
  --year 2020 \
  --version v1 \
  --scope national
```

### Step 5: Verify Output

Check that national maps were created:
```bash
# Main national maps
ls outputs/us_2020_v1/maps/us_all_districts.png
ls outputs/us_2020_v1/maps/us_political_lean.png
ls outputs/us_2020_v1/maps/us_compactness_polsby_popper.png
ls outputs/us_2020_v1/maps/us_demographic_white_percentage.png

# Round progression maps (series)
ls outputs/us_2020_v1/maps/rounds/us_round_*.png
```

Typical file sizes (DPI 150):
- **All districts**: ~3-5 MB (complex geometry)
- **Analysis maps**: ~2-4 MB
- **Round maps**: ~1-3 MB each (9 maps total)

## Alaska and Hawaii Inset Handling

National maps require special handling for Alaska and Hawaii:

### Alaska Inset
- **Scaling**: 30% of actual size (Alaska is huge!)
- **Position**: Bottom-left corner of map
- **Bounding box**: Custom coordinates to fit inset
- **Districts**: 1 at-large district (no internal boundaries)

### Hawaii Inset
- **Scaling**: 100% (actual size works well)
- **Position**: Center-bottom of map (near Alaska)
- **Bounding box**: Custom coordinates
- **Districts**: 2 districts (island-based divisions)

### Continental US (Lower 48)
- **Projection**: Albers Equal Area Conic (standard for US maps)
- **Bounds**: Optimized to show contiguous states
- **Districts**: 432 districts (435 - 1 AK - 2 HI)

### Implementation Pattern

```python
# Pseudocode for national map with insets
import matplotlib.pyplot as plt
import geopandas as gpd

# Main axes for continental US
fig, ax_main = plt.subplots(figsize=(20, 12))

# Load all state geometries
all_states = gpd.read_file('all_tracts.parquet')

# Plot continental US (exclude AK, HI)
continental = all_states[~all_states['state'].isin(['alaska', 'hawaii'])]
continental.plot(ax=ax_main, ...)

# Create Alaska inset
ax_ak = fig.add_axes([0.08, 0.15, 0.15, 0.15])  # [left, bottom, width, height]
alaska = all_states[all_states['state'] == 'alaska']
alaska.plot(ax=ax_ak, ...)

# Create Hawaii inset
ax_hi = fig.add_axes([0.25, 0.15, 0.1, 0.1])
hawaii = all_states[all_states['state'] == 'hawaii']
hawaii.plot(ax=ax_hi, ...)

# Add titles and labels
ax_main.set_title('US Congressional Districts (2020)', fontsize=20)
ax_ak.set_title('Alaska', fontsize=10)
ax_hi.set_title('Hawaii', fontsize=10)

plt.savefig('us_all_districts.png', dpi=150, bbox_inches='tight')
```

## Map Styling Guidelines

### Colors

**All districts map**:
- Use subtle district colors (light pastels)
- Emphasize state boundaries (bold black)
- District boundaries (thin gray)

**Political lean**:
- Strong blue-to-red diverging colormap
- 50% = white (swing districts)
- Colorbar with percentages

**Demographics**:
- Sequential yellow-to-red (`YlOrRd`)
- 0% = light, 100% = dark
- Clear colorbar labels

**Compactness**:
- Red (0.0) to yellow (0.5) to green (1.0)
- Colormap: `RdYlGn`
- Show national mean as reference line

### Boundaries

**State boundaries**:
- Color: Black
- Width: 2.0-3.0 points (prominent)
- Separates state groupings

**District boundaries**:
- Color: Gray (#808080) or white
- Width: 0.3-0.5 points (subtle)
- Shows internal structure

**Inset boundaries**:
- Thin black rectangle around insets
- Separates from main map

### Annotations

**Title**:
- Format: "US Congressional Districts (2020)"
- Font size: 18-24 points
- Position: Top center

**Statistics**:
- Political: "D: 222 | R: 213 (Total: 435)"
- Compactness: "National Mean PP: 0.42"
- Demographics: "Majority-Minority Districts: 145"
- Position: Bottom or top subtitle

**Inset labels**:
- "Alaska" and "Hawaii" labels on insets
- Font size: 10-12 points

## Output Locations

National maps are saved to top-level maps directory:
```
outputs/us_{year}_{version}/maps/
├── us_all_districts.png                    # All 435 districts
├── us_political_lean.png                   # National partisan lean
├── us_compactness_polsby_popper.png        # National PP scores
├── us_compactness_reock.png                # National Reock scores
├── us_demographic_white_percentage.png     # National White %
├── us_demographic_minority_percentage.png  # National Minority %
├── us_demographic_majority_minority.png    # MM districts
└── rounds/                                 # Round progression
    ├── us_round_1.png                      # 2 regions
    ├── us_round_2.png                      # 4 regions
    ├── ...
    └── us_round_9.png                      # 435 districts
```

## Troubleshooting

**Common Issues**:

**Missing state data**:
```
Error: Cannot create national map, missing data for 5 states
Solution: Run redistricting for missing states or skip with --allow-partial
```

**Alaska/Hawaii not showing**:
```
Issue: Insets are blank or missing
Solution: Check that alaska.parquet and hawaii.parquet exist
          Verify inset axes coordinates are correct
```

**Out of memory**:
```
Error: MemoryError when loading all 50 states
Solution: Load states incrementally, or reduce DPI
          Close other applications
```

**Projection errors**:
```
Error: CRS mismatch when combining states
Solution: Reproject all states to same CRS (Albers Equal Area)
          Use gdf.to_crs('EPSG:5070')
```

**Colorbar issues**:
```
Issue: Colorbar doesn't match data range
Solution: Explicitly set vmin/vmax based on data
          Use robust percentile clipping (2nd, 98th percentile)
```

## Performance Notes

**Typical runtime per national map**:
| Map Type | DPI 150 | DPI 300 |
|----------|---------|---------|
| All districts | ~60-90 sec | ~3-5 min |
| Political | ~45-60 sec | ~2-3 min |
| Demographic | ~45-60 sec | ~2-3 min |
| Compactness | ~45-60 sec | ~2-3 min |
| Round series | ~5-8 min | ~15-20 min |

**Bottlenecks**:
- Loading 50 state geometries (~1M tracts)
- Geometry simplification for rendering
- Matplotlib rendering (scales with complexity × DPI²)

**Optimization tips**:
- Simplify geometries: `gdf.simplify(tolerance=100)`
- Load only necessary columns
- Use lower DPI for drafts (DPI 50-75)
- Cache loaded geometries between maps

## Advanced Usage

### Partial National Maps

Create national map with subset of states:
```bash
# Example: Only Western states
python scripts/visualization/create_national_map.py \
  --year 2020 \
  --version v1 \
  --states-filter "california,oregon,washington,nevada,arizona"
```

### Custom Inset Positions

Modify inset positions in script:
```python
# Custom Alaska position (larger, different location)
ax_ak = fig.add_axes([0.05, 0.20, 0.20, 0.20])
```

### High-Resolution for Publication

For papers/presentations:
```bash
python scripts/visualization/create_national_map.py \
  --year 2020 \
  --version v1 \
  --dpi 300 \
  --style paper \
  --format pdf  # Vector format for scaling
```

### Comparison Maps (Side-by-Side)

Compare two years or modes:
```python
# Create figure with two subplots
fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(40, 12))

# Plot 2010 on left
plot_national_map(year=2010, version='v1', ax=ax1)
ax1.set_title('2010 Districts')

# Plot 2020 on right
plot_national_map(year=2020, version='v1', ax=ax2)
ax2.set_title('2020 Districts')

plt.savefig('comparison_2010_vs_2020.png', dpi=150)
```

## Related Skills

- `/create-state-map` - Generate individual state maps
- `/run-redistricting` - Complete pipeline that creates national maps
- `/generate-dashboard` - Include national maps in dashboard
- `/create-pedagogical-example` - Create educational district examples

## Integration with Pipeline

National maps are automatically created during post-processing stage:

**Pipeline sequence**:
1. **Redistricting** (parallel per-state) → Creates `districts.csv` for each state
2. **Analysis** (parallel per-state) → Creates analysis CSVs per state
3. **State maps** (parallel per-state) → Creates per-state PNG maps
4. **Post-processing** (sequential) → **Creates national maps** ← This skill
5. **Dashboard** → Embeds national maps

**Automatic execution**:
```bash
# National maps created automatically at end
python scripts/pipeline/run_complete_redistricting.py \
  --year 2020 \
  --version v1
```

**Manual execution** (regenerate national maps only):
```bash
# Skip redistricting and analysis, just regenerate national maps
python scripts/pipeline/run_complete_redistricting.py \
  --year 2020 \
  --version v1 \
  --skip-redistricting \
  --skip-analysis \
  --force
```

## What You'll Get

After successful creation:
- **US-wide map** showing all 435 districts
- **Alaska inset** (scaled 30%, bottom-left)
- **Hawaii inset** (actual size, center-bottom)
- **Analysis maps** (political, demographic, compactness if data available)
- **Round progression series** (9 maps showing recursive bisection)
- **High-resolution PNGs** ready for web/print/presentation

## Next Steps

- View national maps in output directory
- Include in dashboard with `/generate-dashboard`
- Compare across years (2000, 2010, 2020)
- Compare across modes (edge-weighted vs unweighted)
- Use in presentations/papers
