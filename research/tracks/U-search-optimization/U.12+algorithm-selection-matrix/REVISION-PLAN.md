# Improvement Plan: U.12 Algorithm-Selection Matrix

**Paper**: `U.12+algorithm-selection-matrix`  
**Round**: 1 to 2  
**Date**: 2026-05-11  
**Source**: `reviews/SYNTHESIS.md`

> **Purpose**: This plan is based on AI-generated quality assessment feedback.
> Use it to strengthen the work, not as real reviewer response.

## Summary

The simulated panel found that U.12 has the right methodological scope but
needed a concrete matrix, reusable preflight questions, and stronger caveats
against treating workflow choice as a legal or algorithmic ranking.

## P1: Critical Enhancements

### P1.1: Selection matrix
**Target sections**: `sections/matrix.tex`  
**Tasks**:
- [x] Add method-family rows covering construction, search, exact,
  ensemble/SMC, Pareto selection, and audit verification.
- [x] Include evidence requirements and audit handoffs for each row.

### P1.2: Operator preflight
**Target sections**: `sections/matrix.tex`  
**Tasks**:
- [x] Add questions for claim type, evidence level, CLI/crate surface, and
  RPLAN/RCTX sidecars.

### P1.3: Claim boundaries
**Target sections**: abstract, `sections/audit.tex`, `sections/limitations.tex`  
**Tasks**:
- [x] State that the matrix is not a leaderboard or legal sufficiency test.
- [x] Explain that certificates do not make heuristic, distributional, or legal
  claims on their own.

## P2: Substantial Enhancements

- [ ] Add concrete command examples after U.16 through U.20 finalize CLI
      wording.
- [ ] Consider a shared U-series dependency figure during module-level review.

## Quality Checkpoints

- [x] P1 items addressed.
- [x] Paper builds without errors.
- [x] Claims map to evidence level.
- [ ] Ready for round-2 simulated assessment.
