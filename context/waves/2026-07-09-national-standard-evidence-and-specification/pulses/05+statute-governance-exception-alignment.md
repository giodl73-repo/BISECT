---
pulse: 05
title: Statute governance and exception alignment
status: done
depends_on: 01
wave: national-standard-evidence-and-specification
validation_level: L1 legal design
---

# Pulse 05 - Statute, Governance, And Exception Alignment

## Purpose

Make the model statute implement the canonical specification while preserving
legal judgment, public participation, and a survivable constitutional posture.

## Deliverables

- [x] Align binary versus prime-factor structure, edge weights, seed/search,
      tolerance, resolution, and administering body.
- [x] Separate the binding legal floor from the evidentiary baseline.
- [x] Define VRA, community, state-law, and topology exception procedures.
- [x] Require baseline-to-final diffs and public reasons.
- [x] Add parameter versioning, public challenge, errata, and appeal rules.
- [x] Develop direct-federal, preemption, commission-support, and
      conditional-funding fallback postures.
- [x] Reconcile the quickstart, one-pager, rationale, review notes, and B.02.

## Validation

Run BOUNDARY, COMMONS, COVENANT, DATUM, and hostile constitutional review, then:

```powershell
git --no-pager diff --check
```

Result: passed 2026-07-10. B.02 rebuilt successfully.

## Evidence

- `docs/legal/MODEL_FEDERAL_STATUTE.md`
- `docs/legal/NRS_TECHNICAL_SCHEDULE_V0.1.md`
- `docs/legal/NRS_EVALUATION_SCHEDULE_V0.1.md`
- `context/waves/2026-07-09-national-standard-evidence-and-specification/panels/pulse-05-statute-governance-review.md`

## Closure Rule

Closed as an internal legal-design candidate. The statute and NRS describe the
same benchmark and evaluation procedure. Unresolved constitutional,
appropriations, implementation-readiness, and doctrine questions are explicit
and gate external use.
