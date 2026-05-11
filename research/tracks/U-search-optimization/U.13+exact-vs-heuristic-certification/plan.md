# U.13 - Exact-vs-Heuristic Certification

**Paper Type:** Certification methodology  
**Status:** Planning  
**Track:** U - Search and Optimization  
**Code Home:** `bisect-ilp`, `bisect-column`, `rplan-*`, `bisect verify`

## Research Question

What is the difference between an exact proof, an optimization bound, a
heuristic validity audit, and a reproducibility certificate?

## Hypotheses / Claims

- **H1:** Exact solvers and heuristic constructors can share a final artifact format without sharing proof semantics.
- **H2:** RPLAN audit certificates verify declared constraints, not global optimality or political fairness.
- **H3:** Clear certification categories reduce legal and scientific overclaiming.

Falsification: certificate fields that blur proof status with audit validity.

## Scope Boundary

- **In scope:** proof/bound/audit taxonomy, ILP and column-generation examples, RPLAN certificates.
- **Out of scope:** proving all BISECT algorithms optimal.
- **Generalizability claim:** the taxonomy applies to BISECT paper and report language.

## Evaluation Plan

- Baselines: undifferentiated "verified" language.
- Evidence: U.16/U.17 solve reports, U.20 audit certificates, verifier tests.
- Success criteria: each artifact has explicit semantics and non-semantics.

## Figures and Tables

- Proof-vs-audit ladder.
- Artifact semantics table.
- Example certificate interpretation.

## Limitations

- Legal admissibility is jurisdiction-specific.
- Verification does not replace independent expert analysis.

## Panel Readiness Checklist

- [ ] `main.tex` and sections exist.
- [ ] Taxonomy distinguishes proof, bound, audit, empirical evidence.
- [ ] Examples use current artifact fields.
- [ ] P1 simulated feedback addressed.
