# Wave 9: API Migration

**Date**: 2026-01-25 (Planned)
**Focus**: Add FastAPI backend and React dashboard to existing pipeline without changing core functionality
**Status**: 📋 PLANNED
**Estimated Duration**: 4-6 weeks
**Enhancements**: 58, 59, 60, 61, 62
**Phases**:
- Phase 1: Enhancement 58 - Project Setup (📋 PLANNED)
- Phase 2: Enhancements 59, 60 - Backend API (📋 PLANNED)
- Phase 3: Enhancements 61, 62 - React Dashboard (📋 PLANNED)

---

## Goals

1. Create FastAPI backend wrapping existing Python pipeline
2. Set up PostgreSQL database for run metadata and results
3. Build React dashboard for running and viewing redistricting analyses
4. Integrate with App Manager system (port 8002 backend, 3002 frontend)
5. Enable web-based pipeline execution and monitoring
6. Provide interactive visualization of results
7. Maintain backward compatibility with existing CLI tools

---

## Success Metrics

| Metric | Target | Notes |
|--------|--------|-------|
| API endpoints | 20+ | Full CRUD for districts, runs, states |
| Database tables | 10+ | Runs, districts, states, configurations |
| React components | 30+ | Dashboard, map viewer, run manager |
| Pipeline execution | CLI + Web | Both interfaces supported |
| Result visualization | Interactive React | Replace static HTML |
| User experience | General users | Not just technical users |
| Deployment | PM2 automated | Same as other apps |

---

## Architecture Overview

**Backend (FastAPI - Port 8002)**:
- Wraps existing Python pipeline scripts
- Stores run metadata in PostgreSQL
- Provides REST API for frontend
- Handles async pipeline execution
- Manages result caching

**Frontend (React - Port 3002)**:
- Built with Vite + TypeScript + Tailwind
- Uses @common/ui components from appmanager
- Interactive district visualization
- Pipeline run management
- Real-time progress updates

**Database (PostgreSQL - Port 5434)**:
- Run history and metadata
- District data for quick access
- User preferences
- Cached results

---

## Phases

### Phase 1: Enhancement 58 - Project Setup & Dependencies
**Status**: 📋 PLANNED
**Estimated Effort**: 12-16 hours

Set up backend and frontend infrastructure with database connection.

**Tasks**:
- Backend directory structure (FastAPI, SQLAlchemy, Alembic)
- Frontend directory structure (React, Vite, TypeScript, Tailwind)
- PostgreSQL database connection
- PM2 ecosystem configuration
- Common package integration from appmanager

### Phase 2: Backend Implementation (Enhancements 59, 60)
**Status**: 📋 PLANNED
**Estimated Effort**: 42-50 hours

Backend infrastructure and database.

**Enhancement 58 - Project Setup & Dependencies** (📋 PLANNED):
- Backend directory structure (FastAPI, SQLAlchemy, Alembic)
- Frontend directory structure (React, Vite, TypeScript, Tailwind)
- PostgreSQL database connection
- PM2 ecosystem configuration
- Common package integration from appmanager

### Phase 2: Backend Implementation (Enhancements 59, 60)
**Status**: 📋 PLANNED
**Estimated Effort**: 30-36 hours

Database schema, API endpoints, and pipeline integration.

**Enhancement 59 - Database Schema & Core API Endpoints** (📋 PLANNED):
- Schema design (runs, states, districts, configurations)
- SQLAlchemy models
- Alembic migrations
- Seed data (state configs)
- Run management endpoints (create, list, get, delete)
- District data endpoints (by state, year, run)
- State configuration endpoints
- Pipeline execution endpoints

**Enhancement 60 - Pipeline Integration** (📋 PLANNED):
- Wrap pipeline scripts in async functions
- Subprocess execution handling
- WebSocket progress streaming
- Result storage in database
- Error handling and recovery

### Phase 3: Frontend & Deployment (Enhancements 61, 62)
**Status**: 📋 PLANNED
**Estimated Effort**: 30-36 hours

Frontend components and production deployment.

**Enhancement 61 - Frontend Core Components** (📋 PLANNED):
- Routing (run list, run detail, map viewer)
- Run management components
- District table views
- State/year filters
- Loading states and error handling

**Enhancement 62 - Interactive Map & Deployment** (📋 PLANNED):
- Mapping library integration (Leaflet)
- District boundary rendering
- Interactive tooltips (district stats)
- Color by metric (compactness, partisan lean)
- Zoom/pan controls
- Alaska/Hawaii inset positioning
- PM2 ecosystem configuration
- E2E tests for critical flows
- Integration tests for API
- Performance testing
- Complete API documentation
- User guide for web interface
- Update CLAUDE.md
- App Manager integration

**Original Enhancement 62 - Deployment, Testing & Documentation**: Content merged into Enhancement 62 above
- PM2 ecosystem configuration
- E2E tests for critical flows
- Integration tests for API
- Performance testing
- Complete API documentation
- User guide for web interface
- Update CLAUDE.md
- App Manager integration

---

## Dependencies

**Prerequisites**:
- Completed Wave 8 (Wave Manager Improvements) ✅
- Stable pipeline with all features
- PostgreSQL available (via Docker or system)
- PM2 process manager configured

**Blocking Issues**: None identified

---

## Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Pipeline async integration complex | High | Start with sync execution, add async later |
| Map rendering performance | Medium | Use geometry simplification, virtual scrolling |
| Database schema changes | Medium | Design schema carefully upfront, use migrations |
| WebSocket complexity | Medium | Fallback to polling if WebSockets fail |

---

## Related Enhancements

- [Enhancement 58](../enhancements/58_api_setup.md) - Project Setup & Dependencies (planned)
- [Enhancement 59](../enhancements/59_database_api.md) - Database Schema & Core API Endpoints (planned)
- [Enhancement 60](../enhancements/60_pipeline_integration.md) - Pipeline Integration (planned)
- [Enhancement 61](../enhancements/61_frontend_core.md) - Frontend Core Components (planned)
- [Enhancement 62](../enhancements/62_map_deployment.md) - Interactive Map Visualization & Deployment (planned)

---

## Notes

- This wave maintains full backward compatibility with CLI tools
- Web interface is additive, not replacement
- Can be executed at any time (no dependencies on future waves)
- Good candidate for "next wave" when ready for web interface

---

**Wave 9 Summary**: Transform apportionment project from CLI-only to full web application with interactive dashboard and real-time pipeline execution.
