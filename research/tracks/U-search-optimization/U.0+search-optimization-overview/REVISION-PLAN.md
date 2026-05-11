# Improvement Plan: U.0 Search and Optimization Overview

**Paper**: `U.0+search-optimization-overview`  
**Round**: 1 to 2  
**Date**: 2026-05-11  
**Source**: `reviews/SYNTHESIS.md`

> **Purpose**: This plan is based on AI-generated quality assessment feedback.
> Use it to strengthen the work, not as real reviewer response.

## Summary

The simulated panel found that U.0 is the correct entry point for the U-series
but needs visible taxonomy, reusable evidence categories, and stronger
RPLAN/RCTX audit-limit language before it can guide later papers.

## P1: Critical Enhancements

### P1.1: U-series role map
**Target sections**: `sections/taxonomy.tex`  
**Tasks**:
- [x] Add a table mapping paper bands to role and primary claim type.
- [x] Explain why different algorithm roles should not be compared as if they
  prove the same kind of claim.

### P1.2: Evidence ladder
**Target sections**: `sections/evidence.tex`  
**Tasks**:
- [x] Add a claim-boundary table for fixture correctness, smoke tests,
  benchmarks, empirical sweeps, certificates, and legal interpretation.
- [x] State how staged implementation papers should mark future empirical work.

### P1.3: Fixed-point audit limits
**Target sections**: abstract, `sections/limitations.tex`  
**Tasks**:
- [x] State that RPLAN/RCTX artifacts support reproducibility and lineage, not
  political fairness or legal compliance.
- [x] Tie the limitation to declared profiles and recorded evidence.

## P2: Substantial Enhancements

- [ ] Add a shared U-series dependency figure after U.12 through U.15 settle
      their final vocabulary.
- [ ] Require every downstream U implementation paper to label current evidence
      and future-evidence placeholders.

## Quality Checkpoints

- [x] P1 items addressed.
- [x] Paper builds without errors.
- [x] Claims map to evidence level.
- [ ] Ready for round-2 simulated assessment.
