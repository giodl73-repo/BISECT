---
uuid: 40a1b0
slug: corner-adjacency-filter
name: nhancement: Filter Corner Adjacencies from Adjacency Graphs
wave_uuid: f4a1b4
created: '2026-01-25'
status: PLANNED
---
# Enhancement: Filter Corner Adjacencies from Adjacency Graphs

**Status**: Proposed  
**Priority**: Low  
**Created**: January 14, 2026
**Commits**: (Not yet implemented)
**Size**: (Not yet implemented)

## Problem

Census tracts can touch at single points (corners) without sharing a meaningful boundary. These "corner adjacencies" create edges in the adjacency graph with near-zero boundary length.

**Examples from Minneapolis visualization:**
- Tracts touching only at a single point (corner)
- Boundary length ~0.0 km (or very small like 0.01 km)
- These edges show up in adjacency graph but aren't meaningful for redistricting

## Impact

**Current behavior:**
- Corner adjacencies included in adjacency graph
- METIS sees these as valid edges with very small weights
- Doesn't significantly affect results (near-zero edge weight)
- Creates visual clutter in graph visualizations

**Why it's mostly harmless:**
- Edge weights are near-zero, so METIS effectively ignores them
- Population balance constraint dominates the partitioning
- Contiguity isn't broken (still connected via real edges)

## Proposed Solution

Add boundary length threshold at adjacency graph construction:

```python
# In build_tract_adjacency.py or adjacency.py
MIN_BOUNDARY_LENGTH = 0.1  # km (configurable)

if geom_i.touches(geom_j):
    boundary = geom_i.intersection(geom_j.boundary)
    if not boundary.is_empty:
        length_km = boundary.length / 1000
        
        # Filter corner adjacencies
        if length_km > MIN_BOUNDARY_LENGTH:
            adjacency[i].append(j)
            edge_weights[(i, j)] = length_km
```

## Implementation Options

**Option 1: Hard threshold (Recommended)**
- Filter adjacencies with boundary < 0.1 km
- Simple, effective, no configuration needed
- Threshold based on typical tract sizes

**Option 2: Relative threshold**
- Filter adjacencies < 1% of tract's total boundary
- Adapts to tract size
- More complex, probably unnecessary

**Option 3: Geometry-based filter**
- Use shapely to detect point-only touches vs line touches
- Most accurate but more expensive
- May not be worth the complexity

## Files to Modify

1. `scripts/build_tract_adjacency.py` - Add filter when building graphs
2. `src/apportionment/data/adjacency.py` - Add to adjacency construction
3. `../context/DATA_FORMATS.md` - Document filtering behavior
4. `../context/CODING_PATTERNS.md` - Note the threshold convention

## Testing

Compare results with and without filter:
- Should produce nearly identical district assignments
- May reduce edge count by 5-10%
- Should not affect connectivity or contiguity

## Benefits

- ✅ Cleaner adjacency graphs
- ✅ Slightly faster METIS (fewer edges)
- ✅ Better visualizations
- ✅ More semantically meaningful adjacencies

## Risks

- ⚠️ Must ensure connectivity preserved
- ⚠️ Edge case: very small tracts might have all boundaries filtered
- ⚠️ Need to validate on all 50 states

## Timeline

Not urgent - current behavior works fine. Could implement:
- After main research complete
- When rebuilding adjacency graphs anyway
- As part of a broader data pipeline cleanup

## Related Work

- Visualization already filters these (> 0.1 km threshold)
- Same threshold could be used at construction time
- Would align visualization with underlying data
