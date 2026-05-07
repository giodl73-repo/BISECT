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

Generate high-quality figures for academic presentations, papers, educational materials. Creates schematic diagrams (algorithmic concepts) + real census tract examples (algorithm behavior).

## Prerequisites
**Required**: Python 3.13+ with matplotlib, geopandas, numpy, shapely
**Optional** (real tract examples): Census tract shapefiles (`data/geography/tiger_{year}_tracts/`), population data (`data/processed/census_{year}/`), pipeline outputs with round maps (`outputs/us_{year}_{version}/`)

## When to Use
User says "Create presentation figures/Generate figures for presentation/paper/I need figures/educational examples", needs schematic diagrams (gerrymandering/algorithm concepts), wants real census tract visualizations, preparing slides/documents for academic purposes

## Figure Types

### Schematic Diagrams
1. **Example Gerrymander**: Illinois 4th "Earmuffs", narrow corridor connecting distant communities, grid overlay → `figures/example_gerrymander.png`
2. **Tract-to-Graph Transform**: Side-by-side geographic tracts → abstract graph, shows census tracts become nodes with adjacency edges, population labels → `figures/tract_to_graph.png`
3. **Graph with Cut**: 3x3 grid showing partition, red dashed lines for cut edges, colored nodes by partition, population labels/balance → `figures/graph_with_cut.png`
4. **Edge Weights**: Side-by-side unweighted vs edge-weighted graphs, boundary lengths affect partitioning, thickness scaled by weight, weight labels → `figures/edge_weights_example.png`
5. **Before/After Cut**: Graph before (single region) + after partitioning (2 districts), highlights cut edges/weight → `figures/before_after_cut.png`

### Real Census Tract Examples
6. **Real Tracts to Graph with METIS**: Actual census data (default: Minneapolis, Hennepin County), left panel (geographic map with tract labels/populations/boundary lengths), right panel (abstract graph), METIS partitioning with contiguity, red lines show cuts, total cut length calculated, population balance shown → `figures/real_tracts_to_graph.png`

**Features**: Auto-selects 12 contiguous tracts (BFS), calculates real boundary lengths (km), uses METIS (same params as pipeline), ensures contiguous partitions, labels edges with lengths, highlights cut edges (red, total cut weight)

### Round Progression Maps
7. **Round-by-Round** (copied from pipeline): Minnesota rounds 1-3 (2→4→8 districts), Alabama rounds 1-3 (2→4→7 districts), source `outputs/us_{year}_{version}/states/{state}/maps/rounds/` → `figures/{state}_round_*.png`

## Workflow

### Step 1: Determine Parameters
**Required**: `--year` (2000/2010/2020), `--version` (v1/v2/test)
Ask if not specified or use defaults: Year 2020, Version v1

### Step 2: Navigate to Presentation Directory
```bash
cd presentations/edge_weighted_bisection
```

### Step 3: Run Figure Generation
```bash
python create_figures.py --year 2020 --version v1
```

**Actions**: Creates `outputs/presentations/edge_weighted_bisection/figures/`, copies round maps (if available), generates 6 schematic diagrams, generates real census tract example with METIS (if data available)

### Step 4: Verify Outputs
```bash
ls ../../outputs/presentations/edge_weighted_bisection/figures/
```

**Expected**: `example_gerrymander.png`, `tract_to_graph.png`, `graph_with_cut.png`, `edge_weights_example.png`, `before_after_cut.png`, `real_tracts_to_graph.png` (if data), `minnesota_round_*.png` (if pipeline outputs), `alabama_round_*.png` (if pipeline outputs)

### Step 5: Use in Presentation
**LaTeX**: `\includegraphics[width=0.8\textwidth]{figures/real_tracts_to_graph.png}`
**PowerPoint**: Navigate to figures dir, insert images directly
**Resolution**: All figures 150 DPI (presentation-suitable)

## Technical Details

### Real Census Tract Algorithm
1. **Load Data**: Census tract shapefiles (specified year), population data from CSVs, year-specific field names (GEOID10, GEOID20)
2. **Select Contiguous Cluster**: Filter to Hennepin County Minneapolis (FIPS 27053), start index 50 (central), BFS finds 12 contiguous tracts
3. **Build Adjacency**: Identify touching tracts (`shapely.touches()`), calculate real boundary lengths (`intersection()`), convert to km (handles projected/unprojected CRS)
4. **Run METIS**: Uses `partition_graph()` wrapper, matches main pipeline params (`nparts=2` 50-50 split, `recursive=True`, `ufactor=1.005` 0.5% tolerance, edge weights=boundary lengths, contiguity enforced `-contig`)
5. **Visualize**: Left panel (geographic map, tracts colored by partition blue/red, labels show tract ID A-L + population K, non-cut edges labeled with length, cut edges red with length, total cut length bottom), Right panel (abstract graph, nodes positioned using centroids normalized, edge thickness ∝ boundary length, cut edges red dashed with X marks, all edges labeled, total cut weight bottom)

### METIS Integration
Uses exact same METIS config as main pipeline: Edge-weighted CSR format (code 011), recursive bisection (`-ptype=rb`), contiguity (`-contig`), population balance (ufactor). Ensures educational examples accurately reflect production behavior.

### Data Availability
**Schematic diagrams**: Always generated (no dependencies)
**Real tract examples**: Requires `data/geography/tiger_{year}_tracts/tl_{year}_27_tract{YY}/*.shp` + `data/processed/census_{year}/mn_tracts_{year}_population.csv` (graceful skip with warning if missing)
**Round progression**: Requires `outputs/us_{year}_{version}/states/{minnesota,alabama}/maps/rounds/round_*.png` (warns and skips if missing)

## Output Directory
```
outputs/presentations/edge_weighted_bisection/figures/
├── example_gerrymander.png              # Schematic
├── tract_to_graph.png                   # Transform
├── graph_with_cut.png                   # 3x3 grid
├── edge_weights_example.png             # Unweighted vs weighted
├── before_after_cut.png                 # Before/after
├── real_tracts_to_graph.png             # Real census (if data)
├── minnesota_round_1_2_regions.png      # Copied (if exists)
├── minnesota_round_2_4_regions.png
├── minnesota_round_3_8_regions.png
├── alabama_round_1_2_regions.png
├── alabama_round_2_4_regions.png
└── alabama_round_3_7_regions.png
```

## Troubleshooting

**Missing pipeline outputs**: `[WARNING] Pipeline outputs not found` → `/run-redistricting --year 2020 --version v1`
**Missing census data**: `[WARNING] Census tracts/population not found` → `/census-download --year 2020 --state minnesota`
**METIS not available**: `[WARNING] METIS not available, using simple cut` → Real tract example uses geometric median (less realistic but figure still generated)
**Import errors**: `ModuleNotFoundError: geopandas` → `pip install geopandas matplotlib shapely numpy`

## Customization

**Change target state/county**: Edit `create_figures.py` line 682: `tracts_file = Path(f'...tiger_{args.year}_06_tract.../...')`, `county_field = '037'` (LA County)
**Change tract count**: Edit line 733: `while len(selected_indices) < 18 and queue:` (increase from 12 to 18, more tracts=better algorithm demo but labels may overlap)
**Change partition ratio**: Edit line 804: `target_weights=[0.67, 0.33]` (change from 50-50 to 67-33 split)
**Increase resolution**: Edit `plt.savefig()` calls: `dpi=300` (increase from 150 for publication quality)

## Performance

**Runtime**: Schematic diagrams ~5-10s (matplotlib), round map copying ~1s (file copy), real tract example ~30-60s (GIS operations + METIS)
**Total**: ~1-2 min for all figures

## Quality Notes

**Schematic**: Clean clear illustrations, suitable for slides/papers, no data dependencies (always work)
**Real tract examples**: Authentic census geometries, real METIS partitioning (matches production), boundary lengths in km, population balance validated
**Round progression**: High-quality pipeline outputs, consistent styling across rounds, shows algorithm on real states

## Related Skills
`/create-pedagogical-example` (algorithm examples with quality validation), `/create-state-map` (state-level maps), `/run-redistricting` (generate round progression via pipeline), `/compile-latex` (compile presentation after adding figures)

## Next Steps
Include in LaTeX presentation, compile with `/compile-latex`, review for clarity, adjust parameters if needed (more tracts, different location, etc.)
