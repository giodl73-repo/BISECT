---
name: adjacency-build
description: Build adjacency graphs for census tracts by detecting shared boundaries. Use when tract data exists but graphs are missing. Handles spatial indexing, water boundaries, islands, and connectivity validation.
allowed-tools:
  - Read
  - Write
  - Bash
  - Glob
user-invocable: true
---

# Adjacency Graph Building

Create adjacency graphs representing which census tracts share boundaries. Required inputs for METIS partitioning. Includes edge weights (boundary lengths in meters) for edge-weighted mode.

## Prerequisites
Census tract data exists (`data/tracts/{year}/{state}_tracts_{year}.parquet`), tract geometries are valid polygons

## When to Use
User says "Build adjacency graphs/Create tract graphs for [state]", pipeline fails with missing adjacency graph error, after downloading new census data

## Workflow

### Step 1: Verify Tract Data
```bash
ls data/tracts/2020/california_tracts_2020.parquet  # Check exists
```
If missing → `/census-download` first

### Step 2: Build Adjacency Graph
**Single state**: `python scripts/data/adjacency/build_adjacency_graph.py --year 2020 --state california`
**Multiple states**: Loop with `&` background jobs + `wait`
**All 50 states**: `python scripts/data/adjacency/build_all_adjacency_graphs.py --year 2020`

### Step 3: Process Steps
1. Load tract geometries from parquet
2. Create spatial index (R-tree) for efficient queries
3. Detect adjacencies: Shared boundaries (land), water boundaries (islands/coastal), point adjacencies (corner touches)
4. Calculate edge weights: Measure boundary lengths (meters), convert to centimeters (integers for METIS), special handling for water/point adjacencies
5. Validate connectivity: Check single connected component, identify islands/disconnected regions, add water connections if needed
6. Save graph: Pickle format `data/adjacency/{year}/{state}_adjacency_{year}.pkl`, contains adjacency dict + edge_weights dict

### Step 4: Verify Output
```bash
ls data/adjacency/2020/california_adjacency_2020.pkl
python -c "import pickle; g = pickle.load(open('data/adjacency/2020/california_adjacency_2020.pkl', 'rb')); print(f'Nodes: {len(g[\"adjacency\"])}, Edges: {sum(len(v) for v in g[\"adjacency\"].values())//2}')"
```

## Graph Structure

**Pickle file contains**:
```python
{
  'adjacency': {geoid: [neighbor_geoid1, neighbor_geoid2, ...], ...},  # Dict of GEOID -> list of neighbor GEOIDs
  'edge_weights': {(geoid1, geoid2): weight_cm, ...}  # Dict of edge tuples -> boundary length (cm)
}
```

**Properties**: Undirected graph, edge weights symmetric, nodes = census tract GEOIDs (strings), single connected component (after water connections)

## Adjacency Detection

**Shared boundary** (primary): `geom1.touches(geom2)` and `geom1.intersection(geom2).length > 0`, edge weight = boundary length in cm
**Water connection** (islands/coastal): If graph disconnected, find nearest tracts across water, add edge with weight = distance in cm
**Point adjacency** (corners): `geom1.touches(geom2)` but intersection length = 0, edge weight = 1 cm (minimal)

## Connectivity Validation

**Check**: NetworkX `number_connected_components(G)` should be 1
**If >1 component**: Identify disconnected regions (islands), find nearest pairs across water, add connections with distance-based weights, re-validate
**Special cases**: Hawaii (8 islands → need water connections), Alaska (islands in panhandle), Michigan (upper peninsula), coastal states (barrier islands)

## Performance

| State Size | Tracts | Runtime | Memory |
|------------|--------|---------|--------|
| Small (VT) | ~200 | ~10s | ~50MB |
| Medium (AL)| ~1,200 | ~30s | ~200MB |
| Large (CA) | ~9,000 | ~3min | ~1GB |
| All 50 states (parallel) | ~73K | ~15min | ~10GB peak |

**Bottlenecks**: Spatial index creation (R-tree), intersection calculations (O(n²) naively → O(n log n) with R-tree), connectivity validation (NetworkX)

## Troubleshooting

**Invalid geometries**: `GEOSException: Invalid geometry` → Repair with `buffer(0)` before building
**Memory error**: Large states OOM → Process in chunks, use simplified geometries
**Disconnected graph**: `ValueError: Graph has X components` → Add water connections (automatic in script), verify islands handled
**Missing data**: `FileNotFoundError: tracts parquet not found` → Run `/census-download` first

## Output Validation

**Check node count**: Should match tract count in source data
**Check edge count**: Typical 2-4 edges per node average (2D planar graph)
**Check connectivity**: Single component (graph connected)
**Check edge weights**: All positive integers in cm (1-100,000 typical for shared boundaries)

## What You'll Get

Adjacency graph pickle file (`data/adjacency/{year}/{state}_adjacency_{year}.pkl`), edge weights for METIS (boundary lengths in cm), validated connectivity (single component), water connections added (if needed for islands), ready for METIS partitioning

## Related Skills
`/census-download` (get tract data before building graphs), `/run-redistricting` (uses adjacency graphs), `/data-validate` (validate graphs after building)

## Next Steps
Verify graph exists for all states needed, run `/run-redistricting` to use graphs, regenerate if tract data changes (new year, corrected geometries)
