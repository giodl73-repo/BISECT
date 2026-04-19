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

Generate clear, educational examples of redistricting algorithm using small clusters of real census tracts. Creates dual visualizations showing both geographic representation and abstract graph structure, ideal for academic papers, presentations, teaching materials.

## Prerequisites
Census tract data with geometries available, adjacency graphs built for states, METIS binary available (`bin/gpmetis.exe`), target state selected (preferably with variety of tract shapes)

## When to Use
User says "Create example for [paper/presentation]/Need visual demonstration of algorithm", wants to explain algorithm to non-technical audience, needs figures for academic paper, wants to show algorithm behavior with real data

## What Makes a Good Pedagogical Example

**Quality Criteria**:
- **Geographic clarity**: Small number of tracts (8-16, ideally 12), compact cluster (tracts close together), clear visual boundaries, recognizable shapes (not fragmented)
- **Population balance**: Achieves target ratio within tolerance (±0.5%), Example: 3:2 ratio → 60%/40% split ±0.5%
- **Compactness**: Both regions reasonably compact, Polsby-Popper score ≥ 0.25 for both, not extreme gerrymanders
- **Visual quality**: Regions visually distinguishable, boundary clear and meaningful, good for black/white printing, labels readable

**Validation Parameters**:
```python
MAX_ATTEMPTS = 26         # 25 retries + initial attempt
RATIO_TOLERANCE = 0.005   # 0.5% ratio accuracy
MIN_COMPACTNESS = 0.25    # Polsby-Popper threshold
MIN_TRACTS = 8            # Minimum for meaningful example
MAX_TRACTS = 16           # Maximum for visual clarity
```

## Example Types

**1. Simple Bisection (1:1 ratio)**: 12 tracts → 6 + 6, target 50% each, simplest case, good introductory example → Use case: Introduction to algorithm, basic concept

**2. Unequal Split (3:2 ratio)**: 12 tracts → ~7 + ~5, target 60%/40%, shows algorithm handles unequal populations, more realistic → Use case: Show flexibility, real-world scenarios

**3. Edge-Weighted vs Unweighted Comparison**: Same tract cluster, two partitions (with/without edge weights), shows compactness improvement, side-by-side visualization → Use case: Justify edge-weighted approach, show improvements

**4. Complex Geometry Example**: Tracts with irregular boundaries, water boundaries (coastlines/rivers), non-convex shapes, real-world complexity → Use case: Demonstrate robustness

## Workflow

### Step 1: Select Source Location
```bash
ls data/tracts/2020/*.parquet  # List available states

# Typical choices:
# - Coastal states (complex boundaries): california, florida
# - Diverse geometry: pennsylvania, texas
# - Compact tracts: iowa, kansas
```

**Selection criteria**: State with variety of tract shapes, area with compact tract cluster, avoid extreme fragmentation, prefer suburban/rural (cleaner boundaries)

### Step 2: Set Target Parameters
```python
n_tracts = 12              # Number of tracts in example
target_ratio = (3, 2)      # Population split (e.g., 3:2 for 60/40)
state = 'california'       # Source state
year = '2020'              # Census year
ratio_tolerance = 0.005    # ±0.5% from target
min_compactness = 0.25     # PP score threshold
max_attempts = 26          # Retry limit
```

### Step 3: Run Generation Script

**Basic generation** (tries multiple starting locations):
```bash
python presentations/edge_weighted_bisection/create_appendix_examples.py --state california --year 2020 --n-tracts 12 --ratio 3:2 --mode edge-weighted
```

**With specific starting location**:
```bash
python presentations/edge_weighted_bisection/create_appendix_examples.py --state california --year 2020 --n-tracts 12 --ratio 3:2 --mode edge-weighted --start-index 150
```

**Comparison mode** (edge-weighted vs unweighted):
```bash
python presentations/edge_weighted_bisection/create_appendix_examples.py --state california --year 2020 --n-tracts 12 --ratio 3:2 --comparison
```

### Step 4: Monitor Retry Logic
Script automatically retries with different starting locations if quality criteria not met:

**Retry output**:
```
[ATTEMPT 1] Starting from tract 0...
  Cluster selected: 12 tracts
  Running METIS partition...
  Checking ratio: 0.617 (target: 0.600, tolerance: 0.005)
  FAILED: Ratio outside tolerance

[ATTEMPT 2] Starting from tract 25...
  Cluster selected: 12 tracts
  Checking ratio: 0.602 (target: 0.600, tolerance: 0.005)
  Checking compactness: PP0=0.28, PP1=0.31
  SUCCESS! Example meets all criteria.

Saving to: presentations/edge_weighted_bisection/figures/example_3_2_ratio.png
```

**What gets retried**: Different starting tracts (spatial variation), up to 26 attempts total, first success → stops immediately

**Failure modes**: Ratio too far from target (>0.5% error), compactness too low (PP < 0.25), invalid geometry (disconnected regions), maximum attempts exceeded

### Step 5: Verify Output Quality

**Visual inspection**: `start presentations/edge_weighted_bisection/figures/example_3_2_ratio.png`

**Quality checklist**:
- [ ] Ratio accurate (within ±0.5%)
- [ ] Both regions compact (PP ≥ 0.25)
- [ ] Labels readable (tract IDs, populations, percentages)
- [ ] Boundary clear and meaningful
- [ ] Graph representation matches geographic map
- [ ] Suitable for black/white printing

**Metrics validation**:
```
Region 0: Population: 75,234 (60.1%) | Tracts: 7 | Polsby-Popper: 0.28 | Boundary: 45.2 km
Region 1: Population: 49,876 (39.9%) | Tracts: 5 | Polsby-Popper: 0.31 | Boundary: 38.7 km
Ratio: 1.508 (target: 1.500, error: 0.5%)
```

## Visualization Structure

**Dual Panel Layout**:
- **Left panel: Geographic Map**: Real census tract boundaries, two regions colored (e.g., blue/orange), tract IDs labeled at centroids, region boundary emphasized (thick black line), population per tract annotated, scale bar (optional)
- **Right panel: Abstract Graph**: Nodes = census tracts (circles), node size ∝ population, edges = adjacencies (lines between neighbors), edge thickness ∝ boundary length, same coloring as geographic map, node labels match geographic tract IDs

**Annotations**:
- **Title**: `Example: 3:2 Population Split (12 Census Tracts)`
- **Statistics box**: `Region 0 (Blue): 75,234 (60.1%) | 7 tracts | PP: 0.28` / `Region 1 (Orange): 49,876 (39.9%) | 5 tracts | PP: 0.31` / `Ratio: 1.508 (target: 1.500) | Boundary: 45.2 km`
- **Geographic panel**: Tract IDs (T001, T002), population values ("12.5k" format), compass rose (optional)
- **Graph panel**: Node labels (same as tract IDs), edge weights (boundary lengths km), legend ("Node size = population")

## Output Files
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

**Location**: `presentations/edge_weighted_bisection/create_appendix_examples.py`

**Key functions**:
- `find_compact_cluster(tracts_gdf, start_idx, n_tracts)` - Find spatially compact cluster using nearest neighbors
- `validate_partition_quality(tracts, partition, target_ratio, tolerance)` - Check ratio accuracy and compactness
- `create_dual_visualization(tracts_gdf, partition, output_path)` - Generate geographic + graph dual panel figure
- `retry_until_valid(state, year, n_tracts, target_ratio, max_attempts=26)` - Try multiple starting locations until valid example found

**Command-Line Interface**:
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

**No valid examples after 26 attempts**: `Error: Failed to find valid example after 26 attempts` → Relax tolerance (`--tolerance 0.01`), relax compactness (`--min-compactness 0.20`), try different state (more uniform tract sizes), adjust n_tracts (try 10 or 14 instead of 12)

**Ratio consistently too far from target**: Ratio always 0.65 when targeting 0.60 → Tract populations too variable → Try different starting location or increase tolerance

**Compactness too low**: One region always has PP < 0.25 → Tract geometry forces elongated shapes → Try different state or accept lower threshold

**Graph visualization unclear**: Graph too dense, edges overlap → Reduce n_tracts to 8-10 for clearer visualization or use force-directed layout

**Labels unreadable**: Tract labels overlap or too small → Increase DPI (`--dpi 400`), reduce n_tracts for more space, adjust font sizes in script

## Advanced Usage

**Specific Geographic Features**:
```python
coastal_tracts = tracts_gdf[tracts_gdf['AWATER'] > 0]     # Water boundaries
urban_tracts = tracts_gdf[tracts_gdf['POP100'] > 5000]    # Dense areas
rural_tracts = tracts_gdf[tracts_gdf['POP100'] < 2000]    # Large areas
```

**Custom Ratios**:
```bash
for ratio in 1:1 3:2 5:6 2:3; do
  python create_appendix_examples.py --state california --year 2020 --ratio $ratio --n-tracts 12
done
```

**High-Resolution for Publication**:
```bash
python create_appendix_examples.py --state california --year 2020 --n-tracts 12 --ratio 3:2 --dpi 600 --format pdf
```

**Comparison Series** (before/after):
```bash
# Unweighted version
python create_appendix_examples.py --state california --year 2020 --n-tracts 12 --ratio 3:2 --mode unweighted --start-index 100

# Edge-weighted version (same starting location for fair comparison)
python create_appendix_examples.py --state california --year 2020 --n-tracts 12 --ratio 3:2 --mode edge-weighted --start-index 100
```

## Integration with Papers/Presentations

**LaTeX Integration**:
```latex
\begin{figure}[htbp]
  \centering
  \includegraphics[width=0.9\textwidth]{figures/example_3_2_ratio.png}
  \caption{Pedagogical example showing 3:2 population split using 12 census tracts. Left: Geographic representation. Right: Abstract graph structure with edge weights.}
  \label{fig:pedagogical_example}
\end{figure}
```

**Beamer Slides**:
```latex
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

| Attempts | Time |
|----------|------|
| 1 (immediate success) | ~5-10s |
| 5 attempts | ~30-45s |
| 26 attempts (max) | ~3-5 min |

**Bottlenecks**: METIS partitioning (~1-2s per attempt), geometry operations (compactness calculation), visualization rendering

**Success rates** (typical): 1:1 ratio (~60% first attempt), 3:2 ratio (~40% first attempt), 5:6 ratio (~25% first attempt, harder to achieve)

## Quality Assurance

**Automated Validation**:
```python
ratio_error = abs(ratio_actual - ratio_target) / ratio_target
assert ratio_error <= tolerance  # Within 0.5%
assert pp0 >= min_compactness and pp1 >= min_compactness  # Both ≥ 0.25
assert is_connected(region0) and is_connected(region1)  # No fragments
assert 8 <= n_tracts <= 16  # Readable range
```

**Manual Review Checklist**: Accuracy (ratio within ±0.5%), compactness (both regions PP ≥ 0.25), clarity (labels readable, boundaries clear), aesthetics (visually appealing, good colors), printability (works in black/white), consistency (graph matches geographic map), documentation (statistics accurate and complete)

## Best Practices
Start simple (1:1 ratio, 12 tracts), iterate on location (try different states if quality poor), document parameters (record successful parameters for reproducibility), version control (save parameters with figures as metadata), consistent style (same color scheme across examples), test printing (verify readability in grayscale), annotate thoroughly (include all relevant statistics), archive attempts (keep log of what worked/didn't work)

## What You'll Get
High-quality dual visualization (geographic + graph), validated example meeting all quality criteria, complete annotations (populations, ratios, compactness), publication-ready figure (high DPI, proper formatting), reproducible result (documented parameters), educational value (clear demonstration of algorithm)

## Next Steps
Include in paper/presentation, create series of examples (different ratios), generate comparison figures (weighted vs unweighted), document in figure captions, use in teaching materials

## Related Skills
`/create-state-map` (generate full state maps), `/create-national-map` (generate national maps), `/compile-latex` (integrate examples into papers), `/create-presentation-figures` (generate full figure sets)
