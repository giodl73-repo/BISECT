# U.0 - Search and Optimization Overview

**Paper Type:** Track synthesis / taxonomy  
**Status:** Planning  
**Track:** U - Search and Optimization  
**Code Home:** all U-family crates and `bisect-cli` orchestration  
**Primary Concept:** `docs/concepts/algorithm-family-layer-cake.md`

## Research Question

How should the U-series organize search, optimization, certification, selection,
and audit methods after the RPLAN fixed point?

## Hypotheses / Claims

- **H1:** The U-series can be cleanly partitioned into search, exact optimization, multi-objective selection, and audit/certification roles.
- **H2:** The RPLAN audit fixed point makes heterogeneous algorithm families comparable at the artifact level.
- **H3:** A track overview reduces overclaiming by separating empirical, computational, and legal interpretations.

Falsification: taxonomy ambiguity that prevents mapping a paper or CLI surface
to one primary role.

## Scope Boundary

- **In scope:** U.1-U.20 taxonomy, CLI surfaces, evidence levels, paper dependencies.
- **Out of scope:** new empirical claims, new algorithm implementation.
- **Generalizability claim:** this is a portfolio guide for the BISECT U-track.

## Evaluation Plan

- Baselines: prior U.1-U.11 structure and the algorithm-family roadmap.
- Evidence: implemented CLI/crate surfaces and paper index.
- Success criteria: every U paper has a role, dependency, and evidence class.

## Figures and Tables

- U-track taxonomy tree.
- CLI-to-paper matrix.
- Evidence-level legend.

## Limitations

- Synthesis papers depend on the accuracy of underlying paper drafts.
- New methods may require future taxonomy revisions.

## Panel Readiness Checklist

- [ ] `main.tex` and sections exist.
- [ ] Taxonomy covers U.1-U.20.
- [ ] No empirical claims exceed available evidence.
- [ ] P1 simulated feedback addressed.
