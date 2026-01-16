---
name: create-presentation-figures
description: Generate figures for research presentations. Creates schematic diagrams (gerrymandering examples, tract-to-graph transformations, graph cuts, edge weights), copies round progression maps from pipeline outputs, and generates real census tract examples with METIS cuts. Use when preparing figures for papers, presentations, or educational materials.
allowed-tools:
  - Read
  - Bash
  - Glob
  - Grep
user-invocable: true
---

# Create Presentation Figures

## Overview

Generate high-quality figures for academic presentations, papers, and educational materials. Creates both schematic diagrams illustrating algorithmic concepts and real census tract examples demonstrating algorithm behavior.

## Prerequisites

**Required**:
- Python 3.13+ with matplotlib, geopandas, numpy, shapely installed

**Optional (for real tract examples)**:
- Census tract shapefiles in `data/geography/tiger_{year}_tracts/`
- Population data in `data/processed/census_{year}/`
- Pipeline outputs with round progression maps in `outputs/us_{year}_{version}/`

## When to Use This Skill

- User says: "Create presentation figures" or "Generate figures for presentation"
- User says: "I need figures for my paper"
- User says: "Create educational examples"
- User needs schematic diagrams of gerrymandering or algorithm concepts
- User wants real census tract visualizations
- User is preparing slides or documents for academic purposes

## Figure Types Generated

### Schematic Diagrams

**1. Example Gerrymandered District**
- Illustrates famous gerrymandering case (Illinois 4th "Earmuffs")
- Shows narrow corridor connecting distant communities
- Grid overlay to emphasize unnaturalness
- **Output**: `figures/example_gerrymander.png`

**2. Tract-to-Graph Transformation**
- Side-by-side: Geographic tracts → Abstract graph
- Shows how census tracts become nodes with adjacency edges
- Population labels on both representations
- **Output**: `figures/tract_to_graph.png`

**3. Graph with Cut Visualization**
- 3x3 grid graph showing partition cut
- Red dashed lines for cut edges
- Colored nodes by partition
- Population labels and balance shown
- **Output**: `figures/graph_with_cut.png`

**4. Edge Weights Visualization**
- Side-by-side: Unweighted vs Edge-weighted graphs
- Shows how boundary lengths affect partitioning
- Thickness scaled by weight
- Weight labels on edges
- **Output**: `figures/edge_weights_example.png`

**5. Before/After Cut Comparison**
- Shows graph before partitioning (single region)
- Shows graph after partitioning (2 districts)
- Highlights cut edges and cut weight
- **Output**: `figures/before_after_cut.png`

### Real Census Tract Examples

**6. Real Tracts to Graph with METIS Cut**
- Uses actual census tract data (default: Minneapolis, Hennepin County)
- Left panel: Geographic map with tract labels, populations, boundary lengths
- Right panel: Abstract graph representation
- **METIS partitioning** with contiguity enforcement
- Red lines show cut boundaries
- Total cut length calculated and displayed
- Population balance shown for both regions
- **Output**: `figures/real_tracts_to_graph.png`

**Features**:
- Automatically selects 12 contiguous tracts using BFS
- Calculates real boundary lengths in kilometers
- Uses METIS with same parameters as main pipeline
- Ensures contiguous partitions
- Labels all edges with boundary lengths
- Highlights cut edges in red with total cut weight

### Round Progression Maps

**7. Round-by-Round Visualizations** (copied from pipeline outputs)
- Minnesota rounds 1-3 (2 → 4 → 8 districts)
- Alabama rounds 1-3 (2 → 4 → 7 districts)
- **Source**: `outputs/us_{year}_{version}/states/{state}/maps/rounds/`
- **Output**: `figures/minnesota_round_*.png`, `figures/alabama_round_*.png`

## Workflow

### Step 1: Determine Parameters

**Required Parameters**:
- `--year`: Census year (2000, 2010, or 2020) for round progression and real tracts
- `--version`: Pipeline version tag (e.g., v1, v2, test)

Ask user if not specified, or use defaults:
- Year: 2020 (default)
- Version: v1 (default)

### Step 2: Navigate to Presentation Directory

```bash
cd presentations/edge_weighted_bisection
```

### Step 3: Run Figure Generation Script

```bash
python create_figures.py --year 2020 --version v1
```

**What it does**:
1. Creates output directory: `outputs/presentations/edge_weighted_bisection/figures/`
2. Copies round progression maps from pipeline outputs (if available)
3. Generates 6 schematic diagrams
4. Generates real census tract example with METIS (if data available)

### Step 4: Verify Outputs

Check that figures were created:
```bash
ls ../../outputs/presentations/edge_weighted_bisection/figures/
```

**Expected files**:
- `example_gerrymander.png`
- `tract_to_graph.png`
- `graph_with_cut.png`
- `edge_weights_example.png`
- `before_after_cut.png`
- `real_tracts_to_graph.png` (if census data available)
- `minnesota_round_*.png` (if pipeline outputs exist)
- `alabama_round_*.png` (if pipeline outputs exist)

### Step 5: Use Figures in Presentation

**For LaTeX presentations**:
```latex
\includegraphics[width=0.8\textwidth]{figures/real_tracts_to_graph.png}
```

**For PowerPoint**:
- Navigate to figures directory
- Insert images directly

**Figure resolution**: All figures generated at 150 DPI (suitable for presentations)

## Technical Details

### Real Census Tract Example Algorithm

1. **Load Data**:
   - Census tract shapefiles for specified year
   - Population data from processed CSV files
   - Uses year-specific field names (GEOID10, GEOID20, etc.)

2. **Select Contiguous Cluster**:
   - Filter to Hennepin County, Minneapolis (FIPS: 27053)
   - Start from index 50 (typically central location)
   - BFS to find 12 contiguous tracts

3. **Build Adjacency Graph**:
   - Identify touching tracts using shapely `touches()`
   - Calculate real boundary lengths using `intersection()`
   - Convert to kilometers (handles both projected and unprojected CRS)

4. **Run METIS Partitioning**:
   - Uses project's `partition_graph()` wrapper
   - Parameters match main redistricting pipeline:
     - `nparts=2` (50-50 split)
     - `recursive=True` (recursive bisection)
     - `ufactor=1.005` (0.5% imbalance tolerance)
     - Edge weights = boundary lengths
     - Contiguity enforced via `-contig` flag

5. **Visualize Results**:
   - **Left panel**: Geographic map
     - Tracts colored by partition (blue/red)
     - Labels show tract ID (A-L) and population (K)
     - Non-cut edges labeled with boundary length
     - Cut edges highlighted in red with length
     - Total cut length shown at bottom
   - **Right panel**: Abstract graph
     - Nodes positioned using tract centroids (normalized)
     - Edge thickness proportional to boundary length
     - Cut edges shown as red dashed lines with X marks
     - All edges labeled with boundary lengths
     - Total cut weight shown at bottom

### METIS Integration

The skill uses the exact same METIS configuration as the main redistricting pipeline:
- Edge-weighted CSR format (code 011)
- Recursive bisection algorithm (`-ptype=rb`)
- Contiguity enforcement (`-contig`)
- Population balance constraint (ufactor)

This ensures the educational examples accurately reflect production behavior.

### Data Availability

**Schematic diagrams**: Always generated (no data dependencies)

**Real tract examples**: Requires:
- `data/geography/tiger_{year}_tracts/tl_{year}_27_tract{YY}/tl_{year}_27_tract{YY}.shp`
- `data/processed/census_{year}/mn_tracts_{year}_population.csv`

If data not found, script gracefully skips with warning message.

**Round progression maps**: Requires:
- `outputs/us_{year}_{version}/states/minnesota/maps/rounds/round_*.png`
- `outputs/us_{year}_{version}/states/alabama/maps/rounds/round_*.png`

If pipeline outputs not found, script warns and skips copying.

## Output Directory Structure

```
outputs/presentations/edge_weighted_bisection/figures/
├── example_gerrymander.png              # Schematic gerrymandering example
├── tract_to_graph.png                   # Transformation diagram
├── graph_with_cut.png                   # 3x3 grid graph with cut
├── edge_weights_example.png             # Unweighted vs edge-weighted
├── before_after_cut.png                 # Before/after comparison
├── real_tracts_to_graph.png             # Real census tracts (if data available)
├── minnesota_round_1_2_regions.png      # Copied from pipeline (if exists)
├── minnesota_round_2_4_regions.png      # Copied from pipeline (if exists)
├── minnesota_round_3_8_regions.png      # Copied from pipeline (if exists)
├── alabama_round_1_2_regions.png        # Copied from pipeline (if exists)
├── alabama_round_2_4_regions.png        # Copied from pipeline (if exists)
└── alabama_round_3_7_regions.png        # Copied from pipeline (if exists)
```

## Troubleshooting

**Missing pipeline outputs**:
```
[WARNING] Pipeline outputs not found at: outputs/us_2020_v1
Solution: Run redistricting first: /run-redistricting --year 2020 --version v1
```

**Missing census data**:
```
[WARNING] Census tracts shapefile not found at: data/geography/...
[WARNING] Population data not found at: data/processed/...
Solution: Download census data: /census-download --year 2020 --state minnesota
```

**METIS not available**:
```
[WARNING] METIS not available, using simple cut
Effect: Real tract example uses geometric median instead of METIS
Impact: Less realistic partitioning, but figure still generated
```

**Import errors**:
```
ModuleNotFoundError: No module named 'geopandas'
Solution: Install dependencies: pip install geopandas matplotlib shapely numpy
```

## Customization

### Change Target State/County

Edit `create_figures.py` line 682:
```python
# Change from Minnesota (FIPS 27, County 053)
tracts_file = Path(f'../../data/geography/tiger_{args.year}_06_tract{year_suffix}/...')
county_field = '037'  # Los Angeles County
```

### Change Number of Tracts

Edit line 733:
```python
while len(selected_indices) < 18 and queue:  # Increase from 12 to 18
```

**Note**: More tracts = better for showing algorithm, but labels may overlap

### Change Partition Ratio

Edit line 804:
```python
target_weights=[0.67, 0.33],  # Change from 50-50 to 67-33 split
```

### Increase Resolution

For publication-quality figures, edit DPI in `plt.savefig()` calls:
```python
plt.savefig(figures_dir / 'real_tracts_to_graph.png', dpi=300,  # Increase from 150
```

## Related Skills

- `/create-pedagogical-example` - Create algorithm examples with quality validation
- `/create-state-map` - Generate state-level maps
- `/run-redistricting` - Generate round progression maps via pipeline
- `/compile-latex` - Compile presentation after adding figures

## Usage Examples

### Generate All Figures for 2020 Data

```bash
# From project root
cd presentations/edge_weighted_bisection
python create_figures.py --year 2020 --version v1
```

### Generate for Different Census Year

```bash
python create_figures.py --year 2010 --version v1
```

### Use in LaTeX Beamer Presentation

```latex
\begin{frame}{Real Census Tract Example}
  \begin{figure}
    \centering
    \includegraphics[width=0.95\textwidth]{figures/real_tracts_to_graph.png}
    \caption{Minneapolis census tracts (left) transformed to graph (right) with METIS partitioning}
  \end{figure}
\end{frame}
```

## Performance

**Typical runtime**:
- Schematic diagrams: ~5-10 seconds (fast matplotlib rendering)
- Round map copying: ~1 second (file copy only)
- Real tract example: ~30-60 seconds (GIS operations, METIS partitioning)

**Total**: ~1-2 minutes for all figures

## Quality Notes

**Schematic diagrams**:
- Clean, clear illustrations
- Suitable for slides and papers
- No data dependencies (always work)

**Real tract examples**:
- Authentic census tract geometries
- Real METIS partitioning (matches production)
- Boundary lengths in kilometers
- Population balance validated

**Round progression maps**:
- High-quality pipeline outputs
- Consistent styling across rounds
- Shows algorithm behavior on real states

## Next Steps

After generating figures:
- Include in LaTeX presentation
- Compile presentation with `/compile-latex`
- Review figures for clarity
- Adjust parameters if needed (more tracts, different location, etc.)
