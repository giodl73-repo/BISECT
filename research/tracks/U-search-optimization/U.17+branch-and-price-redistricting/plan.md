# U.17 - Branch-And-Price Redistricting

**Paper Type:** Exact optimization implementation  
**Status:** Planning  
**Track:** U - Search and Optimization  
**Code Home:** `bisect-column`, `bisect-cli::exact_cmd`  
**CLI Surface:** `bisect exact --method branch-and-price`

## Research Question

Can column generation be staged as a reproducible exact-optimization family with
clear pricing, master-problem, bounds, and selected-plan audit artifacts?

## Hypotheses / Claims

- **H1:** Pricing can generate connected balanced columns on canonical fixtures.
- **H2:** The small exact fixture path reports true partition objective and bound metadata.
- **H3:** Solved outputs can emit RPLAN/RCTX/audit certificate/manifest packages.

Falsification: disconnected columns, incorrect objective reporting, or missing
audit sidecars for solved outputs.

## Scope Boundary

- **In scope:** pricing/master contracts, formulation-only reports, small exact fixture solution.
- **Out of scope:** mature large-scale branch-price performance claims.
- **Generalizability claim:** the paper establishes an audit-ready column-generation contract.

## Evaluation Plan

- Baselines: branch-and-cut ILP, formulation-only report, heuristic construction.
- Evidence: L0 pricing/master tests and L1 exact package test.
- Evidence needed: column-pool visualization and larger formulation-only examples.

## Figures and Tables

- Column-generation loop.
- Pricing/master artifact schema.
- Exact fixture solution table.

## Limitations

- Current exact solving is intentionally fixture-scale.
- Branching strategy and large-instance pricing remain staged.

## Panel Readiness Checklist

- [ ] `main.tex` and sections exist.
- [ ] Pricing and master contracts are explicit.
- [ ] Audit package example included.
- [ ] P1 simulated feedback addressed.
