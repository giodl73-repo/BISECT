# Wave F6: Legacy Cleanup

**Date**: TBD (Planned)
**Focus**: Complete block-level data support and 2000 census metro area maps
**Status**: 📋 PLANNED
**Priority**: LOW (nice-to-have)
**Estimated Duration**: 1-2 weeks
**Enhancements**: 8, 16
**Phases**:
- Phase 1: Enhancement 8 - Block Data (🔄 IN PROGRESS)
- Phase 2: Enhancement 16 - Metro Maps 2000 (📋 PLANNED)

---

## Goals

1. Complete block-level data support for all census years
2. Add 2000 census metro area maps for completeness
3. Resolve legacy incomplete features

---

## Success Metrics

| Metric | Target | Notes |
|--------|--------|-------|
| Block-level support | All years (2000, 2010, 2020) | Complete resolution options |
| Metro area maps | All years | Consistent visualization |

---

## Phases

### Phase 1: Enhancement 8 - Block-Level Data Support
**Status**: 🔄 IN PROGRESS (Phase 0 complete for 2010, partial for 2000)
**Priority**: LOW
**Estimated Effort**: 1 week

Complete block-level census data support for higher-resolution redistricting.

**Current Status**:
- 2010: Phase 0 complete (download and processing infrastructure)
- 2000: Partial implementation
- 2020: Not started

**Remaining Work**:
- Complete 2000 block-level processing
- Implement 2020 block-level support
- Test full pipeline with block-level data
- Document performance/quality tradeoffs

**Rationale**: Block-level data provides ~10x higher resolution than tracts, enabling finer-grained districts. However, computational cost is significant. Low priority because tract-level already produces good results.

### Phase 2: Enhancement 16 - 2000 Census Metro Area Maps
**Status**: 📋 PLANNED
**Priority**: LOW
**Estimated Effort**: 3-5 days

Add metro area overlay maps for 2000 census year for consistency.

**Rationale**: 2010 and 2020 have metro area maps, but 2000 is missing. Adding for completeness and consistency across all census years.

**Implementation**:
- Download 2000 metro area shapefiles
- Integrate into visualization pipeline
- Generate maps for all 2000 states
- Validate against 2010/2020 metro definitions

---

## Dependencies

**Prerequisites**: None (can be executed anytime)

**Blocking Issues**: None

---

## Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Block-level performance | High | Make optional, document tradeoffs clearly |
| 2000 metro data availability | Low | Census provides historical metro definitions |

---

## Related Enhancements

- [Enhancement 8](../enhancements/08_block_level_data.md) - Block-Level Data Support (in progress)
- [Enhancement 16](../enhancements/16_metro_2000.md) - 2000 Census Metro Area Maps (planned)

---

## Why This Wave Is Low Priority

1. **Tract-level sufficient**: Current tract-level resolution produces high-quality results
2. **High computational cost**: Block-level increases runtime 5-10x for marginal quality improvement
3. **Research impact**: Not required for current research papers
4. **Nice-to-have**: Completeness is good but not essential

**Recommendation**: Defer until after higher-priority waves (F2, F3, F4, F5)

---

**Wave F6 Summary**: Complete legacy features for full census year coverage and resolution options. Low priority due to sufficient quality with existing implementation.
