# T.15 - Capacity-Constrained Clustering

**Paper Type:** Algorithm implementation + audit-oriented construction  
**Status:** Planning  
**Track:** T - Plan Construction  
**Code Home:** `bisect-clustering`  
**CLI/YAML Surface:** `--structure capacity-clustering`, `structure: capacity-clustering`  
**Primary Specs:** `docs/specs/2026-05-11-t15-capacity-clustering.md`

## Research Question

Can capacity-constrained clustering serve as a reproducible construction family
with explicit capacity status, repair status, and RPLAN audit lineage?

## Hypotheses / Claims

- **H1:** Farthest-point seeding and capacity assignment are deterministic on canonical fixtures.
- **H2:** Valid clustering outputs can be converted to RPLAN sidecars with verifiable audit certificates.
- **H3:** Infeasible capacity cases can return structured status rather than silent invalid plans.

Falsification: nondeterministic seeds, invalid sidecars, missing repair or
capacity witness data for infeasible cases.

## Scope Boundary

- **In scope:** capacity assignment, small repair, status summaries, audit sidecars.
- **Out of scope:** k-means optimality claims, large-scale clustering superiority, legal compliance beyond audit profile.
- **Generalizability claim:** the implementation establishes a staged audited baseline; quality claims require broader state sweeps.

## Evaluation Plan

- Baselines: METIS construction, spectral construction, regionalization.
- Metrics: capacity status, contiguity, population deviation, edge cut, repair status.
- Evidence now: L0 capacity/determinism fixtures and L1 RPLAN audit package tests.
- Evidence needed: real-data feasibility rates and quality comparison.

## Figures and Tables

- Seed placement on path and two-clique fixtures.
- Capacity status and repair flow.
- Table of fixture outcomes.

## Limitations

- The current slice is a deterministic baseline, not a full clustering benchmark suite.
- Repair is deliberately small and auditable.
- Cluster compactness and community coherence require additional metrics.

## Panel Readiness Checklist

- [ ] `main.tex` and sections exist.
- [ ] Capacity and repair witnesses are defined clearly.
- [ ] Audit sidecar example included.
- [ ] P1 simulated feedback addressed.
