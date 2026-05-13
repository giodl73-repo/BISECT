---
wave: pipeline-manager
date_open: 2026-01-17
date_close: 2026-01-26
status: archived
backfill: true
confidence: high
---

# Pipeline Manager

## Mission

Build a configurable multi-year pipeline manager, enhancement manager, and API
dashboard experiment, then remove obsolete management apps after consolidation.

## Evidence

Representative commits:

- `b19442f4` Enhancement 36 Phase 1: Define Configuration Schema and Writer
- `b002f38c` Implement Enhancement 36: Two-level configuration system and multi-year pipeline
- `311f62e7` Refactor pipeline scripts to eliminate code duplication
- `926047a4` Add pipeline orchestrator for unified subprocess management
- `6efe11a1` Create Wave 8 for wave manager improvements and renumber API Migration to Wave 9
- `0797039f` Enhancement 60: API Project Setup & Infrastructure
- `b148f619` Enhancement 64: District Visualization MVP
- `4afd1f5d` Remove obsolete wave-manager

## Tracks

- Configuration schema.
- Pipeline orchestration and stage tracking.
- Wave/enhancement manager UX.
- API/dashboard experiment.

## Established

- The first explicit wave vocabulary.
- Multi-stage pipeline status and management concepts.
- Lessons that static artifacts and Rust CLI surfaces were preferable to the
  temporary FastAPI/React manager for this repo.

## Carry Forward

The durable piece is the wave/pulse idea, now revived in `context/waves`.

