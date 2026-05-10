---
reviewer: FOCUSED ROLE PANEL
roles: BOUNDARY, WARD, COVENANT, CONTOUR, MERIDIAN, BENCHMARK, LEDGER, TRENCH
spec: U.20 Topology Constraint Certification
round: 3
date: 2026-05-10
score: 3.7
---

# Round 3 Focused Review: U.20 Future Lineage

## Summary

Round 3 reviewed the spec specifically for future-proofing. The new
"Future Consumers And Lineage" section materially improves the contract: later
algorithm families now have named certificate fields and extension points rather
than relying on implied metadata.

The key design choice is correct: keep stable validity facts at the top level
and put algorithm-specific provenance in an optional `algorithm_lineage` payload.
This lets external readers verify plan validity even if they do not understand a
future branch-and-price, flow, clustering, or evolutionary-search payload.

Decision: **approved for phase 1 implementation**. No Round 4 needed before
coding U.20 phase 1.

## Scores

| Role | Score | Reason |
|------|-------|--------|
| BOUNDARY | 3.7/4 | Future consumers remain profile-scoped; no later algorithm can imply legal validity without a profile. |
| WARD | 3.5/4 | State/chamber profile gate survives future consumers; state-rule content remains future data work. |
| COVENANT | 3.8/4 | Lineage fields cover solver, parent-plan, parameters, source hashes, and stable content hash. |
| CONTOUR | 3.7/4 | External audit lineage still rests on explicit unit ids and source hashes. |
| MERIDIAN | 3.6/4 | Algorithm-specific metadata is separated cleanly from validity checks. |
| BENCHMARK | 3.7/4 | Future consumers imply concrete tests for unknown optional fields and fail-closed status handling. |
| LEDGER | 3.8/4 | Forward-compatibility rules are explicit enough for v1 readers. |
| TRENCH | 3.7/4 | Major future drift modes now have structural prevention. |
| **Average** | **3.7/4** | Approved for U.20 phase 1 implementation. |

## Resolved By The Lineage Section

### R3-A: Future consumers are now explicit

The spec now lists how `bisect-ilp`, `bisect-column`, `bisect-clustering`,
`bisect-flow`, `bisect-local-search`, `bisect-pareto`, `bisect-smc`,
`bisect-cli::runner`, `bisect report`, and external audit tools consume U.20.
This gives later specs a stable target.

### R3-B: Extension point is controlled

`AlgorithmLineage.extra` gives later crates room to add solver-specific or
algorithm-specific metadata without changing top-level certificate semantics.
The spec correctly forbids duplicating required top-level fields in `extra`.

### R3-C: Reader compatibility behavior is specified

Readers ignore unknown optional fields but fail closed on unknown statuses,
severities, or malformed required fields. This is the right LEDGER/COVENANT
tradeoff: future metadata can be added, but result semantics cannot drift.

## Required Implementation Tests From Round 3

These are phase 1 tests, not spec blockers:

- v1 reader ignores an unknown optional field
- v1 reader rejects an unknown `CheckStatus`
- v1 reader rejects an unknown `Severity`
- `AlgorithmLineage.extra` participates in `content_hash`
- `AlgorithmLineage.extra` cannot override `plan_hash`, `legal_profile_hash`, or
  `source_hashes`
- certificate for a plan produced by a mock future algorithm remains verifiable
  without understanding the mock algorithm payload

## Remaining Later-Phase Work

- Define concrete `algorithm_lineage.extra` schemas in each future family spec.
- Add state-law profile data for WARD concerns.
- Add COI/community-character witnesses for COMMONS concerns.
- Add real external-auditor examples once `bisect certify` exists.

## Role Decision

BOUNDARY/WARD approve because legal meaning is still entirely profile-scoped.
COVENANT/LEDGER approve because future metadata is versioned, hashable, and
ignorable by older readers. CONTOUR/MERIDIAN approve because plan identity and
validity checks remain independent of algorithm-specific metadata. BENCHMARK and
TRENCH approve because the remaining risks are concrete implementation tests,
not design blockers.

