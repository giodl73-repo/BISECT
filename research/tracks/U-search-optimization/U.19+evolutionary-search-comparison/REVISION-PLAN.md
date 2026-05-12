# Improvement Plan: U.19 Evolutionary Search Comparison

**Paper**: `U.19+evolutionary-search-comparison`  
**Round**: 1 to 2  
**Date**: 2026-05-11; updated 2026-05-12  
**Source**: `reviews/SYNTHESIS.md`

> **Purpose**: This plan is based on AI-generated quality assessment feedback.
> Use it to strengthen the work, not as real reviewer response.

## Summary

The simulated panel found that U.19 has a strong selected-frontier audit
position but needed sharper boundaries between operator validity, frontier
quality, selected-entry auditability, and policy/legal preference claims.

## P1: Critical Enhancements

### P1.1: Status boundaries
**Target sections**: `sections/implementation.tex`  
**Tasks**:
- [x] Define operator-valid, operator-fallback, frontier-recorded, selected-packaged, and selection-deferred statuses.
- [x] State that invalid silent operator outputs falsify the implementation contract.

### P1.2: Determinism and lineage
**Target sections**: `sections/audit.tex`, abstract  
**Tasks**:
- [x] Require fixed seed, operator parameters, parent ids, generation id, and selected entry id.
- [x] Require selected packages to include RPLAN/RCTX/certificate/manifest sidecars.

### P1.3: Evidence ladder
**Target sections**: `sections/evaluation.tex`, `sections/limitations.tex`  
**Tasks**:
- [x] Add a claim/evidence/publication-gap table.
- [x] Separate frontier-quality evidence from U.14 selection records and U.15 posture boundaries.

## P2: Substantial Enhancements

- [x] Add a selected-frontier CLI transcript.
- [x] Add seed/objective sensitivity examples.
- [x] Archive a selected frontier package with manifest and verification transcript.

## 2026-05-12 Paper-Quality Pass

- [x] Replaced draft framing with implementation-paper framing.
- [x] Positioned U.19 against U.14 selection records, U.15 posture boundaries,
      and U.20 fixed-point audit semantics.
- [x] Added concrete frontier row, metadata row, selected package, lineage, and
      audit-certificate fields.
- [x] Added selected-index example showing why the chosen row is a trade-off,
      not a single-objective optimum.
- [x] Added CLI/package evidence and seed/objective/budget sensitivity caveats.

## Quality Checkpoints

- [x] P1 items addressed.
- [x] Paper builds without errors.
- [x] Claims map to evidence level.
- [x] Ready for round-2 simulated assessment.
