# Improvement Plan: T.15 Capacity-Constrained Clustering

**Paper**: `T.15+capacity-constrained-clustering`
**Round**: 1 to 2
**Date**: 2026-05-11
**Source**: `reviews/SYNTHESIS.md`

> **Purpose**: This plan is based on AI-generated quality assessment feedback. Use it to strengthen the work, not as real reviewer response.

## Summary

The simulated panel found a useful audit-oriented baseline with three P1 items:
precise capacity/repair semantics, explicit evidence separation, and a clearer
audit contract.

## P1: Critical Enhancements

### P1.1: Capacity assignment and repair semantics
**Target sections**: `sections/method.tex`, `sections/implementation.tex`
**Tasks**:
- [ ] Add deterministic procedure for seeds, assignment, capacity slack, and tie breaking.
- [ ] Add a status taxonomy for valid, repaired, infeasible, and invalid outputs.

### P1.2: Evidence ladder and failure behavior
**Target sections**: `sections/evaluation.tex`, `sections/limitations.tex`
**Tasks**:
- [ ] Add claim-evidence table distinguishing L0 fixtures, L1 sidecars, smoke, and future sweeps.
- [ ] Add baseline comparison focused on failure/witness behavior.

### P1.3: Audit contract and limits
**Target sections**: `sections/audit.tex`, abstract/conclusion
**Tasks**:
- [ ] Define sidecar fields and parameter-hash intent.
- [ ] State that sidecars do not certify fairness, community preservation, or legal compliance.

## P2: Substantial Enhancements

- [ ] Explain graph, spatial, and attribute input assumptions.
- [ ] Mark community-character weights as optional or future work.

## Quality Checkpoints

- [x] P1 items addressed.
- [x] Paper builds without errors.
- [x] Claims map to evidence level.
- [x] Ready for round-2 simulated assessment.

## 2026-05-12 Paper-Quality Pass

- [x] Aligned paper terminology with implemented statuses:
      \texttt{valid}, \texttt{needs-repair}, and
      \texttt{infeasible-capacity}.
- [x] Added a worked capacity-decision table showing candidate rejection by
      population slack.
- [x] Expanded implementation, audit, evidence, and repair-limit language.
- [x] Marked the paper ready for another simulated assessment pass.
