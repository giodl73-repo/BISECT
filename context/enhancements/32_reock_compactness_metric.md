# Enhancement 32: Reock Compactness Metric

**Status**: 📋 PLANNED
**Proposed**: January 16, 2026
**Complexity**: Medium (3-4 hours)
**Priority**: Low
**Commits**: (Not yet implemented)
**Size**: (Not yet implemented)

## Current State

The pipeline calculates **Polsby-Popper** compactness metric but not **Reock** metric.

**Polsby-Popper** (currently implemented):
- Formula: PP = 4π × Area / Perimeter²
- Measures how close a shape is to a circle
- Values: 0 (very non-compact) to 1 (perfect circle)

**Reock** (not implemented):
- Formula: Reock = Area / Area of Minimum Bounding Circle
- Also measures circularity
- Values: 0 (very non-compact) to 1 (perfect circle)
- Considered more intuitive than Polsby-Popper

**Blocker**: Shapely 2.x removed `minimum_bounding_circle()` method. We need to implement MBC calculation ourselves.

## Goal

Add Reock compactness metric to district analysis:
1. Implement minimum bounding circle (MBC) calculation
2. Calculate Reock scores for all districts
3. Add to `district_summary.csv` output
4. Visualize in compactness maps
5. Add comprehensive tests

## Implementation Plan

**Phase 1: Minimum Bounding Circle Implementation**
- Research MBC algorithms (Welzl's algorithm is O(n))
- Implement `calculate_minimum_bounding_circle(polygon)` helper
- Test with various shapes (circle, square, triangle, irregular polygons)
- Validate against known solutions
- Files:
  - `src/apportionment/analysis/geometry_utils.py` (new) - MBC calculation
  - `tests/unit/test_geometry_utils.py` (new) - MBC tests

**Phase 2: Reock Metric Calculation**
- Add Reock calculation to compactness analysis
- Integrate with existing Polsby-Popper logic
- Add to CSV output columns
- Files:
  - `scripts/compactness/analyze_compactness.py` - Add Reock calculation
  - `scripts/compactness/generate_compactness_maps.py` - Add Reock maps

**Phase 3: Visualization**
- Add Reock color maps (similar to Polsby-Popper)
- Update legends and labels
- Generate national Reock map
- Files:
  - `scripts/compactness/generate_compactness_maps.py` - Reock visualization
  - `scripts/compactness/generate_national_compactness_map.py` - National view

**Phase 4: Testing**
- Unit tests for MBC algorithm
- Unit tests for Reock calculation
- Integration tests for pipeline
- Baseline tests (compare to known values)
- Files:
  - `tests/unit/test_geometry_utils.py` - MBC tests
  - `tests/unit/test_compactness_analysis.py` - Reock tests (re-add)

**Phase 5: Dashboard Integration**
- Add Reock column to district tables
- Add Reock maps to compactness tab
- Update statistics displays
- Files:
  - `web/dashboard.html` - Add Reock displays
  - `scripts/web/generate_dashboard.py` - Bake Reock data

**Phase 6: Documentation**
- Document Reock formula and interpretation
- Update ARCHITECTURE.md
- Update DATA_FORMATS.md with new CSV columns
- Add to CHANGELOG.md

## Minimum Bounding Circle Algorithm

**Welzl's Algorithm** (recommended):
- Randomized incremental algorithm
- O(n) expected time complexity
- Compact implementation (~50 lines)

**Alternative: Smallest Enclosing Circle (SEC)**:
- Deterministic algorithm
- O(n²) worst case
- Simpler but slower

**Implementation approach**:
```python
def calculate_minimum_bounding_circle(polygon):
    """
    Calculate minimum bounding circle using Welzl's algorithm.

    Returns
    -------
    tuple
        (center_x, center_y, radius)
    """
    # Extract polygon boundary points
    points = np.array(polygon.exterior.coords)

    # Welzl's algorithm implementation
    center, radius = welzl_mbc(points)

    return center[0], center[1], radius
```

## Testing Plan

1. **Unit tests for MBC**:
   - Perfect circle → MBC = circle itself
   - Square → MBC circumscribes square (radius = diagonal/2)
   - Triangle → Various triangles with known solutions
   - Irregular polygon → Validate radius contains all points

2. **Unit tests for Reock**:
   - Circle → Reock = 1.0
   - Square → Reock ≈ 0.637 (4 / 2π)
   - Elongated rectangle → Reock < 0.3
   - Real tract geometries → Reasonable values (0.3-0.8)

3. **Integration tests**:
   - Vermont (1 district) → Single Reock score
   - Alabama (7 districts) → 7 Reock scores
   - Compare Reock vs Polsby-Popper correlation

4. **Baseline tests**:
   - Known compact districts (circles) → High Reock
   - Known non-compact districts (gerrymanders) → Low Reock
   - Ensure Reock ≈ Polsby-Popper in most cases

## Output Examples

**CSV Output** (`district_summary.csv`):
```csv
state,district,population,polsby_popper,reock,perimeter_m,area_sq_m
alabama,1,717381,0.442,0.516,128430.2,1234567890
alabama,2,718102,0.389,0.448,145230.8,1345678901
```

**Maps**:
- `compactness_reock_alabama_2020.png` - State map colored by Reock
- `compactness_reock_national_2020.png` - National overview

**Dashboard**:
- Add "Reock Score" column to district tables
- Add Reock map to Compactness tab
- Show Reock distribution histogram

## Benefits

1. **Academic credibility** - Reock is widely used in literature
2. **Intuitive interpretation** - "How circular is this district?"
3. **Complementary metric** - Provides second opinion vs Polsby-Popper
4. **Research value** - Compare algorithms by both metrics
5. **Visualization richness** - More compactness maps

## Success Criteria

- [ ] MBC algorithm implemented and tested
- [ ] Reock scores calculated for all districts
- [ ] Reock column added to `district_summary.csv`
- [ ] Reock maps generated (state + national)
- [ ] Dashboard displays Reock scores and maps
- [ ] All tests pass (unit + integration)
- [ ] Documentation updated
- [ ] Correlation with Polsby-Popper documented (expected: 0.7-0.9)

## Estimated Complexity

**Effort**: 3-4 hours
- Phase 1 (MBC): 1 hour (algorithm research + implementation)
- Phase 2 (Reock): 30 minutes (simple calculation)
- Phase 3 (Visualization): 45 minutes (copy Polsby-Popper patterns)
- Phase 4 (Testing): 45 minutes (comprehensive test suite)
- Phase 5 (Dashboard): 30 minutes (add columns + maps)
- Phase 6 (Documentation): 15 minutes

**Risk**: Low
- MBC algorithm is well-documented
- Integration follows existing compactness patterns
- No breaking changes to existing functionality

**Dependencies**: None

## Implementation Notes

**Why Shapely Removed MBC**:
- Computational geometry is complex
- Not used by most GIS applications
- Maintenance burden
- Users can implement themselves if needed

**Alternative Libraries**:
- `scipy.spatial.distance.cdist` for distance calculations
- `miniball` Python package (specialized for MBC)
- Custom implementation (recommended for control)

**Performance**:
- Welzl's algorithm: O(n) per district
- 435 districts × ~100 points each ≈ instant
- No performance concerns

**Quality Assurance**:
- Validate MBC radius ≥ maximum distance from center to any point
- Ensure Reock ≤ 1.0 always
- Check that circles have Reock ≈ 1.0

## References

- **Polsby, D. D., & Popper, R. D. (1991)**. "The Third Criterion: Compactness as a Procedural Safeguard Against Partisan Gerrymandering"
- **Reock, E. C. (1961)**. "Measuring Compactness as a Requirement of Legislative Apportionment"
- **Welzl, E. (1991)**. "Smallest Enclosing Disks (Balls and Ellipsoids)"

## Why This is Low Priority

1. **Polsby-Popper sufficient** - Already have one compactness metric
2. **Research tool** - More useful for papers than production
3. **No user demand** - Not requested by stakeholders
4. **Implementation blocker** - Need to implement MBC ourselves

This enhancement can wait until after higher-priority features (testing, validation, block-level data).
