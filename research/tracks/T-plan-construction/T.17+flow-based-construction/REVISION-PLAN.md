# Improvement Plan: T.17 Flow-Based Construction

**Paper**: `T.17+flow-based-construction`
**Round**: 1 to 2
**Date**: 2026-05-11
**Source**: `reviews/SYNTHESIS.md`

> **Purpose**: This plan is based on AI-generated quality assessment feedback. Use it to strengthen the work, not as real reviewer response.

## Summary

The simulated panel found a promising flow-family baseline with three P1
items: flow/status semantics, evidence separation, and audit contract limits.

## P1: Critical Enhancements

### P1.1: Flow model and status outputs
**Target sections**: `sections/method.tex`, `sections/implementation.tex`
**Tasks**:
- [ ] Add network components, capacities, costs, and assignment extraction.
- [ ] Add status taxonomy for valid, infeasible, and invalid outputs.

### P1.2: Evidence ladder and model-relative witnesses
**Target sections**: `sections/evaluation.tex`, `sections/limitations.tex`
**Tasks**:
- [ ] Add claim-evidence table.
- [ ] Add failure-behavior comparison.
- [ ] State that infeasibility witnesses are model/profile relative.

### P1.3: Audit contract and limits
**Target sections**: `sections/audit.tex`, abstract/conclusion
**Tasks**:
- [ ] Define sidecar fields and hash intent.
- [ ] State that sidecars do not certify fairness, legal sufficiency, or global optimality.

## P2: Substantial Enhancements

- [ ] Bridge to U.16/U.17 for exact optimization claims.
- [ ] Explain geographic modeling assumptions.

## Quality Checkpoints

- [x] P1 items addressed.
- [x] Paper builds without errors.
- [x] Claims map to evidence level.
- [x] Ready for round-2 simulated assessment.

## 2026-05-12 Paper-Quality Pass

- [x] Aligned status taxonomy and summary fields with \texttt{bisect-flow}.
- [x] Reframed the method as capacity-gated frontier growth, not a full
      min-cost-flow solver.
- [x] Added a worked frontier-decision table and model-relative witness
      language.
- [x] Marked the paper ready for another simulated assessment pass.
