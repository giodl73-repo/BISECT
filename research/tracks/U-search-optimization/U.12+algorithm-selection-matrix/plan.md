# U.12 - Algorithm-Selection Matrix

**Paper Type:** Practitioner methodology / taxonomy  
**Status:** Planning  
**Track:** U - Search and Optimization  
**Code Home:** `bisect-cli`, all algorithm-family crates  
**Primary Specs:** `docs/specs/2026-05-10-algorithm-family-roadmap.md`

## Research Question

Given many construction, search, exact, ensemble, and audit paths, how should a
practitioner select an algorithm family for a concrete redistricting task?

## Hypotheses / Claims

- **H1:** Algorithm choice can be expressed as a matrix over purpose, evidence needs, runtime budget, and audit requirements.
- **H2:** The CLI surfaces expose enough metadata to support reproducible selection.
- **H3:** Selection guidance can reduce misuse by naming what each method does not prove.

Falsification: ambiguous cases where multiple methods are recommended without a
decision criterion.

## Scope Boundary

- **In scope:** method-selection guidance, CLI examples, evidence tiers.
- **Out of scope:** claiming a universally best algorithm.
- **Generalizability claim:** the matrix applies to BISECT-supported workflows.

## Evaluation Plan

- Baselines: ad hoc algorithm choice, single-default workflows.
- Evidence: crate taxonomy, CLI surfaces, existing docs.
- Success criteria: each method has when-to-use, avoid-when, output, and audit columns.

## Figures and Tables

- Algorithm-selection matrix.
- Decision tree from task to CLI command.
- Output artifact comparison table.

## Limitations

- Selection depends on data availability and legal context.
- Empirical performance entries should be updated as benchmark sweeps mature.

## Panel Readiness Checklist

- [ ] `main.tex` and sections exist.
- [ ] Matrix covers T.14-T.17 and U.16-U.20.
- [ ] Limitations are visible in the table.
- [ ] P1 simulated feedback addressed.
