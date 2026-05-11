# Improvement Plan: U.18 Large-Neighborhood Search

**Paper**: `U.18+large-neighborhood-search`  
**Round**: 1 to 2  
**Date**: 2026-05-11  
**Source**: `reviews/SYNTHESIS.md`

> **Purpose**: This plan is based on AI-generated quality assessment feedback.
> Use it to strengthen the work, not as real reviewer response.

## Summary

The simulated panel found that U.18 is a useful heuristic-improvement contract
but needed a sharper distinction between implemented one-move behavior, rejected
or no-op outcomes, and future LNS/tabu claims. The paper also needed stronger
parent-child lineage requirements for audited descendant plans.

## P1: Critical Enhancements

### P1.1: Status vocabulary
**Target sections**: `sections/implementation.tex`  
**Tasks**:
- [x] Add improvement, no-op, rejected-move, and staged-method statuses.
- [x] State that rejected or no-improvement outcomes are reportable results.

### P1.2: Parent-child audit lineage
**Target sections**: `sections/audit.tex`, abstract  
**Tasks**:
- [x] Require each output path to identify parent plan, child artifact, method, and objective delta.
- [x] Separate validity checks from improvement claims.

### P1.3: Evidence ladder
**Target sections**: `sections/evaluation.tex`, `sections/limitations.tex`  
**Tasks**:
- [x] Add a claim/evidence/publication-gap table.
- [x] Mark full LNS/tabu comparisons and scaling claims as future benchmark work.

## P2: Substantial Enhancements

- [ ] Add command transcripts for improved, no-op, and staged-method fixtures.
- [ ] Add a move summary schema example with before/after district metrics.
- [ ] Archive a public descendant RPLAN/RCTX package for a fixture plan.

## Quality Checkpoints

- [x] P1 items addressed.
- [x] Paper builds without errors.
- [x] Claims map to evidence level.
- [ ] Ready for round-2 simulated assessment.
