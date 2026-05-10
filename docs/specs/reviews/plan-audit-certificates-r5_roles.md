---
reviewer: ROLE PANEL
roles: BOUNDARY, WARD, COVENANT, CONTOUR, MERIDIAN, BENCHMARK, SCALE, PRECINCT, DATUM, COMMONS, LEDGER, SURVEY, TRENCH
spec: U.20 Plan Audit Certificates
round: 5
date: 2026-05-10
score: 3.5
---

# Round 5 Role Review: U.20 Plan Audit Certificates

## Summary

Round 4 blockers are resolved. U.20 now consumes the RPLAN v0.2 plan/context
model directly: `AuditContext` wraps `RplanContext`, context identity is tracked
with `context_hash`, population context can be missing, and `rplan audit` is
context-artifact first.

Decision: **approved for `rplan-audit` phase 1 implementation after
`rplan-core` and `rplan-io` land**.

## Scores

| Role | Score | Reason |
|------|-------|--------|
| BOUNDARY | 3.5/4 | Generic audit layer is cleanly separated from bisect algorithms. |
| WARD | 3.5/4 | The profile-disclaimer model avoids legal overclaiming. |
| COVENANT | 3.5/4 | Certificate identity now includes plan and context identity. |
| CONTOUR | 3.5/4 | `RplanContext` removes duplicate graph-unit state. |
| MERIDIAN | 3.5/4 | Contiguity behavior is tied to explicit graph context. |
| BENCHMARK | 3.5/4 | L0/L1 tests cover missing context and context changes. |
| SCALE | 3.5/4 | Optional context supports lightweight format checks. |
| PRECINCT | 3.5/4 | Multi-unit support flows through the RPLAN plan model. |
| DATUM | 3.5/4 | `context_hash` and source hashes cover provenance needs. |
| COMMONS | 4/4 | Non-bisect audit tools are first-class consumers. |
| LEDGER | 3.5/4 | Audit certificate schema is versioned and hashable. |
| SURVEY | 3.5/4 | `rplan audit --context` is clear enough for phase 1. |
| TRENCH | 3/4 | Phase 1 is implementable; geometry/VRA remain later phases. |
| **Average** | **3.5/4** | Approved for phase 1. |

## Resolved Since Round 4

### R5-A: `AuditContext` now wraps `RplanContext`

The spec no longer duplicates `graph_unit_ids`. The audit rule is based on
matching `unit_universe_hash`.

### R5-B: Population context is optional

`populations: Option<Vec<i64>>` allows missing-input audit results instead of
sentinel vectors.

### R5-C: Certificates include `context_hash`

Audit certificates distinguish the same plan under different graph/population
contexts.

### R5-D: CLI is context-artifact first

`rplan audit` now uses `--context PATH`; `--data-dir` is only a possible bisect
wrapper convenience.

### R5-E: Certification wording is narrowed

The operational language now says audit/pass/fail under supplied profiles,
not legal certification.

## Phase 1 Implementation Scope

Approved implementation surface:

- plan-shape audit
- context matching by `unit_universe_hash`
- population audit with `MissingInput`
- contiguity audit from `UnitGraph`
- `AuditCertificate` v1 with `context_hash`
- `rplan audit` fixture CLI

Not approved yet:

- VRA liability analysis
- state-specific split-law compliance beyond profile-gated checks
- geometric compactness claims without `rplan-geo`
- legal certification wording
