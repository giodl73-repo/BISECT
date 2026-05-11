# T.14 - Spectral Partitioning

**Paper Type:** Algorithm implementation + deterministic construction baseline  
**Status:** Planning  
**Track:** T - Plan Construction  
**Code Home:** `bisect-apportion::spectral`, `bisect-cli` runner  
**CLI/YAML Surface:** `--structure spectral`, `structure: spectral`  
**Primary Specs:** `docs/specs/2026-05-11-t14-spectral-partitioning.md`, `docs/specs/2026-05-10-algorithm-family-roadmap.md`

## Research Question

Can a deterministic spectral split provide a cheap, auditable construction
baseline that complements METIS-family bisection without making stronger
optimality claims than the implementation supports?

## Hypotheses / Claims

- **H1:** On canonical synthetic graph fixtures, the Fiedler-vector sweep cut is deterministic and preserves population-balance constraints.
- **H2:** Recursive spectral construction can handle odd and non-power-of-two district counts when proportional target fractions are passed through the recursion.
- **H3:** Spectral construction is best presented as a transparent baseline, not as a replacement for METIS or exact optimization.

Falsification: nondeterministic fixture output, population-balance failures on
stated fixtures, or recursive splits that mishandle uneven district counts.

## Scope Boundary

- **In scope:** deterministic graph partitioning, synthetic fixtures, CLI integration, RPLAN audit sidecars.
- **Out of scope:** claims of global optimality, large-scale performance superiority, legal neutrality.
- **Generalizability claim:** results apply to construction baselines over tract adjacency graphs; empirical quality claims require real-data sweeps.

## Evaluation Plan

- Baselines: standard METIS bisection, GeoSection, and capacity-clustering where available.
- Metrics: validity, determinism, edge cut, population deviation, audit pass/fail.
- Evidence now: L0 path/two-clique/determinism tests and recursive split hardening.
- Evidence needed: real-data smoke table and runtime/quality comparison.

## Figures and Tables

- Fiedler-vector sweep-cut diagram.
- Recursive split tree for odd `k`.
- Table comparing spectral, METIS, and GeoSection on fixture and smoke cases.

## Limitations

- Spectral cuts are relaxations, not exact districting optima.
- Quality depends on graph structure and available population weights.
- The paper should not infer political fairness from deterministic construction.

## Panel Readiness Checklist

- [x] `main.tex` and sections exist.
- [ ] Related work covers spectral graph partitioning and redistricting baselines.
- [ ] Evaluation distinguishes fixture correctness from empirical sweeps.
- [ ] P1 simulated feedback addressed.
