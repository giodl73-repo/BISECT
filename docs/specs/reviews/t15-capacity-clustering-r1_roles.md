---
reviewer: ROLE PANEL
roles: BOUNDARY, WARD, COVENANT, CONTOUR, MERIDIAN, BENCHMARK, SCALE, PRECINCT, DATUM, COMMONS, LEDGER, SURVEY, TRENCH
spec: T.15 Capacity-Constrained Clustering
round: 1
date: 2026-05-11
score: 3.4
---

# Role Review: T.15 Capacity-Constrained Clustering

## Summary

The T.15 staging spec is implementation-ready for a crate-first Stage 1. It
keeps clustering out of `bisect-cli` until the kernel has deterministic fixture
coverage, makes U.20 audit lineage the eventual certification path, and avoids
claiming that clustering alone creates legal validity.

Decision: **approved for Stage 1 crate implementation**. CLI integration and
valid final-plan claims remain blocked until repair is implemented and
`rplan-audit` verifies the resulting plan.

## Scores

| Role | Score | Reason |
|------|-------|--------|
| BOUNDARY | 3.5/4 | Legal claims are deferred to audit profiles; Stage 1 cannot label disconnected plans valid. |
| WARD | 3/4 | Subdivision rules are not in Stage 1, but the spec leaves room for county-aware seeding. |
| COVENANT | 3.5/4 | Lineage fields are concrete and use the U.20 builder boundary. |
| CONTOUR | 3/4 | Graph-distance Stage 1 is acceptable; later geometry/COI inputs need source hashes. |
| MERIDIAN | 3.5/4 | New crate boundary is justified and CLI is deferred correctly. |
| BENCHMARK | 3.5/4 | Canonical fixtures and exact assertions are named. |
| SCALE | 3/4 | Runtime expectations are not yet quantified; acceptable for synthetic Stage 1. |
| PRECINCT | 3.5/4 | Spec avoids neutrality claims and partisan conclusions. |
| DATUM | 3/4 | Falsification is implicit through infeasible/needs-repair statuses. |
| COMMONS | 3/4 | COI hooks are later work; spec distinguishes them from administrative clustering. |
| LEDGER | 3.5/4 | Versioned summary and lineage hash are required from the start. |
| SURVEY | 3.5/4 | Minimal CLI target is clear and not over-promised. |
| TRENCH | 3.5/4 | Stop lines prevent half-valid plans from entering user workflows. |
| **Average** | **3.4/4** | Approved for Stage 1 implementation. |

## Required For Stage 1

- Keep `bisect-clustering` independent of `bisect-cli`.
- Use deterministic tie-breaking everywhere.
- Return structured errors/statuses for impossible capacities.
- Preserve assignment order as explicit unit index order once RPLAN integration
  begins.
- Do not mark `needs-repair` outputs as valid.

## Deferred Blockers

- CLI wiring requires a repair path.
- RPLAN sidecar emission requires final-plan audit certificates.
- Geometry-distance or COI-aware clustering requires source-hash/provenance
  fields beyond Stage 1.
