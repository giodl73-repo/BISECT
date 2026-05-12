# Improvement Plan: T.16 Hierarchical Regionalization

**Paper**: `T.16+hierarchical-regionalization`
**Round**: 1 to 2
**Date**: 2026-05-11
**Source**: `reviews/SYNTHESIS.md`

> **Purpose**: This plan is based on AI-generated quality assessment feedback. Use it to strengthen the work, not as real reviewer response.

## Summary

The simulated panel found a promising inspectable regionalization baseline with
three P1 items: merge/witness semantics, evidence separation, and audit-limit
framing.

## P1: Critical Enhancements

### P1.1: Merge and witness semantics
**Target sections**: `sections/method.tex`, `sections/implementation.tex`
**Tasks**:
- [ ] Add deterministic procedure for candidate merges, scoring, capacity checks, and tie breaking.
- [ ] Add a witness field table.

### P1.2: Evidence ladder and failure behavior
**Target sections**: `sections/evaluation.tex`, `sections/limitations.tex`
**Tasks**:
- [ ] Add claim-evidence table.
- [ ] Add baseline comparison by failure and witness behavior.

### P1.3: Audit contract and limits
**Target sections**: `sections/audit.tex`, abstract/conclusion
**Tasks**:
- [ ] State that merge lineage verifies provenance under a declared profile.
- [ ] State that it does not certify fairness, legal sufficiency, or optimality.

## P2: Substantial Enhancements

- [ ] Explain graph and connectedness assumptions.
- [ ] Reserve a merge-tree figure/table.

## Quality Checkpoints

- [x] P1 items addressed.
- [x] Paper builds without errors.
- [x] Claims map to evidence level.
- [x] Ready for round-2 simulated assessment.

## 2026-05-12 Paper-Quality Pass

- [x] Aligned merge scoring and witness fields with the implemented
      regionalization crate.
- [x] Added a worked merge-candidate table showing capacity eligibility before
      scoring.
- [x] Expanded implementation, audit, evidence, and limitation boundaries.
- [x] Marked the paper ready for another simulated assessment pass.
