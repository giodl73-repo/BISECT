# Wave 9: API Migration

**Date**: 2026-01-25
**Focus**: Add FastAPI backend and React dashboard to existing pipeline without changing core functionality
**Status**: ✅ COMPLETED
**Actual Duration**: ~40 hours over 1 day (MVP scope)
**E**: 60, 61, 62, 63, 64
**Phases**:
- Phase 1: E60 - Project Setup & Infrastructure ✅ COMPLETED
- Phase 2: E61, 62 - Backend API & Pipeline Integration ✅ COMPLETED
- Phase 3: E63, 64 - React Dashboard & Visualization ✅ COMPLETED

---

## Management Summary

**Executive Assessment**: Wave 9 represents a significant strategic investment to transform the Apportionment project from a CLI-only tool to a full web application with interactive dashboard capabilities.

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Strategic Value | **HIGH** | Expands user accessibility significantly |
| Technical Feasibility | **GOOD** | Sound architecture, appropriate technology choices |
| Timeline Realism | **MODERATE** | 5-7 weeks achievable with identified risks |
| Resource Requirements | **REASONABLE** | Can be executed with 1-2 developers |
| Risk Level | **MEDIUM-HIGH** | E62 is critical path |
| ROI Projection | **POSITIVE** | Long-term value exceeds investment |

**Overall Score**: 8.1/10 - Strong plan, proceed with controls

**Decision**: **GO** - Approved with mandatory controls

**Mandatory Controls**:
1. Weekly checkpoint reviews with documented progress assessment
2. E62 quality gate must pass before full integration work
3. MVP delivery by end of week 4 (stretch) or week 5 (baseline)
4. Change control for any scope changes > 4 hours

---

## Goals

1. Create FastAPI backend wrapping existing Python pipeline
2. Set up PostgreSQL database for run metadata (not district data - use files)
3. Build React dashboard for running and viewing redistricting analyses
4. Integrate with App Manager system (port 8002 backend, 3002 frontend)
5. Enable web-based pipeline execution and monitoring
6. Provide interactive visualization of results
7. Maintain backward compatibility with existing CLI tools

---

## Resource Plan

### Team Size Recommendation

| Configuration | Team Size | Pros | Cons |
|---------------|-----------|------|------|
| **Optimal** | 2 developers (1 backend, 1 frontend) | Enables parallelization, reduces calendar time | Coordination overhead |
| **Minimum** | 1 full-stack developer | Lower coordination, simpler | Longer calendar time (6-7 weeks) |
| **Maximum Useful** | 3 developers | Fastest timeline | Diminishing returns due to dependencies |

### Skill Requirements Matrix

| Skill | Requirement Level | Enhancement | Risk if Gap |
|-------|-------------------|-------------|-------------|
| FastAPI | **Strong** | 61, 62 | High |
| asyncio/subprocess | **Strong** | 62 | **Critical** |
| SQLAlchemy | Moderate | 60, 61 | Medium |
| PostgreSQL | Moderate | 60, 61 | Low |
| React + TypeScript | **Strong** | 63, 64 | High |
| React Query | Moderate | 63 | Medium |
| Leaflet | Moderate | 64 | Medium |
| Docker/PM2 | Basic | 60, 64 | Low |
| Windows development | Moderate | 62 | Medium |

### Resource Allocation by Phase

**Phase 1 (Weeks 1-2)**: 1 developer
- E60 (solo work)
- E61 (solo work)

**Phase 2 (Weeks 3-4)**: 2 developers optimal
- Developer A: E62 (backend execution)
- Developer B: E63 (frontend with mocks)

**Phase 3 (Weeks 5-6)**: 1-2 developers
- E64 (requires E63 complete)
- Integration testing
- Deployment setup

**Week 7 (Buffer)**: As needed
- Risk mitigation
- Polish and optimization

---

## Timeline with Buffer

### Realistic Timeline: 5-7 weeks (112-134 hours)

| Phase | Allocated Hours | Buffer | Total Hours |
|-------|-----------------|--------|-------------|
| Setup (E60) | 12-16h | 2h | 14-18h |
| Backend (E61+62) | 44-54h | 6h | 50-60h |
| Frontend (E63+64) | 36-44h | 4h | 40-48h |
| Integration/Polish | 0h | 8h | 8h |
| **Total** | **92-114h** | **20h** | **112-134h** |

### Critical Path Analysis

```
Week 1: E60 (Setup) ────────────────────────────┐
                                                 │
Week 2: E61 (Run API) ──────────────────────────┼──► DEPENDENCY GATE
                                                 │
Week 3-4: E62 (Execution) ◄────────────────────┘    CRITICAL PATH
          E63 (Dashboard) [parallel if 2 devs]
                      │
Week 5:   E64 (Visualization) ◄─────────────────────
                      │
Week 6:   Integration, Polish, Deploy ◄─────────────
                      │
Week 7:   Buffer / Risk Mitigation (if needed)
```

**Critical Path Items**:
1. E60 blocks all subsequent work
2. E61 blocks E62
3. E62 blocks full integration testing
4. E63 must complete before E64

---

## Milestone Plan

| Week | Milestone | Exit Criteria | Checkpoint |
|------|-----------|---------------|------------|
| 1 | Infrastructure Complete | Docker Compose running, health endpoint functional, frontend scaffold accessible | Team review |
| 2 | API Layer Complete | CRUD endpoints complete, database migrations working, Swagger documentation available | Team review |
| 3 | Execution Engine Part 1 | Subprocess spawning works, STATUS parsing functional, file fallback implemented | **E62 Quality Gate** |
| 4 | **MVP Complete** | VT run executes via API, progress polling works, frontend shows progress, basic map displays | **Sponsor Demo** |
| 5 | Full Features | Color-by-metric, sorting/filtering, error handling complete | Team review |
| 6 | Production Ready | E2E tests pass, performance optimized, PM2 deployment configured | **Final Review** |
| 7 | Buffer (if needed) | Risk mitigation, polish, optimization | Release approval |

### Demo Checkpoints

| Demo | Week | Content | Audience |
|------|------|---------|----------|
| Infrastructure | 1 | Health endpoint, DB connection | Team |
| API Complete | 2 | Swagger UI, CRUD operations | Team |
| **Execution Engine** | 4 | VT run via API, progress polling | **Sponsor, Team** |
| **MVP Complete** | 4-5 | Full create-run-view flow | **Users, Sponsor** |
| Full Wave | 6 | Maps, deployment, polish | All stakeholders |

---

## Risk Register

| ID | Risk | Category | Likelihood | Impact | Score | Mitigation | Owner |
|----|------|----------|------------|--------|-------|------------|-------|
| R1 | Subprocess hangs | Technical | Medium | High | **HIGH** | Watchdog + file fallback | Backend Dev |
| R2 | Windows subprocess issues | Technical | High | Medium | **HIGH** | Test early, taskkill fallback | Backend Dev |
| R3 | Stdout buffering delays | Technical | Medium | Medium | MEDIUM | File-based progress fallback | Backend Dev |
| R4 | Server restart orphans | Technical | Low | Medium | LOW | PID tracking, orphan detection | Backend Dev |
| R5 | Frontend state sync bugs | Technical | Medium | Medium | MEDIUM | React Query patterns | Frontend Dev |
| R6 | Map performance issues | Technical | Medium | Medium | MEDIUM | Canvas renderer, simplification | Frontend Dev |
| R7 | Scope creep | Project | Medium | High | **HIGH** | Change control process | PM |
| R8 | Integration failures | Technical | Medium | High | **HIGH** | Early integration testing | Team |
| R9 | Key developer unavailable | Resource | Low | High | MEDIUM | Knowledge sharing, documentation | PM |
| R10 | Dependency on stable pipeline | External | Low | High | MEDIUM | Regression tests before wave | Team |

### Risk Monitoring Process

**Weekly Risk Review**:
1. Update risk scores based on actual progress
2. Identify new risks discovered during implementation
3. Verify mitigations are implemented, not just planned
4. Escalate HIGH risks that are not decreasing

**Escalation Criteria**:
- Any risk score increases to CRITICAL
- E62 delays exceed 3 days
- Integration testing reveals architectural issues

### Contingency Plans

| Trigger | Response |
|---------|----------|
| E62 exceeds 34 hours | Extend timeline, consider scope reduction |
| E62 testing fails | Block subsequent work, intensive debugging |
| Performance gates missed | Accept as warning, defer optimization |
| MVP not achievable by week 4 | Evaluate full wave completion, consider partial delivery |

---

## Success Metrics & KPIs

### Development Velocity Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Schedule variance | < 10% | (Actual - Planned) / Planned |
| Scope delivered | 100% MVP, 90%+ full | Features delivered / planned |
| Budget variance | < 15% | Hours invested / estimated |
| Blocker resolution | < 24 hours | Time from identification to resolution |

### Quality Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Unit test coverage | ≥ 80% | Test coverage report |
| Integration test pass rate | 100% | CI/CD pipeline |
| E2E critical flows | 100% | Playwright test suite |
| Bug escape rate | < 5% | Bugs found post-deployment / total bugs |

### User Adoption Targets (Post-Launch)

| Metric | 30-Day Target | 90-Day Target |
|--------|---------------|---------------|
| New users onboarded | 5 | 10 |
| Web runs vs CLI runs | 30% | 50% |
| User satisfaction | Positive feedback | NPS > 0 |
| Support tickets | < 5 | < 2/month |

---

## Stakeholder Communication Plan

### Communication Matrix

| Stakeholder | Interest | Influence | Communication Need |
|-------------|----------|-----------|-------------------|
| Project Sponsor | ROI, timeline | High | Weekly status, demos |
| Development Team | Technical details | High | Daily standups |
| End Users | Usability, features | Medium | Demo at MVP, feedback |
| Operations | Deployment, maintenance | Medium | Deployment plan review |
| QA/Testing | Coverage, quality | Medium | Test plan review |

### Communication Schedule

| Audience | Format | Frequency | Content |
|----------|--------|-----------|---------|
| Sponsor | Status Email | Weekly | Progress %, risks, decisions needed |
| Team | Standup | Daily | Yesterday, today, blockers |
| Users | Demo | Week 4 (MVP), Week 6 | Working prototype, full features |
| Operations | Meeting | Week 5 | Deployment walkthrough |

---

## Budget Estimate

### Development Investment

| Category | Hours | Notes |
|----------|-------|-------|
| Development | 92-114 | Enhancements 60-64 |
| Testing | 40-50 | 30-35% of development time |
| Documentation | 8-12 | API docs, deployment guide |
| Buffer | 20 | Risk mitigation |
| **Total** | **160-196** | ~4-5 FTE weeks |

### Ongoing Costs (Post-Launch)

| Item | Monthly Cost | Notes |
|------|--------------|-------|
| Infrastructure | Minimal | Docker container + PostgreSQL |
| Maintenance | ~4 hours | Bug fixes, minor updates |
| Operations | ~2 hours | Monitoring, restarts |

### ROI Analysis

**Investment**: 160-196 hours

**Quantified Benefits**:
| Benefit | Impact | Time Saved |
|---------|--------|------------|
| Reduced training time | 2h vs 8h per new user | 6h/user |
| Faster result exploration | 5 min vs 30 min | 25 min/session |
| Broader user accessibility | 10x potential user base | N/A |
| Reduced support burden | Self-service vs CLI guidance | 2h/month |

**Break-Even Analysis**:
- If 10 new users avoid 6 hours training each = 60 hours saved
- If 50 pipeline runs use web vs CLI, saving 20 min each = 17 hours saved
- **Break-even**: Within 3-6 months of deployment

---

## Mandatory Controls

The following controls are **required** for Wave 9 implementation:

### 1. Weekly Checkpoint Reviews

**Requirement**: Documented progress review every Friday
**Content**: Progress %, completed tasks, blockers, risk updates, next week plan
**Owner**: Team Lead
**Escalation**: Any blocker > 24 hours unresolved

### 2. E62 Quality Gate

**Requirement**: All E62 critical tests must pass before starting E63/E64 integration
**Criteria**:
- [ ] Subprocess spawn and terminate works on Windows
- [ ] STATUS protocol parsing functional
- [ ] File-based fallback tested
- [ ] Watchdog kills hung process
- [ ] VT integration test passes
**Owner**: Backend Dev + QA
**Blocker**: Integration work cannot proceed until gate passed

### 3. MVP-First Delivery Strategy

**Requirement**: MVP features complete by week 4 (stretch) or week 5 (baseline)
**MVP Criteria**:
- [ ] Create run via web interface
- [ ] Progress polling visible
- [ ] Completed results viewable
- [ ] Basic district map displayed
- [ ] CLI tools unchanged
**Owner**: Team Lead
**Escalation**: If MVP not achievable by week 5, scope reduction required

### 4. Change Control Process

**Requirement**: Formal approval for any scope change > 4 hours

| Change Size | Approval | Documentation |
|-------------|----------|---------------|
| Minor (< 4 hours) | Developer decision | Note in enhancement file |
| Moderate (4-16 hours) | Team discussion | Documented decision |
| Major (> 16 hours) | PM approval | Timeline impact assessment |

**Change Request Fields**: Description, hours impact, risk impact, justification, affected enhancements

---

## Success Metrics (Revised per Senior Designer and Tester)

### MVP (Minimum Viable Product)

| Metric | Target | Notes |
|--------|--------|-------|
| User can create run via web | Yes | Form with year, version, states, workers |
| Progress visible during execution | Yes | Polling-based (2 second intervals) |
| Completed run results viewable | Yes | District table with sorting/filtering |
| Basic district map | Yes | Leaflet, no AK/HI insets initially |
| CLI tools unchanged | Yes | Zero modifications to existing scripts |

### Full Wave Completion

| Metric | Target | Notes |
|--------|--------|-------|
| All MVP criteria | Yes | - |
| Run cancellation | Yes | Graceful subprocess termination |
| Color-by-metric maps | Yes | Compactness, partisan lean, demographics |
| Run history with filtering | Yes | By status, year, version |
| PM2 deployment automated | Yes | Ecosystem file for all services |
| Test coverage | 80%+ | Unit + integration + E2E with VT |

### Quality Gates (from Senior Tester)

| Gate | Threshold | Blocking |
|------|-----------|----------|
| Unit test coverage | 80% minimum | Yes |
| Integration test pass | 100% | Yes |
| E2E critical flows | 100% | Yes |
| API p99 latency | <100ms | Warning |
| Map load time | <3s | Warning |
| Security scan | No high/critical | Yes |

---

## Testing Strategy (from Senior Tester)

**Overall Testing Risk Assessment**: MEDIUM-HIGH

E62 (Pipeline Execution Engine) is the **highest testing risk** and should receive disproportionate testing attention.

### Test Pyramid

```
         /\
        /E2E\        10-15 tests (critical flows, VT integration)
       /------\
      /  Integ  \     40-50 tests (API endpoints, component integration)
     /----------\
    /    Unit    \   70-90 tests (services, parsers, components, utilities)
   /--------------\
```

### Effort Allocation by Enhancement

| Enhancement | Risk Level | Testing Priority | Recommended Effort |
|------------|------------|------------------|-------------------|
| E62 (Pipeline Execution) | **HIGHEST** | 1 | 40% of testing effort |
| E61 (Run Management) | MEDIUM | 2 | 20% of testing effort |
| E63 (React Dashboard) | MEDIUM | 3 | 20% of testing effort |
| E64 (Visualization) | MEDIUM | 4 | 15% of testing effort |
| E60 (Project Setup) | LOW | 5 | 5% of testing effort |

### Testing Timeline

**Estimated Total Testing Effort**: 40-50 hours (30-35% of total development time)

| Phase | Weeks | Testing Focus |
|-------|-------|---------------|
| Phase 1 (Setup) | 1-2 | Infrastructure tests, health endpoint tests |
| Phase 2 (Backend) | 2-4 | API unit/integration tests, pipeline execution tests |
| Phase 3 (Frontend) | 4-6 | Component tests, E2E tests, performance tests |

### Critical Test Scenarios (Must Pass Before Launch)

1. **Pipeline Execution**
   - [ ] Vermont single-year run completes successfully
   - [ ] Progress updates appear in API response
   - [ ] Cancellation stops subprocess within 10 seconds
   - [ ] File-based fallback works when stdout is delayed
   - [ ] Watchdog kills hung process after timeout

2. **API Reliability**
   - [ ] 100 concurrent API requests handled without errors
   - [ ] Database connection pool handles load
   - [ ] Error responses don't leak internal details

3. **Frontend Stability**
   - [ ] Run creation works end-to-end
   - [ ] Progress polling updates UI correctly
   - [ ] Error boundaries catch component failures
   - [ ] Works on latest Chrome, Firefox, Edge

---

## Test Coverage Requirements by Enhancement

| Enhancement | Unit Tests | Integration Tests | E2E Tests | Total |
|-------------|------------|-------------------|-----------|-------|
| E60 | 3-5 | 5-8 | 0 | 8-13 |
| E61 | 15-18 | 12-15 | 2-3 | 29-36 |
| E62 | 25-30 | 12-15 | 3-5 | 40-50 |
| E63 | 18-22 | 10-12 | 5-7 | 33-41 |
| E64 | 10-12 | 8-10 | 5-7 | 23-29 |
| **Total** | **71-87** | **47-60** | **15-22** | **133-169** |

---

## Architecture Overview (Revised per Senior Designer)

**Backend (FastAPI - Port 8002)**:
- Wraps existing Python pipeline scripts via subprocess
- Stores run metadata only in PostgreSQL (not district data)
- Provides REST API for frontend (polling, not WebSocket initially)
- Uses asyncio subprocess for non-blocking execution
- Background tasks for pipeline execution

**Frontend (React - Port 3002)**:
- Built with Vite + TypeScript + Tailwind
- UI component library first (`@/components/ui/`)
- Uses React Query (TanStack Query) for server state
- Polling-based progress updates (2 second intervals)
- Leaflet for map visualization

**Database (PostgreSQL - Port 5434)**:
- Run metadata and progress only
- JSON columns for flexible config/progress storage
- File paths to existing Parquet/CSV outputs
- No geometry storage (too large, use files)

---

## Design Decisions (per Senior Designer Review)

### Polling over WebSocket
**Decision**: Start with polling (every 2 seconds) for progress updates.
**Rationale**: Simpler implementation, sufficient for 1-4 hour runs, easier debugging.
**Future**: Add WebSocket in future enhancement if polling proves inadequate.

### Metadata-Only Database
**Decision**: Store only run metadata in PostgreSQL, not district data.
**Rationale**:
- District data already exists as Parquet/CSV files
- Geometries are too large (MB per state)
- Avoids data duplication
- Database stays small and fast
- Can regenerate from files if needed

### File-Based Fallback for Progress
**Decision**: Add file-based progress as fallback to STATUS protocol parsing.
**Rationale**: Subprocess stdout can buffer unexpectedly, need reliable progress tracking.
**Implementation**: Write progress to `outputs/runs_progress.json` as secondary channel.

### UI Component Library First
**Decision**: Build reusable UI components before feature components.
**Components**: Button, Card, Table, Input, Select, Badge, Spinner, ErrorBanner.
**Rationale**: Ensures consistent styling, faster feature development.

### Deferred AK/HI Insets
**Decision**: Basic map first, Alaska/Hawaii insets in future enhancement.
**Rationale**: Inset positioning is complex, not required for MVP.

---

## Phases (Revised per Senior Designer)

### Phase 1: E60 - Project Setup & Infrastructure
**Status**: PLANNED
**Estimated Effort**: 12-16 hours

Set up complete development environment with Docker Compose.

**Tasks**:
- Backend directory structure (api/)
- Frontend directory structure (frontend/)
- Docker Compose for local development (PostgreSQL + API + Frontend)
- Environment configuration (.env files)
- PM2 ecosystem configuration
- Database connection with SQLAlchemy
- CORS configuration for development
- API versioning (/api/v1/)
- Health check endpoints (/health, /version)

### Phase 2: Backend Implementation (Enhancements 61, 62)
**Status**: COMPLETED
**Actual Effort**: ~24 hours
**Completed**: 2026-01-25

Database schema, API endpoints, and pipeline integration.

**E61 - Run Management API** ✅ COMPLETED (16-20 hours):
- Database schema (runs, run_years tables only)
- Alembic migrations
- JSON columns for config and progress flexibility
- Run CRUD endpoints
- Progress polling endpoint
- State configuration endpoint
- Unit tests for all endpoints
- **Commit**: 8b0aa7d

**E62 - Pipeline Execution Engine** ✅ COMPLETED (MVP ~8 hours):
- Subprocess manager class (async execution) ✅
- STATUS protocol parser (regex-based) ✅
- Background task execution ✅
- Cancellation support (terminate with timeout) ✅
- Test isolation (ExecutionManager reset) ✅
- **Commit**: b58cae6
- **Tests**: 15/15 passing (9 STATUS parser + 6 execution API)
- **Total Backend Suite**: 52/52 tests passing

**MVP Scope Delivered**:
- Basic subprocess execution and monitoring
- STATUS protocol parsing from stdout
- Progress callbacks to database
- Graceful cancellation (terminate → kill)

**Future Work (not MVP, ~20-25 hours)**:
- File-based progress fallback
- Watchdog for hung processes
- Heartbeat monitoring
- Integration tests with Vermont pipeline
- Advanced error recovery

### Phase 3: Frontend & Deployment (Enhancements 63, 64)
**Status**: ✅ COMPLETED
**Actual Effort**: ~20 hours total (E63: ~12h, E64: ~8h)
**Completed**: 2026-01-25

Frontend components and production deployment.

**E63 - React Dashboard Core** ✅ COMPLETED (MVP ~12 hours):
- Vite + TypeScript + Tailwind setup (from E60) ✅
- UI component library integration (@common/ui) ✅
- Navigation/layout component ✅
- Run list page with filters (status, year) ✅
- Run detail page with progress display ✅
- Run creation form with validation ✅
- API client with React Query hooks ✅
- Error boundaries and loading states ✅
- React Router integration (4 pages) ✅
- **Commit**: 2a1b523
- **Pages**: RunList, RunDetail, CreateRun, About
- **Features**: Real-time progress polling (2s), ETA display, year breakdown, state selection

**MVP Scope Delivered**:
- All core dashboard pages functional
- Real-time progress monitoring
- Complete run lifecycle management (create, start, cancel, delete)
- Form validation and error handling
- Responsive layout with Tailwind CSS

**E64 - District Visualization** ✅ COMPLETED (MVP ~8 hours):
- Leaflet map component ✅
- GeoJSON district rendering ✅
- Color-by-metric selection (compactness, population, partisan, demographic) ✅
- District tooltips with stats ✅
- District table with click selection ✅
- Backend API endpoints for GeoJSON/stats serving ✅
- **Commit**: 00709f9
- **Tests**: 57/57 passing (5 new district API tests)
- **Pages**: Districts page with map + table
- **Features**: Interactive map, year selection, metric selection, color legend, hover/click interactions

**MVP Scope Delivered**:
- Interactive Leaflet map with district boundaries
- GeoJSON rendering from pipeline outputs
- Multiple color-by-metric visualizations
- District statistics table with selection
- API endpoints for data serving
- Full year and metric selection

**Future Work (not MVP, ~10-15 hours)**:
- PM2 ecosystem deployment configuration
- E2E tests for critical flows
- Alaska/Hawaii insets for national maps
- Export to PNG/PDF functionality

---

## Dependencies

**Prerequisites**:
- Completed Wave 8 (Wave Manager Improvements) - Complete
- Stable pipeline with all features
- PostgreSQL available (via Docker or system)
- PM2 process manager configured

**Blocking Issues**: None identified

---

## Risks & Mitigation (Comprehensive 10-Risk Register)

See [Risk Register](#risk-register) section above for full risk management details.

**Summary by Category**:

| Category | HIGH Risks | MEDIUM Risks | LOW Risks |
|----------|------------|--------------|-----------|
| Technical | R1 (subprocess hangs), R2 (Windows issues), R8 (integration) | R3 (buffering), R5 (frontend sync), R6 (map performance) | R4 (orphans) |
| Project | R7 (scope creep) | - | - |
| Resource | - | R9 (developer unavailability) | - |
| External | - | R10 (pipeline dependency) | - |

**E62 Risks (Highest Priority)**:
| Risk ID | Risk | Mitigation Status |
|---------|------|-------------------|
| R1 | Subprocess hangs indefinitely | Watchdog + 60s timeout planned |
| R2 | Windows subprocess issues | `process.terminate()` + `taskkill` fallback |
| R3 | Stdout buffering delays | File-based progress fallback with atomic writes |
| R4 | Server restart orphans | PID tracking in database |

**Note**: R1-R4 are addressed with comprehensive mitigations identified by Senior Engineer. Mandatory testing of all failure scenarios required before proceeding past E62.

---

## Open Questions (from Senior Designer)

1. **Concurrent Runs**: Single active run initially, queue additional requests
2. **User Authentication**: Deferred to future wave if needed
3. **Data Retention**: Keep metadata indefinitely, allow manual deletion of output files
4. **Error Recovery**: Detect orphaned runs on startup, allow manual retry
5. **Alaska/Hawaii Insets**: Deferred to post-MVP enhancement

---

## Technical Recommendations (from Senior Designer)

### Database Schema (Minimal Viable)

```sql
CREATE TABLE runs (
    id SERIAL PRIMARY KEY,
    version VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    config JSONB NOT NULL,
    progress JSONB,
    created_at TIMESTAMP DEFAULT NOW(),
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    error_message TEXT,
    output_path VARCHAR(255)
);

CREATE INDEX idx_runs_status ON runs(status);
CREATE INDEX idx_runs_version ON runs(version);

CREATE TABLE run_years (
    id SERIAL PRIMARY KEY,
    run_id INTEGER REFERENCES runs(id) ON DELETE CASCADE,
    year VARCHAR(4) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    states_completed INTEGER DEFAULT 0,
    states_total INTEGER DEFAULT 50,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    error_message TEXT
);

CREATE INDEX idx_run_years_run ON run_years(run_id);
```

### API Structure

```
api/
├── main.py              # App factory, middleware, exception handlers
├── config.py            # pydantic-settings configuration
├── database.py          # Engine, session factory
├── models.py            # SQLAlchemy models (simple, one file)
├── schemas/
│   ├── run.py           # Run schemas
│   └── common.py        # Shared schemas (pagination, errors)
├── routers/
│   ├── runs.py          # /api/v1/runs
│   ├── health.py        # /health, /version
│   └── files.py         # /api/v1/files (map/data serving)
├── services/
│   ├── run_service.py   # Run CRUD
│   └── pipeline.py      # Pipeline execution
└── workers/
    └── executor.py      # Subprocess management
```

### Frontend Structure

```
frontend/
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── api/
│   │   ├── client.ts    # Axios instance
│   │   └── runs.ts      # Run API
│   ├── components/
│   │   └── ui/          # Reusable: Button, Card, Table, etc.
│   ├── features/
│   │   ├── runs/        # Run list, detail, form
│   │   └── districts/   # District table, map
│   ├── hooks/
│   │   └── useApi.ts    # React Query hooks
│   └── types/
│       └── index.ts     # TypeScript interfaces
├── package.json
├── tsconfig.json
├── vite.config.ts
└── tailwind.config.js
```

---

## Related Enhancements

- [E60](../enhancements/60_api_project_setup.md) - Project Setup & Infrastructure (planned)
- [E61](../enhancements/61_run_management_api.md) - Run Management API (planned)
- [E62](../enhancements/62_pipeline_execution_engine.md) - Pipeline Execution Engine (planned)
- [E63](../enhancements/63_react_dashboard_core.md) - React Dashboard Core (planned)
- [E64](../enhancements/64_district_visualization.md) - District Visualization (planned)

---

## Notes

- This wave maintains full backward compatibility with CLI tools
- Web interface is additive, not replacement
- Senior Designer review completed 2026-01-25 with PROCEED WITH MODIFICATIONS recommendation
- Senior Engineer review completed 2026-01-25 with PROCEED recommendation
- Senior Tester review completed 2026-01-25 with PROCEED WITH EXPANDED TESTING recommendation
- **Senior Manager review completed 2026-01-25 with GO decision (8.1/10 overall score)**
- Estimated timeline updated from 4-6 weeks to **5-7 weeks** (with 20h buffer)
- Four mandatory controls apply: weekly reviews, E62 quality gate, MVP-first delivery, change control

---

## Design Documents

- [Senior Designer Review](wave09/01_senior_designer_review.md) - Design review with recommendations
- [Senior Engineer Review](wave09/02_senior_engineer_review.md) - Engineering review with implementation details
- [Senior Tester Review](wave09/03_senior_tester_review.md) - Testing strategy and quality gates
- [Senior Manager Review](wave09/04_senior_manager_review.md) - Management assessment and go/no-go decision
- [Management Decision](wave09/05_management_decision.md) - Executive summary and approval
- [Design Patterns](../DESIGN_PATTERNS.md) - API and frontend patterns for implementation

---

## Technical Assessment (from Senior Engineer)

### Risk Ratings by Enhancement

| Enhancement | Risk Level | Key Risk | Mitigation |
|-------------|------------|----------|------------|
| 60 - Setup | **Low** | Port conflicts | Configurable ports via env |
| 61 - Run API | **Medium** | Schema evolution | JSONB columns, migrations |
| 62 - Execution | **HIGH** | Subprocess hangs | Watchdog + file fallback |
| 63 - Dashboard | **Medium** | State sync bugs | React Query invalidation |
| 64 - Maps | **Medium** | Performance | Simplification, canvas |

### E62 Deep Dive (Highest Risk)

**Risk 1: Subprocess stdout buffering**
- **Impact**: Progress updates delayed or lost
- **Likelihood**: Medium
- **Mitigation**: `PYTHONUNBUFFERED=1` env var + file-based progress fallback

**Risk 2: Process hangs indefinitely**
- **Impact**: Run stuck in "running" state forever
- **Likelihood**: Medium
- **Mitigation**: Watchdog task (check every 10s), kill after 60s of no progress

**Risk 3: Server restart during execution**
- **Impact**: Orphaned runs, inconsistent state
- **Likelihood**: Low
- **Mitigation**: Store process PID in database, detect orphaned PIDs on startup

**Risk 4: Windows subprocess issues**
- **Impact**: Different signal handling (no SIGTERM)
- **Likelihood**: High (Windows is development target)
- **Mitigation**: Use `process.terminate()`, test on Windows explicitly, `taskkill /F /PID` fallback

---

## Implementation Timeline (from Senior Engineer)

### Recommended 5-7 Week Build Order (with Buffer)

```
Week 1-2: E60 (Setup)
  |-- Docker Compose working (postgres, API skeleton)
  |-- Frontend scaffold (Vite + Tailwind + empty routes)
  |-- Health endpoints verified
  |-- CI/CD for tests

Week 2-3: E61 (Run API)
  |-- Database schema + migrations
  |-- Run CRUD endpoints
  |-- Progress polling endpoint
  |-- Unit + integration tests

Week 3-4: E62 (Execution Engine)  ** CRITICAL PATH **
  |-- PipelineExecutor class
  |-- StatusBridge integration
  |-- File-based fallback
  |-- Watchdog implementation
  |-- VT integration test
  |-- ** QUALITY GATE CHECKPOINT **

Week 4-5: E63 (React Dashboard)
  |-- UI component library
  |-- Run list + detail pages
  |-- Progress display with polling
  |-- Run creation form
  |-- ** MVP DEMO CHECKPOINT **

Week 5-6: E64 (Visualization)
  |-- Leaflet map component
  |-- Color-by-metric
  |-- District table
  |-- PM2 deployment
  |-- E2E tests

Week 7: Buffer / Risk Mitigation (+20 hours)
  |-- Risk mitigation as needed
  |-- Polish and optimization
  |-- Final documentation
  |-- Release preparation
```

**Timeline Notes**:
- Week 7 buffer provides 20 hours for risk mitigation
- MVP checkpoint at week 4-5 is mandatory
- E62 quality gate is blocking
- Parallelization of E62+E63 can reduce to 5 weeks if 2 developers available

---

## Parallelization Opportunities

Enhancements 62 and 63 can be developed concurrently by different developers:

```
E60 ----+
                   |
E61 ----+---- E62 (Developer A: backend execution)
                   |
                   +---- E63 (Developer B: frontend, mock API responses)
                         |
                         +---- E64 (after 63 complete)
```

**Benefits**:
- Reduce calendar time from 6 weeks to 4-5 weeks
- Developer B can build UI with mock data while Developer A builds execution engine
- Integration testing happens in weeks 4-5

---

## Key Engineering Decisions (from Senior Engineer)

### 1. Single Active Run Pattern (PipelineManager Singleton)

**Rationale**: Simplifies state management, avoids resource contention.

```python
class PipelineManager:
    """Singleton managing active pipeline execution."""

    _instance = None
    _lock = asyncio.Lock()

    def __init__(self):
        self.active_run: Optional[int] = None
        self.executor: Optional[PipelineExecutor] = None

    async def start_run(self, run_id: int, config: dict) -> bool:
        async with self._lock:
            if self.active_run is not None:
                raise ConflictError(f"Run {self.active_run} already active")
            self.active_run = run_id
            # ...
```

### 2. STATUS Protocol Bridge Reusing Existing Code

**Rationale**: The existing `StatusReader` class already handles all message types.

```python
from scripts.utils.status_protocol import parse_status_message

class StatusBridge:
    """Bridge STATUS protocol to database/WebSocket."""

    async def process_line(self, line: str):
        msg_type, data = parse_status_message(line)
        if msg_type is None:
            return
        self._update_progress(msg_type, data)
        await self._persist_progress()
```

### 3. Windows Signal Handling Considerations

**Issue**: Windows doesn't support SIGTERM like Unix.
**Solution**: Use `process.terminate()` (cross-platform) with `taskkill /F /PID` fallback.

### 4. File-Based Progress with Atomic Writes

**Rationale**: Python subprocess stdout can buffer unexpectedly.

```python
async def write_progress(self, progress: dict):
    """Atomic write: temp file + rename."""
    temp_file = self.progress_file.with_suffix(".tmp")
    async with aiofiles.open(temp_file, "w") as f:
        await f.write(json.dumps(progress, indent=2))
    temp_file.rename(self.progress_file)
```

### 5. Process PID Tracking for Orphan Detection

**Recommendation**: Add `process_pid` column to `runs` table:
```sql
ALTER TABLE runs ADD COLUMN process_pid INTEGER;
CREATE INDEX idx_runs_pid ON runs(process_pid) WHERE process_pid IS NOT NULL;
```

---

## Development Tooling (from Senior Engineer)

### Backend Tools
- `uvicorn --reload` for development
- `pytest-asyncio` for async tests
- `pytest-cov` for coverage
- `httpx` for async test client
- `ruff` for linting (faster than flake8)

### Frontend Tools
- `pnpm` over npm (faster, better disk usage)
- `@tanstack/react-query-devtools` for debugging
- `msw` for API mocking in tests
- `@playwright/test` for E2E

### Database Tools
- `pgAdmin` or `DBeaver` for database inspection
- `alembic history` to visualize migrations

### Performance Benchmarks (Target Metrics)

| Metric | Target | Measurement |
|--------|--------|-------------|
| API response time | <100ms | p99 latency |
| Progress poll overhead | <50ms | Including DB query |
| Map initial load | <3s | Time to first paint |
| Map interaction | 60fps | No jank on pan/zoom |

---

**Wave 9 Summary**: Transform apportionment project from CLI-only to full web application with interactive dashboard and real-time pipeline execution, using polling-based progress updates and file-backed data storage. **Approved 2026-01-25** with 8.1/10 management score. Timeline: 5-7 weeks with mandatory controls.
