# Improvement Plan: U.20 Plan Audit Certificates

**Paper**: `U.20+plan-audit-certificates`  
**Round**: 1 to 2  
**Date**: 2026-05-11  
**Source**: `reviews/SYNTHESIS.md`

> **Purpose**: This plan is based on AI-generated quality assessment feedback.
> Use it to strengthen the work, not as real reviewer response.

## Summary

The simulated panel found that U.20 is the correct fixed-point capstone but
needed more explicit artifact ownership, canonical-order invariants,
profile-scoped verifier semantics, and failure-mode evidence before it can
serve as the portfolio integration paper.

## P1: Critical Enhancements

### P1.1: Artifact ownership and invariants
**Target sections**: `sections/schema.tex`  
**Tasks**:
- [x] Define RPLAN, RCTX, certificate, and manifest ownership.
- [x] Add canonical-order, hash, profile, and lineage invariants.

### P1.2: Verifier semantics
**Target sections**: `sections/audit.tex`, abstract  
**Tasks**:
- [x] State that certificates verify declared profiles and package integrity.
- [x] State that certificates do not settle fairness, neutrality, or legal sufficiency.

### P1.3: Evidence ladder and failure modes
**Target sections**: `sections/evaluation.tex`, `sections/limitations.tex`  
**Tasks**:
- [x] Add a claim/evidence/publication-gap table.
- [x] Mark public examples and tamper/failure-mode catalog as publication gaps.

## P2: Substantial Enhancements

- [x] Add a concrete manifest and certificate snippet.
- [x] Add tamper tests for unit-order mismatch, hash mismatch, and profile mismatch.
- [ ] Archive public examples from construction, exact, local-search, and Pareto outputs.

## Quality Checkpoints

- [x] P1 items addressed.
- [x] Paper builds without errors.
- [x] Claims map to evidence level.
- [ ] Ready for round-2 simulated assessment.
