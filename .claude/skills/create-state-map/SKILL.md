---
name: create-state-map
description: Generate state-level visualization maps for redistricting results. Creates maps with customizable color schemes for districts, political lean, demographics, compactness, or round progression. Use when you need to visualize state-specific redistricting data.
allowed-tools:
  - Read
  - Bash
  - Glob
  - Grep
user-invocable: true
---

# Create State Map

Generate high-quality state-level visualization maps for congressional redistricting. Supports multiple map types with consistent styling.

## Prerequisites
**Validate**: District assignments exist (`outputs/us_{year}_{version}/states/{state}/data/districts.csv`), census tract data available (`data/tracts/{year}/{state}_tracts_{year}.parquet`), analysis data exists for specific types (e.g., `political_lean.csv`)

## When to Use
User says "Create map for [state]/Visualize [political/demographic/compactness] for [state]", regenerate maps after data changes, custom visualization for specific state, higher resolution maps

## Map Types

### 1. District Assignment
**Purpose**: Basic district boundaries + assignments
**Shows**: District boundaries (thick black), tract boundaries (thin white), distinct colors per district, district numbers at centroids
**Data**: `districts.csv`
**Command**: `python scripts/visualization/visualize_state.py --state california --year 2020 --version v1 --dpi 150`

### 2. Political Lean
**Purpose**: Partisan lean (2020 presidential election)
**Shows**: Blue-red gradient (D→R), district boundaries (black), D/R vote % per district, total D/R seat counts
**Data**: `political_lean.csv` (2020 only with election data)
**Command**: `python scripts/political/analyze_districts.py --state california --year 2020 --version v1 --scope state`

### 3. Demographic Composition
**Purpose**: Racial/ethnic composition
**Shows**: Color intensity by %, 3 variants (White %, Minority %, Majority-minority status), district boundaries (black), % labels
**Data**: `demographic_composition.csv`
**Command**: `python scripts/demographic/analyze_demographics.py --state california --year 2020 --version v1 --scope state`

### 4. Compactness Scores
**Purpose**: District compactness
**Shows**: Gradient low (red) → high (green), 2 metrics (Polsby-Popper, Reock), district boundaries (black), score labels (0.0-1.0)
**Data**: `compactness.csv`
**Command**: `python scripts/compactness/analyze_compactness.py --state california --year 2020 --version v1 --scope state`

### 5. Round Progression
**Purpose**: Districts created at each recursive bisection round
**Shows**: Districts colored by creation round, Round 1 (2 districts) → Round N (final), algorithm behavior
**Data**: `rounds_hierarchy.csv`
**Command**: `python scripts/visualization/visualize_rounds.py --state california --year 2020 --version v1 --scope state`

## Workflow

### Step 1: Identify Parameters
Ask if not specified: State (lowercase_underscores: `california`, `new_york`), map type (districts/political/demographic/compactness/rounds), census year (2000/2010/2020), version (v1/v2/test)

### Step 2: Validate Data
Check required files exist:
```bash
ls outputs/us_2020_v1/states/california/data/districts.csv              # Required
ls outputs/us_2020_v1/states/california/data/political_lean.csv         # For political
ls outputs/us_2020_v1/states/california/data/demographic_composition.csv # For demographic
ls outputs/us_2020_v1/states/california/data/compactness.csv            # For compactness
```
**If missing**: Districts → `/run-redistricting`, Analysis → run analysis script or `/run-analysis-only`

### Step 3: Set Parameters
**Required**: `--state` (lowercase_underscores), `--year` (2000/2010/2020), `--version` (output tag), `--scope state`
**Optional**: `--dpi` (50-300, default 150, higher=sharper/larger), `--output` (custom path), `--no-labels` (omit district numbers), `--style` (paper/presentation/web)

### Step 4: Generate Map
Run script by type (see Map Types section for commands)

### Step 5: Verify Output
Check map created:
```bash
ls outputs/us_2020_v1/states/california/maps/districts.png
ls outputs/us_2020_v1/states/california/maps/political_lean.png
ls outputs/us_2020_v1/states/california/maps/compactness_polsby_popper.png
```
**File sizes**: DPI 50 (~200-500KB, web), DPI 150 (~800KB-2MB, standard), DPI 300 (~2-5MB, print)

## Styling

**Colors**: Districts (qualitative `tab20`), Political (blue-red `seismic/RdBu_r`), Demographics (sequential `YlOrRd/viridis`), Compactness (red-yellow-green `RdYlGn`)
**Boundaries**: Tracts (white 0.1-0.3pt semi-transparent), Districts (black 1.5-2.0pt opaque)
**Labels**: District numbers at centroids, 8-12pt, white/black for contrast, optional stroke/halo
**Annotations**: Title "State Congressional Districts (Year)", stats (political "D: 42 | R: 11", compactness "Mean PP: 0.45", demographic "Majority-Minority: 15")

## Output Locations
```
outputs/us_{year}_{version}/states/{state}/maps/
├── districts.png                        # Basic
├── political_lean.png                   # Partisan (2020)
├── compactness_polsby_popper.png       # PP
├── compactness_reock.png               # Reock
├── demographic_white_percentage.png    # White %
├── demographic_minority_percentage.png # Minority %
├── demographic_majority_minority.png   # Majority-minority
└── rounds_*.png                        # Progression
```

## Troubleshooting

**Missing data**: `Error: districts.csv not found` → Run `/run-redistricting --states "california"`
**Low quality**: Pixelated/blurry → Increase DPI (200-300 for print)
**Unicode errors (Win)**: `UnicodeEncodeError` → Check file paths use ASCII not Unicode
**Memory errors**: Large states OOM → Reduce DPI or close apps
**Wrong colors**: Political map wrong party → Verify data loaded, check NaN values

## Advanced Usage

**Custom colors**: Modify script, use custom `LinearSegmentedColormap`:
```python
from matplotlib.colors import LinearSegmentedColormap
colors = ['#0000FF', '#FFFFFF', '#FF0000']  # Blue-White-Red
cmap = LinearSegmentedColormap.from_list('custom', colors)
```

**Batch multiple states**:
```bash
for state in california texas florida new_york; do
  python scripts/visualization/visualize_state.py --state $state --year 2020 --version v1 --dpi 150
done
```

**High-res publication**: `--dpi 300 --style paper`

## Performance

**Runtime per map**:
| State Size | DPI 50 | DPI 150 | DPI 300 |
|------------|--------|---------|---------|
| Small (VT) | ~2s    | ~5s     | ~15s    |
| Medium (AL)| ~5s    | ~10s    | ~30s    |
| Large (CA) | ~15s   | ~30s    | ~90s    |

**Bottlenecks**: Geometry simplification (large tract counts), matplotlib rendering (scales DPI²), file I/O (large PNGs)

## Related Skills
`/create-national-map` (US-wide), `/run-analysis-only` (regenerate all), `/generate-dashboard` (interactive web), `/create-pedagogical-example` (educational)

## Next Steps
View maps in output dir, include in dashboard (`/generate-dashboard`), compare across years/modes, create national map (`/create-national-map`)
