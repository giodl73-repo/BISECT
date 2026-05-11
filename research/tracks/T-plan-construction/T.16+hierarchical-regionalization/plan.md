# T.16 - Hierarchical Regionalization

**Paper Type:** Algorithm implementation + regionalization construction  
**Status:** Planning  
**Track:** T - Plan Construction  
**Code Home:** `bisect-clustering::regionalization`  
**CLI/YAML Surface:** `--structure regionalization`, `structure: regionalization`  
**Primary Specs:** `docs/specs/2026-05-11-t16-hierarchical-regionalization.md`

## Research Question

Can deterministic agglomerative regionalization produce connected district
regions while emitting merge witnesses and audit-ready lineage?

## Hypotheses / Claims

- **H1:** The regionalization baseline produces deterministic merge logs on canonical fixtures.
- **H2:** Valid outputs preserve contiguity and capacity constraints on supported fixtures.
- **H3:** Merge witnesses make regionalization more explainable than opaque one-shot partitioning.

Falsification: unstable merge order, disconnected outputs, missing or
uninformative merge witnesses.

## Scope Boundary

- **In scope:** deterministic agglomeration, merge logs, regionalization summaries, CLI wiring.
- **Out of scope:** full SKATER/Max-p equivalence, claims of superior compactness without sweeps.
- **Generalizability claim:** the staged baseline demonstrates the audit contract for regionalization-family methods.

## Evaluation Plan

- Baselines: capacity-clustering, METIS, flow-construction.
- Metrics: merge count, connectedness, population deviation, edge cut, audit status.
- Evidence now: L0 hierarchy/capacity/determinism fixtures and lineage sidecars.
- Evidence needed: real-data comparison and visual examples.

## Figures and Tables

- Agglomerative merge sequence diagram.
- Merge-witness table.
- Fixture comparison table.

## Limitations

- The current algorithm is a deterministic baseline, not a full regionalization taxonomy.
- Quality depends on merge scoring and graph topology.
- Real-data claims need state-level sweeps.

## Panel Readiness Checklist

- [x] `main.tex` and sections exist.
- [ ] Merge witness semantics are precise.
- [ ] Evaluation separates fixtures from future empirical sweeps.
- [ ] P1 simulated feedback addressed.
