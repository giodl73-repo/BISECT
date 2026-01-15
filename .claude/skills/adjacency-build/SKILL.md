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

# Adjacency Graph Building Skill

## Overview

Create adjacency graphs that represent which census tracts share boundaries. These graphs are required inputs for METIS partitioning. Includes edge weights (boundary lengths in meters) for edge-weighted redistricting mode.

## Prerequisites

- Census tract data exists: `data/tracts/{year}/{state}_tracts_{year}.parquet`
- Tract geometries are valid polygons

## When to Use This Skill

- User says: "Build adjacency graphs"
- User says: "Create tract graphs for [state]"
- Pipeline fails with missing adjacency graph error
- After downloading new census data

## Workflow

### Step 1: Verify Tract Data Exists

```bash
# Check for tract data
ls data/tracts/2020/california_tracts_2020.parquet
```

If missing, use `/census-download` first.

### Step 2: Build Adjacency Graph

**Single state**:
```bash
python scripts/data/adjacency/build_adjacency_graph.py \
  --year 2020 \
  --state california
```

**Multiple states**:
```bash
# Build graphs for all states in parallel
for state in california texas florida; do
  python scripts/data/adjacency/build_adjacency_graph.py \
    --year 2020 \
    --state $state &
done
wait
```

**All 50 states**:
```bash
python scripts/data/adjacency/build_all_adjacency_graphs.py --year 2020
```

### Step 3: Process Steps

The script performs:

1. **Load tract geometries** from parquet file
2. **Create spatial index** (R-tree) for efficient queries
3. **Detect adjacencies**:
   - Shared boundaries (land connections)
   - Water boundaries (for islands/coastal areas)
   - Point adjacencies (corner touches)
4. **Calculate edge weights**:
   - Measure boundary lengths in meters
   - Convert to centimeters (integers for METIS)
   - Special handling for water/point adjacencies
5. **Validate connectivity**:
   - Check graph forms single connected component
   - Report islands or disconnected regions
6. **Save graph** as pickle file

### Step 4: Adjacency Detection

**Shared boundaries (primary)**:
```python
# Two tracts are adjacent if they share a boundary
boundary = tract_i.intersection(tract_j.boundary)
if not boundary.is_empty and boundary.length > 0:
    # Land adjacency with measurable boundary
    edge_weight = boundary.length  # meters
```

**Water boundaries (secondary)**:
```python
# For islands, use median land boundary length
if tract_has_no_land_neighbors:
    # Find nearest mainland tract across water
    edge_weight = median_of_all_land_boundaries
```

**Point adjacencies (tertiary)**:
```python
# Corners touching but no shared edge
if boundary.length < 0.1:  # Very small or point touch
    edge_weight = 0.1  # Minimal weight (10cm)
```

### Step 5: Connectivity Validation

**Check for single connected component**:
```bash
python scripts/data/geography/check_graph_connectivity.py \
  --year 2020 \
  --state california
```

Expected output:
```
Graph has 1 connected component (valid)
9,000 nodes, 27,000 edges
Average degree: 3.0
```

**If multiple components**:
```
Error: Graph has 2 connected components
Component 1: 8,995 tracts
Component 2: 5 tracts (islands)

Solution: Add water connections for islands
```

## Edge Weight Handling

**METIS requirements**:
- Edge weights must be positive integers
- Represent "cost" of cutting that boundary
- Longer boundaries = higher cost = less likely to cut

**Conversion**:
```python
# Boundary length in meters
length_m = boundary.length

# Convert to centimeters (integer)
edge_weight = int(length_m * 100)

# Special cases
if length_m < 0.001:  # Point adjacency
    edge_weight = 10  # 10cm minimum
if is_water_boundary:
    edge_weight = median_land_boundary_length_cm
```

**Edge weight statistics**:
- Typical range: 100 - 10,000 cm (1m - 100m)
- Water boundaries: ~2,000 cm (median)
- Point adjacencies: 10 cm (minimum)
- Maximum: ~100,000 cm (1km for very long boundaries)

## Output Format

**Pickle file contains**:
```python
{
    'adjacency': {
        0: [1, 2, 5],      # Tract 0 neighbors 1, 2, 5
        1: [0, 2, 3],      # Tract 1 neighbors 0, 2, 3
        # ... for all tracts
    },
    'edge_weights': {
        (0, 1): 1500,      # Boundary between 0 and 1 is 15m
        (0, 2): 2000,      # Boundary between 0 and 2 is 20m
        # ... for all edges
    },
    'num_nodes': 9000,
    'num_edges': 27000
}
```

**Saved to**: `data/adjacency/{year}/{state}_adjacency_{year}.pkl`

## Build Time Estimates

| State Size | Tracts | Time |
|------------|--------|------|
| Small (VT, DE, WY) | 50-200 | 10-30 seconds |
| Medium (AL, KY) | 500-1,500 | 1-2 minutes |
| Large (CA, TX, FL) | 5,000-10,000 | 5-10 minutes |
| **All 50 states** | ~73,000 | ~2-3 hours (parallel) |

## Common Issues

**Issue 1: Invalid Geometries**:
```
Error: Invalid geometry for tract 06001400100
Solution: Repair with buffer(0) in tract data
```

**Issue 2: Disconnected Components**:
```
Error: Graph has 3 connected components
Solution: Check for islands, add water connections
```

**Issue 3: Missing Nodes**:
```
Error: Node 1234 has no neighbors
Solution: Ensure all nodes explicitly added to graph before edges
```

**Critical pattern**:
```python
# ✅ DO: Add all nodes first
for i in range(num_nodes):
    graph.add_node(i)  # Even if no edges yet

# Then add edges
for i, neighbors in enumerate(adjacency):
    for j in neighbors:
        graph.add_edge(i, j)
```

**Issue 4: Memory Errors**:
```
MemoryError: Unable to create spatial index
Solution: Process by county, then merge county graphs
```

## Validation Checklist

After building:
- [ ] Pickle file exists at correct path
- [ ] Contains 'adjacency' and 'edge_weights' dicts
- [ ] All tracts represented (num_nodes matches tract count)
- [ ] Graph is connected (single component)
- [ ] Edge weights are positive integers
- [ ] Average degree ~2-4 (typical for geographic graphs)
- [ ] No isolated nodes (all tracts have ≥1 neighbor)

## Output Structure

```
data/adjacency/
├── 2000/
│   ├── california_adjacency_2000.pkl
│   ├── texas_adjacency_2000.pkl
│   └── ... (50 states)
├── 2010/
│   ├── california_adjacency_2010.pkl
│   └── ...
└── 2020/
    ├── california_adjacency_2020.pkl
    └── ...
```

## What You'll Get

- Adjacency graph with all tract neighbors
- Edge weights (boundary lengths in cm)
- Validated single connected component
- Ready for METIS partitioning

## Next Steps

After building:
- Use `/data-validate` to verify graph quality
- Use `/run-redistricting` to generate districts
- If connectivity fails, rebuild with adjusted water connection logic

## Related Scripts

- `scripts/data/adjacency/build_adjacency_graph.py` - Single state builder
- `scripts/data/adjacency/build_all_adjacency_graphs.py` - Batch builder
- `scripts/data/geography/check_graph_connectivity.py` - Validator
