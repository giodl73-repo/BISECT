# Water-Based Adjacency Implementation Guide

## Overview

Water-based adjacency is a **critical feature** required for **30+ states** with island or coastal geography. Without it, island tracts would be isolated and METIS would fail to create contiguous districts.

## The Fundamental Requirement

**CRITICAL**: Each state's adjacency graph must form a **single connected component**. Every tract must be reachable from every other tract through adjacency links.

Without water adjacency, island states would have **disconnected subgraphs**, causing METIS to fail.

## The Problem

Census tracts on islands have no land-based adjacency to the mainland. This creates **disconnected graph components** within a state. Examples:
- Hawaiian islands (Oahu, Maui, Big Island, Kauai, etc.)
- Massachusetts islands (Martha's Vineyard, Nantucket)
- Alaska's extensive island communities
- Washington's San Juan Islands (100+ islands)
- Michigan's Great Lakes islands
- New York (Long Island, Staten Island)
- California's Channel Islands
- Florida Keys
- Maine's 3,000+ coastal islands
- And 20+ more states...

## The Solution: County-Aware Water Adjacency

### Core Algorithm

**For each island tract** (tract with zero land-based neighbors):
1. Identify it as an island (no existing adjacencies OR only water)
2. Find the nearest tract **with the same county GEOID prefix**
3. Create a bidirectional adjacency link across water
4. Result: Island is now connected within its proper county

### Why County-Aware Matching is Critical

❌ **WRONG**: Connect island to nearest tract regardless of county
- Martha's Vineyard (Dukes County) could connect to Nantucket (Nantucket County)
- Hawaiian islands could connect across county lines
- Breaks county boundaries in redistricting

✅ **CORRECT**: Connect island to nearest tract **in same county**
- Martha's Vineyard connects to mainland Dukes County
- Nantucket connects to mainland Nantucket County (if exists) or stays as own district
- Hawaiian islands connect within Honolulu County, Maui County, Hawaii County, or Kauai County respectively
- Respects county boundaries as redistricting typically does

## Implementation Details

### Current 2020 Implementation

**File**: `scripts/create_adjacency_with_water.py`

**Key Code Pattern**:
```python
# 1. Identify island tracts
island_tracts = []
for i, tract in enumerate(tracts):
    if len(adjacency[i]) == 0:  # No neighbors
        island_tracts.append(i)

# 2. For each island, find nearest same-county tract
for island_idx in island_tracts:
    island_geoid = tracts.iloc[island_idx]['GEOID']
    county_code = island_geoid[:5]  # First 5 digits = state + county

    # Find all tracts in same county that are NOT islands
    same_county_tracts = []
    for j, tract in enumerate(tracts):
        if j != island_idx:
            tract_geoid = tracts.iloc[j]['GEOID']
            if tract_geoid.startswith(county_code):
                if len(adjacency[j]) > 0:  # Not another island
                    same_county_tracts.append(j)

    # Find nearest
    min_distance = float('inf')
    nearest_idx = None
    island_centroid = tracts.iloc[island_idx].geometry.centroid

    for j in same_county_tracts:
        tract_centroid = tracts.iloc[j].geometry.centroid
        distance = island_centroid.distance(tract_centroid)
        if distance < min_distance:
            min_distance = distance
            nearest_idx = j

    # Create bidirectional link
    if nearest_idx is not None:
        adjacency[island_idx].append(nearest_idx)
        adjacency[nearest_idx].append(island_idx)
```

### GEOID Structure (Critical for County Matching)

**2020 Census**: GEOID is 11 digits
- Positions 1-2: State FIPS code
- Positions 3-5: County FIPS code
- Positions 6-11: Tract code
- **County Code = GEOID[:5]** (first 5 digits)

Example:
- `06075021700` = California (06), San Francisco County (075), Tract 0217.00
- `06075021800` = California (06), San Francisco County (075), Tract 0218.00
- Both match on `06075` = same county

**2010 Census**: GEOID10 field (same structure)
**2000 Census**: May use CTIDFP00 or different field name (verify!)

## States Requiring Water Adjacency (Comprehensive List)

### Tier 1: Critical (Major Island Populations)
1. **Hawaii** - Multiple major islands, 4 counties
2. **Alaska** - Extensive island communities
3. **Massachusetts** - Martha's Vineyard, Nantucket
4. **Washington** - San Juan Islands, Whidbey Island
5. **California** - Channel Islands, Bay Area islands
6. **New York** - Long Island, Staten Island
7. **Michigan** - Great Lakes islands
8. **Florida** - Florida Keys, barrier islands
9. **Maine** - 3,000+ coastal islands

### Tier 2: Significant Island/Coastal Areas
10. Rhode Island
11. Maryland (Chesapeake Bay)
12. Virginia (Eastern Shore)
13. North Carolina (Outer Banks)
14. South Carolina (Sea Islands)
15. Georgia (Sea Islands)
16. Louisiana (barrier islands)
17. Texas (Galveston, Padre Island)
18. Connecticut (Long Island Sound)
19. New Jersey (barrier islands)
20. Wisconsin (Apostle Islands)
21. Ohio (Lake Erie islands)
22. Oregon (coastal islands)

### Tier 3: Smaller Island Areas
23. Delaware
24. Minnesota
25. Pennsylvania
26. Vermont (Lake Champlain)
27. New Hampshire
28. Mississippi (barrier islands)
29. Alabama (Mobile Bay)
30. Illinois (river islands)
31. Tennessee (river islands)

## Testing & Validation

### Connectivity Validation (CRITICAL)

**Before running redistricting**, validate that each state forms a single connected component:

```python
# Pseudocode for connectivity validation
def validate_state_connectivity(adjacency_graph, num_tracts):
    """Verify all tracts are reachable from tract 0."""
    visited = set()
    queue = [0]

    while queue:
        current = queue.pop(0)
        if current in visited:
            continue
        visited.add(current)

        for neighbor in adjacency_graph[current]:
            if neighbor not in visited:
                queue.append(neighbor)

    if len(visited) != num_tracts:
        disconnected_tracts = set(range(num_tracts)) - visited
        raise ValueError(
            f"State has disconnected tracts! "
            f"Connected: {len(visited)}, Disconnected: {len(disconnected_tracts)}"
        )

    return True  # Fully connected
```

**This validation MUST pass before METIS can run successfully.**

### Automated Tests
```bash
# Validate state connectivity (MUST run first!)
python scripts/validate_state_connectivity.py --state HI --year 2020

# Test island connectivity for a state
python scripts/test_water_adjacency.py --state HI --year 2020

# Validate all districts are contiguous
python scripts/validate_contiguity.py --state HI --year 2020
```

### Manual Verification Checklist

For each census year (2020, 2010, 2000):

**Hawaii** (Most Critical):
- [ ] Verify Oahu tracts connect within Honolulu County
- [ ] Verify Maui, Lanai, Molokai connect within Maui County
- [ ] Verify Big Island tracts connect within Hawaii County
- [ ] Verify Kauai tracts connect within Kauai County
- [ ] Verify NO cross-county water connections

**Massachusetts**:
- [ ] Verify Martha's Vineyard (Dukes County) connects to mainland Dukes County
- [ ] Verify Nantucket (Nantucket County) handled appropriately
- [ ] Verify Boston Harbor islands connect properly

**Alaska**:
- [ ] Verify Aleutian Islands connect within proper boroughs
- [ ] Verify Alexander Archipelago islands connect properly
- [ ] Verify no orphaned island communities

**Washington**:
- [ ] Verify San Juan Islands connect within San Juan County
- [ ] Verify Whidbey Island connects within Island County

**New York**:
- [ ] Verify Long Island tracts have proper connections
- [ ] Verify Staten Island (Richmond County) connects properly

### Common Issues to Watch For

1. **Orphaned Islands**: Island tract with NO water adjacency created
   - Symptom: METIS fails with "disconnected graph" error
   - Fix: Verify county-matching logic is working

2. **Cross-County Connections**: Island connected to wrong county
   - Symptom: District crosses county lines inappropriately
   - Fix: Verify GEOID prefix matching (first 5 digits)

3. **Missing GEOID Field**: Script can't find county code
   - Symptom: All islands remain disconnected
   - Fix: Update field name for census year (GEOID vs GEOID10 vs CTIDFP00)

4. **Multiple Islands Need Same Connection**: Two islands both nearest to same mainland tract
   - This is OK! Multiple islands can connect to the same tract
   - METIS will handle multiple connections properly

## Historical Census Considerations

### 2010 Census
- Field name: **GEOID10** (not GEOID)
- Same 11-digit structure
- Script must use: `county_code = tract['GEOID10'][:5]`

### 2000 Census
- Field name: **CTIDFP00** or **GEOID** (verify!)
- May have different digit structure (research needed)
- Script must handle legacy format

## References

- Current implementation: `scripts/create_adjacency_with_water.py`
- Census TIGER/Line documentation: https://www.census.gov/geographies/mapping-files/time-series/geo/tiger-line-file.html
- GEOID structure: https://www.census.gov/programs-surveys/geography/guidance/geo-identifiers.html

## Key Takeaway

**Water-based adjacency is NOT optional.** It is required for 30+ states and must be county-aware to produce valid redistricting results. When implementing historical census support, this feature MUST be included from the start.

---

**Last Updated**: 2026-01-09
**Implementation Status**: ✅ 2020 Complete, ⏳ 2010/2000 Pending
