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

## Overview

Generate high-quality state-level visualization maps for congressional redistricting results. Supports multiple map types with consistent styling across all visualizations.

## Prerequisites

Before creating maps, validate:
1. **District assignments** exist for the state (`outputs/us_{year}_{version}/states/{state}/data/districts.csv`)
2. **Census tract data** available (`data/tracts/{year}/{state}_tracts_{year}.parquet`)
3. **Analysis data** exists for specific map types (e.g., political_lean.csv for political maps)

## When to Use This Skill

- User says: "Create a map for [state]"
- User says: "Visualize [political/demographic/compactness] data for [state]"
- User wants to regenerate maps after data changes
- User needs custom visualization for specific state
- User wants higher resolution maps than default

## Map Types

### 1. District Assignment Map
**Purpose**: Show the basic district boundaries and assignments

**What it shows**:
- District boundaries in thick black lines
- Tract boundaries in thin white lines
- Each district colored distinctly
- District numbers labeled at centroids

**Required data**:
- `districts.csv` - Tract-to-district assignments

**Example command**:
```bash
python scripts/visualization/visualize_state.py \
  --state california \
  --year 2020 \
  --version v1 \
  --type districts \
  --dpi 150
```

### 2. Political Lean Map
**Purpose**: Show partisan lean based on 2020 presidential election

**What it shows**:
- Blue-to-red gradient (Democratic to Republican)
- District boundaries in black
- D/R vote percentages per district
- Total D/R seat counts annotated

**Required data**:
- `political_lean.csv` - Partisan vote shares
- Only available for 2020 with election data

**Example command**:
```bash
python scripts/political/analyze_districts.py \
  --state california \
  --year 2020 \
  --version v1 \
  --scope state
```

### 3. Demographic Composition Maps
**Purpose**: Show racial/ethnic composition of districts

**What it shows**:
- Color intensity by percentage
- Three variants: White %, Minority %, Majority minority status
- District boundaries in black
- Percentage labels

**Required data**:
- `demographic_composition.csv` - Race/ethnicity percentages

**Example command**:
```bash
python scripts/demographic/analyze_demographics.py \
  --state california \
  --year 2020 \
  --version v1 \
  --scope state
```

### 4. Compactness Score Maps
**Purpose**: Show how compact each district is

**What it shows**:
- Color gradient from low (red) to high (green) compactness
- Two metrics: Polsby-Popper and Reock
- District boundaries in black
- Score labels (0.0 - 1.0)

**Required data**:
- `compactness.csv` - PP and Reock scores

**Example command**:
```bash
python scripts/compactness/analyze_compactness.py \
  --state california \
  --year 2020 \
  --version v1 \
  --scope state
```

### 5. Round Progression Map
**Purpose**: Show districts created at each recursive bisection round

**What it shows**:
- Districts colored by the round they were created
- Round 1 (2 districts) → Round N (final count)
- Helps understand algorithm behavior

**Required data**:
- `rounds_hierarchy.csv` - Bisection tree structure

**Example command**:
```bash
python scripts/visualization/visualize_rounds.py \
  --state california \
  --year 2020 \
  --version v1 \
  --scope state
```

## Workflow

### Step 1: Identify Map Type and State

Ask user if not specified:
- Which state? (lowercase with underscores: `california`, `new_york`)
- Which map type? (districts, political, demographic, compactness, rounds)
- Which census year? (2000, 2010, 2020)
- Which version? (v1, v2, test, etc.)

### Step 2: Validate Data Availability

Check that required files exist:
```bash
# Check district assignments
ls outputs/us_2020_v1/states/california/data/districts.csv

# Check analysis data (for specific map types)
ls outputs/us_2020_v1/states/california/data/political_lean.csv
ls outputs/us_2020_v1/states/california/data/demographic_composition.csv
ls outputs/us_2020_v1/states/california/data/compactness.csv
```

If data missing:
- For districts: Must run redistricting first (`/run-redistricting`)
- For analysis: Run specific analysis script or use `/run-analysis-only`

### Step 3: Set Parameters

**Required Parameters**:
- `--state`: State name (lowercase with underscores)
- `--year`: Census year (2000, 2010, or 2020)
- `--version`: Output version tag
- `--scope`: Must be `state` for state-level maps

**Optional Parameters**:
| Parameter | Values | Default | Purpose |
|-----------|--------|---------|---------|
| `--dpi` | 50-300 | 150 | Resolution (higher = sharper, larger files) |
| `--output` | File path | Auto | Custom output location |
| `--no-labels` | Flag | False | Omit district number labels |
| `--style` | paper, presentation, web | default | Style preset |

### Step 4: Generate Map

Run the appropriate script based on map type:

**District Assignment**:
```bash
python scripts/visualization/visualize_state.py \
  --state california \
  --year 2020 \
  --version v1 \
  --dpi 150
```

**Political Lean** (2020 only):
```bash
python scripts/political/analyze_districts.py \
  --state california \
  --year 2020 \
  --version v1 \
  --scope state
```

**Demographics**:
```bash
python scripts/demographic/analyze_demographics.py \
  --state california \
  --year 2020 \
  --version v1 \
  --scope state
```

**Compactness**:
```bash
python scripts/compactness/analyze_compactness.py \
  --state california \
  --year 2020 \
  --version v1 \
  --scope state
```

**Round Progression**:
```bash
python scripts/visualization/visualize_rounds.py \
  --state california \
  --year 2020 \
  --version v1 \
  --scope state
```

### Step 5: Verify Output

Check that map was created:
```bash
# District map
ls outputs/us_2020_v1/states/california/maps/districts.png

# Other map types
ls outputs/us_2020_v1/states/california/maps/political_lean.png
ls outputs/us_2020_v1/states/california/maps/compactness_polsby_popper.png
ls outputs/us_2020_v1/states/california/maps/demographic_white_percentage.png
```

Map file sizes:
- **DPI 50**: ~200-500 KB per map (web preview)
- **DPI 150**: ~800 KB - 2 MB per map (standard quality)
- **DPI 300**: ~2-5 MB per map (print quality)

## Map Styling Guidelines

### Colors

**District assignment**:
- Qualitative colormap (distinct colors for neighboring districts)
- Typically uses `tab20` or custom palette

**Political lean**:
- Blue (Democratic) to Red (Republican)
- Diverging colormap centered at 50%
- Colormap: `seismic` or `RdBu_r`

**Demographics**:
- Sequential colormap (low to high percentage)
- Colormap: `YlOrRd` or `viridis`

**Compactness**:
- Red (low) to Yellow (medium) to Green (high)
- Colormap: `RdYlGn`

### Boundaries

**Tract boundaries**:
- Color: White (#FFFFFF)
- Width: 0.1-0.3 points
- Semi-transparent

**District boundaries**:
- Color: Black (#000000)
- Width: 1.5-2.0 points
- Fully opaque

### Labels

**District numbers**:
- Placed at district centroids
- Font size: 8-12 points (scales with DPI)
- Color: White or black for contrast
- Optional stroke/halo for visibility

### Annotations

**State title**:
- Top center or top left
- Format: "California Congressional Districts (2020)"

**Statistics** (if applicable):
- Political: "D: 42 | R: 11" (party seat totals)
- Compactness: "Mean PP: 0.45"
- Demographics: "Majority-Minority: 15 districts"

## Output Locations

Maps are saved to state-specific directories:
```
outputs/us_{year}_{version}/states/{state}/maps/
├── districts.png                        # Basic district map
├── political_lean.png                   # Partisan lean (2020 only)
├── compactness_polsby_popper.png       # PP compactness
├── compactness_reock.png               # Reock compactness
├── demographic_white_percentage.png    # White % by district
├── demographic_minority_percentage.png # Minority % by district
├── demographic_majority_minority.png   # Majority-minority districts
└── rounds_*.png                        # Round progression maps
```

## Troubleshooting

**Common Issues**:

**Missing data files**:
```
Error: districts.csv not found for california
Solution: Run redistricting first: /run-redistricting --states "california"
```

**Low quality maps**:
```
Issue: Maps look pixelated or blurry
Solution: Increase DPI (use 200-300 for print quality)
```

**Unicode errors (Windows)**:
```
Error: UnicodeEncodeError when saving file
Solution: Check that file paths use ASCII, not Unicode characters
```

**Memory errors for large states**:
```
Error: MemoryError when creating California map
Solution: Reduce DPI or close other applications
```

**Wrong colors/legend**:
```
Issue: Political map shows wrong party colors
Solution: Verify data loaded correctly, check for NaN values
```

## Advanced Usage

### Custom Color Schemes

Modify visualization script to use custom colormaps:
```python
# In visualize_state.py or analysis script
import matplotlib.pyplot as plt

# Custom colormap
from matplotlib.colors import LinearSegmentedColormap
colors = ['#0000FF', '#FFFFFF', '#FF0000']  # Blue-White-Red
cmap = LinearSegmentedColormap.from_list('custom', colors)
```

### Multiple States Batch

Generate maps for multiple states:
```bash
# Loop through states
for state in california texas florida new_york; do
  python scripts/visualization/visualize_state.py \
    --state $state \
    --year 2020 \
    --version v1 \
    --dpi 150
done
```

Or use pipeline to regenerate all:
```bash
python scripts/pipeline/run_complete_redistricting.py \
  --year 2020 \
  --version v1 \
  --skip-redistricting \
  --force
```

### High-Resolution for Publication

For academic papers or presentations:
```bash
python scripts/visualization/visualize_state.py \
  --state california \
  --year 2020 \
  --version v1 \
  --dpi 300 \
  --style paper
```

## Related Skills

- `/create-national-map` - Generate US-wide visualization
- `/run-analysis-only` - Regenerate all analysis and maps
- `/generate-dashboard` - Create interactive web dashboard
- `/create-pedagogical-example` - Create educational examples

## Performance Notes

**Typical runtime per map**:
| State Size | DPI 50 | DPI 150 | DPI 300 |
|------------|--------|---------|---------|
| Small (VT) | ~2 sec | ~5 sec | ~15 sec |
| Medium (AL)| ~5 sec | ~10 sec | ~30 sec |
| Large (CA) | ~15 sec| ~30 sec | ~90 sec |

**Bottlenecks**:
- Geometry simplification (for large tract counts)
- Matplotlib rendering (scales with DPI²)
- File I/O (writing large PNG files)

## Next Steps

After creating state maps:
- View maps in output directory
- Include in dashboard (regenerate with `/generate-dashboard`)
- Compare across different years/modes
- Create national map with `/create-national-map`
