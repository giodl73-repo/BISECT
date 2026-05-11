# Improvement Plan: U.16 Branch-and-Cut Redistricting

**Paper**: `U.16+branch-and-cut-redistricting`  
**Round**: 1 to 2  
**Date**: 2026-05-11  
**Source**: `reviews/SYNTHESIS.md`

> **Purpose**: This plan is based on AI-generated quality assessment feedback.
> Use it to strengthen the work, not as real reviewer response.

## Summary

The simulated panel found that U.16 is a good exact-optimization implementation
contract but needed sharper status vocabulary, separation between solver proof
and RPLAN audit validity, and a visible claim/evidence ladder.

## P1: Critical Enhancements

### P1.1: Status vocabulary
**Target sections**: `sections/implementation.tex`  
**Tasks**:
- [x] Add formulation-only, cut-active, bounded, proven, and fallback statuses.
- [x] State that timeouts and fallbacks are first-class outputs.

### P1.2: Solver report vs audit package
**Target sections**: `sections/audit.tex`, abstract  
**Tasks**:
- [x] State that a valid RPLAN is not an optimality proof.
- [x] Require solver report and audit package to travel together.

### P1.3: Evidence ladder
**Target sections**: `sections/evaluation.tex`, `sections/limitations.tex`  
**Tasks**:
- [x] Add claim/evidence/publication-gap table.
- [x] Mark scalability and real-data exact performance as future benchmark work.

## P2: Substantial Enhancements

- [ ] Add concrete command transcripts and solver versions.
- [ ] Add a cut/separation report schema example.
- [ ] Archive a public small-instance exact package.

## Quality Checkpoints

- [x] P1 items addressed.
- [x] Paper builds without errors.
- [x] Claims map to evidence level.
- [ ] Ready for round-2 simulated assessment.
