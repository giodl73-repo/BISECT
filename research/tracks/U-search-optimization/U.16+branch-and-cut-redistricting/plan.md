# U.16 - Branch-And-Cut Redistricting

**Paper Type:** Exact optimization implementation  
**Status:** Planning  
**Track:** U - Search and Optimization  
**Code Home:** `bisect-ilp`, `bisect-cli` runner  
**CLI/YAML Surface:** `--structure ilp --ilp-method branch-and-cut`

## Research Question

How should branch-and-cut redistricting be exposed so connectivity cuts, solver
status, fallback behavior, and audit lineage remain reproducible?

## Hypotheses / Claims

- **H1:** Connectivity-cut metadata can distinguish true branch-and-cut behavior from fallback or formulation-only paths.
- **H2:** Solver reports and audit summaries can be verified independently of the final RPLAN certificate.
- **H3:** Exact optimization claims require proof/bound metadata, not only a valid final plan.

Falsification: missing solve status, missing cut/separation evidence, or
ambiguous fallback reporting.

## Scope Boundary

- **In scope:** branch-and-cut lifecycle, solve reports, audit summary, lineage.
- **Out of scope:** claiming scalable exact solution for every state.
- **Generalizability claim:** the implementation is a baseline exact-optimization contract.

## Evaluation Plan

- Baselines: formulation-only ILP, heuristic METIS/fallback plan.
- Evidence: ILP solve reports, audit summary verifier, RPLAN audit certificate.
- Evidence needed: small real-data or synthetic exact example table.

## Figures and Tables

- Branch-and-cut lifecycle diagram.
- Cut/separation report schema.
- Proof-status table.

## Limitations

- Exact methods may be limited by instance scale.
- Valid final plans do not prove optimality without solver proof metadata.

## Panel Readiness Checklist

- [ ] `main.tex` and sections exist.
- [ ] Proof, fallback, and audit semantics are separated.
- [ ] Solver reports are reproducible.
- [ ] P1 simulated feedback addressed.
