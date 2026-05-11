# Improvement Plan: T.14 Spectral Partitioning

**Paper**: `T.14+spectral-partitioning`  
**Round**: 1 to 2  
**Date**: 2026-05-11  
**Source**: `reviews/SYNTHESIS.md`  

> **Purpose**: This plan is based on AI-generated quality assessment feedback. Use it to strengthen the work, not as real reviewer response.

## Summary

The simulated panel found a promising draft with adequate average score
(2.6/4) but three P1 items before readiness: precise algorithm specification,
explicit evidence separation, and stronger audit-limit language.

## P1: Critical Enhancements

### P1.1: Reproducible spectral method
**Target sections**: `sections/method.tex`, `sections/implementation.tex`  
**Tasks**:
- [ ] Add an algorithm-style procedure for graph construction, Laplacian setup, Fiedler sweep, tie breaking, and recursive split targets.
- [ ] Add deterministic proof obligations for fixture runs.

### P1.2: Evidence ladder
**Target sections**: `sections/evaluation.tex`  
**Tasks**:
- [ ] Add a claim-evidence table distinguishing L0 fixture correctness, L1 audit checks, real-data smoke, and future empirical sweeps.
- [ ] Label METIS/GeoSection/clustering comparisons as future work until commands and outputs are recorded.

### P1.3: Audit claim limits
**Target sections**: `sections/audit.tex`, `sections/limitations.tex`, abstract/conclusion  
**Tasks**:
- [ ] State that RPLAN/RCTX sidecars verify declared profile and lineage, not fairness or legal compliance.
- [ ] Move the limitation into the paper's contribution framing, not only the audit details.

## P2: Substantial Enhancements

### P2.1: Portfolio positioning
**Target sections**: `sections/introduction.tex`, `sections/evaluation.tex`  
**Tasks**:
- [ ] Add a constructor comparison table covering spectral, METIS, GeoSection, capacity clustering, and regionalization.

### P2.2: Geographic assumptions and failure modes
**Target sections**: `sections/method.tex`, `sections/limitations.tex`  
**Tasks**:
- [ ] Add adjacency/weight/disconnected-component assumptions.
- [ ] Name weak-eigenvector and near-symmetry failure modes.

## P3: Refinements

- [ ] Add or reserve a recursive odd-k split figure.
- [ ] Use "deterministic baseline" consistently.

## Quality Checkpoints

- [x] P1 items addressed.
- [x] Paper builds without errors.
- [x] Claims map to evidence level.
- [ ] Ready for round-2 simulated assessment.
