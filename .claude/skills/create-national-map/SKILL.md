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

Generate comprehensive national maps showing all 435 congressional districts across US. Includes Alaska/Hawaii insets with proper scaling/positioning.

## Prerequisites
All state data exists (`outputs/us_{year}_{version}/states/*/data/districts.csv` for 50 states + DC), census tract data for all states, analysis completed (if creating analysis maps), national aggregation completed (post-processing stage)

## When to Use
User says "Create national/US map/Show all 435 districts", visualize national patterns (political/demographic/compactness), overview of entire redistricting, after 50-state pipeline run

## Map Types

### 1. All Districts
**Purpose**: All 435 districts on single US map
**Shows**: All district boundaries, distinct colors, AK/HI insets (scaled), state boundaries (bold), 435 districts
**Data**: `districts.csv` for all 50 states
**Command**: `python scripts/visualization/create_national_map.py --year 2020 --version v1 --type districts --dpi 150`

### 2. Political Lean
**Purpose**: Partisan lean (2020 only)
**Shows**: Blue-red gradient, total D/R counts ("D: 222 | R: 213"), AK/HI insets, swing districts highlighted
**Data**: `political_lean.csv` for all states (2020 with election data)
**Command**: `python scripts/political/analyze_districts.py --year 2020 --version v1 --scope national`

### 3. Demographics
**Purpose**: Racial/ethnic composition nationally
**Shows**: Color intensity by %, 3 variants (White %, Minority %, Majority-minority), AK/HI insets, national stats
**Data**: `demographic_composition.csv` for all states
**Command**: `python scripts/demographic/analyze_demographics.py --year 2020 --version v1 --scope national`

### 4. Compactness
**Purpose**: Compactness scores nationally
**Shows**: Red (low) → green (high) gradient, 2 metrics (Polsby-Popper, Reock), AK/HI insets, national mean/median
**Data**: `compactness.csv` for all states
**Command**: `python scripts/compactness/analyze_compactness.py --year 2020 --version v1 --scope national`

### 5. Round Progression
**Purpose**: Districts created at each recursive bisection round
**Shows**: Series (9 maps, one per round), Round 1 (2 regions) → Round 9 (435 districts), color by round, algorithm hierarchy
**Data**: `rounds_hierarchy.csv` for all states, `rounds_aggregated.csv` (national)
**Command**: `python scripts/visualization/visualize_rounds.py --year 2020 --version v1 --scope national`

## Workflow

### Step 1: Validate All States Complete
```bash
ls outputs/us_2020_v1/states/*/data/districts.csv | wc -l   # Should be 51
python scripts/validation/check_data_completeness.py --year 2020 --version v1
```
If missing → Run redistricting or continue with partial (graceful degradation)

### Step 2: Determine Map Type
Ask if not specified: All districts, political (2020 only), demographics (White%/Minority%/MM), compactness (PP/Reock), round progression

### Step 3: Run National Aggregation (if needed)
```bash
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version v1 --skip-redistricting --skip-analysis
```
Creates: `outputs/us_2020_v1/data/national_summary.csv`, `rounds_aggregated.csv`

### Step 4: Generate National Map
Run script with `--scope national` (see Map Types for commands)

### Step 5: Verify Output
```bash
ls outputs/us_2020_v1/maps/us_all_districts.png
ls outputs/us_2020_v1/maps/us_political_lean.png
ls outputs/us_2020_v1/maps/us_compactness_polsby_popper.png
ls outputs/us_2020_v1/maps/rounds/us_round_*.png
```
**File sizes** (DPI 150): All districts ~3-5MB (complex geom), Analysis maps ~2-4MB, Round maps ~1-3MB each (9 total)

## Alaska & Hawaii Insets

**Alaska**: Scaling 30% (huge!), position bottom-left, custom bbox, 1 at-large district (no internal boundaries)
**Hawaii**: Scaling 100% (actual size works), position center-bottom (near AK), custom bbox, 2 districts (island-based)
**Continental (Lower 48)**: Albers Equal Area Conic projection, optimized bounds, 432 districts (435 - 1 AK - 2 HI)

**Implementation**:
```python
fig, ax_main = plt.subplots(figsize=(20, 12))   # Main axes for continental
continental = all_states[~all_states['state'].isin(['alaska', 'hawaii'])]
continental.plot(ax=ax_main, ...)
ax_ak = fig.add_axes([0.08, 0.15, 0.15, 0.15])  # Alaska inset
alaska.plot(ax=ax_ak, ...)
ax_hi = fig.add_axes([0.25, 0.15, 0.1, 0.1])    # Hawaii inset
hawaii.plot(ax=ax_hi, ...)
```

## Styling

**Colors**: All districts (light pastels, emphasize state boundaries), Political (strong blue-red diverging, 50%=white swing), Demographics (sequential `YlOrRd`, 0%=light 100%=dark), Compactness (red 0.0 → yellow 0.5 → green 1.0 `RdYlGn`, show national mean)
**Boundaries**: State (black 2.0-3.0pt prominent), District (gray/white 0.3-0.5pt subtle), Inset (thin black rectangle)
**Annotations**: Title "US Congressional Districts (Year)" 18-24pt top center, Stats (political "D: 222 | R: 213", compactness "Mean PP: 0.42", demo "MM Districts: 145"), Inset labels "Alaska"/"Hawaii" 10-12pt

## Output Locations
```
outputs/us_{year}_{version}/maps/
├── us_all_districts.png                    # All 435
├── us_political_lean.png                   # Partisan
├── us_compactness_polsby_popper.png        # PP
├── us_compactness_reock.png                # Reock
├── us_demographic_white_percentage.png     # White%
├── us_demographic_minority_percentage.png  # Minority%
├── us_demographic_majority_minority.png    # MM
└── rounds/                                 # Progression
    ├── us_round_1.png → us_round_9.png   # 2 → 435 districts
```

## Troubleshooting

**Missing data**: `Error: Missing 5 states` → Run redistricting or `--allow-partial`
**AK/HI not showing**: Insets blank → Check `alaska.parquet`/`hawaii.parquet` exist, verify inset axes coordinates
**Out of memory**: `MemoryError loading 50 states` → Load incrementally, reduce DPI, close apps
**Projection errors**: `CRS mismatch` → Reproject to same CRS (Albers Equal Area): `gdf.to_crs('EPSG:5070')`
**Colorbar issues**: Doesn't match range → Set vmin/vmax explicitly, use robust percentile clipping (2nd, 98th)

## Performance

| Map Type | DPI 150 | DPI 300 |
|----------|---------|---------|
| All districts | ~60-90s | ~3-5min |
| Political | ~45-60s | ~2-3min |
| Demographic | ~45-60s | ~2-3min |
| Compactness | ~45-60s | ~2-3min |
| Round series | ~5-8min | ~15-20min |

**Bottlenecks**: Loading 50 states (~1M tracts), geometry simplification, matplotlib rendering (complexity × DPI²)
**Optimization**: Simplify geometries `gdf.simplify(100)`, load only necessary columns, lower DPI for drafts (50-75), cache geometries

## Advanced Usage

**Partial maps**: `--states-filter "california,oregon,washington,nevada,arizona"` (Western states only)
**Custom inset positions**: Modify `ax_ak = fig.add_axes([0.05, 0.20, 0.20, 0.20])` (larger AK, different location)
**High-res publication**: `--dpi 300 --style paper --format pdf` (vector for scaling)
**Comparison side-by-side**: `fig, (ax1, ax2) = plt.subplots(1, 2)` → Plot 2010 left, 2020 right

## Pipeline Integration

**Automatic**: National maps created during post-processing (step 4 after state maps)
**Sequence**: Redistricting (parallel) → Analysis (parallel) → State maps (parallel) → **Post-processing (national maps)** → Dashboard
**Manual regeneration**: `--skip-redistricting --skip-analysis --force` (regenerate national maps only)

## What You'll Get
US-wide map (all 435 districts), Alaska inset (30% scaled, bottom-left), Hawaii inset (100%, center-bottom), analysis maps (political/demographic/compactness if data available), round progression series (9 maps), high-res PNGs (web/print/presentation)

## Related Skills
`/create-state-map` (individual states), `/run-redistricting` (full pipeline), `/generate-dashboard` (include in dashboard), `/create-pedagogical-example` (educational examples)

## Next Steps
View maps in output dir, include in dashboard (`/generate-dashboard`), compare years (2000/2010/2020), compare modes (edge-weighted vs unweighted), use in presentations/papers
