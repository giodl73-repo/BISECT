# Wave F4: Algorithm Improvements

**Date**: TBD (Planned)
**Focus**: Algorithm refinements with Reock metric, corner adjacency filtering, and real-world constraints
**Status**: 📋 PLANNED
**Priority**: MEDIUM
**Estimated Duration**: 3-4 weeks
**E**: 32, 40, 44
**Phases**:
- Phase 1: E32 - Reock Metric (📋 PLANNED)
- Phase 2: E40 - Corner Filtering (📋 PLANNED)
- Phase 3: E44 - Legal Constraints (📋 PLANNED)

---

## Goals

1. Add Reock compactness metric as alternative to Polsby-Popper
2. Filter corner adjacencies from adjacency graphs for cleaner topology
3. Implement real-world redistricting constraints (VRA, COI)
4. Improve algorithm quality and legal compliance

---

## Success Metrics

| Metric | Target | Notes |
|--------|--------|-------|
| Compactness metrics | 2 (P-P + Reock) | Dual metric validation |
| Corner adjacencies | Filtered | Cleaner graph topology |
| VRA compliance | Validated | Voting Rights Act constraints |
| COI integration | Implemented | Communities of Interest respected |

---

## Phases

### Phase 1: E32 - Reock Compactness Metric
**Status**: 📋 PLANNED
**Priority**: LOW
**Estimated Effort**: 1 week

Add Reock compactness metric (circle-based) as complement to Polsby-Popper (perimeter-based).

**Rationale**: Reock metric captures different aspect of compactness. Some districts may score well on one metric but poorly on another. Having both provides fuller picture.

**Implementation**:
- Calculate minimum bounding circle for each district
- Compute Reock score (district area / circle area)
- Add to district summary CSV
- Create Reock visualization maps
- Compare P-P vs Reock rankings

### Phase 2: E40 - Filter Corner Adjacencies
**Status**: 📋 PLANNED
**Priority**: LOW
**Estimated Effort**: 1 week

Remove corner-only adjacencies from adjacency graphs to enforce shared border requirement.

**Rationale**: Current adjacency graphs include tracts that only touch at corners. This can create districts connected only at points, which looks unnatural. Filtering to shared-edge adjacencies creates cleaner districts.

**Implementation**:
- Detect corner-only adjacencies in graph construction
- Filter to only shared-edge adjacencies
- Validate contiguity still satisfied
- Compare filtered vs unfiltered compactness
- Document impact on district quality

### Phase 3: E44 - Real-World Redistricting Constraints
**Status**: 📋 PLANNED
**Priority**: MEDIUM
**Estimated Effort**: 1-2 weeks

Implement legal constraints: Voting Rights Act (VRA) compliance and Communities of Interest (COI).

**Rationale**: Real redistricting must satisfy legal requirements. VRA requires majority-minority districts in certain states. COI considerations (keeping cities/counties together) improve representation quality.

**Implementation**:
- VRA compliance:
  - Identify VRA-required states/regions
  - Ensure sufficient majority-minority districts
  - Validate against DOJ criteria
- COI integration:
  - Define communities (cities, counties, regions)
  - Add soft constraints to keep COIs together
  - Balance COI vs compactness tradeoffs
- Documentation and validation

---

## Dependencies

**Prerequisites**:
- WAVE-F3 (Baseline data) - Needed for VRA/COI validation

**Blocking Issues**: None

---

## Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| VRA criteria complexity | High | Start with simple majority-minority requirement |
| COI definition ambiguity | Medium | Use standard definitions (cities, counties) |
| Performance impact | Medium | Make constraints optional, profile carefully |

---

## Related Enhancements

- [E32](../enhancements/32_reock_metric.md) - Reock Compactness Metric (planned)
- [E40](../enhancements/40_corner_adjacencies.md) - Filter Corner Adjacencies (planned)
- [E44](../enhancements/44_real_world_constraints.md) - Real-World Constraints (planned)

---

**Wave F4 Summary**: Refine algorithm with additional metrics, cleaner topology, and legal compliance constraints for real-world applicability.
