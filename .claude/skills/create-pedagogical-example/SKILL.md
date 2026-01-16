---
name: create-pedagogical-example
description: Create educational examples demonstrating redistricting algorithm with small, clear tract clusters. Generates dual visualizations (geographic map + abstract graph) with strict quality validation. Use for papers, presentations, or teaching materials.
allowed-tools:
  - Read
  - Bash
  - Glob
  - Grep
  - Edit
user-invocable: true
---

# Create Pedagogical Example

## Overview

Generate clear, educational examples of the redistricting algorithm using small clusters of real census tracts. Creates dual visualizations showing both geographic representation and abstract graph structure, ideal for academic papers, presentations, and teaching materials.

## Prerequisites

Before creating examples:
1. **Census tract data** with geometries available
2. **Adjacency graphs** built for states
3. **METIS binary** available (`bin/gpmetis.exe`)
4. **Target state** selected (preferably with variety of tract shapes)

## When to Use This Skill

- User says: "Create an example for [paper/presentation]"
- User says: "I need a visual demonstration of the algorithm"
- User wants to explain algorithm to non-technical audience
- User needs figures for academic paper
- User wants to show algorithm behavior with real data

## What Makes a Good Pedagogical Example

### Quality Criteria

**Geographic clarity**:
- Small number of tracts (8-16, ideally 12)
- Compact cluster (tracts close together)
- Clear visual boundaries between regions
- Recognizable shapes (not fragmented)

**Population balance**:
- Achieves target ratio within tolerance (±0.5%)
- Example: 3:2 ratio → 60%/40% split ±0.5%

**Compactness**:
- Both resulting regions reasonably compact
- Polsby-Popper score ≥ 0.25 for both regions
- Not extreme gerrymanders

**Visual quality**:
- Regions visually distinguishable
- Boundary clear and meaningful
- Good for black/white printing
- Labels readable

### Validation Parameters

The skill implements strict quality checks:
```python
# Retry parameters
MAX_ATTEMPTS = 26         # 25 retries + initial attempt
RATIO_TOLERANCE = 0.005   # 0.5% ratio accuracy
MIN_COMPACTNESS = 0.25    # Polsby-Popper threshold
MIN_TRACTS = 8            # Minimum for meaningful example
MAX_TRACTS = 16           # Maximum for visual clarity
```

## Example Types

### 1. Simple Bisection (1:1 ratio)
**Purpose**: Show basic equal split

**Characteristics**:
- 12 tracts → 6 + 6
- Target ratio: 1:1 (50% each)
- Simplest case to understand
- Good introductory example

**Use case**: Introduction to algorithm, basic concept

### 2. Unequal Split (3:2 ratio)
**Purpose**: Demonstrate non-equal partitioning

**Characteristics**:
- 12 tracts → ~7 + ~5 (approximately)
- Target ratio: 3:2 (60%/40%)
- Shows algorithm handles unequal populations
- More realistic than 1:1

**Use case**: Show flexibility, real-world scenarios

### 3. Edge-Weighted vs Unweighted Comparison
**Purpose**: Demonstrate edge-weight impact

**Characteristics**:
- Same tract cluster
- Two partitions: with and without edge weights
- Shows compactness improvement
- Side-by-side visualization

**Use case**: Justify edge-weighted approach, show improvements

### 4. Complex Geometry Example
**Purpose**: Show algorithm handles difficult shapes

**Characteristics**:
- Tracts with irregular boundaries
- Water boundaries (coastlines, rivers)
- Non-convex shapes
- Real-world complexity

**Use case**: Demonstrate robustness

## Workflow

### Step 1: Select Source Location

Choose state and starting location:
```bash
# List available states
ls data/tracts/2020/*.parquet

# Typical choices:
# - coastal states (complex boundaries): california, florida
# - diverse geometry: pennsylvania, texas
# - compact tracts: iowa, kansas
```

**Selection criteria**:
- State with variety of tract shapes
- Area with compact tract cluster
- Avoid extreme fragmentation
- Prefer suburban/rural (cleaner boundaries)

### Step 2: Set Target Parameters

Define example characteristics:
```python
# Core parameters
n_tracts = 12              # Number of tracts in example
target_ratio = (3, 2)      # Population split (e.g., 3:2 for 60/40)
state = 'california'       # Source state
year = '2020'              # Census year

# Quality thresholds
ratio_tolerance = 0.005    # ±0.5% from target
min_compactness = 0.25     # PP score threshold
max_attempts = 26          # Retry limit
```

### Step 3: Run Generation Script

Execute the pedagogical example generator:

**Basic generation** (tries multiple starting locations):
```bash
python presentations/edge_weighted_bisection/create_appendix_examples.py \
  --state california \
  --year 2020 \
  --n-tracts 12 \
  --ratio 3:2 \
  --mode edge-weighted
```

**With specific starting location**:
```bash
python presentations/edge_weighted_bisection/create_appendix_examples.py \
  --state california \
  --year 2020 \
  --n-tracts 12 \
  --ratio 3:2 \
  --mode edge-weighted \
  --start-index 150  # Specific starting tract
```

**Comparison mode** (edge-weighted vs unweighted):
```bash
python presentations/edge_weighted_bisection/create_appendix_examples.py \
  --state california \
  --year 2020 \
  --n-tracts 12 \
  --ratio 3:2 \
  --comparison
```

### Step 4: Monitor Retry Logic

Script automatically retries with different starting locations if quality criteria not met:

**Retry output**:
```
[ATTEMPT 1] Starting from tract 0...
  Cluster selected: 12 tracts
  Running METIS partition...
  Checking ratio: 0.617 (target: 0.600, tolerance: 0.005)
  FAILED: Ratio outside tolerance (0.617 vs 0.600 ± 0.005)

[ATTEMPT 2] Starting from tract 25...
  Cluster selected: 12 tracts
  Running METIS partition...
  Checking ratio: 0.602 (target: 0.600, tolerance: 0.005)
  Checking compactness: PP0=0.28, PP1=0.31
  SUCCESS! Example meets all criteria.

Saving to: presentations/edge_weighted_bisection/figures/example_3_2_ratio.png
```

**What gets retried**:
- Different starting tracts (spatial variation)
- Up to 26 attempts total
- First success → stops immediately

**Failure modes**:
- Ratio too far from target (>0.5% error)
- Compactness too low (PP < 0.25)
- Invalid geometry (disconnected regions)
- Maximum attempts exceeded

### Step 5: Verify Output Quality

Check generated example:

**Visual inspection**:
```bash
# Open generated figure
start presentations/edge_weighted_bisection/figures/example_3_2_ratio.png
```

**Quality checklist**:
- [ ] Ratio accurate (within ±0.5%)
- [ ] Both regions compact (PP ≥ 0.25)
- [ ] Labels readable (tract IDs, populations, percentages)
- [ ] Boundary clear and meaningful
- [ ] Graph representation matches geographic map
- [ ] Suitable for black/white printing

**Metrics validation**:
```
Region 0:
  Population: 75,234 (60.1%)
  Tracts: 7
  Polsby-Popper: 0.28
  Boundary length: 45.2 km

Region 1:
  Population: 49,876 (39.9%)
  Tracts: 5
  Polsby-Popper: 0.31
  Boundary length: 38.7 km

Ratio: 1.508 (target: 1.500, error: 0.5%)
```

## Visualization Structure

### Dual Panel Layout

**Left panel: Geographic Map**
- Real census tract boundaries
- Two regions colored (e.g., blue and orange)
- Tract IDs labeled at centroids
- Region boundary emphasized (thick black line)
- Population per tract annotated
- Scale bar (optional)

**Right panel: Abstract Graph**
- Nodes = census tracts (circles)
- Node size proportional to population
- Edges = adjacencies (lines between neighbors)
- Edge thickness proportional to boundary length
- Same coloring as geographic map
- Node labels match geographic tract IDs

### Annotations

**Title**:
```
Example: 3:2 Population Split (12 Census Tracts)
```

**Statistics box** (below or beside):
```
Region 0 (Blue):  75,234 (60.1%) | 7 tracts | PP: 0.28
Region 1 (Orange): 49,876 (39.9%) | 5 tracts | PP: 0.31
Ratio: 1.508 (target: 1.500) | Boundary: 45.2 km
```

**Geographic panel annotations**:
- Tract IDs: T001, T002, etc.
- Population values: "12.5k" format
- Compass rose (optional)

**Graph panel annotations**:
- Node labels: Same as tract IDs
- Edge weights: Boundary lengths (km)
- Legend: "Node size = population"

## Output Files

Examples saved to presentation/paper directories:
```
presentations/edge_weighted_bisection/figures/
├── example_1_1_ratio.png           # Equal split example
├── example_3_2_ratio.png           # Unequal split example
├── example_comparison.png          # Edge-weighted vs unweighted
└── example_complex_geometry.png    # Irregular boundaries

papers/03_combined_recursive_bisection/figures/
├── pedagogical_example_1.png
├── pedagogical_example_2.png
└── pedagogical_example_comparison.png
```

## Script Reference

### Main Generation Script

**Location**: `presentations/edge_weighted_bisection/create_appendix_examples.py`

**Key functions**:
```python
def find_compact_cluster(tracts_gdf, start_idx, n_tracts):
    """Find spatially compact cluster of n tracts starting from start_idx"""
    # Uses nearest neighbors to build compact cluster
    pass

def validate_partition_quality(tracts, partition, target_ratio, tolerance):
    """Check if partition meets quality criteria"""
    # Validates ratio accuracy and compactness
    pass

def create_dual_visualization(tracts_gdf, partition, output_path):
    """Generate geographic + graph dual panel figure"""
    # Creates side-by-side visualization
    pass

def retry_until_valid(state, year, n_tracts, target_ratio, max_attempts=26):
    """Try multiple starting locations until valid example found"""
    # Automatic retry with different starting points
    pass
```

### Command-Line Interface

```bash
python presentations/edge_weighted_bisection/create_appendix_examples.py \
  --state STATE \           # Source state name
  --year YEAR \             # Census year (2000/2010/2020)
  --n-tracts N \            # Number of tracts (8-16)
  --ratio A:B \             # Target ratio (e.g., 3:2, 1:1)
  --mode MODE \             # edge-weighted or unweighted
  --tolerance TOL \         # Ratio tolerance (default: 0.005)
  --min-compactness MIN \   # PP threshold (default: 0.25)
  --max-attempts MAX \      # Retry limit (default: 26)
  --start-index IDX \       # Starting tract index (optional)
  --comparison \            # Create comparison figure
  --dpi DPI                 # Output resolution (default: 300)
```

## Troubleshooting

**Common Issues**:

**No valid examples found after 26 attempts**:
```
Error: Failed to find valid example after 26 attempts
Suggestions:
  1. Relax tolerance: --tolerance 0.01 (1% instead of 0.5%)
  2. Relax compactness: --min-compactness 0.20
  3. Try different state (more uniform tract sizes)
  4. Adjust n_tracts (try 10 or 14 instead of 12)
```

**Ratio consistently too far from target**:
```
Issue: Ratio always 0.65 when targeting 0.60 (3:2)
Cause: Tract populations too variable
Solution: Try different starting location or increase tolerance
```

**Compactness too low**:
```
Issue: One region always has PP < 0.25
Cause: Tract geometry forces elongated shapes
Solution: Try different state or accept lower threshold
```

**Graph visualization unclear**:
```
Issue: Graph too dense, edges overlap
Solution: Reduce n_tracts to 8-10 for clearer visualization
         Or use force-directed layout for node positions
```

**Labels unreadable**:
```
Issue: Tract labels overlap or too small
Solution: Increase DPI (--dpi 400)
         Reduce n_tracts for more space
         Adjust font sizes in script
```

## Advanced Usage

### Specific Geographic Features

Target examples with specific characteristics:
```python
# Find coastal tracts (demonstrate water boundaries)
coastal_tracts = tracts_gdf[tracts_gdf['AWATER'] > 0]

# Find urban tracts (demonstrate dense areas)
urban_tracts = tracts_gdf[tracts_gdf['POP100'] > 5000]

# Find rural tracts (demonstrate large areas)
rural_tracts = tracts_gdf[tracts_gdf['POP100'] < 2000]
```

### Custom Ratios

Create examples for various target ratios:
```bash
# Different ratios for series of examples
for ratio in 1:1 3:2 5:6 2:3; do
  python create_appendix_examples.py \
    --state california \
    --year 2020 \
    --ratio $ratio \
    --n-tracts 12
done
```

### High-Resolution for Publication

Generate publication-quality figures:
```bash
python create_appendix_examples.py \
  --state california \
  --year 2020 \
  --n-tracts 12 \
  --ratio 3:2 \
  --dpi 600 \
  --format pdf  # Vector format for perfect scaling
```

### Comparison Series

Create before/after series:
```bash
# Unweighted version
python create_appendix_examples.py \
  --state california \
  --year 2020 \
  --n-tracts 12 \
  --ratio 3:2 \
  --mode unweighted \
  --start-index 100  # Fix starting location

# Edge-weighted version (same starting location)
python create_appendix_examples.py \
  --state california \
  --year 2020 \
  --n-tracts 12 \
  --ratio 3:2 \
  --mode edge-weighted \
  --start-index 100  # Same location for fair comparison
```

## Integration with Papers/Presentations

### LaTeX Integration

Include generated figures in papers:
```latex
\begin{figure}[htbp]
  \centering
  \includegraphics[width=0.9\textwidth]{figures/example_3_2_ratio.png}
  \caption{Pedagogical example showing 3:2 population split using 12 census tracts from California. Left: Geographic representation with tract boundaries. Right: Abstract graph structure with edges weighted by boundary length.}
  \label{fig:pedagogical_example}
\end{figure}
```

### Presentation Slides

PowerPoint/Beamer integration:
```latex
% Beamer slide
\begin{frame}{Algorithm Demonstration}
  \begin{columns}
    \column{0.5\textwidth}
    \includegraphics[width=\textwidth]{figures/example_unweighted.png}
    \caption{Without edge weights}

    \column{0.5\textwidth}
    \includegraphics[width=\textwidth]{figures/example_weighted.png}
    \caption{With edge weights}
  \end{columns}

  \vspace{1em}
  Edge weights improve compactness by 52.8\%
\end{frame}
```

## Performance Notes

**Typical runtime**:
| Attempts | Time |
|----------|------|
| 1 (immediate success) | ~5-10 seconds |
| 5 attempts | ~30-45 seconds |
| 26 attempts (max) | ~3-5 minutes |

**Bottlenecks**:
- METIS partitioning (~1-2 sec per attempt)
- Geometry operations (compactness calculation)
- Visualization rendering

**Success rates** (typical):
- 1:1 ratio: ~60% first attempt success
- 3:2 ratio: ~40% first attempt success
- 5:6 ratio: ~25% first attempt success (harder to achieve)

## Quality Assurance

### Automated Validation

Script performs these checks:
```python
# 1. Ratio accuracy
ratio_actual = pop0 / pop1
ratio_target = target_ratio[0] / target_ratio[1]
ratio_error = abs(ratio_actual - ratio_target) / ratio_target
assert ratio_error <= tolerance  # Must be within 0.5%

# 2. Compactness
pp0 = polsby_popper(region0)
pp1 = polsby_popper(region1)
assert pp0 >= min_compactness and pp1 >= min_compactness  # Both ≥ 0.25

# 3. Connectivity
assert is_connected(region0) and is_connected(region1)  # No fragments

# 4. Visual quality
assert n_tracts >= 8 and n_tracts <= 16  # Readable range
```

### Manual Review Checklist

After generation, verify:
- [ ] **Accuracy**: Ratio within ±0.5% of target
- [ ] **Compactness**: Both regions PP ≥ 0.25
- [ ] **Clarity**: Labels readable, boundaries clear
- [ ] **Aesthetics**: Visually appealing, good colors
- [ ] **Printability**: Works in black/white
- [ ] **Consistency**: Graph matches geographic map
- [ ] **Documentation**: Statistics accurate and complete

## Related Skills

- `/create-state-map` - Generate full state maps
- `/create-national-map` - Generate national maps
- `/compile-latex` - Integrate examples into papers
- `/create-presentation-figures` - Generate full figure sets

## Best Practices

1. **Start simple**: Begin with 1:1 ratio, 12 tracts
2. **Iterate on location**: Try different states if quality poor
3. **Document parameters**: Record successful parameters for reproducibility
4. **Version control**: Save parameters with figures (metadata)
5. **Consistent style**: Use same color scheme across examples
6. **Test printing**: Verify readability in grayscale
7. **Annotate thoroughly**: Include all relevant statistics
8. **Archive attempts**: Keep log of what worked/didn't work

## What You'll Get

After successful creation:
- **High-quality dual visualization** (geographic + graph)
- **Validated example** meeting all quality criteria
- **Complete annotations** (populations, ratios, compactness)
- **Publication-ready figure** (high DPI, proper formatting)
- **Reproducible result** (documented parameters)
- **Educational value** (clear demonstration of algorithm)

## Next Steps

- Include in paper/presentation
- Create series of examples (different ratios)
- Generate comparison figures (weighted vs unweighted)
- Document in figure captions
- Use in teaching materials
