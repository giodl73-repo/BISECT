# Improvement Plan: U.13 Exact-vs-Heuristic Certification

**Paper**: `U.13+exact-vs-heuristic-certification`  
**Round**: 1 to 2  
**Date**: 2026-05-11  
**Source**: `reviews/SYNTHESIS.md`

> **Purpose**: This plan is based on AI-generated quality assessment feedback.
> Use it to strengthen the work, not as real reviewer response.

## Summary

The simulated panel found that U.13 has a necessary claim-discipline role but
needed sharper tables and rules so proof, empirical evidence, audit validation,
and legal interpretation remain separate.

## P1: Critical Enhancements

### P1.1: Certificate class table
**Target sections**: `sections/certificates.tex`  
**Tasks**:
- [x] Add artifact classes and their supported claims.
- [x] Add "does not establish alone" boundaries.

### P1.2: Required certificate nouns
**Target sections**: abstract, `sections/certificates.tex`  
**Tasks**:
- [x] Require model, instance, objective, verifier, and claim boundary.

### P1.3: Staged claim limits
**Target sections**: `sections/limitations.tex`, `sections/audit.tex`  
**Tasks**:
- [x] Mark U.16/U.17 exact certificates as implementation-paper contracts until
  solver evidence is documented.
- [x] Mark U.18/U.19 heuristic claims as validity and empirical claims, not
  dominance or exactness claims.

## P2: Substantial Enhancements

- [ ] Add concrete examples after U.16 through U.19 drafts settle.

## Quality Checkpoints

- [x] P1 items addressed.
- [x] Paper builds without errors.
- [x] Claims map to evidence level.
- [ ] Ready for round-2 simulated assessment.
