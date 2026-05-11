# T.17 - Flow-Based Construction

**Paper Type:** Algorithm implementation + constructive flow baseline  
**Status:** Planning  
**Track:** T - Plan Construction  
**Code Home:** `bisect-flow`  
**CLI/YAML Surface:** `--structure flow-construction`, `structure: flow-construction`  
**Primary Specs:** `docs/specs/2026-05-11-t17-flow-construction.md`

## Research Question

Can a flow-style construction baseline produce auditable assignments while
making infeasibility and capacity behavior explicit?

## Hypotheses / Claims

- **H1:** The flow-construction baseline is deterministic on path fixtures.
- **H2:** Infeasible capacity inputs return structured witnesses.
- **H3:** Flow summaries can be converted into algorithm lineage for final-plan audit.

Falsification: nondeterministic fixture output, missing infeasibility witness,
or lineage that cannot be verified through the audit path.

## Scope Boundary

- **In scope:** small deterministic flow construction, capacity/cost summaries, infeasibility witnesses.
- **Out of scope:** industrial min-cost-flow scaling claims, global optimality, legal fairness.
- **Generalizability claim:** the paper establishes the audit-friendly shape of flow construction before larger solvers are introduced.

## Evaluation Plan

- Baselines: METIS, capacity-clustering, regionalization.
- Metrics: validity, edge cut, population deviation, capacity status, audit pass/fail.
- Evidence now: L0 capacity/infeasibility/determinism fixtures and L1 sidecar tests.
- Evidence needed: scale characterization and real-data smoke examples.

## Figures and Tables

- Flow network schematic.
- Infeasibility witness example.
- Fixture outcomes table.

## Limitations

- Current flow implementation is deliberately small.
- Capacity feasibility does not imply high-quality districts.
- Larger flow solvers may need separate dependency and certificate treatment.

## Panel Readiness Checklist

- [ ] `main.tex` and sections exist.
- [ ] Flow network and witness definitions are reproducible.
- [ ] Audit lineage path is shown.
- [ ] P1 simulated feedback addressed.
