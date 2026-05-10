---
reviewer: ROLE PANEL
roles: BOUNDARY, WARD, COVENANT, CONTOUR, MERIDIAN, BENCHMARK, SCALE, PRECINCT, DATUM, COMMONS, LEDGER, SURVEY, TRENCH
spec: U.20 Topology Constraint Certification
round: 2
date: 2026-05-10
score: 3.5
---

# Round 2 Role Review: U.20 Topology Constraint Certification

## Summary

Round 1 blockers are resolved. The spec is now implementation-ready for U.20
phase 1: plan shape, population, contiguity, certificate JSON, stable hashing,
and canonical fixtures.

Round 2 found one remaining issue that affected testability: canonical JSON was
still deferred. That has now been patched into the spec, along with a distinction
between the full document hash and stable certificate content hash.

Decision: **approved for phase 1 implementation**.

## Scores

| Role | Score | Reason |
|------|-------|--------|
| BOUNDARY | 3.5/4 | Legal-profile scope is clear; no legal safe-harbor overclaim remains. |
| WARD | 3/4 | State-law certification remains profile-gated, which is correct for phase 1. |
| COVENANT | 3.5/4 | Stable content hashing and provenance fields are now sufficient for implementation. |
| CONTOUR | 3.5/4 | Graph-unit identity is explicit; loader implementation must preserve GEOIDs. |
| MERIDIAN | 3.5/4 | Contiguity via induced graph components is clear and testable. |
| BENCHMARK | 3.5/4 | Canonical JSON, fixtures, and exit-code behavior are now testable. |
| SCALE | 3.5/4 | No quantitative overclaim remains in this spec. |
| PRECINCT | 3.5/4 | VRA/partisan matters are kept out of phase 1 certification claims. |
| DATUM | 3.5/4 | The spec has concrete falsification tests. |
| COMMONS | 2.5/4 | COI witnesses remain later-phase work; acceptable for phase 1. |
| LEDGER | 3.5/4 | Versioned schema plus canonical JSON is enough to start. |
| SURVEY | 3.5/4 | CLI behavior is operationally clear. |
| TRENCH | 3.5/4 | Known phase 1 failure modes have structural checks. |
| **Average** | **3.5/4** | Approved for implementation of phase 1 only. |

## Resolved Since Round 1

### R2-A: Canonical JSON is now specified

The spec now defines UTF-8 encoding, whitespace, key ordering, array ordering,
integer formatting, float constraints, and null/absent distinction. This closes
the main BENCHMARK/LEDGER/COVENANT concern for stable hashes.

### R2-B: Full document hash and stable content hash are separated

The run manifest now stores both:

- `certificate_sha256`: hash of the certificate file as written
- `certificate_content_hash`: stable hash excluding volatile metadata

This prevents tests and audit tools from relying on timestamps.

### R2-C: Phase 1 witness schema is scoped

The spec now requires concrete phase 1 witness structs for population,
contiguity, and missing input. Split/VRA/geometry witnesses can remain reserved
schema variants until later phases.

## Remaining Non-Blocking Items

### P2-A: Example JSON certificates

Add examples during implementation:

- valid 3x3 fixture
- disconnected path-graph failure
- missing VAP warning/failure fixture

### P2-B: Profile seed files

Add `profiles/us-congressional-project-v1.json` with the implementation PR.
Add a state-legislative negative fixture proving profile-required behavior.

### P2-C: Pitfall records

When code starts, add pitfall entries for assignment-order drift, volatile
certificate hashes, missing-input false passes, and project-profile overclaim.

## Role Decision

No role raises a phase 1 blocker. BOUNDARY/WARD approve because legal claims are
profile-scoped. COVENANT/LEDGER approve because hashes and schema versions are
stable. MERIDIAN/BENCHMARK approve because phase 1 checks are concrete and
testable. COMMONS remains the lowest score, but its concerns apply to later COI
reporting rather than the phase 1 validity kernel.

