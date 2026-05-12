# Improvement Plan: U.17 Branch-and-Price Redistricting

**Paper**: `U.17+branch-and-price-redistricting`  
**Round**: 1 to 2  
**Date**: 2026-05-11; updated 2026-05-12  
**Source**: `reviews/SYNTHESIS.md`

> **Purpose**: This plan is based on AI-generated quality assessment feedback.
> Use it to strengthen the work, not as real reviewer response.

## Summary

The simulated panel found that U.17 is correctly staged as a fixture-scale
column-generation contract but needs stronger status vocabulary, clearer
column-generation audit metadata, and explicit evidence-level boundaries.

## P1: Critical Enhancements

### P1.1: Status vocabulary
**Target sections**: `sections/implementation.tex`  
**Tasks**:
- [x] Add pricing-valid, pricing-failed, master-bounded, fixture-solved, and
  formulation-only statuses.
- [x] State that partial pools and formulation-only reports are not solved exact
  instances.

### P1.2: Column-generation evidence vs RPLAN assignment
**Target sections**: `sections/audit.tex`  
**Tasks**:
- [x] Require column pool, selected columns, master objective, and fixture id in
  audit package metadata.
- [x] Keep solver-specific evidence outside generic RPLAN assignment semantics.

### P1.3: Evidence ladder
**Target sections**: `sections/evaluation.tex`, `sections/limitations.tex`  
**Tasks**:
- [x] Add claim/evidence/publication-gap table.
- [x] Mark large-instance branch-price performance as future benchmark work.

## P2: Substantial Enhancements

- [x] Add solver logs, command transcripts, and column-pool examples.
- [x] Add larger formulation-only examples before performance comparison.
- [x] Tie exact fixture examples to archived RPLAN/RCTX packages.

## 2026-05-12 Paper-Quality Pass

- [x] Replaced draft framing with implementation-paper framing.
- [x] Positioned U.17 against U.16 branch-and-cut and U.20 fixed-point audit
      semantics.
- [x] Added the path4 column-generation example showing why a valid middle
      column can still be rejected by exact cover.
- [x] Added concrete branch-price report fields from \texttt{bisect-column}.
- [x] Added CLI/test/package evidence and kept large-instance performance out
      of scope.

## Quality Checkpoints

- [x] P1 items addressed.
- [x] Paper builds without errors.
- [x] Claims map to evidence level.
- [x] Ready for round-2 simulated assessment.
