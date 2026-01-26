# Wave F2: Production Polish

**Date**: TBD (Planned)
**Focus**: Production-ready features with experimental variants configuration and Pipeline Manager web app
**Status**: 📋 PLANNED
**Priority**: HIGH - Strong candidate for Wave 08
**Estimated Duration**: 2-3 weeks
**Enhancements**: 36, 51
**Phases**:
- Phase 1: Enhancement 36 - Variant System (📋 PLANNED)
- Phase 2: Enhancement 51 - Pipeline Manager (📋 PLANNED)

---

## Goals

1. Build experimental variants configuration system for algorithm testing
2. Create Pipeline Manager web app for production pipeline control
3. Enable systematic algorithm experimentation
4. Provide web-based pipeline management and monitoring

---

## Success Metrics

| Metric | Target | Notes |
|--------|--------|-------|
| Variant configurations | 5+ | Edge-weighted modes, parameter sets |
| Experiment tracking | Automated | Track all variant runs systematically |
| Pipeline Manager UI | Web app | Replace command-line pipeline control |
| Real-time monitoring | Live updates | WebSocket-based progress streaming |
| Run history | Persistent | Database-backed run tracking |

---

## Phases

### Phase 1: Enhancement 36 - Experimental Variants Configuration System
**Status**: 📋 PLANNED
**Priority**: HIGH
**Estimated Effort**: 1-2 weeks

Build system for defining and running algorithm variants systematically.

**Rationale**: Critical for research - need to test different edge weighting modes, population tolerances, and METIS parameters. Current ad-hoc approach makes experiments hard to reproduce.

**Features**:
- YAML-based variant definitions
- Parameter sweeps (edge weight scaling, population tolerance)
- Automated result comparison
- Statistical analysis of variant performance
- Reproducible experiment tracking

### Phase 2: Enhancement 51 - Pipeline Manager Web App
**Status**: 📋 PLANNED
**Priority**: HIGH
**Estimated Effort**: 1-2 weeks

Build web application for pipeline execution, monitoring, and result browsing.

**Rationale**: Command-line pipeline control is too technical for broader use. Web UI enables easier pipeline management, real-time monitoring, and result exploration.

**Features**:
- Start/stop/restart pipeline runs
- Real-time progress monitoring with hierarchical display
- Run history and comparison
- Result file browsing
- Error log viewing
- State-by-state status tracking

---

## Dependencies

**Prerequisites**:
- Completed historical waves (WAVE01-07) ✅
- Stable pipeline infrastructure
- Enhancement Manager (35) as reference implementation

**Blocking Issues**: None

---

## Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Variant system complexity | Medium | Start with simple parameters, extend gradually |
| Pipeline Manager UI scope creep | Medium | MVP first, iterate based on usage |
| Configuration format changes | Low | Use versioned YAML schemas |

---

## Related Enhancements

- [Enhancement 36](../enhancements/36_experimental_variants.md) - Experimental Variants Configuration (planned)
- [Enhancement 51](../enhancements/51_pipeline_manager.md) - Pipeline Manager Web App (planned)

---

## Why This Wave Should Be Next (Wave 08)

1. **High Impact**: Both enhancements are HIGH priority and enable critical research workflows
2. **No Dependencies**: Can be executed immediately after WAVE07
3. **Research Enablement**: Variant system unlocks systematic algorithm experimentation needed for papers
4. **Usability**: Pipeline Manager makes system accessible to non-technical users
5. **Moderate Scope**: 2-3 weeks is achievable without being too ambitious

---

**Wave F2 Summary**: Polish production system with experimental variants for research and web-based pipeline management for broader accessibility.
